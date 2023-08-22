mod ascii;
mod cli;
mod ffmpeg;
mod math;
mod rumpeg;
mod video;

use ascii::Color;
use std::time::Instant;
use video::Video;

macro_rules! unwrap {
  (Some $wrapped: expr, Err $( $err: expr ),*) => {
    match $wrapped {
      Some(v) => v,
      None => {
        log!(ln err@$( $err ),*);
        println!();
        return;
      }
    }
  };
  (Ok $wrapped: expr, Err $( $err: expr ),*) => {
    match $wrapped {
      Ok(v) => v,
      Err(e) => {
        log!(err@$( $err ),*);
        log!(": {e}\n");
        return;
      }
    }
  };
}

fn main() {
  log!(success@"FFMPEG VERSION: {}\n", rumpeg::version());
  let args = unwrap!(Ok cli::CLIArgs::read(), Err "Error");
  rumpeg::set_log_level(args.log_level);
  let mut video = unwrap!(Ok Video::open(&args.filepath), Err "Failed to open video");
  unwrap!(Ok video.resize_output(args.width, args.height), Err "Failed to resize image");
  if args.debug {
    println!("{}", video);
  }
  let start_time = Instant::now();
  if args.atlas {
    unwrap!(
      Ok video.burst_frames("temp/image", args.seek_position, args.end, args.step),
      Err "Failed to get burst frames"
    );
  }
  unwrap!(Ok video.get_frame(args.seek_position, "temp/image"), Err "Failed to get frame");
  let end_time = Instant::now();
  log!(success@"Done in {:?}", end_time - start_time)
}
