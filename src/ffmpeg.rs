#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

// error handling
pub const fn AVERROR(e: i32) -> i32 {
  if EDOM > 0 {
    -(e)
  } else {
    e
  }
}

pub const fn AVUNERROR(e: i32) -> i32 {
  if EDOM > 0 {
    -(e)
  } else {
    e
  }
}

pub const fn FFERRTAG(a: u8, b: u8, c: u8, d: u8) -> i32 {
  -(MKTAG(a, b, c, d) as i32)
}

pub const fn MKTAG(a: u8, b: u8, c: u8, d: u8) -> u32 {
  (a as u32) | ((b as u32) << 8) | ((c as u32) << 16) | ((d as u32) << 24)
}

pub const AVERROR_BSF_NOT_FOUND: i32 = FFERRTAG(0xF8, b'B', b'S', b'F');
pub const AVERROR_BUG: i32 = FFERRTAG(b'B', b'U', b'G', b'!');
pub const AVERROR_BUFFER_TOO_SMALL: i32 = FFERRTAG(b'B', b'U', b'F', b'S');
pub const AVERROR_DECODER_NOT_FOUND: i32 = FFERRTAG(0xF8, b'D', b'E', b'C');
pub const AVERROR_DEMUXER_NOT_FOUND: i32 = FFERRTAG(0xF8, b'D', b'E', b'M');
pub const AVERROR_ENCODER_NOT_FOUND: i32 = FFERRTAG(0xF8, b'E', b'N', b'C');
pub const AVERROR_EOF: i32 = FFERRTAG(b'E', b'O', b'F', b' ');
pub const AVERROR_EXIT: i32 = FFERRTAG(b'E', b'X', b'I', b'T');
pub const AVERROR_EXTERNAL: i32 = FFERRTAG(b'E', b'X', b'T', b' ');
pub const AVERROR_FILTER_NOT_FOUND: i32 = FFERRTAG(0xF8, b'F', b'I', b'L');
pub const AVERROR_INVALIDDATA: i32 = FFERRTAG(b'I', b'N', b'D', b'A');
pub const AVERROR_MUXER_NOT_FOUND: i32 = FFERRTAG(0xF8, b'M', b'U', b'X');
pub const AVERROR_OPTION_NOT_FOUND: i32 = FFERRTAG(0xF8, b'O', b'P', b'T');
pub const AVERROR_PATCHWELCOME: i32 = FFERRTAG(b'P', b'A', b'W', b'E');
pub const AVERROR_PROTOCOL_NOT_FOUND: i32 = FFERRTAG(0xF8, b'P', b'R', b'O');
pub const AVERROR_STREAM_NOT_FOUND: i32 = FFERRTAG(0xF8, b'S', b'T', b'R');
pub const AVERROR_BUG2: i32 = FFERRTAG(b'B', b'U', b'G', b' ');
pub const AVERROR_UNKNOWN: i32 = FFERRTAG(b'U', b'N', b'K', b'N');
pub const AVERROR_HTTP_BAD_REQUEST: i32 = FFERRTAG(0xF8, b'4', b'0', b'0');
pub const AVERROR_HTTP_UNAUTHORIZED: i32 = FFERRTAG(0xF8, b'4', b'0', b'1');
pub const AVERROR_HTTP_FORBIDDEN: i32 = FFERRTAG(0xF8, b'4', b'0', b'3');
pub const AVERROR_HTTP_NOT_FOUND: i32 = FFERRTAG(0xF8, b'4', b'0', b'4');
pub const AVERROR_HTTP_OTHER_4XX: i32 = FFERRTAG(0xF8, b'4', b'X', b'X');
pub const AVERROR_HTTP_SERVER_ERROR: i32 = FFERRTAG(0xF8, b'5', b'X', b'X');

include!(concat!(env!("OUT_DIR"), "/ffmpeg.rs"));
