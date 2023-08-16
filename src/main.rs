mod ffmpeg {
  #![allow(non_upper_case_globals)]
  #![allow(non_camel_case_types)]
  #![allow(non_snake_case)]
  #![allow(dead_code)]
  include!(concat!(env!("OUT_DIR"), "/ffmpeg.rs"));
}

mod rumpeg;
mod video;

use std::env;
use video::Video;

fn main() {
  let args: Vec<String> = env::args().collect();
  let filepath = args.get(1).unwrap_or_exit("Missing file path").as_str();
  let mut video = Video::open(filepath).unwrap_or_exit("Failed to open video");
  println!("VIDEO {:#?}", video);
  for i in 0..9 {
    video
      .get_thumbnail(i * 5, format!("temp/image-{i}.webp").as_str())
      .unwrap_or_exit("Failed to get thumbnail");
  }
}

trait GracefulExit<T> {
  fn unwrap_or_exit(self, msg: impl std::fmt::Display) -> T;
}

impl<T, E> GracefulExit<T> for Result<T, E>
where
  E: std::fmt::Display,
{
  fn unwrap_or_exit(self, msg: impl std::fmt::Display) -> T {
    match self {
      Ok(t) => t,
      Err(e) => {
        eprintln!("{msg}: {e}");
        std::process::exit(0)
      }
    }
  }
}

impl<T> GracefulExit<T> for Option<T> {
  fn unwrap_or_exit(self, msg: impl std::fmt::Display) -> T {
    match self {
      Some(t) => t,
      None => {
        eprintln!("{msg}");
        std::process::exit(1)
      }
    }
  }
}
