use super::*;
use std::{collections::HashMap, io::Write, net::TcpStream};

#[derive(Debug, Default)]
pub enum HttpStatus {
  #[default]
  OK,
  BadRequest(HttpRequestError),
  NotFound,
  InternalServerError(ServerError),
}

impl HttpStatus {
  fn as_tuple(&self) -> (u16, &'static str) {
    match *self {
      HttpStatus::OK => (200, "OK"),
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
      ]),
      content: Vec::new(),
    }
  }
}

impl HttpResponse {
  pub fn new() -> Self {
    HttpResponse {
      status_code: HttpStatus::OK,
      headers: HashMap::new(),
      content: Vec::new(),
    }
  }

  pub fn add_header(&mut self, key: &str, value: &str) {
    self.headers.insert(key.to_string(), value.to_string());
  }

  pub fn add_content(&mut self, content: &[u8]) {
    self.add_header("Content-Length", &content.len().to_string());
    self.content.extend_from_slice(content);
  }

  pub fn set_status(&mut self, status: HttpStatus) {
    self.status_code = status;
  }

  pub fn send(&self, stream: &mut TcpStream) -> ServerResult {
    let (code, reason) = self.status_code.as_tuple();

    let mut response = format!("HTTP/1.1 {} {}\r\n", code, reason);
    for (key, value) in &self.headers {
      response.push_str(&format!("{}: {}\r\n", key, value));
    }
    response.push_str("\r\n");

    stream.write_all(response.as_bytes())?;
    if self.content.is_empty() {
      if let Some(e) = match self.status_code {
        HttpStatus::InternalServerError(ref e) => Some(e.to_string()),
        HttpStatus::BadRequest(ref e) => Some(e.to_string()),
        _ => None,
      } {
        stream.write_all(e.as_bytes())?
      }
    } else {
      stream.write_all(&self.content)?;
    }
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
