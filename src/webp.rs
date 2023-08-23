#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use thiserror::Error;

use crate::rumpeg::AVFrame;

include!(concat!(env!("OUT_DIR"), "/libwebp.rs"));

pub fn encode_frame_as_webp(frame: &AVFrame, quality_factor: f32) -> WebPResult<Vec<u8>> {
  let mut output_ptr: *mut u8 = std::ptr::null_mut();
  let result = unsafe {
    WebPEncodeRGB(
      frame.data().as_ptr(),
      frame.width,
      frame.height,
      frame.linesize[0],
      quality_factor,
      &mut output_ptr,
    )
  };

  if result == 0 {
    return Err(WebPError::Encoding);
  }

  let encoded_data = unsafe {
    let encoded_size = result;
    let data = std::slice::from_raw_parts(output_ptr, encoded_size).to_vec();
    WebPFree(output_ptr as *mut _);
    data
  };

  Ok(encoded_data)
}

#[derive(Debug, Error)]
pub enum WebPError {
  #[error("Could not encode webp")]
  Encoding,
}

type WebPResult<T = ()> = Result<T, WebPError>;

pub struct WebP {
  quality_factor: f32,
}

impl WebP {
  pub fn new(quality_factor: f32) -> Self {
    Self { quality_factor }
  }

  pub fn encode(&self, rgb_image: &[u8], width: i32, height: i32, stride: i32) -> Option<Vec<u8>> {
    let mut output_ptr: *mut u8 = std::ptr::null_mut();
    let result = unsafe {
      WebPEncodeRGB(
        rgb_image.as_ptr(),
        width,
        height,
        stride,
        self.quality_factor,
        &mut output_ptr,
      )
    };

    if result == 0 {
      return None;
    }

    let encoded_data = unsafe {
      let encoded_size = result;
      let data = std::slice::from_raw_parts(output_ptr, encoded_size).to_vec();
      WebPFree(output_ptr as *mut _);
      data
    };

    Some(encoded_data)
  }
}
