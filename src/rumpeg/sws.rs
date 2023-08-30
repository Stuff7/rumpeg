use std::ptr;

use super::*;

use crate::ascii::Color;
use crate::ascii::RESET;
use crate::ffmpeg;
use crate::math::Matrix3x3;

#[derive(Debug)]
pub struct SwsContext {
  ptr: *mut ffmpeg::SwsContext,
  input: SwsFrameProperties,
  output: SwsFrameProperties,
}

impl SwsContext {
  pub fn resize_output(&mut self, width: i32, height: i32) -> RumpegResult {
    self.output.width = width;
    self.output.height = height;
    self.output.copy_aspect_ratio(self.input);
    self.ptr = Self::get_context_ptr(self.ptr, self.input, self.output)?;
    Ok(())
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
      let mut output = AVFrame::new(
        self.output.pixel_format,
        self.output.width,
        self.output.height,
      )?;

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
    ptr: *mut ffmpeg::SwsContext,
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

      let ptr = ffmpeg::sws_getCachedContext(
        ptr,
        input.width,
        input.height,
        input.pixel_format,
        output.width,
        output.height,
        output.pixel_format,
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

impl Display for SwsContext {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "- {title}Input:{RESET}\n  {}\n- {title}Output:{RESET}\n  {}",
      self.input.to_string().replace('\n', "\n  "),
      self.output.to_string().replace('\n', "\n  "),
      title = "".rgb(75, 200, 200),
    )
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

#[derive(Debug)]
pub struct SwsContextBuilder {
  input: SwsFrameProperties,
  output: SwsFrameProperties,
}

impl SwsContextBuilder {
  pub fn from_codec_context(codec_context: &AVCodecContext) -> Self {
    Self {
      input: SwsFrameProperties {
        width: codec_context.width,
        height: codec_context.height,
        pixel_format: codec_context.format,
      },
      output: SwsFrameProperties {
        width: 0,
        height: 0,
        pixel_format: ffmpeg::AVPixelFormat_AV_PIX_FMT_YUV420P,
      },
    }
  }

  pub fn build(&mut self) -> RumpegResult<SwsContext> {
    self.output.copy_aspect_ratio(self.input);
    Ok(SwsContext {
      ptr: SwsContext::get_context_ptr(ptr::null_mut(), self.input, self.output)?,
      input: self.input,
      output: self.output,
    })
  }

  pub fn width(&mut self, w: i32) -> &mut Self {
    self.output.width = w;
    self
  }

  pub fn height(&mut self, h: i32) -> &mut Self {
    self.output.height = h;
    self
  }

  pub fn pixel_format(&mut self, f: i32) -> &mut Self {
    self.output.pixel_format = f;
    self
  }
}

#[derive(Debug, Clone, Copy)]
pub struct SwsFrameProperties {
  width: i32,
  height: i32,
  pixel_format: i32,
}

impl SwsFrameProperties {
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

impl Display for SwsFrameProperties {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "- {title}Width:{RESET} {}\n\
      - {title}Height:{RESET} {}\n\
      - {title}Format:{RESET} {}",
      self.width,
      self.height,
      self.pixel_format,
      title = "".rgb(75, 200, 200),
    )
  }
}
