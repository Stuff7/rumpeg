use crate::ascii::Color;
use crate::ascii::RESET;
use crate::math;
use crate::rumpeg;
use crate::rumpeg::RumpegResult;
use crate::rumpeg::SeekPosition;
use std::fmt::Display;
use thiserror::Error;

#[derive(Debug)]
pub struct Video {
  pub duration_us: u64,
  pub extensions: &'static str,
  pub format_name: &'static str,
  pub height: i32,
  pub mime_type: &'static str,
  pub width: i32,
  codec_context: rumpeg::AVCodecContext,
  display_matrix: Option<math::Matrix3x3>,
  format_context: rumpeg::AVFormatContext,
  sws_context: rumpeg::SwsContext,
}

#[derive(Error, Debug)]
pub enum VideoError {
  #[error(transparent)]
  Rumpeg(#[from] rumpeg::RumpegError),
}

type VideoResult<T = ()> = Result<T, VideoError>;

impl Video {
  pub fn open(filepath: &str) -> VideoResult<Video> {
    let format_context = rumpeg::AVFormatContext::new(filepath)?;
    let codecpar = rumpeg::AVCodecParameters::new(format_context.stream.codecpar)?;
    let codec_context = rumpeg::AVCodecContext::new(&codecpar)?;
    let iformat = rumpeg::AVInputFormat::new(format_context.iformat);
    let display_matrix = format_context.stream.display_matrix();

    Ok(Self {
      duration_us: format_context.duration as u64,
      extensions: iformat.extensions,
      format_name: iformat.format_name,
      height: codecpar.height,
      mime_type: iformat.mime_type,
      width: codecpar.width,
      codec_context,
      display_matrix,
      format_context,
      sws_context: rumpeg::SWSContextBuilder::from_codecpar(codecpar).build()?,
    })
  }

  pub fn resize_output(&mut self, width: i32, height: i32) -> VideoResult {
    Ok(self.sws_context.resize_output(width, height)?)
  }

  pub fn get_frame(&mut self, position: SeekPosition, thumbnail_path: &str) -> VideoResult {
    self.seek(position)?;

    if let Some(mut frame) = self.frames(None).next() {
      let webp = self
        .sws_context
        .transform(&mut frame, self.display_matrix)?
        .encode_as_webp();
      std::fs::write(format!("{thumbnail_path}.webp"), &*webp).expect("Failed to save image");
    }

    Ok(())
  }

  pub fn burst_frames(&mut self, position: SeekPosition, thumbnail_path: &str) -> VideoResult {
    self.seek(position)?;
    let mut i = 0;

    while let Some(mut curr_frame) = self.frames(Some(SeekPosition::Seconds(5))).next() {
      let webp = self
        .sws_context
        .transform(&mut curr_frame, self.display_matrix)?
        .encode_as_webp();
      std::fs::write(format!("{thumbnail_path}-{i}.webp"), &*webp).expect("Failed to save image");
      i += 1;
    }

    Ok(())
  }

  fn seek(&self, position: SeekPosition) -> RumpegResult {
    self.codec_context.flush();
    self.format_context.seek(position)
  }

  fn frames(&mut self, step: Option<SeekPosition>) -> rumpeg::AVPacketIter {
    rumpeg::AVPacketIter::new(&mut self.format_context, &mut self.codec_context, step)
  }
}

impl Display for Video {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{title}VIDEO INFO{RESET}\n\
      - {title}Display Matrix:{RESET} {}\n\
      - {title}Rotation:{RESET} {}°\n\
      {}\n\
      - {title}Duration:{RESET} {} seconds\n\
      - {title}Extensions:{RESET} {}\n\
      - {title}Format:{RESET} {}\n\
      - {title}Mime Type:{RESET} {}",
      self
        .display_matrix
        .map(|m| format!("\n{m}"))
        .unwrap_or("None".into()),
      self.display_matrix.map(|m| m.rotation()).unwrap_or(0.),
      self.sws_context,
      self.duration_us as f64 / 1_000_000.,
      self.extensions,
      self.format_name,
      self.mime_type,
      title = "".rgb(75, 205, 94).bold(),
    )
  }
}
