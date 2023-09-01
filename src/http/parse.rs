use super::HttpRequestResult;
use std::str::FromStr;

pub trait FromQueryString {
  fn from_query_string(query_string: &str) -> HttpRequestResult<Self>
  where
    Self: std::marker::Sized;
}

pub trait FromPath {
  fn from_path(path: &str) -> HttpRequestResult<Self>
  where
    Self: std::marker::Sized;
}

pub fn find_query_flag(query: &[&str], key_name: &str) -> bool {
  query.iter().any(|key| *key == key_name)
}

pub fn find_query_arg<F: FromStr + Default>(query: &[&str], key_name: &str) -> F {
  query
    .iter()
    .find(|q| q.starts_with(&format!("{key_name}=")))
    .and_then(|q| q.split_once('='))
    .map(|q| q.1)
    .and_then(|n| n.parse::<F>().ok())
    .unwrap_or_default()
}

pub(super) fn decode_uri(uri: &str) -> String {
  let mut decoded_uri = String::with_capacity(uri.len());
  let mut chars = uri.chars();

  while let Some(c) = chars.next() {
    match c {
      '%' => {
        let encoded_byte = chars.by_ref().take(2).collect::<String>();
        if let Ok(byte) = u8::from_str_radix(&encoded_byte, 16) {
          decoded_uri.push(char::from(byte));
        } else {
          decoded_uri.push('%');
          decoded_uri.push_str(&encoded_byte);
        }
      }
      _ => decoded_uri.push(c),
    }
  }

  decoded_uri
}
