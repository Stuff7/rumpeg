use std::ops::DerefMut;

use thiserror::Error;

use crate::ffmpeg;
use crate::rumpeg;

#[derive(Debug)]
pub struct Video {
  pub duration_us: u64,
  pub extensions: &'static str,
  pub format_name: &'static str,
  pub height: i32,
  pub mime_type: &'static str,
  pub width: i32,
  codec_context: rumpeg::AVCodecContext,
  format_context: rumpeg::AVFormatContext,
  pixel_format: i32,
  stream_index: i32,
  sws_context: rumpeg::SWSContext,
}

#[derive(Error, Debug)]
pub enum VideoError {
  #[error("No frame found at second {0}")]
  FrameOutOfBounds(i64),
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
    stream.display_matrix()?;

    Ok(Self {
      duration_us: format_context.duration as u64,
      extensions: iformat.extensions,
      format_name: iformat.format_name,
      height: codecpar.height,
      mime_type: iformat.mime_type,
      width: codecpar.width,
      codec_context,
      format_context,
      pixel_format: codecpar.pixel_format,
      stream_index: stream.index,
      sws_context: rumpeg::SWSContext::new(codecpar.width, codecpar.height, codecpar.pixel_format)?,
    })
  }

  pub fn get_thumbnail(&mut self, seconds: i64, thumbnail_path: &str) -> VideoResult {
    unsafe {
      let mut frame = rumpeg::AVFrame::new()?;
      let mut packet = rumpeg::AVPacket::new();
      let mut found_keyframe = false;

      self.format_context.seek(seconds)?;

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
        return Err(VideoError::FrameOutOfBounds(seconds));
      }

      let encoded_data = self.sws_context.encode_as_webp(&mut frame)?;
      std::fs::write(thumbnail_path, &*encoded_data).expect("Failed to save image");

      self.codec_context.flush();
      Ok(())
    }
  }
}
