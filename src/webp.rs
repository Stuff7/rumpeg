mod libwebp {
  #![allow(non_upper_case_globals)]
  #![allow(non_camel_case_types)]
  #![allow(non_snake_case)]
  #![allow(dead_code)]

  include!(concat!(env!("OUT_DIR"), "/libwebp.rs"));

  /// Should always be called, to initialize a fresh WebPConfig structure before
  /// modification. Returns false in case of version mismatch. WebPConfigInit()
  /// must have succeeded before using the 'config' object.
  /// Note that the default values are lossless=0 and quality=75.
  #[inline]
  pub unsafe fn WebPConfigInit(config: *mut WebPConfig) -> i32 {
    WebPConfigInitInternal(
      config,
      WebPPreset_WEBP_PRESET_DEFAULT,
      75.,
      WEBP_ENCODER_ABI_VERSION as i32,
    )
  }

  /// This function will initialize the configuration according to a predefined
  /// set of parameters (referred to by 'preset') and a given quality factor.
  /// This function can be called as a replacement to WebPConfigInit(). Will
  /// return false in case of error.
  #[inline]
  pub unsafe fn WebPConfigPreset(config: *mut WebPConfig, preset: WebPPreset, quality: f32) -> i32 {
    WebPConfigInitInternal(config, preset, quality, WEBP_ENCODER_ABI_VERSION as i32)
  }

  /// Should always be called, to initialize the structure. Returns false in case
  /// of version mismatch. WebPPictureInit() must have succeeded before using the
  /// 'picture' object.
  /// Note that, by default, use_argb is false and colorspace is WEBP_YUV420.
  #[inline]
  pub unsafe fn WebPPictureInit(picture: *mut WebPPicture) -> i32 {
    WebPPictureInitInternal(picture, WEBP_ENCODER_ABI_VERSION as i32)
  }

  pub fn webp_error<'a>(error_code: i32) -> &'a str {
    match error_code {
      WebPEncodingError_VP8_ENC_OK => "Everything ok [VP8_ENC_OK]",
      WebPEncodingError_VP8_ENC_ERROR_OUT_OF_MEMORY => {
        "Memory error allocating objects [VP8_ENC_ERROR_OUT_OF_MEMORY]"
      }
      WebPEncodingError_VP8_ENC_ERROR_BITSTREAM_OUT_OF_MEMORY => {
        "Memory error while flushing bits [VP8_ENC_ERROR_BITSTREAM_OUT_OF_MEMORY]"
      }
      WebPEncodingError_VP8_ENC_ERROR_NULL_PARAMETER => {
        "A pointer parameter is NULL [VP8_ENC_ERROR_NULL_PARAMETER]"
      }
      WebPEncodingError_VP8_ENC_ERROR_INVALID_CONFIGURATION => {
        "Configuration is invalid [VP8_ENC_ERROR_INVALID_CONFIGURATION]"
      }
      WebPEncodingError_VP8_ENC_ERROR_BAD_DIMENSION => {
        "Picture has invalid width/height [VP8_ENC_ERROR_BAD_DIMENSION]"
      }
      WebPEncodingError_VP8_ENC_ERROR_PARTITION0_OVERFLOW => {
        "Partition is bigger than 512k [VP8_ENC_ERROR_PARTITION0_OVERFLOW]"
      }
      WebPEncodingError_VP8_ENC_ERROR_PARTITION_OVERFLOW => {
        "Partition is bigger than 16M [VP8_ENC_ERROR_PARTITION_OVERFLOW]"
      }
      WebPEncodingError_VP8_ENC_ERROR_BAD_WRITE => {
        "Error while flushing bytes [VP8_ENC_ERROR_BAD_WRITE]"
      }
      WebPEncodingError_VP8_ENC_ERROR_FILE_TOO_BIG => {
        "File is bigger than 4G [VP8_ENC_ERROR_FILE_TOO_BIG]"
      }
      WebPEncodingError_VP8_ENC_ERROR_USER_ABORT => {
        "Abort request by user [VP8_ENC_ERROR_USER_ABORT]"
      }
      WebPEncodingError_VP8_ENC_ERROR_LAST => "List terminator. always last. [VP8_ENC_ERROR_LAST]",
      _ => "Unknown error code",
    }
  }
}

use crate::ffmpeg;
use crate::rumpeg::AVFrame;
use crate::rumpeg::AVPixelFormatMethods;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WebPError {
  #[error("Webp encoding failed (Code {0}): {1}")]
  Encoding(i32, String),
  #[error("Webp config initialization failed")]
  WebPConfigInit,
  #[error("Format \"{0}\" is not supported")]
  FormatNotSupported(&'static str),
}

impl WebPError {
  pub fn from_code(error_code: i32) -> Self {
    Self::Encoding(error_code, libwebp::webp_error(error_code).into())
  }

  pub fn from_format(format: ffmpeg::AVPixelFormat) -> Self {
    Self::FormatNotSupported(format.av_pix_fmt_name())
  }
}

type WebPResult<T = ()> = Result<T, WebPError>;

pub struct WebPEncoder {
  pic: libwebp::WebPPicture,
  config: libwebp::WebPConfig,
}

impl WebPEncoder {
  pub fn new(frame: &AVFrame, quality: f32) -> WebPResult<Self> {
    unsafe {
      let mut config = libwebp::WebPConfig::default();
      if libwebp::WebPConfigInit(&mut config) == 0 {
        return Err(WebPError::WebPConfigInit);
      }

      config.quality = quality;

      let mut pic = libwebp::WebPPicture::default();
      if libwebp::WebPPictureInit(&mut pic) == 0 {
        return Err(WebPError::from_code(pic.error_code));
      }

      pic.width = frame.width;
      pic.height = frame.height;

      match frame.format {
        ffmpeg::AVPixelFormat_AV_PIX_FMT_YUV420P => {
          if libwebp::WebPPictureAlloc(&mut pic) == 0 {
            libwebp::WebPPictureFree(&mut pic);
            return Err(WebPError::from_code(pic.error_code));
          }
          pic.y = frame.data[0];
          pic.y_stride = frame.linesize[0];
          pic.u = frame.data[1];
          pic.v = frame.data[2];
          pic.uv_stride = frame.linesize[1];
        }
        ffmpeg::AVPixelFormat_AV_PIX_FMT_RGB24
          if libwebp::WebPPictureImportRGB(&mut pic, frame.data[0], frame.linesize[0]) == 0 =>
        {
          libwebp::WebPPictureFree(&mut pic);
          return Err(WebPError::from_code(pic.error_code));
        }
        format => return Err(WebPError::from_format(format)),
      }

      Ok(Self { pic, config })
    }
  }

  pub fn encode<'a>(&mut self) -> WebPResult<&'a [u8]> {
    unsafe {
      let mut writer = libwebp::WebPMemoryWriter::default();
      libwebp::WebPMemoryWriterInit(&mut writer);
      self.pic.writer = Some(libwebp::WebPMemoryWrite);
      self.pic.custom_ptr = &mut writer as *mut _ as *mut std::ffi::c_void;
      let encode_result = libwebp::WebPEncode(&self.config, &mut self.pic);

      if encode_result == 0 {
        return Err(WebPError::from_code(self.pic.error_code));
      }

      Ok(std::slice::from_raw_parts(writer.mem, writer.size))
    }
  }
}

impl Drop for WebPEncoder {
  fn drop(&mut self) {
    unsafe {
      libwebp::WebPPictureFree(&mut self.pic);
    }
  }
}

pub fn version() -> String {
  let version = unsafe { libwebp::WebPGetEncoderVersion() };
  let major = ((version >> 16) & 0xFF) as u8;
  let minor = ((version >> 8) & 0xFF) as u8;
  let revision = (version & 0xFF) as u8;
  format!("{}.{}.{}", major, minor, revision)
}
