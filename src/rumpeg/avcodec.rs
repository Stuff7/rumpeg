use super::*;
use crate::ffmpeg;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ptr;

#[derive(Debug)]
pub struct AVCodecContext {
  ptr: *mut ffmpeg::AVCodecContext,
  pub format: ffmpeg::AVPixelFormat,
}

impl AVCodecContext {
  pub fn new(codecpar: *mut ffmpeg::AVCodecParameters) -> RumpegResult<Self> {
    unsafe {
      let codec = ffmpeg::avcodec_find_decoder((*codecpar).codec_id);
      if codec.is_null() {
        return Err(RumpegError::DecoderMissing);
      }

      let ptr = ffmpeg::avcodec_alloc_context3(codec);
      if ptr.is_null() {
        return Err(RumpegError::AVCodecContextAllocFail);
      }

      let result = ffmpeg::avcodec_parameters_to_context(ptr, codecpar);
      if result < 0 {
        return Err(RumpegError::from_code(
          result,
          "Could not get AVCodecContext from codec parameters",
        ));
      }

      let result = ffmpeg::avcodec_open2(ptr, codec, ptr::null_mut());
      if result < 0 {
        return Err(RumpegError::from_code(result, "Could not open AVCodec"));
      }

      let format = if (*ptr).pix_fmt == ffmpeg::AVPixelFormat_AV_PIX_FMT_NONE {
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
        (*ptr).pix_fmt
      };

      Ok(Self {
        ptr,
        format: match format {
          // These are deprecated
          ffmpeg::AVPixelFormat_AV_PIX_FMT_YUVJ420P => ffmpeg::AVPixelFormat_AV_PIX_FMT_YUV420P,
          ffmpeg::AVPixelFormat_AV_PIX_FMT_YUVJ422P => ffmpeg::AVPixelFormat_AV_PIX_FMT_YUV422P,
          ffmpeg::AVPixelFormat_AV_PIX_FMT_YUVJ444P => ffmpeg::AVPixelFormat_AV_PIX_FMT_YUV444P,
          ffmpeg::AVPixelFormat_AV_PIX_FMT_YUVJ440P => ffmpeg::AVPixelFormat_AV_PIX_FMT_YUV440P,
          ffmpeg::AVPixelFormat_AV_PIX_FMT_YUVJ411P => ffmpeg::AVPixelFormat_AV_PIX_FMT_YUV411P,
          _ => format,
        },
      })
    }
  }

  /// Clear any buffered packets or frames
  pub fn flush(&self) {
    unsafe {
      ffmpeg::avcodec_flush_buffers(self.ptr);
    }
  }

  pub fn as_ptr(&self) -> *mut ffmpeg::AVCodecContext {
    self.ptr
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
      ffmpeg::avcodec_free_context(&mut self.ptr);
    }
  }
}
