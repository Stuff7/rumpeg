mod ffmpeg {
  #![allow(non_upper_case_globals)]
  #![allow(non_camel_case_types)]
  #![allow(non_snake_case)]
  #![allow(dead_code)]
  include!(concat!(env!("OUT_DIR"), "/ffmpeg.rs"));
}

mod math;
mod rumpeg;
mod video;

use std::env;
use video::Video;

macro_rules! unwrap {
  (Some $wrapped: expr, Err $( $err: expr ),*) => {
    match $wrapped {
      Some(v) => v,
      None => {
        eprint!("\x1b[38;2;255;75;75m");
        eprintln!($( $err ),*);
        eprint!("\x1b[0m");
        return;
      }
    }
  };
  (Ok $wrapped: expr, Err $( $err: expr ),*) => {
    match $wrapped {
      Ok(v) => v,
      Err(e) => {
        eprint!("\x1b[38;2;255;75;75m");
        eprint!($( $err ),*);
        eprint!(": {e}\n");
        eprint!("\x1b[0m");
        return;
      }
    }
  };
}

fn main() {
  let args: Vec<String> = env::args().collect();
  let filepath = unwrap!(Some args.get(1), Err "Missing file path").as_str();
  let mut video = unwrap!(Ok Video::open(filepath), Err "Failed to open video");
  println!("VIDEO {:#?}", video);
  // for i in 0..9 {
  //   unwrap!(
  //     Ok video.get_frame(rumpeg::SeekPosition::Seconds(i * 5), format!("temp/image-{i}.webp").as_str()),
  //     Err "Failed to get frame"
  //   );
  // }
  unwrap!(
    Ok video.resize_output(
      args
        .get(2)
        .and_then(|n| n.parse::<i32>().ok())
        .unwrap_or_default(),
      args
        .get(3)
        .and_then(|n| n.parse::<i32>().ok())
        .unwrap_or_default(),
    ),
    Err "Failed to resize image"
  );
  unwrap!(Ok video.get_frame(rumpeg::SeekPosition::Percentage(0.5), "temp/image.webp"), Err "Failed to get frame");
}
