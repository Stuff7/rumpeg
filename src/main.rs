#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
extern crate libc;

use libc::c_char;
use std::env;
use std::ffi::CString;
use std::ptr;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

fn main() {
  let args: Vec<String> = env::args().collect();

  let filename = CString::new(if args.len() > 1 {
    args[1].as_str()
  } else {
    println!("Missing file path");
    return;
  })
  .expect("CString creation failed");

  unsafe {
    let mut format_context = avformat_alloc_context();
    if format_context.is_null() {
      eprintln!("avformat_alloc_context failed");
      return;
    }

    let result = avformat_open_input(
      &mut format_context,
      filename.as_ptr(),
      ptr::null_mut(),
      ptr::null_mut(),
    );

    if result < 0 {
      let mut error_buffer: [c_char; 256] = [0; 256];
      av_strerror(result, error_buffer.as_mut_ptr(), error_buffer.len());
      eprintln!(
        "avformat_open_input failed: {}",
        std::ffi::CStr::from_ptr(error_buffer.as_ptr()).to_string_lossy()
      );
      return;
    }

    let iformat = (*format_context).iformat;
    let duration = (*format_context).duration;
    if !iformat.is_null() {
      let format_name = std::ffi::CStr::from_ptr((*iformat).long_name);
      println!(
        "Format {}, duration {} us",
        format_name.to_str().unwrap_or("N/A"),
        duration
      );
    }

    avformat_close_input(&mut format_context);
  }
}
