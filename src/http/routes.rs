use super::*;
use crate::{ascii::LogDisplay, log, rumpeg::SeekPosition, video::Video};

pub(super) fn get_frame(request: &HttpRequest) -> ServerResult<HttpResponse> {
  log!("QUERY: {:#?}", request.query_params);
  let Some(filepath) = request.path[1..].split_once('/').map(|(_, filepath)| {
    filepath.to_string()
  }) else {
    return Ok(HttpResponse::from_status(HttpStatusCode::NotFound));
  };
  log!("FILE: {filepath:#?}");
  let Ok(video) = Video::open(
    &filepath,
    request.query_params.get("width").cloned().unwrap_or_default().parse().unwrap_or(0),
    request.query_params.get("height").cloned().unwrap_or_default().parse().unwrap_or(0),
  ) else {
    return Ok(HttpResponse::from_status(HttpStatusCode::NotFound));
  };
  let frame = video
    .frames(
      request
        .query_params
        .get("start")
        .cloned()
        .unwrap_or_default()
        .parse()
        .unwrap_or_default(),
      request
        .query_params
        .get("end")
        .cloned()
        .unwrap_or_default()
        .parse()
        .unwrap_or(SeekPosition::Percentage(1.)),
      request
        .query_params
        .get("step")
        .cloned()
        .unwrap_or_default()
        .parse()
        .unwrap_or(SeekPosition::TimeBase(1)),
    )?
    .next();

  let mut response = HttpResponse::new();
  if let Some(mut frame) = frame {
    let frame = video.frame_to_webp(&mut frame)?;
    response.set_status(HttpStatusCode::OK);
    response.add_header("Content-Type", "image/webp");
    response.add_content(frame);
  } else {
    response.set_status(HttpStatusCode::NotFound);
  }
  Ok(response)
}
