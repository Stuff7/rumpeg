use std::ops::{Deref, DerefMut};

use super::RumpegError;
use super::RumpegResult;

use crate::ffmpeg;

pub struct AVFrame {
  ptr: *mut ffmpeg::AVFrame,
}

impl AVFrame {
  pub fn new() -> RumpegResult<Self> {
    unsafe {
      let ptr = ffmpeg::av_frame_alloc();
      if ptr.is_null() {
        return Err(RumpegError::AVFrameCreation);
      }
      Ok(Self { ptr })
    }
  }
}

impl Drop for AVFrame {
  fn drop(&mut self) {
    unsafe {
      println!("DROPPING AVFrame");
      ffmpeg::av_frame_free(&mut self.ptr);
    }
  }
}

impl Deref for AVFrame {
  type Target = ffmpeg::AVFrame;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.ptr }
  }
}

impl DerefMut for AVFrame {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.ptr }
  }
}

pub struct RGBBuffer {
  ptr: *mut u8,
}

impl RGBBuffer {
  pub fn new(width: i32, height: i32) -> RumpegResult<Self> {
    unsafe {
      let rgb_size =
        ffmpeg::av_image_get_buffer_size(ffmpeg::AVPixelFormat_AV_PIX_FMT_RGB24, width, height, 1);
      let ptr = libc::malloc(rgb_size as usize) as *mut u8;
      Ok(Self { ptr })
    }
  }
}

impl Drop for RGBBuffer {
  fn drop(&mut self) {
    unsafe {
      println!("DROPPING RGBBuffer");
      libc::free(self.ptr as *mut libc::c_void);
    }
  }
}

impl Deref for RGBBuffer {
  type Target = *mut u8;

  fn deref(&self) -> &Self::Target {
    &self.ptr
  }
}

impl DerefMut for RGBBuffer {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.ptr
  }
}
