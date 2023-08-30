use std::ptr;

use super::*;

use crate::ffmpeg;
use crate::math::Matrix3x3;

#[derive(Debug)]
pub struct SwsContext {
  ptr: *mut ffmpeg::SwsContext,
  input: SwsFrameProperties,
  output: SwsFrameProperties,
}

impl SwsContext {
  pub fn new(input: SwsFrameProperties, out_w: i32, out_h: i32) -> RumpegResult<Self> {
    let output = input.output(out_w, out_h);
    Ok(Self {
      input,
      output,
      ptr: Self::get_context_ptr(input, output)?,
    })
  }

  pub fn width(&self) -> i32 {
    self.output.width
  }

  pub fn height(&self) -> i32 {
    self.output.height
  }

  pub fn transform(
    &self,
    input: &mut AVFrame,
    transform: Option<Matrix3x3>,
  ) -> RumpegResult<AVFrame> {
    unsafe {
      let mut output = AVFrame::new(self.output.format, self.output.width, self.output.height)?;

      ffmpeg::sws_scale(
        self.ptr,
        input.data.as_ptr() as *const *const _,
        input.linesize.as_ptr() as *const _,
        0,
        self.input.height,
        output.data.as_ptr() as *const *mut _,
        output.linesize.as_ptr() as *mut _,
      );

      if let Some(matrix) = transform {
        output.transform(matrix)?
      }

      Ok(output)
    }
  }

  #[inline]
  fn get_context_ptr(
    input: SwsFrameProperties,
    output: SwsFrameProperties,
  ) -> RumpegResult<*mut ffmpeg::SwsContext> {
    unsafe {
      let mut flags = ffmpeg::SWS_SINC as i32;

      // workaround for "right band" issue
      // https://ffmpeg.org/pipermail/libav-user/2012-July/002451.html
      if (input.width & 0x7 != 0) || (input.height & 0x7 != 0) {
        flags |= ffmpeg::SWS_ACCURATE_RND as i32
      }

      let ptr = ffmpeg::sws_getContext(
        input.width,
        input.height,
        input.format,
        output.width,
        output.height,
        output.format,
        flags,
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
      );

      if ptr.is_null() {
        Err(RumpegError::SwsContextCreation)
      } else {
        Ok(ptr)
      }
    }
  }
}

impl std::ops::Deref for SwsContext {
  type Target = ffmpeg::SwsContext;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.ptr }
  }
}

impl std::ops::DerefMut for SwsContext {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.ptr }
  }
}

impl Drop for SwsContext {
  fn drop(&mut self) {
    unsafe {
      ffmpeg::sws_freeContext(self.ptr);
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct SwsFrameProperties {
  pub width: i32,
  pub height: i32,
  pub format: i32,
}

impl SwsFrameProperties {
  pub fn output(&self, width: i32, height: i32) -> Self {
    let mut output = Self {
      width,
      height,
      format: ffmpeg::AVPixelFormat_AV_PIX_FMT_YUV420P,
    };
    output.copy_aspect_ratio(*self);
    output
  }

  fn copy_aspect_ratio(&mut self, other: Self) {
    if self.width < 1 {
      self.width = if self.height > 0 {
        self.height * other.width / other.height
      } else {
        other.width
      };
    }
    if self.height < 1 {
      self.height = if self.width > 0 {
        self.width * other.height / other.width
      } else {
        other.height
      };
    }
  }
}

impl From<&AVFrame> for SwsFrameProperties {
  fn from(frame: &AVFrame) -> Self {
    Self {
      width: frame.width,
      height: frame.height,
      format: frame.format,
    }
  }
}

impl From<&AVCodecContext> for SwsFrameProperties {
  fn from(codec_context: &AVCodecContext) -> Self {
    Self {
      width: codec_context.width,
      height: codec_context.height,
      format: codec_context.format,
    }
  }
}
