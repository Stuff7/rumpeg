mod avcodec;
mod avformat;
mod avframe;
mod avpacket;
mod avpixel;
mod avstream;
mod sws;

pub use avcodec::*;
pub use avformat::*;
pub use avframe::*;
pub use avpacket::*;
pub use avpixel::*;
pub use avstream::*;
pub use sws::*;

use crate::{ffmpeg, math::MathError, webp::WebPError};
use std::{
  ffi::{CStr, NulError},
  str::FromStr,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RumpegError {
  #[error("Could not allocate AVCodecContext")]
  AVCodecContextAllocFail,
  #[error("{0}: AVError - {2} (Code {1})")]
  AVError(String, i32, String),
  #[error("Could not allocate AVFormatContext")]
  AVFormatContextAllocFail,
  #[error("Could not allocate AVFrame")]
  AVFrameCreation,
  #[error("Could not create CString\n{0}")]
  CStringCreation(#[from] NulError),
  #[error("No decoder found")]
  DecoderMissing,
  #[error(transparent)]
  Math(#[from] MathError),
  #[error("Unknown codec, could not determine pixel format (Codec ID {0})")]
  PixelFormatMissing(i32),
  #[error("Could not create SwsContext")]
  SwsContextCreation,
  #[error("Unknown log level")]
  UnknownLogLevel,
  #[error("No video format found")]
  VideoFormatMissing,
  #[error(transparent)]
  WebPError(#[from] WebPError),
}

impl RumpegError {
  fn from_code(code: i32, msg: &str) -> Self {
    unsafe {
      let mut error_buffer: [libc::c_char; 256] = [0; 256];
      ffmpeg::av_strerror(code, error_buffer.as_mut_ptr(), error_buffer.len());
      let error_msg = CStr::from_ptr(error_buffer.as_ptr())
        .to_string_lossy()
        .to_string();

      RumpegError::AVError(msg.to_string(), code, error_msg)
    }
  }
}

pub type RumpegResult<T = ()> = Result<T, RumpegError>;

pub fn version() -> &'static str {
  unsafe { ptr_to_str(ffmpeg::av_version_info()).unwrap_or("N/A") }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum LogLevel {
  Quiet = ffmpeg::AV_LOG_QUIET as isize,
  Panic = ffmpeg::AV_LOG_PANIC as isize,
  Fatal = ffmpeg::AV_LOG_FATAL as isize,
  Error = ffmpeg::AV_LOG_ERROR as isize,
  #[default]
  Warning = ffmpeg::AV_LOG_WARNING as isize,
  Info = ffmpeg::AV_LOG_INFO as isize,
  Verbose = ffmpeg::AV_LOG_VERBOSE as isize,
  Debug = ffmpeg::AV_LOG_DEBUG as isize,
  Trace = ffmpeg::AV_LOG_TRACE as isize,
}

impl FromStr for LogLevel {
  type Err = RumpegError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let s = s.to_lowercase();
    if s == "quiet" {
      Ok(Self::Quiet)
    } else if s == "panic" {
      Ok(Self::Panic)
    } else if s == "fatal" {
      Ok(Self::Fatal)
    } else if s == "error" {
      Ok(Self::Error)
    } else if s == "warning" {
      Ok(Self::Warning)
    } else if s == "info" {
      Ok(Self::Info)
    } else if s == "verbose" {
      Ok(Self::Verbose)
    } else if s == "debug" {
      Ok(Self::Debug)
    } else if s == "trace" {
      Ok(Self::Trace)
    } else {
      Err(RumpegError::UnknownLogLevel)
    }
  }
}

pub fn set_log_level(level: LogLevel) {
  unsafe {
    ffmpeg::av_log_set_level(level as i32);
  }
}

pub fn ptr_to_str(ptr: *const i8) -> Option<&'static str> {
  unsafe {
    (!ptr.is_null())
      .then(|| CStr::from_ptr(ptr).to_str().ok())
      .flatten()
  }
}
