use std::io::Write;
use std::net::TcpStream;

use http::{header::HeaderName, StatusCode};
use tokio::io::AsyncWriteExt;

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


    async fn write_headers_async(&self, stream: &mut tokio::net::TcpStream) -> Result<(), std::io::Error> {
        for header in self.headers.iter() {
            let header_line = format!("{}: {}\r\n\r\n", header.0.as_str(), header.1);
            match stream.write(header_line.as_bytes()).await {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        }

        return Ok(());
    }

    fn status_line(&self) -> String {
        format!(
            "HTTP/1.1 {} {}\r\n",
            self.status_code.as_str(),
            self.status_code
                .canonical_reason()
                .unwrap_or("Unknown status code")
        )
    }

    fn write_status(&self, stream: &mut TcpStream) -> Result<(), std::io::Error> {
        let status_line = self.status_line();
        return match stream.write(status_line.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        };
    }

    async fn write_status_async(&self, stream: &mut tokio::net::TcpStream) -> Result<(), std::io::Error> {
        let status_line = self.status_line();
        return match stream.write(status_line.as_bytes()).await {
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

    async fn write_body_async(&self, stream: &mut tokio::net::TcpStream) -> Result<(), std::io::Error> {
        let body = match &self.body {
            Some(body) => body,
            None => "",
        };

        let response = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
        return match stream.write(response.as_bytes()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        };
    }

    pub fn write(&self, stream: &mut TcpStream) -> Result<(), std::io::Error> {
        self.write_status(stream)?;
        self.write_headers(stream)?;
        self.write_body(stream)
    }

    pub async fn write_async(&self, stream: &mut tokio::net::TcpStream) -> Result<(), std::io::Error> {
        self.write_status_async(stream).await?;
        self.write_headers_async(stream).await?;
        self.write_body_async(stream).await
    }
}
