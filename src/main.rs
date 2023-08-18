mod ffmpeg {
  #![allow(non_upper_case_globals)]
  #![allow(non_camel_case_types)]
  #![allow(non_snake_case)]
  #![allow(dead_code)]
  include!(concat!(env!("OUT_DIR"), "/ffmpeg.rs"));
}

mod cli;
mod math;
mod rumpeg;
mod video;

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
  let args = unwrap!(Ok cli::CLIArgs::read(), Err "Error");
  let mut video = unwrap!(Ok Video::open(&args.filepath), Err "Failed to open video");
  unwrap!(Ok video.resize_output(args.width, args.height), Err "Failed to resize image");
  // for i in 0..9 {
  //   unwrap!(
  //     Ok video.get_frame(rumpeg::SeekPosition::Seconds(i * 5), format!("temp/a-image-{i}.webp").as_str()),
  //     Err "Failed to get frame"
  //   );
  // }
  if args.debug {
    println!("{}", video);
  }
  unwrap!(Ok video.get_frame(args.seek_position, "temp/image.webp"), Err "Failed to get frame");
}
