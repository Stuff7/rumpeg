mod ascii;
mod cli;
mod ffmpeg;
mod math;
mod rumpeg;
mod video;

use ascii::Color;
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
  let mut video = unwrap!(Ok Video::open(&args.filepath), Err "Failed to open video");
  unwrap!(Ok video.resize_output(args.width, args.height), Err "Failed to resize image");
  // for i in 0..9 {
  //   unwrap!(
  //     Ok video.get_frame(rumpeg::SeekPosition::Seconds(i * 5), format!("temp/a-image-{i}").as_str()),
  //     Err "Failed to get frame"
  //   );
  // }
  if args.debug {
    println!("{}", video);
  }
  unwrap!(Ok video.burst_frames(args.seek_position, "temp/image"), Err "Failed to get burst frames");
  unwrap!(Ok video.get_frame(args.seek_position, "temp/image"), Err "Failed to get frame");
}
