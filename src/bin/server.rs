#![deny(warnings)]

use std::convert::Infallible;
use std::net::SocketAddr;

use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper::Method;
use hyper::StatusCode;
use tokio::fs;
use hyper_util::rt::{TokioIo, TokioTimer};
use tokio::net::TcpListener;

// This would normally come from the `hyper-util` crate, but we can't depend
// on that here because it would be a cyclical dependency.


// An async function that serves static files from the "static" directory.
async fn hello(req: Request<impl hyper::body::Body>) -> Result<Response<Full<Bytes>>, Infallible> {
    // Only handle GET requests
    if req.method() != Method::GET {
        let resp = Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Full::new(Bytes::from("Method Not Allowed")))
            .unwrap();
        return Ok(resp);
    }

    // Map "/" to "index.html", otherwise strip leading '/' from the path
    let path = match req.uri().path() {
        "/" => "static/index.html".to_string(),
        p if p.starts_with('/') => format!("static{}", p),
        p => format!("static/{}", p),
    };

    // Attempt to read the file asynchronously
    match fs::read(&path).await {
        Ok(contents) => {
            let resp = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html; charset=utf-8")
                .body(Full::new(Bytes::from(contents)))
                .unwrap();
            Ok(resp)
        }
        Err(_) => {
            let resp = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "text/html; charset=utf-8")
                .body(Full::new(Bytes::from("<h1>404 Not Found</h1>")))
                .unwrap();
            Ok(resp)
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
   

    // This address is localhost
    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();

    // Bind to the port and listen for incoming TCP connections
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);
    loop {
        // When an incoming TCP connection is received grab a TCP stream for
        // client<->server communication.
        //
        // Note, this is a .await point, this loop will loop forever but is not a busy loop. The
        // .await point allows the Tokio runtime to pull the task off of the thread until the task
        // has work to do. In this case, a connection arrives on the port we are listening on and
        // the task is woken up, at which point the task is then put back on a thread, and is
        // driven forward by the runtime, eventually yielding a TCP stream.
        let (tcp, _) = listener.accept().await?;
        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(tcp);

        // Spin up a new task in Tokio so we can continue to listen for new TCP connection on the
        // current task without waiting for the processing of the HTTP1 connection we just received
        // to finish
        tokio::task::spawn(async move {
            // Handle the connection from the client using HTTP1 and pass any
            // HTTP requests received on that connection to the `hello` function
            if let Err(err) = http1::Builder::new()
                .timer(TokioTimer::new())
                .serve_connection(io, service_fn(hello))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}