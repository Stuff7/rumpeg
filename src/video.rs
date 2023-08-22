use crate::ascii::Color;
use crate::ascii::RESET;
use crate::math;
use crate::rumpeg::*;
use std::fmt;
use thiserror::Error;
use webp::WebPMemory;

#[derive(Debug)]
pub struct Video {
  pub duration_us: u64,
  pub extensions: &'static str,
  pub format_name: &'static str,
  pub height: i32,
  pub mime_type: &'static str,
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
}

type VideoResult<T = ()> = Result<T, VideoError>;

impl Video {
  pub fn open(filepath: &str) -> VideoResult<Video> {
    let format_context = AVFormatContext::new(filepath)?;
    let codec_context = AVCodecContext::new(format_context.stream.codecpar)?;
    let iformat = AVInputFormat::new(format_context.iformat);
    let display_matrix = format_context.stream.display_matrix();

    Ok(Self {
      duration_us: format_context.duration as u64,
      extensions: iformat.extensions,
      format_name: iformat.format_name,
      height: codec_context.height,
      mime_type: iformat.mime_type,
      width: codec_context.width,
      sws_context: SwsContextBuilder::from_codec_context(&codec_context).build()?,
      codec_context,
      display_matrix,
      format_context,
    })
  }

  pub fn resize_output(&mut self, width: i32, height: i32) -> VideoResult {
    Ok(self.sws_context.resize_output(width, height)?)
  }

  pub fn frame_to_webp(&self, frame: &mut AVFrame) -> VideoResult<WebPMemory> {
    Ok(
      self
        .sws_context
        .transform(frame, self.display_matrix)?
        .encode_as_webp(),
    )
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

impl fmt::Display for Video {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{title}VIDEO INFO{RESET}\n\
      - {title}File Name:{RESET} {}\n\
      - {title}Display Matrix:{RESET} {}\n\
      - {title}Rotation:{RESET} {}°\n\
      {}\n\
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
      self.sws_context,
      self.duration_us as f64 / 1_000_000.,
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
