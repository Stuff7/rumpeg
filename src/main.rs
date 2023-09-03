mod ascii;
mod cli;
mod ffmpeg;
mod http;
mod math;
mod routes;
mod rumpeg;
mod video;
mod webp;

use crate::cli::CLIArgs;
use crate::http::{Router, Server};
use ascii::LogDisplay;
use rumpeg::*;
use std::fs::write;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::time::Instant;
use video::Video;

macro_rules! unwrap {
  (Some $wrapped: expr, Err $( $err: expr ),*) => {
    match $wrapped {
      Some(v) => v,
      None => {
        log!(err@$( $err ),*);
        return;
      }
    }
  };
  (Ok $wrapped: expr, Err $( $err: expr ),*) => {
    match $wrapped {
      Ok(v) => v,
      Err(e) => {
        log!(err@$( $err ),*);
        log!(err@"{e}");
        return;
      }
    }
  };
}

pub static MEDIA_FOLDER: AtomicPtr<String> = AtomicPtr::new(std::ptr::null_mut());

fn main() {
  log!(ok@"Using Ffmpeg v{} and libwebp v{}", rumpeg::version(), webp::version());
  let mut args = unwrap!(Ok CLIArgs::read(), Err "Error");
  rumpeg::set_log_level(args.log_level);

  if args.host {
    let mut router = Router::new();
    router
      .get("/frame/*", routes::get_frame)
      .get("/media/*", routes::get_asset)
      .get("/favicon.ico", routes::favicon)
      .get("/*", routes::index);
    let server = unwrap!(Ok Server::new("0.0.0.0:8080", router), Err "Could not create server");
    MEDIA_FOLDER.store(&mut args.filepath as *mut _, Ordering::SeqCst);

    unwrap!(Ok server.listen(), Err "Server could not listen");
    return;
  }

  let video = unwrap!(
    Ok Video::open(&args.filepath, args.width, args.height),
    Err "Failed to open video"
  );

  if args.debug {
    println!("{}", video);
  }

  let start_time = Instant::now();

  unwrap!(
    Ok save_image(&video, "temp/image", args.seek_position),
    Err "Failed to save image"
  );

  if args.film {
    unwrap!(
      Ok save_film_strip(&video, "temp/image", args.seek_position, args.end, args.step),
      Err "Failed to save film roll"
    );
  }

  let end_time = Instant::now();

  log!(ok@"Done in {:?}", end_time - start_time)
}

fn save_film_strip(
  video: &Video,
  thumbnail_path: &str,
  start: SeekPosition,
  end: SeekPosition,
  step: SeekPosition,
) -> Result<(), Box<dyn std::error::Error>> {
  write(
    format!("{thumbnail_path}-film.webp"),
    video.film_strip(start, end, step)?.encode_as_webp()?,
  )?;

  Ok(())
}

fn save_image(
  video: &Video,
  thumbnail_path: &str,
  position: SeekPosition,
) -> Result<(), Box<dyn std::error::Error>> {
  if let Some(mut frame) = video
    .frames(
      position,
      SeekPosition::Percentage(1.),
      SeekPosition::default(),
    )?
    .next()
  {
    let image = video.frame_to_webp(&mut frame)?;
    write(format!("{thumbnail_path}.webp"), image)?;
  }

  Ok(())
}
