use std::{
  fs::File,
  io::{Read, Seek, SeekFrom},
  ops::Deref,
};
use thiserror::Error;

pub const MAX_ASSET_SIZE: usize = 5 * 1024 * 1024;

#[derive(Debug, Error)]
pub enum AssetError {
  #[error("Asset error [IO]\n{0}")]
  IO(#[from] std::io::Error),
}

pub type AssetResult<T = Asset> = Result<T, AssetError>;

pub struct Asset {
  file: File,
  pub content_type: &'static str,
  pub size: usize,
}

impl Asset {
  pub fn open(file_path: &str) -> AssetResult {
    let file = File::open(file_path)?;
    Ok(Asset {
      size: file.metadata().map(|m| m.len() as usize).unwrap_or(0),
      file,
      content_type: get_content_type(file_path),
    })
  }

  pub fn read(&mut self, start: usize, end: usize) -> AssetResult<Vec<u8>> {
    let start = std::cmp::min(start, self.size);
    let end = std::cmp::min(end, self.size);
    let mut buffer = vec![0u8; end - start];
    self.file.seek(SeekFrom::Start(start as u64))?;
    self.file.read_exact(&mut buffer)?;
    Ok(buffer)
  }

  pub fn bytes(&mut self) -> AssetResult<Vec<u8>> {
    let mut buffer = Vec::with_capacity(self.size);
    self.file.read_to_end(&mut buffer)?;
    Ok(buffer)
  }
}

impl Deref for Asset {
  type Target = File;
  fn deref(&self) -> &Self::Target {
    &self.file
  }
}

fn get_content_type(filepath: &str) -> &'static str {
  match filepath.to_lowercase() {
    filepath if filepath.ends_with(".html") => "text/html",
    filepath if filepath.ends_with(".css") => "text/css",
    filepath if filepath.ends_with(".js") => "application/javascript",
    filepath if filepath.ends_with(".ico") => "image/x-icon",
    filepath if filepath.ends_with(".jpg") || filepath.ends_with(".jpeg") => "image/jpeg",
    filepath if filepath.ends_with(".png") => "image/png",
    filepath if filepath.ends_with(".gif") => "image/gif",
    filepath if filepath.ends_with(".mp4") => "video/mp4",
    filepath if filepath.ends_with(".mov") => "video/quicktime",
    _ => "application/octet-stream",
  }
}
