use crate::ascii::Color;
use crate::ascii::RESET;
use crate::ffmpeg;
use crate::math;
use crate::rumpeg::*;
use std::fmt;
use thiserror::Error;

const MAX_FILM_WIDTH: i32 = 8;

#[derive(Debug)]
pub struct Video<'a> {
  pub duration_ms: i64,
  pub extensions: &'a str,
  pub format_name: &'a str,
  pub height: i32,
  pub mime_type: &'a str,
  pub width: i32,
  codec_context: AVCodecContext,
  display_matrix: Option<math::Matrix3x3>,
  format_context: AVFormatContext,
  sws_context: SwsContext,
}

#[derive(Error, Debug)]
pub enum VideoError {
  #[error(transparent)]
  Rumpeg(#[from] RumpegError),
  #[error("At least 1 frame is needed to create a film strip, found {0}")]
  NoFramesInFilmStrip(i32),
}

type VideoResult<T = ()> = Result<T, VideoError>;

impl<'a> Video<'a> {
  pub fn open(filepath: &'a str, w: i32, h: i32) -> VideoResult<Video> {
    let format_context = AVFormatContext::new(filepath)?;
    let codec_context = AVCodecContext::new(format_context.stream.codecpar)?;
    let iformat = AVInputFormat::new(format_context.iformat);
    let display_matrix = format_context.stream.display_matrix();

    Ok(Self {
      duration_ms: format_context.stream.duration_millis(),
      extensions: iformat.extensions,
      format_name: iformat.format_name,
      height: codec_context.height,
      mime_type: iformat.mime_type,
      width: codec_context.width,
      sws_context: SwsContext::new(SwsFrameProperties::from(&codec_context), w, h)?,
      codec_context,
      display_matrix,
      format_context,
    })
  }

  pub fn frame_to_webp(&self, frame: &mut AVFrame) -> VideoResult<&[u8]> {
    Ok(
      self
        .sws_context
        .transform(frame, self.display_matrix)?
        .encode_as_webp()?,
    )
  }

  pub fn film_strip(
    &self,
    start: SeekPosition,
    end: SeekPosition,
    step: SeekPosition,
  ) -> VideoResult<AVFrame> {
    let tile_count = {
      let start = self.format_context.stream.as_time_base(start);
      let end = self.format_context.stream.as_time_base(end);
      let step = self.format_context.stream.as_time_base(step);
      ((end - start) as f64 / step as f64).ceil() as i32
    };

    if tile_count < 1 {
      return Err(VideoError::NoFramesInFilmStrip(tile_count));
    }

    let (tile_w, tile_h) = (self.sws_context.width(), self.sws_context.height());
    let mut tile_cols = std::cmp::min(tile_count, MAX_FILM_WIDTH);
    let mut tile_rows = (tile_count as f64 / MAX_FILM_WIDTH as f64).ceil() as i32;
    let rotation = self
      .display_matrix
      .map(|m| m.rotation() as i32)
      .unwrap_or(0);

    if rotation.abs() == 90 {
      std::mem::swap(&mut tile_cols, &mut tile_rows);
    }

    let mut film_strip = AVFrame::new(
      ffmpeg::AVPixelFormat_AV_PIX_FMT_YUV420P,
      tile_w * tile_cols,
      tile_h * tile_rows,
    )?;
    film_strip.data_mut(0).fill(0);
    film_strip.data_mut(1).fill(128);
    film_strip.data_mut(2).fill(128);

    for (thumb_pos, mut frame) in self.frames(start, end, step)?.enumerate() {
      frame = self.sws_context.transform(&mut frame, None)?; // Rotating the final film frame performs better

      let mut tile_x = thumb_pos as i32 % MAX_FILM_WIDTH;
      let mut tile_y = thumb_pos as i32 / MAX_FILM_WIDTH;

      if rotation.abs() == 90 {
        std::mem::swap(&mut tile_x, &mut tile_y);
      }

      for plane in 0..3 {
        let frame_stride = frame.linesize[plane];
        let frame_height = if plane == 0 {
          frame.plane_height(0) as f32
        } else {
          frame.plane_height(0) as f32 / 2.
        };

        let film_stride = film_strip.linesize[plane];
        let film_height = film_strip.plane_height(plane);

        let (tile_x_offset, tile_y_offset) = match rotation {
          -180 | 180 => (
            film_stride - tile_x * frame_stride - frame_stride,
            (film_height as f32 - tile_y as f32 * frame_height - frame_height) as i32,
          ),
          -90 => (
            film_stride - tile_x * frame_stride - frame_stride,
            (tile_y as f32 * frame_height) as i32,
          ),
          90 => (
            tile_x * frame_stride,
            (film_height as f32 - tile_y as f32 * frame_height - frame_height) as i32,
          ),
          _ => (tile_x * frame_stride, (tile_y as f32 * frame_height) as i32),
        };

        let film_data_start = tile_x_offset + film_stride * tile_y_offset;

        let frame_data = frame.data(plane);
        let film_data = film_strip.data_mut(plane);
        for y in 0..frame_height as i32 {
          let film_row_start = (film_data_start + film_stride * y) as usize;
          let film_row_end = film_row_start + frame_stride as usize;
          let frame_row_start = (y * frame_stride) as usize;
          let frame_row_end = frame_row_start + frame_stride as usize;

          if film_row_end <= film_data.len() {
            film_data[film_row_start..film_row_end]
              .copy_from_slice(&frame_data[frame_row_start..frame_row_end]);
          }
        }
      }
    }

    if let Some(matrix) = self.display_matrix {
      film_strip.transform(matrix)?;
    }

    Ok(film_strip)
  }

  pub fn frames(
    &self,
    start: SeekPosition,
    end: SeekPosition,
    step: SeekPosition,
  ) -> VideoResult<AVFrameIter> {
    self.seek(start)?;
    Ok(
      self
        .format_context
        .frames(self.codec_context.as_ptr(), start, end, step),
    )
  }

  fn seek(&self, position: SeekPosition) -> RumpegResult {
    self.codec_context.flush();
    self.format_context.seek(position)
  }
}

impl<'a> fmt::Display for Video<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{title}VIDEO INFO{RESET}\n\
      - {title}File Name:{RESET} {}\n\
      - {title}Display Matrix:{RESET} {}\n\
      - {title}Rotation:{RESET} {}°\n\
      - {title}Input{RESET}\n  \
      - {title}Width:{RESET} {}\n  \
      - {title}Height:{RESET} {}\n\
      - {title}Output{RESET}\n  \
      - {title}Width:{RESET} {}\n  \
      - {title}Height:{RESET} {}\n\
      - {title}Duration:{RESET} {} seconds\n\
      - {title}Extensions:{RESET} {}\n\
      - {title}Format:{RESET} {}\n\
      - {title}Sample Aspect Ratio:{RESET} {:?}\n\
      - {title}Stream Time Base:{RESET} {:?}\n\
      - {title}Stream Duration:{RESET} {}\n\
      - {title}Framerate:{RESET} {:?}\n\
      - {title}Average Framerate:{RESET} {:?}\n\
      - {title}Base Framerate:{RESET} {:?}\n\
      - {title}GOP Size:{RESET} {}\n\
      - {title}Mime Type:{RESET} {}",
      ptr_to_str(self.format_context.url).unwrap_or("N/A"),
      self
        .display_matrix
        .map(|m| format!("\n{m}"))
        .unwrap_or("None".into()),
      self.display_matrix.map(|m| m.rotation()).unwrap_or(0.),
      self.codec_context.width,
      self.codec_context.height,
      self.sws_context.width(),
      self.sws_context.height(),
      self.duration_ms as f64 / 1000.,
      self.extensions,
      self.format_name,
      self.codec_context.sample_aspect_ratio,
      self.format_context.stream.time_base,
      self.format_context.stream.duration,
      self.codec_context.framerate,
      self.format_context.stream.avg_frame_rate,
      self.format_context.stream.r_frame_rate,
      self.codec_context.gop_size,
      self.mime_type,
      title = "".rgb(75, 205, 94).bold(),
    )
  }
}
