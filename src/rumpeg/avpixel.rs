use super::*;
use crate::ffmpeg;

pub trait AVPixelFormatMethods: Into<i32> + Copy {
  fn av_pix_fmt_descriptor(&self) -> Option<ffmpeg::AVPixFmtDescriptor> {
    unsafe {
      ffmpeg::av_pix_fmt_desc_get((*self).into())
        .as_ref()
        .copied()
    }
  }

  fn av_pix_fmt_name<'a>(&self) -> &'a str {
    unsafe {
      ffmpeg::av_get_pix_fmt_name((*self).into())
        .as_ref()
        .and_then(|ptr| ptr_to_str(ptr))
        .unwrap_or("N/A")
    }
  }
}

impl AVPixelFormatMethods for ffmpeg::AVPixelFormat {}
