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
  pub router: Router,
}

impl Server {
  pub fn new(addr: &str) -> ServerResult<Self> {
    Ok(Self {
      listener: TcpListener::bind(addr)?,
      router: Router::new(),
    })
  }

  pub fn listen(&self) -> ServerResult {
    let addr = self.listener.local_addr()?;
    let ctrl_c_thread = create_ctrl_c_thread(addr)?;

    log!(success@"Server listening on {addr:?}");
    for (mut stream, request) in RequestIter::new(&self.listener) {
      match request {
        Request::Http(request) => self.router.route(request).send(&mut stream)?,
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

type Route = Box<dyn Fn(&HttpRequest) -> ServerResult<HttpResponse>>;
pub struct Router {
  endpoints_get: Vec<(String, Route)>,
}

impl Router {
  pub fn new() -> Self {
    Self {
      endpoints_get: Vec::new(),
    }
  }

  pub fn get(
    &mut self,
    endpoint: &str,
    route: impl Fn(&HttpRequest) -> ServerResult<HttpResponse> + 'static,
  ) -> &mut Self {
    self
      .endpoints_get
      .push((endpoint.to_string(), Box::new(route)));
    self
  }

  fn route(&self, request: HttpRequestResult) -> HttpResponse {
    let request = match request {
      Ok(r) => r,
      Err(e) => {
        return HttpStatus::BadRequest(e).into();
      }
    };

    let mut endpoints = match request.method {
      HttpMethod::Get => self.endpoints_get.iter(),
    };

    endpoints
      .find(|ep| request.path.starts_with(&ep.0))
      .map(|ep| ep.1(&request))
      .unwrap_or(Ok(HttpStatus::NotFound.into()))
      .unwrap_or_else(|e| {
        match e {
          ServerError::BadRequest(e) => HttpStatus::BadRequest(e),
          _ => HttpStatus::InternalServerError(e),
        }
        .into()
      })
  }
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
            let mut stream = TcpStream::connect(format!("127.0.0.1:{}", addr.port()))?;
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
