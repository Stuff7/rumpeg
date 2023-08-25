use std::{
  array::TryFromSliceError,
  fmt,
  ops::{Deref, Index, IndexMut},
};

use thiserror::Error;

use crate::ffmpeg;

#[derive(Error, Debug)]
pub enum MathError {
  #[error("Failed to parse display matrix\n{0}")]
  DisplayMatrixParsingFail(#[from] TryFromSliceError),
  #[error("Packet side data size invalid {0:?}")]
  InvalidSideDataSize(ffmpeg::AVPacketSideData),
}

type MathResult<T = ()> = Result<T, MathError>;

#[derive(Debug, Clone, Copy)]
pub struct Matrix3x3 {
  data: [i32; 9],
}

impl Matrix3x3 {
  pub fn from_side_data(side_data: ffmpeg::AVPacketSideData) -> MathResult<Self> {
    if side_data.size != 36 {
      return Err(MathError::InvalidSideDataSize(side_data));
    }

    unsafe {
      let mut matrix: [i32; 9] =
        std::slice::from_raw_parts(side_data.data as *const i32, 9).try_into()?;
      // loop 3x3 matrix
      for (i, value) in matrix.iter_mut().enumerate() {
        // | 0 1 2 |
        // | 3 4 5 |
        // | 6 7 8 |
        // All numbers are stored in native endianness, as 16.16 fixed-point values,
        // except for 2, 5 and 8, which are stored as 2.30 fixed-point values.
        *value = to_fixed_point(*value, if i == 2 || i == 5 || i == 8 { 30 } else { 16 });
      }
      Ok(Self { data: matrix })
    }
  }

  /// Extract the rotation component of the transformation matrix and
  /// returns the angle (in degrees) by which the transformation rotates
  /// the frame counterclockwise. The angle will be in range `[-180.0, 180.0]`,
  /// or `None` if the matrix is singular
  ///
  /// # Arguments
  /// * `matrix` - The transformation matrix
  ///
  /// *Note: This is a translated implementation from
  /// [libavutil](https://ffmpeg.org/doxygen/trunk/display_8c_source.html#l00035)*
  pub fn rotation(&self) -> f32 {
    let mut scale = [0_f32; 2];

    scale[0] = f32::hypot(self.data[0] as f32, self.data[3] as f32);
    scale[1] = f32::hypot(self.data[1] as f32, self.data[4] as f32);

    if scale[0] == 0.0 || scale[1] == 0.0 {
      return 0.;
    }

    let rotation = f32::atan2(
      (self.data[1] as f32) / scale[1],
      (self.data[0] as f32) / scale[0],
    ) * 180_f32
      / std::f32::consts::PI;

    -rotation
  }
}

impl Index<(usize, usize)> for Matrix3x3 {
  type Output = i32;

  fn index(&self, index: (usize, usize)) -> &i32 {
    let (i, j) = index;
    &self.data[3 * i + j]
  }
}

impl IndexMut<(usize, usize)> for Matrix3x3 {
  fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
    let (i, j) = index;
    &mut self.data[3 * i + j]
  }
}

impl fmt::Display for Matrix3x3 {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for i in 0..3 {
      write!(f, "|")?;
      for j in 0..3 {
        write!(f, "{:^8}", self[(i, j)])?;
      }
      if i == 2 {
        write!(f, "|")?
      } else {
        writeln!(f, "|")?
      };
    }
    Ok(())
  }
}

impl Deref for Matrix3x3 {
  type Target = [i32; 9];

  fn deref(&self) -> &Self::Target {
    &self.data
  }
}

fn to_fixed_point(x: i32, n: i32) -> i32 {
  ((x as f32) / (1 << n) as f32) as i32
}
