use std::ptr;
use std::slice;

use webp::Encoder;
use webp::WebPMemory;

use super::RumpegError;
use super::RumpegResult;

use crate::ffmpeg;

#[derive(Debug)]
pub struct SWSContext {
  ptr: *mut ffmpeg::SwsContext,
  width: i32,
  height: i32,
}

impl SWSContext {
  pub fn new(width: i32, height: i32, pixel_format: i32) -> RumpegResult<Self> {
    unsafe {
      let ptr = ffmpeg::sws_getContext(
        width,
        height,
        pixel_format,
        width,
        height,
        ffmpeg::AVPixelFormat_AV_PIX_FMT_RGB24,
        ffmpeg::SWS_SINC as i32,
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
      );
      if ptr.is_null() {
        Err(RumpegError::SWSContextCreation)
      } else {
        Ok(Self { ptr, width, height })
      }
    }
  }

  pub fn encode_as_webp(&self, frame: &mut super::AVFrame) -> RumpegResult<WebPMemory> {
    unsafe {
      let mut frame_rgb = super::AVFrame::new()?;

      let rgb_buffer = super::RGBBuffer::new(self.width, self.height)?;

      ffmpeg::av_image_fill_arrays(
        frame_rgb.data.as_mut_ptr() as *mut *mut u8,
        frame_rgb.linesize.as_mut_ptr(),
        *rgb_buffer,
        ffmpeg::AVPixelFormat_AV_PIX_FMT_RGB24,
        self.width,
        self.height,
        1,
      );

      ffmpeg::sws_scale(
        self.ptr,
        frame.data.as_mut_ptr() as *mut *const u8,
        frame.linesize.as_mut_ptr(),
        0,
        self.height,
        frame_rgb.data.as_mut_ptr(),
        frame_rgb.linesize.as_mut_ptr(),
      );

      let encoder = Encoder::from_rgb(
        slice::from_raw_parts(*rgb_buffer, (self.width * self.height * 3) as usize), // Assuming RGB format with 3 bytes per pixel
        self.width as u32,
        self.height as u32,
      );

      Ok(encoder.encode(50.))
    }
  }
}

impl Drop for SWSContext {
  fn drop(&mut self) {
    unsafe {
      println!("DROPPING SWSContext");
      ffmpeg::sws_freeContext(self.ptr);
    }
  }
}
