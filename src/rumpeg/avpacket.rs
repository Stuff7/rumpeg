use std::ops::{Deref, DerefMut};

use super::*;
use crate::ffmpeg;

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
  pub fn read(&mut self, format: *mut ffmpeg::AVFormatContext) -> RumpegResult {
    unsafe {
      match ffmpeg::av_read_frame(format, self.deref_mut()) {
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
