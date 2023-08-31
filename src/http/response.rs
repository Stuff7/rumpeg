use super::*;
use std::{collections::HashMap, io::Write, net::TcpStream};

#[derive(Debug, Default)]
pub enum HttpStatusCode {
  #[default]
  OK,
  NotFound,
  InternalServerError(ServerError),
}

impl HttpStatusCode {
  fn as_tuple(&self) -> (u16, &'static str) {
    match *self {
      HttpStatusCode::OK => (200, "OK"),
      HttpStatusCode::NotFound => (404, "Not Found"),
      HttpStatusCode::InternalServerError(..) => (500, "Internal Server Error"),
    }
  }
}

pub struct HttpResponse {
  status_code: HttpStatusCode,
  headers: HashMap<String, String>, // Using a HashMap for headers
  content: Vec<u8>,
}

impl HttpResponse {
  pub fn new() -> Self {
    HttpResponse {
      status_code: HttpStatusCode::OK,
      headers: HashMap::new(), // Initialize with a HashMap
      content: Vec::new(),
    }
  }

  pub fn from_status(status_code: HttpStatusCode) -> Self {
    HttpResponse {
      status_code,
      headers: HashMap::new(), // Initialize with a HashMap
      content: Vec::new(),
    }
  }

  pub fn add_header(&mut self, key: &str, value: &str) {
    self.headers.insert(key.to_string(), value.to_string()); // Use insert for HashMap
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
    if self.content.is_empty() {
      if let HttpStatusCode::InternalServerError(ref e) = self.status_code {
        stream.write_all(e.to_string().as_bytes())?;
      }
    } else {
      stream.write_all(&self.content)?;
    }
    stream.flush()?;
    Ok(())
  }
}
