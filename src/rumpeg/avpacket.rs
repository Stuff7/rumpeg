use std::ops::{Deref, DerefMut};

use crate::ffmpeg;

pub struct AVPacket {
  ptr: *mut ffmpeg::AVPacket,
}

impl AVPacket {
  pub fn new() -> Self {
    unsafe {
      Self {
        ptr: ffmpeg::av_packet_alloc(),
      }
    }
  }
}

impl Drop for AVPacket {
  fn drop(&mut self) {
    unsafe {
      println!("DROPPING AVPacket");
      ffmpeg::av_packet_free(&mut self.ptr);
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
