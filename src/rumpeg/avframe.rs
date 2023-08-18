use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::slice;

use webp::{Encoder, WebPMemory};

use super::RumpegError;
use super::RumpegResult;

use crate::ffmpeg;
use crate::math;

const PX_BYTES: usize = 3;

#[derive(Debug)]
pub struct AVFrame {
  ptr: *mut ffmpeg::AVFrame,
  image_data: ImageBuffer,
}

impl Display for AVFrame {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "AVFrame {{ format: {}, width: {}, height: {} }}",
      self.format, self.width, self.height
    )
  }
}

impl AVFrame {
  pub fn empty() -> RumpegResult<Self> {
    unsafe {
      let ptr = ffmpeg::av_frame_alloc();
      if ptr.is_null() {
        return Err(RumpegError::AVFrameCreation);
      }
      Ok(Self {
        ptr,
        image_data: ImageBuffer::empty(),
      })
    }
  }

  pub fn new(format: i32, width: i32, height: i32) -> RumpegResult<Self> {
    unsafe {
      let mut frame = Self::empty()?;
      frame.format = format;
      frame.width = width;
      frame.height = height;

      let code = ffmpeg::av_frame_get_buffer(frame.ptr, 32);
      if code < 0 {
        return Err(RumpegError::from_code(
          code,
          format!("Failed to allocate frame {frame}"),
        ));
      }

      frame.image_data = ImageBuffer::new(format, width, height)?;

      ffmpeg::av_image_fill_arrays(
        frame.data.as_mut_ptr() as *mut *mut u8,
        frame.linesize.as_mut_ptr(),
        *frame.image_data,
        frame.format,
        frame.width,
        frame.height,
        1,
      );

      Ok(frame)
    }
  }

  /// Rotates `src_frame` using `transform` matrix and stores it in `dst_frame`
  ///
  /// The transformation maps a point `(p, q)` in the source (pre-transformation) frame
  /// to the point `(p', q')` in the destination (post-transformation) frame as follows:
  /// ```
  /// //             | a b u |
  /// // (p, q, 1) . | c d v | = z * (p', q', 1)
  /// //             | x y w |
  /// ```
  /// The transformation can also be more explicitly written in components as follows:
  /// ```ignore
  /// let dp = (a * p + c * q + x) / z;
  /// let dq = (b * p + d * q + y) / z;
  /// let z  =  u * p + v * q + w;
  /// ```
  ///
  /// *Reference: [ffmpeg docs](https://ffmpeg.org/doxygen/trunk/group__lavu__video__display.html)*
  pub fn transform(&self, transform: math::Matrix3x3) -> RumpegResult<Self> {
    let src_width = self.width as usize;
    let src_data = self.data();

    let dst_width = self.height;
    let dst_height = self.width;
    let mut dest = Self::new(self.format, dst_width, dst_height)?;
    let dst_data = dest.data_mut();

    let [a, b, u, c, d, v, x, y, w] = *transform;

    for i in 0..src_data.len() {
      let (p, q) = ((i % src_width) as i32, (i / src_width) as i32);

      let z = u * p + v * q + w;
      let dp = (a * p + c * q + x) / z;
      let dq = (b * p + d * q + y) / z;
      let di = (dp + dst_width * dq) * PX_BYTES as i32;

      if di < 0 {
        continue;
      }

      let di = di as usize;
      if di < dst_data.len() {
        for color_idx in 0..PX_BYTES {
          if i * PX_BYTES + color_idx < src_data.len() {
            dst_data[di + color_idx] = src_data[i * PX_BYTES + color_idx];
          }
        }
      }
    }

    Ok(dest)
  }

  pub fn encode_as_webp(&self) -> WebPMemory {
    Encoder::from_rgb(self.image_data(), self.width as u32, self.height as u32).encode(50.)
  }

  pub fn image_data(&self) -> &[u8] {
    unsafe {
      slice::from_raw_parts(
        *self.image_data,
        (*self.ptr).linesize[0] as usize * (*self.ptr).height as usize,
      )
    }
  }

  pub fn data(&self) -> &[u8] {
    unsafe {
      slice::from_raw_parts(
        (*self.ptr).data[0],
        (*self.ptr).linesize[0] as usize * (*self.ptr).height as usize,
      )
    }
  }

  pub fn data_mut(&mut self) -> &mut [u8] {
    unsafe {
      slice::from_raw_parts_mut(
        (*self.ptr).data[0],
        (*self.ptr).linesize[0] as usize * (*self.ptr).height as usize,
      )
    }
  }
}

impl Drop for AVFrame {
  fn drop(&mut self) {
    unsafe {
      ffmpeg::av_frame_free(&mut self.ptr);
    }
  }
}

impl Deref for AVFrame {
  type Target = ffmpeg::AVFrame;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.ptr }
  }
}

impl DerefMut for AVFrame {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.ptr }
  }
}

#[derive(Debug)]
pub struct ImageBuffer {
  ptr: *mut u8,
}

impl ImageBuffer {
  pub fn empty() -> Self {
    Self {
      ptr: std::ptr::null_mut(),
    }
  }
  pub fn new(pixel_format: ffmpeg::AVPixelFormat, width: i32, height: i32) -> RumpegResult<Self> {
    unsafe {
      let rgb_size = ffmpeg::av_image_get_buffer_size(pixel_format, width, height, 1);
      if rgb_size < 0 {
        return Err(RumpegError::from_code(
          rgb_size,
          "Failed to allocated ImageBuffer",
        ));
      }
      let ptr = libc::malloc(rgb_size as usize) as *mut u8;
      Ok(Self { ptr })
    }
  }
}

impl Drop for ImageBuffer {
  fn drop(&mut self) {
    unsafe {
      libc::free(self.ptr as *mut libc::c_void);
    }
  }
}

impl Deref for ImageBuffer {
  type Target = *mut u8;

  fn deref(&self) -> &Self::Target {
    &self.ptr
  }
}

impl DerefMut for ImageBuffer {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.ptr
  }
}
