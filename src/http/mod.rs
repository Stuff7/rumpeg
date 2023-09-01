mod parse;
mod request;
mod response;
mod server;

pub use parse::*;
pub use request::*;
pub use response::*;
pub use server::*;

use crate::rumpeg::RumpegError;
use crate::video::VideoError;
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
  #[error("Server Error [IO]: {0}")]
  IO(#[from] std::io::Error),
  #[error("Server Error [Video]: {0}")]
  Video(#[from] VideoError),
  #[error("Server Error [WebP]: {0}")]
  WebP(#[from] RumpegError),
  #[error("Failed to set Ctrl+C handler")]
  ExitHandler,
  #[error("Server Error [WebP]: {0}")]
  BadRequest(#[from] HttpRequestError),
}

pub type ServerResult<T = ()> = Result<T, ServerError>;

const TRUE: i32 = 1;
const FALSE: i32 = 0;

const CTRL_C_EVENT: u32 = 0;

extern "system" {
  fn SetConsoleCtrlHandler(
    handlerRoutine: Option<unsafe extern "system" fn(dwCtrlType: u32) -> i32>,
    add: i32,
  ) -> i32;
}

static CTRL_C_PRESSED: AtomicBool = AtomicBool::new(false);

unsafe extern "system" fn ctrl_handler(ctrl_type: u32) -> i32 {
  match ctrl_type {
    CTRL_C_EVENT => {
      CTRL_C_PRESSED.store(true, Ordering::SeqCst);
      TRUE
    }
    _ => FALSE,
  }
}
