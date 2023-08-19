use super::{RumpegError, RumpegResult};
use crate::ffmpeg;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ptr;

#[derive(Debug)]
pub struct AVCodecContext {
  ptr: *mut ffmpeg::AVCodecContext,
}

impl AVCodecContext {
  pub fn new(codecpar: &AVCodecParameters) -> RumpegResult<Self> {
    unsafe {
      let codec = ffmpeg::avcodec_find_decoder(codecpar.codec_id);
      if codec.is_null() {
        return Err(RumpegError::DecoderMissing);
      }

      let ptr = ffmpeg::avcodec_alloc_context3(codec);
      if ptr.is_null() {
        return Err(RumpegError::AVCodecContextAllocFail);
      }

      let result = ffmpeg::avcodec_parameters_to_context(ptr, codecpar.ptr);
      if result < 0 {
        return Err(RumpegError::from_code(
          result,
          "avcodec_parameters_to_context failed",
        ));
      }

      let result = ffmpeg::avcodec_open2(ptr, codec, ptr::null_mut());
      if result < 0 {
        return Err(RumpegError::from_code(result, "avcodec_open2 failed"));
      }

      Ok(Self { ptr })
    }
  }

  /// Clear any buffered packets or frames
  pub fn flush(&self) {
    unsafe {
      ffmpeg::avcodec_flush_buffers(self.ptr);
    }
  }
}

impl Deref for AVCodecContext {
  type Target = ffmpeg::AVCodecContext;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.ptr }
  }
}

impl DerefMut for AVCodecContext {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.ptr }
  }
}

impl Drop for AVCodecContext {
  fn drop(&mut self) {
    unsafe {
      ffmpeg::avcodec_close(self.ptr);
    }
  }
}

#[derive(Debug)]
pub struct AVCodecParameters {
  ptr: *mut ffmpeg::AVCodecParameters,
  pub pixel_format: i32,
}

impl AVCodecParameters {
  pub fn new(ptr: *mut ffmpeg::AVCodecParameters) -> RumpegResult<Self> {
    unsafe {
      let pixel_format = if (*ptr).format == ffmpeg::AVPixelFormat_AV_PIX_FMT_NONE {
        match (*ptr).codec_id {
          ffmpeg::AVCodecID_AV_CODEC_ID_H264
          | ffmpeg::AVCodecID_AV_CODEC_ID_HEVC
          | ffmpeg::AVCodecID_AV_CODEC_ID_MPEG2VIDEO
          | ffmpeg::AVCodecID_AV_CODEC_ID_VP9
          | ffmpeg::AVCodecID_AV_CODEC_ID_AV1
          | ffmpeg::AVCodecID_AV_CODEC_ID_VP8 => ffmpeg::AVPixelFormat_AV_PIX_FMT_YUV420P,
          id => return Err(RumpegError::PixelFormatMissing(id)),
        }
      } else {
        (*ptr).format
      };
      Ok(Self { ptr, pixel_format })
    }
  }
}

impl Deref for AVCodecParameters {
  type Target = ffmpeg::AVCodecParameters;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.ptr }
  }
}

impl DerefMut for AVCodecParameters {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.ptr }
  }
}
