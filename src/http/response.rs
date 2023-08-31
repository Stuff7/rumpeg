use super::*;
use std::{io::Write, net::TcpStream};

#[derive(Debug, Default)]
pub enum HttpStatusCode {
  #[default]
  OK = 200,
  NotFound = 404,
}

impl HttpStatusCode {
  fn as_tuple(&self) -> (u16, &'static str) {
    match *self {
      HttpStatusCode::OK => (200, "OK"),
      HttpStatusCode::NotFound => (404, "Not Found"),
    }
  }
}

pub struct HttpResponse {
  status_code: HttpStatusCode,
  headers: Vec<(String, String)>,
  content: Vec<u8>,
}

impl HttpResponse {
  pub fn new() -> Self {
    HttpResponse {
      status_code: HttpStatusCode::OK,
      headers: Vec::new(),
      content: Vec::new(),
    }
  }

  pub fn from_status(status_code: HttpStatusCode) -> Self {
    HttpResponse {
      status_code,
      headers: Vec::new(),
      content: Vec::new(),
    }
  }

  pub fn add_header(&mut self, key: &str, value: &str) {
    self.headers.push((key.to_string(), value.to_string()));
  }

  pub fn add_content(&mut self, content: &[u8]) {
    self.add_header("Content-Length", &content.len().to_string());
    self.content.extend_from_slice(content);
  }

  pub fn set_status(&mut self, status: HttpStatusCode) {
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
    stream.write_all(&self.content)?;
    stream.flush()?;
    Ok(())
  }
}
