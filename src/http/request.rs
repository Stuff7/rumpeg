use crate::{ascii::LogDisplay, log};
use std::{
  collections::HashMap,
  io::Read,
  net::{TcpListener, TcpStream},
  str::from_utf8,
};

#[derive(Debug)]
pub struct HttpRequest {
  pub method: String,
  pub path: String,
  pub query_params: HashMap<String, String>,
  headers: Vec<(String, String)>,
  http_version: String,
}

impl HttpRequest {
  fn parse(raw_data: &[u8]) -> Option<HttpRequest> {
    let Ok(request) = from_utf8(raw_data) else {
      return None
    };

    let lines: Vec<&str> = request.lines().collect();

    let request_line_parts: Vec<&str> = lines[0].split_whitespace().collect();
    if request_line_parts.len() != 3 {
      return None;
    }

    let method = request_line_parts[0].to_string();
    let mut path = request_line_parts[1].to_string();
    let http_version = request_line_parts[2].to_string();

    let mut query_params = HashMap::new();
    if let Some(pos) = path.find('?') {
      let query_string = &path[pos + 1..];
      for pair in query_string.split('&') {
        let parts: Vec<&str> = pair.splitn(2, '=').collect();
        if parts.len() == 2 {
          query_params.insert(parts[0].to_string(), parts[1].to_string());
        }
      }
      path = path[..pos].to_string();
    }

    let mut headers = Vec::new();
    for line in &lines[1..] {
      if let Some((key, value)) = parse_header(line) {
        headers.push((key, value));
      }
    }

    Some(HttpRequest {
      method,
      path,
      http_version,
      headers,
      query_params,
    })
  }
}

pub enum Request {
  Http(HttpRequest),
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
                return HttpRequest::parse(received_data).map(|req| (stream, Request::Http(req)));
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

fn parse_header(header: &str) -> Option<(String, String)> {
  let parts: Vec<&str> = header.splitn(2, ':').map(|s| s.trim()).collect();
  if parts.len() == 2 {
    Some((parts[0].to_string(), parts[1].to_string()))
  } else {
    None
  }
}
