use std::io::Write;
use std::net::{TcpListener, TcpStream};

use nom::ParseTo;

use crate::response::HttpResponse;
use crate::route::Router;

use crate::request::parse_stream;

fn flush_stream(stream: &mut TcpStream) {
    match stream.flush() {
        Ok(_) => (),
        Err(_) => eprintln!("An error occured when flushing stream"),
    }
}

pub struct Server {
    port: u32,
    host: String,
    router: Router,
}

impl Server {
    pub fn new(port: u32, host: String) -> Self {
        return Server {
            port,
            host,
            router: Router::new(),
        };
    }

    pub fn from_address(address: &str) -> Option<Self> {
        match address.split_once(":") {
            Some((host, port)) => Some(Self::new(port.parse_to().unwrap(), host.to_string())),
            None => None,
        }
    }

    pub fn router(&mut self) -> &mut Router {
        return &mut self.router;
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
        for raw_stream in listener.incoming() {
            match raw_stream {
                Ok(mut stream) => {
                    self.process_stream(&mut stream);
                }
                Err(e) => {
                    println!("error: {}", e);
                }
            }
        }
    }

    fn write_response(&self, response: HttpResponse, stream: &mut TcpStream) {
        match response.write(stream) {
            Ok(_) => (),
            Err(e) => eprintln!("error: {}", e),
        }
    }
    fn process_stream(&self, stream: &mut TcpStream) {
        match parse_stream(stream) {
            Ok(context) => {
                let response = self.router.handle(context);
                self.write_response(response, stream);
                flush_stream(stream);
            }
            Err(e) => eprintln!("stream error: {}", e),
        }

        let _ = stream.shutdown(std::net::Shutdown::Both);
    }
}
