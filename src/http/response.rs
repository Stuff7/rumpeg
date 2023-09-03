use super::*;
use std::{collections::HashMap, io::Write, net::TcpStream};

#[derive(Debug, Default)]
pub enum HttpStatus {
  #[default]
  OK,
  PartialContent,
  BadRequest(HttpRequestError),
  NotFound,
  InternalServerError(ServerError),
}

impl HttpStatus {
  fn as_tuple(&self) -> (u16, &'static str) {
    match *self {
      HttpStatus::OK => (200, "OK"),
      HttpStatus::PartialContent => (206, "Partial Content"),
      HttpStatus::BadRequest(..) => (400, "Bad Request"),
      HttpStatus::NotFound => (404, "Not Found"),
      HttpStatus::InternalServerError(..) => (500, "Internal Server Error"),
    }
  }
}

#[derive(Debug)]
pub struct HttpResponse {
  status_code: HttpStatus,
  headers: HashMap<String, String>,
  content: Vec<u8>,
}

impl Default for HttpResponse {
  fn default() -> Self {
    Self {
      status_code: HttpStatus::default(),
      headers: HashMap::from([
        ("Content-Type".to_string(), "text/plain".to_string()),
        ("Cache-Control".to_string(), "no-cache".to_string()),
        ("Content-Length".to_string(), 0.to_string()),
      ]),
      content: Vec::new(),
    }
  }
}

impl HttpResponse {
  pub fn from_asset(asset_path: &str, request: &HttpRequest) -> ServerResult<Self> {
    let mut response = Self::default();
    let mut asset = Asset::open(asset_path)?;
    response.add_asset(&mut asset, request.range())?;

    Ok(response)
  }

  pub fn add_header(&mut self, key: &str, value: &str) {
    self.headers.insert(key.to_string(), value.to_string());
  }

  pub fn add_content(&mut self, content: &[u8]) {
    self.add_header("Content-Length", &content.len().to_string());
    self.content.extend_from_slice(content);
  }

  pub fn add_asset(&mut self, asset: &mut Asset, range: Option<(usize, usize)>) -> ServerResult {
    self.add_header("Content-Type", asset.content_type);

    let content = if range.is_some() || asset.size > PARTIAL_CONTENT_SIZE {
      let length = asset.size;
      let (start, end) = range.unwrap_or((0, ASSET_CHUNK_SIZE));
      let end = std::cmp::min(
        match end - start {
          s if s > ASSET_CHUNK_SIZE || s == 0 => start + ASSET_CHUNK_SIZE,
          _ => end,
        },
        length,
      );

      self.set_status(HttpStatus::PartialContent);
      self.add_header("Connection", "keep-alive");
      self.add_header("Keep-Alive", "timeout=5");
      self.add_header("Accept-Ranges", "bytes");
      self.add_header("Content-Range", &format!("bytes {start}-{end}/{length}"));

      asset.read(start, end)?
    } else {
      asset.bytes()?
    };
    self.add_content(&content);
    Ok(())
  }

  pub fn set_status(&mut self, status: HttpStatus) {
    self.status_code = status;
  }

  pub fn raw(&self) -> String {
    let (code, reason) = self.status_code.as_tuple();

    let mut response = format!("HTTP/1.1 {} {}\r\n", code, reason);
    for (key, value) in &self.headers {
      response.push_str(&format!("{}: {}\r\n", key, value));
    }
    response.push_str("\r\n");
    response
  }

  pub fn send(&mut self, stream: &mut TcpStream) -> ServerResult {
    if self.content.is_empty() {
      if let Some(e) = match self.status_code {
        HttpStatus::InternalServerError(ref e) => Some(e.to_string()),
        HttpStatus::BadRequest(ref e) => Some(e.to_string()),
        _ => None,
      } {
        self.add_content(e.as_bytes());
      }
    }
    stream.write_all(self.raw().as_bytes())?;
    stream.write_all(&self.content)?;
    stream.flush()?;
    Ok(())
  }
}

impl From<HttpStatus> for HttpResponse {
  fn from(status_code: HttpStatus) -> Self {
    Self {
      status_code,
      ..Default::default()
    }
  }
}
