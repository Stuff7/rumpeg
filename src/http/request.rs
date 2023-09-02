use super::*;
use std::collections::HashMap;
use std::str::{from_utf8, Utf8Error};
use thiserror::Error;

#[derive(Debug)]
pub struct HttpRequest {
  pub method: HttpMethod,
  pub path: String,
  pub query_string: String,
  pub headers: HashMap<String, String>,
  pub http_version: String,
}

impl HttpRequest {
  pub fn parse(raw_data: &[u8]) -> HttpRequestResult {
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

    let mut headers = HashMap::new();
    for line in &lines[1..] {
      if let Some((key, value)) = parse_header(line) {
        headers.insert(key.to_lowercase(), value);
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

  pub fn range(&self) -> Option<(usize, usize)> {
    self.headers.get("range").and_then(|r| {
      r.split_once('=').and_then(|r| {
        r.1.find('-').map(|i| {
          let start = r
            .1
            .get(..i)
            .map(|s| s.parse::<usize>().unwrap_or(0))
            .unwrap_or(0);
          let end = r
            .1
            .get(i + 1..r.1.len())
            .map(|s| s.parse::<usize>().unwrap_or(start + MAX_ASSET_SIZE))
            .unwrap_or(start + MAX_ASSET_SIZE);
          (start, end)
        })
      })
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
