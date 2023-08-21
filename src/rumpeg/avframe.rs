use super::*;
use crate::ascii::Color;
use crate::ffmpeg;
use crate::log;
use crate::math;
use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::slice;
use webp::{Encoder, WebPMemory};

const PX_BYTES: usize = 3;

#[derive(Debug)]
pub struct AVFrame {
  ptr: *mut ffmpeg::AVFrame,
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
      Ok(Self { ptr })
    }
  }

  pub fn new(format: i32, width: i32, height: i32) -> RumpegResult<Self> {
    let mut frame = Self::empty()?;
    frame.format = format;
    frame.width = width;
    frame.height = height;
    let code = unsafe { ffmpeg::av_frame_get_buffer(frame.ptr, 1) };
    if code < 0 {
      return Err(RumpegError::from_code(
        code,
        &format!("Failed to allocate frame {frame}"),
      ));
    }

    Ok(frame)
  }

  /// Applies transformations to frame using `transform` matrix, and returns the transformed frame
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

    let rotation = transform.rotation();
    let (dst_width, dst_height) = if rotation.abs() == 90. {
      (self.height, self.width)
    } else {
      (self.width, self.height)
    };
    let mut dest = Self::new(self.format, dst_width, dst_height)?;
    let dst_data = dest.data_mut();

    let [a, b, u, c, d, v, x, y, w] = *transform;
    let x = if x != 0 { dst_width - 1 } else { x };
    let y = if y != 0 { dst_height - 1 } else { y };

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
    Encoder::from_rgb(self.data(), self.width as u32, self.height as u32).encode(50.)
  }

  pub fn data(&self) -> &[u8] {
    unsafe {
      slice::from_raw_parts(
        self.data[0],
        self.linesize[0] as usize * self.height as usize,
      )
    }
  }

  pub fn data_mut(&mut self) -> &mut [u8] {
    unsafe {
      slice::from_raw_parts_mut(
        self.data[0],
        self.linesize[0] as usize * self.height as usize,
      )
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

impl Drop for AVFrame {
  fn drop(&mut self) {
    unsafe {
      ffmpeg::av_frame_free(&mut self.ptr);
    }
  }
}

#[derive(Debug)]
pub struct AVFrameIter {
  format_context: *mut ffmpeg::AVFormatContext,
  codec_context: *mut ffmpeg::AVCodecContext,
  stream_index: i32,
  duration: i64,
  step: i64,
  next_timestamp: i64,
}

impl AVFrameIter {
  pub fn new(
    format_context: *mut ffmpeg::AVFormatContext,
    codec_context: *mut ffmpeg::AVCodecContext,
    stream_index: i32,
    duration: i64,
    step: i64,
  ) -> Self {
    Self {
      format_context,
      codec_context,
      stream_index,
      duration,
      step,
      next_timestamp: 0,
    }
  }
}

impl Iterator for AVFrameIter {
  type Item = AVFrame;

  fn next(&mut self) -> Option<<Self as Iterator>::Item> {
    let Ok(mut frame) = AVFrame::empty() else {return None};
    let mut packet = AVPacket::empty();

    loop {
      if self.next_timestamp >= self.duration {
        return None;
      }
      match packet.read(self.format_context) {
        Ok(..) => unsafe {
          if packet.stream_index == self.stream_index {
            ffmpeg::avcodec_send_packet(self.codec_context, &*packet);
            let result = ffmpeg::avcodec_receive_frame(self.codec_context, &mut *frame);
            if result == 0 {
              self.next_timestamp = frame.pts + self.step;
              ffmpeg::avcodec_flush_buffers(self.codec_context);
              ffmpeg::avformat_seek_file(
                self.format_context,
                self.stream_index,
                0,
                self.next_timestamp,
                self.next_timestamp,
                ffmpeg::AVSEEK_FLAG_BACKWARD as i32,
              );
              println!("PTS {} NTS {}", frame.pts, self.next_timestamp);
              return Some(frame);
            }
            if result != ffmpeg::AVERROR(ffmpeg::EAGAIN as i32) {
              log!(err@
                "{}",
                RumpegError::from_code(result, "Encountered AVError while receiving frame")
              );
              return None;
            }
          }
        },
        Err(e) => {
          if let RumpegError::AVError(_, code, _) = e {
            if code == ffmpeg::AVERROR_EOF {
              return None;
            }
          }
          log!(err@"Encountered AVError while reading frame {e}");
        }
      }
    }
  }
}
