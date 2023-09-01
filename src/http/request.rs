use super::*;
use crate::{ascii::LogDisplay, log};
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::str::{from_utf8, Utf8Error};
use thiserror::Error;

#[derive(Debug)]
pub struct HttpRequest {
  pub method: HttpMethod,
  pub path: String,
  pub query_string: String,
  pub headers: Vec<(String, String)>,
  pub http_version: String,
}

impl HttpRequest {
  fn parse(raw_data: &[u8]) -> HttpRequestResult {
    let request = from_utf8(raw_data)?;

    let lines: Vec<&str> = request.lines().collect();

    let request_line_parts: Vec<&str> = lines[0].split_whitespace().collect();
    if request_line_parts.len() != 3 {
      return Err(HttpRequestError::Header(lines[0].to_string()));
    }

    let method = request_line_parts[0].try_into()?;
    let path = decode_uri(request_line_parts[1]);
    let (path, query_string) = path.split_once('?').unwrap_or((&path, ""));
    let http_version = request_line_parts[2].to_string();

    let mut headers = Vec::new();
    for line in &lines[1..] {
      if let Some((key, value)) = parse_header(line) {
        headers.push((key, value));
      }
    }

    Ok(Self {
      method,
      path: path.to_string(),
      http_version,
      headers,
      query_string: query_string.to_string(),
    })
  }

  pub fn path<Q: FromPath>(&self) -> HttpRequestResult<Q> {
    Q::from_path(&self.path)
  }

  pub fn query<Q: FromQueryString>(&self) -> HttpRequestResult<Q> {
    Q::from_query_string(&self.query_string)
  }
}

#[derive(Debug, Error)]
pub enum HttpRequestError {
  #[error("Request is not valid utf8\n{0}")]
  Data(#[from] Utf8Error),
  #[error("Invalid request header {0:?}")]
  Header(String),
  #[error("Invalid request method {0:?}")]
  Method(String),
  #[error("Could not parse request {0}")]
  Parse(String),
}

pub type HttpRequestResult<T = HttpRequest> = Result<T, HttpRequestError>;

pub enum Request {
  Http(HttpRequestResult<HttpRequest>),
  Exit,
}

pub struct RequestIter<'a> {
  listener: &'a TcpListener,
}

impl<'a> RequestIter<'a> {
  pub fn new(listener: &'a TcpListener) -> Self {
    Self { listener }
  }
}

impl<'a> Iterator for RequestIter<'a> {
  type Item = (TcpStream, Request);
  fn next(&mut self) -> Option<Self::Item> {
    for stream in self.listener.incoming() {
      match stream {
        Ok(mut stream) => {
          let mut buffer = [0; 1024];
          match stream.read(&mut buffer) {
            Ok(size) => {
              let received_data = &buffer[..size];

              if received_data == b"exit" {
                return Some((stream, Request::Exit));
              } else {
                return Some((stream, Request::Http(HttpRequest::parse(received_data))));
              }
            }
            Err(e) => {
              log!(err@"Read error: {}", e);
            }
          }
        }
        Err(e) => {
          log!(err@"Error accepting connection: {}", e);
        }
      }
    }
    None
  }
}

#[derive(Debug)]
pub enum HttpMethod {
  Get,
}

impl TryFrom<&str> for HttpMethod {
  type Error = HttpRequestError;
  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "GET" => Ok(Self::Get),
      method => Err(HttpRequestError::Method(method.to_string())),
    }
  }
}

fn parse_header(header: &str) -> Option<(String, String)> {
  let parts: Vec<&str> = header.splitn(2, ':').map(|s| s.trim()).collect();
  if parts.len() == 2 {
    Some((parts[0].to_string(), parts[1].to_string()))
  } else {
    None
  }
}
