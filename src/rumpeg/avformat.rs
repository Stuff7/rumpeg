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
  pub stream: AVStream,
}

impl AVFormatContext {
  pub fn new(filepath: &str) -> RumpegResult<Self> {
    let filename = CString::new(filepath)?;

    unsafe {
      let mut ptr = ffmpeg::avformat_alloc_context();
      if ptr.is_null() {
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
        return Err(RumpegError::from_code(
          result,
          "Could not open input stream",
        ));
      }

      let iformat = (*ptr).iformat;

      if iformat.is_null() {
        ffmpeg::avformat_close_input(&mut ptr);
        return Err(RumpegError::VideoFormatMissing);
      }

      let stream = AVStream::new(ptr).map_err(|e| {
        ffmpeg::avformat_close_input(&mut ptr);
        e
      })?;

      Ok(Self { ptr, stream })
    }
  }

  pub fn seek(&self, position: SeekPosition) -> RumpegResult {
    unsafe {
      let timestamp = self.stream.as_time_base(position);

      match ffmpeg::avformat_seek_file(
        &mut *self.ptr,
        self.stream.index,
        0,
        timestamp,
        timestamp,
        ffmpeg::AVSEEK_FLAG_BACKWARD as i32,
      ) {
        s if s >= 0 => Ok(()),
        e => Err(RumpegError::from_code(
          e,
          &format!("Failed to seek to {timestamp} of {}", self.stream.duration),
        )),
      }
    }
  }

  pub fn frames(
    &self,
    codec_context: *mut ffmpeg::AVCodecContext,
    start: SeekPosition,
    end: SeekPosition,
    step: SeekPosition,
  ) -> AVFrameIter {
    let step = std::cmp::max(1, self.stream.as_time_base(step));
    AVFrameIter::new(
      self.ptr,
      codec_context,
      self.stream.index,
      self.stream.as_time_base(start),
      self.stream.as_time_base(end),
      step,
      step > self.stream.as_time_base(SeekPosition::Seconds(2)),
    )
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

impl Drop for AVFormatContext {
  fn drop(&mut self) {
    unsafe {
      ffmpeg::avformat_close_input(&mut self.ptr);
    }
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
  Milliseconds(i64),
  Percentage(f64),
  TimeBase(i64),
}

impl FromStr for SeekPosition {
  type Err = Box<dyn std::error::Error>;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(if let Some(s) = s.strip_suffix("ms") {
      Self::Milliseconds(s.parse()?)
    } else if let Some(s) = s.strip_suffix('%') {
      Self::Percentage(s.parse::<f64>()? / 100.)
    } else if let Some(s) = s.strip_suffix("ts") {
      Self::TimeBase(s.parse()?)
    } else if let Some(s) = s.strip_suffix('s') {
      Self::Seconds(s.parse()?)
    } else {
      Self::Seconds(s.parse()?)
    })
  }
}

impl Default for SeekPosition {
  fn default() -> Self {
    Self::TimeBase(0)
  }
}
