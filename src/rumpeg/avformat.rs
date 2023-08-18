use std::ffi::CStr;
use std::ffi::CString;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ptr;
use std::str::FromStr;

use super::*;
use crate::ffmpeg;

#[derive(Debug)]
pub struct AVFormatContext {
  ptr: *mut ffmpeg::AVFormatContext,
}

impl AVFormatContext {
  pub fn new(filepath: &str) -> RumpegResult<Self> {
    let filename = CString::new(filepath).expect("CString creation failed");

    unsafe {
      let mut ptr = ffmpeg::avformat_alloc_context();
      if ptr.is_null() {
        ffmpeg::avformat_close_input(&mut ptr);
        return Err(RumpegError::AVFormatContextAllocFail);
      }

      let result = ffmpeg::avformat_open_input(
        &mut ptr,
        filename.as_ptr(),
        ptr::null_mut(),
        ptr::null_mut(),
      );

      if result < 0 {
        ffmpeg::avformat_close_input(&mut ptr);
        return Err(RumpegError::from_code(result, "avformat_open_input failed"));
      }

      let iformat = (*ptr).iformat;
      if iformat.is_null() {
        ffmpeg::avformat_close_input(&mut ptr);
        return Err(RumpegError::VideoFormatMissing);
      }

      Ok(Self { ptr })
    }
  }

  pub fn stream(&mut self) -> RumpegResult<AVStream> {
    unsafe {
      let stream_index = ffmpeg::av_find_best_stream(
        self.ptr,
        ffmpeg::AVMediaType_AVMEDIA_TYPE_VIDEO,
        -1,
        -1,
        ptr::null_mut(),
        0,
      );

      if stream_index < 0 {
        ffmpeg::avformat_close_input(&mut self.ptr);
        return Err(RumpegError::from_code(
          stream_index,
          "No video stream found",
        ));
      }

      Ok(AVStream::new(
        *self.streams.offset(stream_index as isize),
        stream_index,
      ))
    }
  }

  pub fn seek(&mut self, position: SeekPosition) -> RumpegResult {
    unsafe {
      let seconds = match position {
        SeekPosition::Seconds(n) => ffmpeg::av_rescale_q(
          n,
          ffmpeg::AVRational { den: 1, num: 1 },
          ffmpeg::av_get_time_base_q(),
        ),
        SeekPosition::Percentage(n) => (self.duration as f64 * n) as i64,
      };
      match ffmpeg::av_seek_frame(
        &mut *self.ptr,
        -1,
        seconds,
        ffmpeg::AVSEEK_FLAG_BACKWARD as i32,
      ) {
        s if s >= 0 => Ok(()),
        e => Err(RumpegError::from_code(
          e,
          &format!("Failed to seek to {seconds} of {}", self.duration),
        )),
      }
    }
  }
}

impl Drop for AVFormatContext {
  fn drop(&mut self) {
    unsafe {
      ffmpeg::avformat_close_input(&mut self.ptr);
    }
  }
}

impl Deref for AVFormatContext {
  type Target = ffmpeg::AVFormatContext;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.ptr }
  }
}

impl DerefMut for AVFormatContext {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.ptr }
  }
}

#[derive(Debug)]
pub struct AVInputFormat {
  ptr: *const ffmpeg::AVInputFormat,
  pub extensions: &'static str,
  pub format_name: &'static str,
  pub mime_type: &'static str,
}

impl AVInputFormat {
  pub fn new(ptr: *const ffmpeg::AVInputFormat) -> Self {
    unsafe {
      Self {
        ptr,
        extensions: ptr_to_str((*ptr).extensions).unwrap_or("N/A"),
        format_name: ptr_to_str((*ptr).long_name).unwrap_or("N/A"),
        mime_type: ptr_to_str((*ptr).mime_type).unwrap_or("N/A"),
      }
    }
  }
}

impl Deref for AVInputFormat {
  type Target = ffmpeg::AVInputFormat;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.ptr }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum SeekPosition {
  Seconds(i64),
  Percentage(f64),
}

impl FromStr for SeekPosition {
  type Err = Box<dyn std::error::Error>;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(if let Some(s) = s.strip_suffix('s') {
      Self::Seconds(s.parse()?)
    } else if let Some(s) = s.strip_suffix('%') {
      Self::Percentage(s.parse::<f64>()? / 100.)
    } else {
      Self::Seconds(s.parse()?)
    })
  }
}

impl Default for SeekPosition {
  fn default() -> Self {
    Self::Seconds(0)
  }
}

fn ptr_to_str(ptr: *const i8) -> Option<&'static str> {
  unsafe {
    (!ptr.is_null())
      .then(|| CStr::from_ptr(ptr).to_str().ok())
      .flatten()
  }
}
