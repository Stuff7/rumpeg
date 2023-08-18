use std::fmt::Display;
use std::ops::DerefMut;

use thiserror::Error;

use crate::ffmpeg;
use crate::math;
use crate::rumpeg;
use crate::rumpeg::SeekPosition;

#[derive(Debug)]
pub struct Video {
  pub duration_us: u64,
  pub extensions: &'static str,
  pub format_name: &'static str,
  pub height: i32,
  pub mime_type: &'static str,
  pub width: i32,
  codec_context: rumpeg::AVCodecContext,
  display_matrix: Option<math::Matrix3x3>,
  format_context: rumpeg::AVFormatContext,
  stream_index: i32,
  sws_context: rumpeg::SWSContext,
}

#[derive(Error, Debug)]
pub enum VideoError {
  #[error("No frame found at second {0:?}")]
  FrameOutOfBounds(SeekPosition),
  #[error(transparent)]
  Rumpeg(#[from] rumpeg::RumpegError),
}

type VideoResult<T = ()> = Result<T, VideoError>;

impl Video {
  pub fn open(filepath: &str) -> VideoResult<Video> {
    let mut format_context = rumpeg::AVFormatContext::new(filepath)?;
    let stream = format_context.stream()?;
    let codecpar = rumpeg::AVCodecParameters::new(stream.codecpar)?;
    let codec_context = rumpeg::AVCodecContext::new(&codecpar)?;
    let iformat = rumpeg::AVInputFormat::new(format_context.iformat);
    let display_matrix = stream.display_matrix();

    Ok(Self {
      duration_us: format_context.duration as u64,
      extensions: iformat.extensions,
      format_name: iformat.format_name,
      height: codecpar.height,
      mime_type: iformat.mime_type,
      width: codecpar.width,
      codec_context,
      display_matrix,
      format_context,
      stream_index: stream.index,
      sws_context: rumpeg::SWSContextBuilder::from_codecpar(codecpar).build()?,
    })
  }

  pub fn resize_output(&mut self, width: i32, height: i32) -> VideoResult {
    Ok(self.sws_context.resize_output(width, height)?)
  }

  pub fn get_frame(&mut self, position: SeekPosition, thumbnail_path: &str) -> VideoResult {
    unsafe {
      let mut frame = rumpeg::AVFrame::empty()?;
      let mut packet = rumpeg::AVPacket::empty();
      let mut found_keyframe = false;

      self.format_context.seek(position)?;

      while ffmpeg::av_read_frame(&mut *self.format_context, packet.deref_mut()) >= 0 {
        if packet.stream_index == self.stream_index {
          ffmpeg::avcodec_send_packet(&mut *self.codec_context, &*packet);
          let result = ffmpeg::avcodec_receive_frame(&mut *self.codec_context, &mut *frame);
          if result == 0 {
            found_keyframe = true;
            break;
          }
        }
      }

      if !found_keyframe {
        return Err(VideoError::FrameOutOfBounds(position));
      }

      let webp = self
        .sws_context
        .transform(&mut frame, self.display_matrix)?
        .encode_as_webp();
      std::fs::write(thumbnail_path, &*webp).expect("Failed to save image");

      self.codec_context.flush();
      Ok(())
    }
  }
}

impl Display for Video {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "VIDEO INFO\n\
      - Display Matrix: {}\n\
      - Rotation: {}°\n\
      {}\n\
      - Duration: {} seconds\n\
      - Extensions: {}\n\
      - Format: {}\n\
      - Mime Type: {}",
      self
        .display_matrix
        .map(|m| format!("\n{m}"))
        .unwrap_or("None".into()),
      self.display_matrix.map(|m| m.rotation()).unwrap_or(0.),
      self.sws_context,
      self.duration_us as f64 / 1_000_000.,
      self.extensions,
      self.format_name,
      self.mime_type,
    )
  }
}
