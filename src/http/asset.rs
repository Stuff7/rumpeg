use std::{
  collections::HashMap,
  fs::File,
  io::{Read, Seek, SeekFrom},
  ops::Deref,
  sync::OnceLock,
};
use thiserror::Error;

pub const PARTIAL_CONTENT_SIZE: usize = 25 * 1024 * 1024;
pub const ASSET_CHUNK_SIZE: usize = 512 * 1024;

#[derive(Debug, Error)]
pub enum AssetError {
  #[error("Asset error [IO - {}]\n{0}", .0.kind())]
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

static CONTENT_TYPES: OnceLock<HashMap<&str, &str>> = OnceLock::new();

fn get_content_type(filepath: &str) -> &'static str {
  let content_types = CONTENT_TYPES.get_or_init(|| {
    HashMap::from([
      (".html", "text/html"),
      (".css", "text/css"),
      (".js", "application/javascript"),
      (".ico", "image/x-icon"),
      (".jpg", "image/jpeg"),
      (".jpeg", "image/jpeg"),
      (".png", "image/png"),
      (".gif", "image/gif"),
      (".mp4", "video/mp4"),
      (".mov", "video/quicktime"),
      (".pdf", "application/pdf"),
      (".txt", "text/plain"),
      (".xml", "application/xml"),
      (".json", "application/json"),
      (".csv", "text/csv"),
      (".svg", "image/svg+xml"),
      (".mp3", "audio/mpeg"),
      (".wav", "audio/wav"),
      (".zip", "application/zip"),
      (".tar", "application/x-tar"),
      (".gz", "application/gzip"),
      (".gzip", "application/gzip"),
      (".ogg", "audio/ogg"),
      (".woff", "font/woff"),
      (".woff2", "font/woff2"),
      (".eot", "application/vnd.ms-fontobject"),
      (".ttf", "font/ttf"),
      (".otf", "font/otf"),
      (".webp", "image/webp"),
      (".avi", "video/x-msvideo"),
      (".flv", "video/x-flv"),
      (".wmv", "video/x-ms-wmv"),
      (".mkv", "video/x-matroska"),
      (".3gp", "video/3gpp"),
      (".3g2", "video/3gpp2"),
      (".ogv", "video/ogg"),
      (".webm", "video/webm"),
      (".mpg", "video/mpeg"),
      (".mpeg", "video/mpeg"),
      (".m4v", "video/x-m4v"),
      (".mng", "video/x-mng"),
      (".mpv", "video/x-matroska"),
      (".ts", "video/mp2t"),
      (".asf", "video/x-ms-asf"),
      (".asx", "video/x-ms-asf"),
      (".vob", "video/dvd"),
      (".m2ts", "video/MP2T"),
      (".divx", "video/divx"),
      (".xvid", "video/x-xvid"),
      (".rm", "application/vnd.rn-realmedia"),
      (".rmvb", "application/vnd.rn-realmedia-vbr"),
      (".f4v", "video/x-f4v"),
      (".mpeg4", "video/mp4"),
      (".mp4v", "video/mp4"),
      (".3gpp", "video/3gpp"),
      (".mj2", "video/mj2"),
      (".mk3d", "video/x-matroska-3d"),
      (".mks", "video/x-matroska"),
      (".h264", "video/h264"),
      (".h265", "video/h265"),
    ])
  });

  if let Some(index) = filepath.rfind('.') {
    if let Some(content_type) = content_types.get(&filepath.to_lowercase()[index..]) {
      return content_type;
    }
  }

  "application/octet-stream"
}
