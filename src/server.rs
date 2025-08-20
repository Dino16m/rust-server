use std::io::{BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

use tokio::{io::{self, AsyncReadExt, AsyncWriteExt}};

use nom::ParseTo;
use threadpool::ThreadPool;

use crate::response::HttpResponse;
use crate::route::Router;

use crate::request::{parse_stream, parse_stream_async};

pub struct Server {
    port: u32,
    host: String,
    router: Arc<Router>,
}

impl Server {
    pub fn new(port: u32, host: String) -> Self {
        return Server {
            port,
            host,
            router: Arc::new(Router::new()),
        };
    }

    pub fn from_address(address: &str) -> Option<Self> {
        match address.split_once(":") {
            Some((host, port)) => Some(Self::new(port.parse_to().unwrap(), host.to_string())),
            None => None,
        }
    }

    pub fn use_router(&mut self, router: Router) {
        self.router = Arc::new(router);
    }

    fn address(&self) -> String {
        return format!("{}:{}", self.host, self.port);
    }
    pub fn run<TCallback>(&self, cb: TCallback)
    where
        TCallback: Fn() + 'static,
    {
        let listener = TcpListener::bind(self.address()).unwrap();
        cb();
        let pool = ThreadPool::new(8);
        for raw_stream in listener.incoming() {
            let router = self.router.clone();
            match raw_stream {
                Ok(mut stream) => pool.execute(move || process_stream(router, &mut stream)),
                Err(e) => {
                    println!("error: {}", e);
                }
            }
        }
    }

    pub async   fn run_async<TCallback>(&self, cb: TCallback)
        where
        TCallback: Fn() + 'static,
    {
        let listener = tokio::net::TcpListener::bind(self.address()).await.unwrap();
        cb();
        loop {
            let conn  = listener.accept().await;
            let router = self.router.clone();
            match  conn {
                Ok((mut stream, _)) => {
                    tokio::spawn(async move {
                        process_stream_async(router, &mut stream).await;
                    });
                },
                Err(e) => println!("error: {}", e),
            }
    }
    }
}

fn write_response(response: HttpResponse, stream: &mut TcpStream) {
    match response.write(stream) {
        Ok(_) => (),
        Err(e) => eprintln!("error: {}", e),
    }
}

async fn write_response_async(response: HttpResponse, stream: &mut tokio::net::TcpStream) {
    match response.write_async(stream).await {
        Ok(_) => (),
        Err(e) => eprintln!("error: {}", e),
    }
}
async fn process_stream_async(router: Arc<Router>, stream: &mut tokio::net::TcpStream) {
    match  parse_stream_async(stream).await {
        Ok(context) => {
            let response = router.handle(context);
            write_response_async(response, stream).await;
            match stream.flush().await {
                Ok(_) => (),
                Err(_) => eprintln!("An error occured when flushing stream"),
            }
        }
        Err(e) => eprintln!("stream error: {}", e),
    }
}
fn process_stream(router: Arc<Router>, stream: &mut TcpStream) {
    match parse_stream(stream) {
        Ok(context) => {
            let response = router.handle(context);
            write_response(response, stream);
            flush_stream(stream);
        }
        Err(e) => eprintln!("stream error: {}", e),
    }

    let _ = stream.shutdown(std::net::Shutdown::Both);
}

fn flush_stream(stream: &mut TcpStream) {
    match stream.flush() {
        Ok(_) => (),
        Err(_) => eprintln!("An error occured when flushing stream"),
    }
}
