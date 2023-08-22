use std::{env, str::FromStr};
use thiserror::Error;

use crate::rumpeg::{LogLevel, SeekPosition};

#[derive(Debug)]
pub struct CLIArgs {
  pub atlas: bool,
  pub debug: bool,
  pub filepath: String,
  pub height: i32,
  pub seek_position: SeekPosition,
  pub width: i32,
  pub log_level: LogLevel,
  pub end: SeekPosition,
  pub step: SeekPosition,
}

impl CLIArgs {
  pub fn read() -> CLIResult<Self> {
    let args: Vec<String> = env::args().collect();
    Ok(Self {
      atlas: Self::find_flag(&args, "-a"),
      debug: Self::find_flag(&args, "-d"),
      filepath: args.get(1).ok_or(CLIError::FilepathMissing)?.clone(),
      height: Self::find_arg(&args, "-h"),
      seek_position: Self::find_arg(&args, "-s"),
      width: Self::find_arg(&args, "-w"),
      log_level: Self::find_arg(&args, "-l"),
      end: Self::find_arg(&args, "-end"),
      step: Self::find_arg(&args, "-step"),
    })
  }

  fn find_flag(args: &[String], arg_name: &str) -> bool {
    args.iter().any(|arg| arg == arg_name)
  }

  fn find_arg<F: FromStr + Default>(args: &[String], arg_name: &str) -> F {
    args
      .iter()
      .position(|arg| arg == arg_name)
      .and_then(|i| args.get(i + 1))
      .and_then(|n| n.parse::<F>().ok())
      .unwrap_or_default()
  }
}

#[derive(Error, Debug)]
pub enum CLIError {
  #[error("Missing filepath")]
  FilepathMissing,
}

pub type CLIResult<T = ()> = Result<T, CLIError>;
