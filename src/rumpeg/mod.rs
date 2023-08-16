mod avcodec;
mod avformat;
mod avframe;
mod avpacket;
mod avstream;
mod sws;

pub use avcodec::*;
pub use avformat::*;
pub use avframe::*;
pub use avpacket::*;
pub use avstream::*;
pub use sws::*;

use crate::{ffmpeg, math::MathError};
use std::ffi::CStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RumpegError {
  #[error("avcodec_alloc_context3 failed")]
  AVCodecContextAllocFail,
  #[error("{0} (Code {1}): {2}")]
  AVError(&'static str, i32, String),
  #[error("avformat_alloc_context failed")]
  AVFormatContextAllocFail,
  #[error("av_frame_alloc failed")]
  AVFrameCreation,
  #[error("No decoder found")]
  DecoderMissing,
  #[error("Unknown codec, could not determine pixel format")]
  PixelFormatMissing,
  #[error("sws_getContext failed")]
  SWSContextCreation,
  #[error("No video format found")]
  VideoFormatMissing,
  #[error(transparent)]
  Math(#[from] MathError),
}

impl RumpegError {
  fn from_code(code: i32, msg: &'static str) -> Self {
    unsafe {
      let mut error_buffer: [libc::c_char; 256] = [0; 256];
      ffmpeg::av_strerror(code, error_buffer.as_mut_ptr(), error_buffer.len());
      let error_msg = CStr::from_ptr(error_buffer.as_ptr())
        .to_string_lossy()
        .to_string();

      RumpegError::AVError(msg, code, error_msg)
    }
  }
}

pub type RumpegResult<T = ()> = Result<T, RumpegError>;
