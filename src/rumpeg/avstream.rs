use std::ops::{Deref, DerefMut};

use super::*;
use crate::ascii::LogDisplay;
use crate::{ffmpeg, log, math::Matrix3x3};

#[derive(Debug)]
pub struct AVStream {
  ptr: *mut ffmpeg::AVStream,
  pub index: i32,
}

impl AVStream {
  pub(super) fn new(format_context: *mut ffmpeg::AVFormatContext) -> RumpegResult<Self> {
    unsafe {
      let result = ffmpeg::avformat_find_stream_info(format_context, std::ptr::null_mut());
      if result < 0 {
        return Err(RumpegError::from_code(result, "Could not find stream info"));
      }

      let index = ffmpeg::av_find_best_stream(
        format_context,
        ffmpeg::AVMediaType_AVMEDIA_TYPE_VIDEO,
        -1,
        -1,
        std::ptr::null_mut(),
        0,
      );
      if index < 0 {
        return Err(RumpegError::from_code(index, "No video stream found"));
      }

      Ok(Self {
        ptr: *(*format_context).streams.offset(index as isize),
        index,
      })
    }
  }

  pub fn as_time_base(&self, position: SeekPosition) -> i64 {
    unsafe {
      match position {
        SeekPosition::Seconds(n) => {
          ffmpeg::av_rescale_q(n, ffmpeg::AVRational { den: 1, num: 1 }, self.time_base)
        }
        SeekPosition::Milliseconds(n) => {
          ffmpeg::av_rescale_q(n, ffmpeg::AVRational { num: 1, den: 1000 }, self.time_base)
        }
        SeekPosition::Percentage(n) => {
          let n = self.duration as f64 * n;
          (n - n % (self.time_base.den as f64 / self.r_frame_rate.num as f64)) as i64
        }
        SeekPosition::TimeBase(n) => n,
      }
    }
  }

  pub fn display_matrix(&self) -> Option<Matrix3x3> {
    unsafe {
      let mut current = 0;
      while current < self.nb_side_data {
        let side_data = *self.side_data.offset(current as isize);
        current += 1;
        if side_data.type_ == ffmpeg::AVPacketSideDataType_AV_PKT_DATA_DISPLAYMATRIX {
          return match Matrix3x3::from_side_data(side_data) {
            Ok(display_matrix) => Some(display_matrix),
            Err(e) => {
              log!(err@"Found display matrix but failed to parse it\n{e}");
              None
            }
          };
        }
      }
      None
    }
  }
}

impl Deref for AVStream {
  type Target = ffmpeg::AVStream;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.ptr }
  }
}

impl DerefMut for AVStream {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.ptr }
  }
}
