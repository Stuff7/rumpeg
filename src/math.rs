use std::{array::TryFromSliceError, ops::Range};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MathError {
  #[error("Failed to parse display matrix at {0:?}\n\nErr:{1:?}")]
  DisplayMatrixParsingFail(Range<usize>, TryFromSliceError),
}

type MathResult<T = ()> = Result<T, MathError>;

/// Converts display matrix bytes into 3x3 integer matrix `[u8; 36]` => `[i32; 9]`
/// # Arguments
/// * `bytes` - Display matrix side data
pub fn parse_display_matrix(bytes: &[u8]) -> MathResult<[i32; 9]> {
  let mut matrix = [0; 9];
  // loop 3x3 matrix
  for i in 0..9 {
    let chunk_range = (i * 4)..(i * 4 + 4);
    // Split bytes slice &[u8] into array chunk [u8; 4]
    let conversion: Result<[u8; 4], std::array::TryFromSliceError> = bytes[chunk_range].try_into();

    match conversion {
      Ok(chunk) => {
        // | 0 1 2 |
        // | 3 4 5 |
        // | 6 7 8 |
        // All numbers are stored in native endianness, as 16.16 fixed-point values,
        // except for 2, 5 and 8, which are stored as 2.30 fixed-point values.
        let value = i32::from_ne_bytes(chunk);
        matrix[i] = if i == 2 || i == 5 || i == 8 {
          to_fixed_point(value, 30)
        } else {
          to_fixed_point(value, 16)
        }
      }
      Err(e) => return Err(MathError::DisplayMatrixParsingFail((i * 4)..(i * 4 + 4), e)),
    }
  }
  Ok(matrix)
}

fn to_fixed_point(x: i32, n: i32) -> i32 {
  ((x as f32) / (1 << n) as f32) as i32
}
