use std::ops::{Deref, DerefMut};

use crate::{ffmpeg, math::Matrix3x3};

pub struct AVStream {
  ptr: *mut ffmpeg::AVStream,
  pub index: i32,
}

impl AVStream {
  pub(super) fn new(ptr: *mut ffmpeg::AVStream, index: i32) -> Self {
    Self { ptr, index }
  }

  pub fn display_matrix(&self) -> Option<Matrix3x3> {
    unsafe {
      let mut current = 0;
      while current < self.nb_side_data {
        let side_data = *self.side_data.offset(current as isize);
        current += 1;
        if side_data.type_ == ffmpeg::AVPacketSideDataType_AV_PKT_DATA_DISPLAYMATRIX {
          // println!(
          //   "AV ROTATION: {}",
          //   ffmpeg::av_display_rotation_get(side_data.data as *const _) as i64
          // );
          // ffmpeg::av_display_rotation_set(side_data.data as *mut _, 0.);
          // println!(
          //   "AV ROTATION: {}",
          //   ffmpeg::av_display_rotation_get(side_data.data as *const _) as i64
          // );
          return match Matrix3x3::from_side_data(side_data) {
            Ok(display_matrix) => Some(display_matrix),
            Err(e) => {
              eprintln!("Found display matrix but failed to parse it {e}");
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
