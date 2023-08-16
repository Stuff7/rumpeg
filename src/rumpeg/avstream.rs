use std::{
  ops::{Deref, DerefMut},
  ptr,
};

use crate::ffmpeg;
use crate::math;

use super::RumpegResult;

pub struct AVStream {
  ptr: *mut ffmpeg::AVStream,
  pub index: i32,
}

impl AVStream {
  pub(super) fn new(ptr: *mut ffmpeg::AVStream, index: i32) -> Self {
    Self { ptr, index }
  }

  pub fn display_matrix(&self) -> RumpegResult {
    unsafe {
      let mut current = 0;
      while current < self.nb_side_data {
        let side_data = self.side_data.offset(current as isize);
        current += 1;
        if (*side_data).type_ == ffmpeg::AVPacketSideDataType_AV_PKT_DATA_DISPLAYMATRIX {
          println!("Raw Display Matrix:");
          for i in 0..3 {
            println!(
              "{:>8} {:>8} {:>8}",
              *(*side_data).data.offset(i * 3),
              *(*side_data).data.offset(i * 3 + 1),
              *(*side_data).data.offset(i * 3 + 2)
            );
          }
          let display_matrix =
            math::parse_display_matrix(std::slice::from_raw_parts((*side_data).data, 36))?;
          println!("Parsed Display Matrix:");
          for i in 0..3 {
            println!(
              "{:>8} {:>8} {:>8}",
              display_matrix[i * 3],
              display_matrix[i * 3 + 1],
              display_matrix[i * 3 + 2]
            );
          }
          return Ok(());
        }
      }
      println!("DISPLAY MATRIX NOT FOUND");
      Ok(())
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
