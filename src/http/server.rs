use super::*;
use crate::ascii::LogDisplay;
use crate::log;
use std::io::prelude::*;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::ops::Index;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct Server {
  listener: TcpListener,
  router: Arc<Router>,
}

impl Server {
  pub fn new(addr: &str, router: Router) -> ServerResult<Self> {
    Ok(Self {
      listener: TcpListener::bind(addr)?,
      router: Arc::new(router),
    })
  }

  pub fn listen(&self) -> ServerResult {
    if unsafe { SetConsoleCtrlHandler(Some(ctrl_handler), TRUE) } == FALSE {
      return Err(ServerError::ExitHandler);
    }

    let addr = self.listener.local_addr()?;
    let mut connections = Vec::new();
    self.listener.set_nonblocking(true)?;

    log!(ok@"Server listening on {addr:?}");
    while !CTRL_C_PRESSED.load(Ordering::SeqCst) {
      match self.listener.accept() {
        Ok((stream, addr)) => {
          let router = self.router.clone();
          connections.push(
            thread::Builder::new()
              .name(addr.to_string())
              .spawn(move || serve_client(stream, router))?,
          );
          connections.retain(|connection| !connection.is_finished());
          log!(info@"Connections: {}", connections.len());
        }
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
          std::thread::sleep(std::time::Duration::from_millis(50));
        }
        Err(e) => {
          log!(err@"Error accepting connection\n{e}");
        }
      }
    }

    log!(info@"Ctrl C pressed, exiting...");
    for connection in connections {
      connection.join().expect("Could not join connection")?;
    }

    Ok(())
  }
}

fn serve_client(mut stream: TcpStream, router: Arc<Router>) -> ServerResult {
  stream.set_read_timeout(Some(Duration::from_secs(5)))?;
  let thread = thread::current();
  let name = thread.name().unwrap_or("Unnamed Connection");

  log!(ok@"[{name}] New connection");

  let mut received_data = [0; 1024];
  while !CTRL_C_PRESSED.load(Ordering::SeqCst) {
    match stream.read(&mut received_data) {
      Ok(size) if size > 0 => {
        let mut response = router.route(HttpRequest::parse(&received_data));
        response.send(&mut stream)?;
      }
      Ok(_) => break,
      Err(_) => break,
    }
  }

  if let Err(e) = stream.shutdown(Shutdown::Both) {
    log!(err@"[{name}] Error closing stream: {e}");
  }

  log!(info@"[{name}] Connection closed");
  Ok(())
}

type Route = Box<dyn Fn(&HttpRequest) -> ServerResult<HttpResponse> + Send + Sync>;
pub struct Router {
  get: Vec<(String, Route)>,
}

impl Router {
  pub fn new() -> Self {
    Self { get: Vec::new() }
  }

  pub fn get(
    &mut self,
    endpoint: &str,
    route: impl Fn(&HttpRequest) -> ServerResult<HttpResponse> + 'static + Send + Sync,
  ) -> &mut Self {
    self.get.push((endpoint.to_string(), Box::new(route)));
    self
  }

  fn route(&self, request: HttpRequestResult) -> HttpResponse {
    let request = match request {
      Ok(r) => r,
      Err(e) => {
        return HttpStatus::BadRequest(e).into();
      }
    };

    self[request.method]
      .iter()
      .find(|ep| {
        if let Some(path) = ep.0.strip_suffix("/*") {
          request.path.starts_with(path)
        } else {
          request.path == ep.0
        }
      })
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

impl Index<HttpMethod> for Router {
  type Output = Vec<(String, Route)>;
  fn index(&self, index: HttpMethod) -> &Self::Output {
    match index {
      HttpMethod::Get => &self.get,
    }
  }
}
