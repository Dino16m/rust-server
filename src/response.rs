use std::io::Write;
use std::net::TcpStream;

use http::{header::HeaderName, StatusCode};

pub struct HttpResponse {
    status_code: StatusCode,
    headers: Vec<(HeaderName, String)>,
    body: Option<String>,
}

impl HttpResponse {
    pub fn new() -> Self {
        return HttpResponse {
            status_code: StatusCode::OK,
            headers: vec![],
            body: None,
        };
    }
    pub fn set_status(&mut self, status_code: StatusCode) -> &mut Self {
        self.status_code = status_code;
        return self;
    }

    pub fn set_body(&mut self, body: &str) -> &mut Self {
        self.body = Some(body.to_string());
        return self;
    }

    pub fn set_header(&mut self, key: HeaderName, value: &str) -> &mut Self {
        self.headers.push((key, value.to_string()));
        return self;
    }

    fn write_headers(&self, stream: &mut TcpStream) -> Result<(), std::io::Error> {
        for header in self.headers.iter() {
            let header_line = format!("{}: {}\r\n\r\n", header.0.as_str(), header.1);
            match stream.write(header_line.as_bytes()) {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        }

        return Ok(());
    }

    fn write_status(&self, stream: &mut TcpStream) -> Result<(), std::io::Error> {
        let status_line = format!(
            "HTTP/1.1 {} {}\r\n",
            self.status_code.as_str(),
            self.status_code
                .canonical_reason()
                .unwrap_or("Unknown status code")
        );
        return match stream.write(status_line.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        };
    }

    fn write_body(&self, stream: &mut TcpStream) -> Result<(), std::io::Error> {
        let body = match &self.body {
            Some(body) => body,
            None => "",
        };

        let response = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
        return match stream.write(response.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        };
    }
    pub fn write(&self, stream: &mut TcpStream) -> Result<(), std::io::Error> {
        println!("status {}", self.status_code.as_str());
        self.write_status(stream)?;
        self.write_headers(stream)?;
        self.write_body(stream)
    }
}
