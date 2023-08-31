use super::*;
use crate::ascii::LogDisplay;
use crate::log;
use std::io::prelude::*;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::Ordering;
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct Server {
  listener: TcpListener,
}

impl Server {
  pub fn new(addr: &str) -> ServerResult<Self> {
    Ok(Self {
      listener: TcpListener::bind(addr)?,
    })
  }

  pub fn listen(&self) -> ServerResult {
    let addr = self.listener.local_addr()?;
    let ctrl_c_thread = create_ctrl_c_thread(addr)?;

    log!(success@"Server listening on {addr:?}");
    for (mut stream, request) in RequestIter::new(&self.listener) {
      match request {
        Request::Http(request) => route(&request).send(&mut stream)?,
        Request::Exit => {
          log!(info@"Ctrl+C pressed, exiting...");
          break;
        }
      }
    }

    ctrl_c_thread
      .join()
      .expect("Could not join ctrl_c_thread")?;

    Ok(())
  }
}

fn route(request: &HttpRequest) -> HttpResponse {
  match request.method.as_str() {
    "GET" if request.path.starts_with("/frame") => routes::get_frame(request),
    _ => Ok(HttpResponse::from_status(HttpStatusCode::NotFound)),
  }
  .unwrap_or_else(|e| {
    let mut response = HttpResponse::from_status(HttpStatusCode::InternalServerError(e));
    response.add_header("Content-Type", "text/plain");
    response
  })
}

fn create_ctrl_c_thread(addr: SocketAddr) -> ServerResult<JoinHandle<ServerResult>> {
  let ctrl_c_handler = unsafe { SetConsoleCtrlHandler(Some(ctrl_handler), TRUE) };
  if ctrl_c_handler == FALSE {
    return Err(ServerError::ExitHandler);
  }

  Ok(
    thread::Builder::new()
      .name("Ctrl + C".into())
      .spawn(move || -> ServerResult {
        loop {
          if CTRL_C_PRESSED.load(Ordering::SeqCst) {
            let mut stream = TcpStream::connect(addr)?;
            stream.write_all("exit".as_bytes())?;
            stream.shutdown(std::net::Shutdown::Write)?;
            break;
          }
          thread::sleep(Duration::from_millis(100));
        }
        Ok(())
      })?,
  )
}
