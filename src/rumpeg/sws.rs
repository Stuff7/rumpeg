use std::ptr;

use super::*;

use crate::{ffmpeg, math::Matrix3x3};

#[derive(Debug)]
pub struct SWSContext {
  ptr: *mut ffmpeg::SwsContext,
  input: SWSFrameProperties,
  output: SWSFrameProperties,
}

impl SWSContext {
  pub fn resize_output(&mut self, width: i32, height: i32) -> RumpegResult {
    self.output.width = width;
    self.output.height = height;
    self.ptr = Self::get_context_ptr(self.ptr, self.input, &mut self.output)?;
    Ok(())
  }

  pub fn transform(
    &self,
    input: &mut AVFrame,
    transform: Option<Matrix3x3>,
  ) -> RumpegResult<AVFrame> {
    unsafe {
      let output = AVFrame::new(
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

      Ok(if let Some(matrix) = transform {
        output.transform(matrix)?
      } else {
        output
      })
    }
  }

  #[inline]
  fn get_context_ptr(
    ptr: *mut ffmpeg::SwsContext,
    input: SWSFrameProperties,
    output: &mut SWSFrameProperties,
  ) -> RumpegResult<*mut ffmpeg::SwsContext> {
    unsafe {
      output.copy_aspect_ratio(input);
      let ptr = ffmpeg::sws_getCachedContext(
        ptr,
        input.width,
        input.height,
        input.pixel_format,
        output.width,
        output.height,
        output.pixel_format,
        ffmpeg::SWS_SINC as i32,
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
      );

      if ptr.is_null() {
        Err(RumpegError::SWSContextCreation)
      } else {
        Ok(ptr)
      }
    }
  }
}

impl Display for SWSContext {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "- Input:\n\t{}\n- Output:\n\t{}",
      self.input.to_string().replace('\n', "\n\t"),
      self.output.to_string().replace('\n', "\n\t"),
    )
  }
}

impl Drop for SWSContext {
  fn drop(&mut self) {
    unsafe {
      ffmpeg::sws_freeContext(self.ptr);
    }
  }
}

#[derive(Debug)]
pub struct SWSContextBuilder {
  input: SWSFrameProperties,
  output: SWSFrameProperties,
}

impl SWSContextBuilder {
  pub fn from_codecpar(codecpar: AVCodecParameters) -> Self {
    Self {
      input: SWSFrameProperties {
        width: codecpar.width,
        height: codecpar.height,
        pixel_format: codecpar.pixel_format,
      },
      output: SWSFrameProperties {
        width: 0,
        height: 0,
        pixel_format: ffmpeg::AVPixelFormat_AV_PIX_FMT_RGB24,
      },
    }
  }

  pub fn build(&mut self) -> RumpegResult<SWSContext> {
    Ok(SWSContext {
      ptr: SWSContext::get_context_ptr(ptr::null_mut(), self.input, &mut self.output)?,
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
pub struct SWSFrameProperties {
  width: i32,
  height: i32,
  pixel_format: i32,
}

impl SWSFrameProperties {
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

impl Display for SWSFrameProperties {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "- Width: {}\n\
      - Height: {}\n\
      - Format: {}",
      self.width, self.height, self.pixel_format,
    )
  }
}
