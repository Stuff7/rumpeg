use crate::http::{
  find_query_arg, find_query_flag, FromPath, FromQueryString, HttpRequest, HttpRequestError,
  HttpRequestResult, HttpResponse, HttpStatus, ServerResult,
};
use crate::rumpeg::SeekPosition;
use crate::video::Video;
use crate::MEDIA_FOLDER;
use std::ops::Deref;
use std::sync::atomic::Ordering;

pub fn index(request: &HttpRequest) -> ServerResult<HttpResponse> {
  HttpResponse::from_asset("public/index.html", request)
}

pub fn favicon(request: &HttpRequest) -> ServerResult<HttpResponse> {
  HttpResponse::from_asset("public/favicon.ico", request)
}

pub fn get_frame(request: &HttpRequest) -> ServerResult<HttpResponse> {
  let query: VideoArgs = request.query()?;
  let videopath: FilePath = request.path()?;

  let Ok(video) = Video::open(
    &videopath,
    query.width,
    query.height,
  ) else {
    return Ok(HttpStatus::NotFound.into());
  };

  let image = if query.film {
    video
      .film_strip(query.seek_position, query.end, query.step)?
      .encode_as_webp()?
  } else {
    let Some(mut frame) = video
      .frames(query.seek_position, query.end, query.step)?
      .next() else {
        return Ok(HttpStatus::NotFound.into())
      };
    video.frame_to_webp(&mut frame)?
  };

  let mut response = HttpResponse::new();
  response.set_status(HttpStatus::OK);
  response.add_header("Content-Type", "image/webp");
  response.add_header("X-Video-Width", &video.width.to_string());
  response.add_header("X-Video-Height", &video.height.to_string());
  response.add_header("X-Video-Duration", &video.duration_ms.to_string());
  response.add_header("X-Video-Extensions", video.extensions);
  response.add_content(image);

  Ok(response)
}

pub fn get_asset(request: &HttpRequest) -> ServerResult<HttpResponse> {
  let filepath: FilePath = request.path()?;
  HttpResponse::from_asset(&filepath, request)
}

#[derive(Debug)]
pub struct VideoArgs {
  pub film: bool,
  pub height: i32,
  pub seek_position: SeekPosition,
  pub width: i32,
  pub end: SeekPosition,
  pub step: SeekPosition,
}

impl FromQueryString for VideoArgs {
  fn from_query_string(query_string: &str) -> HttpRequestResult<Self> {
    let query = query_string.split('&').collect::<Vec<_>>();
    Ok(Self {
      film: find_query_flag(&query, "film"),
      height: find_query_arg(&query, "height"),
      seek_position: find_query_arg(&query, "start"),
      width: find_query_arg(&query, "width"),
      end: match find_query_arg(&query, "end") {
        SeekPosition::TimeBase(0) => SeekPosition::Percentage(1.),
        n => n,
      },
      step: match find_query_arg(&query, "step") {
        SeekPosition::TimeBase(0) => SeekPosition::TimeBase(1),
        n => n,
      },
    })
  }
}

#[derive(Debug)]
pub struct FilePath(String);

impl FromPath for FilePath {
  fn from_path(path: &str) -> HttpRequestResult<Self> {
    unsafe {
      path[1..]
        .split_once('/')
        .map(|(_, filepath)| {
          Self(format!(
            "{}/{filepath}",
            *MEDIA_FOLDER.load(Ordering::SeqCst)
          ))
        })
        .ok_or(HttpRequestError::Parse("Missing filepath".into()))
    }
  }
}

impl Deref for FilePath {
  type Target = String;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
