use std::ops::{Deref, DerefMut};

use super::*;
use crate::ascii::Color;
use crate::{ffmpeg, log};

pub struct AVPacket {
  ptr: *mut ffmpeg::AVPacket,
}

impl AVPacket {
  pub fn empty() -> Self {
    unsafe {
      Self {
        ptr: ffmpeg::av_packet_alloc(),
      }
    }
  }
}

impl AVPacket {
  pub fn read(&mut self, format: &mut AVFormatContext) -> RumpegResult {
    unsafe {
      match ffmpeg::av_read_frame(format.deref_mut(), self.deref_mut()) {
        0 => Ok(()),
        e => Err(RumpegError::from_code(e, "Failed to read packet")),
      }
    }
  }
}

impl Deref for AVPacket {
  type Target = ffmpeg::AVPacket;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.ptr }
  }
}

impl DerefMut for AVPacket {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.ptr }
  }
}

impl Drop for AVPacket {
  fn drop(&mut self) {
    unsafe {
      ffmpeg::av_packet_free(&mut self.ptr);
    }
  }
}

pub struct AVPacketIter<'a> {
  format_context: &'a mut AVFormatContext,
  codec_context: &'a mut AVCodecContext,
  step: SeekPosition,
}

impl<'a> AVPacketIter<'a> {
  pub fn new(
    format_context: &'a mut AVFormatContext,
    codec_context: &'a mut AVCodecContext,
    step: Option<SeekPosition>,
  ) -> Self {
    Self {
      format_context,
      codec_context,
      step: step.unwrap_or_default(),
    }
  }
}

impl<'a> Iterator for AVPacketIter<'a> {
  type Item = AVFrame;

  fn next(&mut self) -> Option<<Self as Iterator>::Item> {
    let Ok(mut frame) = AVFrame::empty() else {return None};
    let mut packet = AVPacket::empty();
    let step = self.format_context.stream.to_time_base(self.step);

    loop {
      match packet.read(self.format_context) {
        Ok(..) => unsafe {
          if packet.stream_index == self.format_context.stream.index {
            ffmpeg::avcodec_send_packet(self.codec_context.deref_mut(), &*packet);
            let result = ffmpeg::avcodec_receive_frame(self.codec_context.deref_mut(), &mut *frame);
            if result == 0 {
              self.codec_context.flush();
              self
                .format_context
                .seek(SeekPosition::TimeBase(frame.pts + step))
                .ok();
              return Some(frame);
            }
            if result != ffmpeg::AVERROR(ffmpeg::EAGAIN as i32) {
              log!(err@
                "{}",
                RumpegError::from_code(result, "Encountered AVError while receiving frame")
              );
              return None;
            }
          }
        },
        Err(e) => {
          if let RumpegError::AVError(_, code, _) = e {
            if code == ffmpeg::AVERROR_EOF {
              return None;
            }
          }
          log!(err@"Encountered AVError while reading frame {e}");
        }
      }
    }
  }
}
