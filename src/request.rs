use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;

#[derive(Debug, PartialEq)]
pub enum HTTPMethod {
    POST,
    GET,
    PATCH,
    DELETE,
    PUT,
    OPTIONS,
}

impl ToString for HTTPMethod {
    fn to_string(&self) -> String {
        return match self {
            HTTPMethod::POST => "POST".to_string(),
            HTTPMethod::GET => "GET".to_string(),
            HTTPMethod::PATCH => "PATCH".to_string(),
            HTTPMethod::DELETE => "DELETE".to_string(),
            HTTPMethod::PUT => "PUT".to_string(),
            HTTPMethod::OPTIONS => "OPTIONS".to_string(),
        };
    }
}

impl From<&str> for HTTPMethod {
    fn from(value: &str) -> Self {
        return match value.to_uppercase().as_str() {
            "GET" => HTTPMethod::GET,
            "POST" => HTTPMethod::POST,
            "PATCH" => HTTPMethod::PATCH,
            "DELETE" => HTTPMethod::DELETE,
            "PUT" => HTTPMethod::PUT,
            "OPTIONS" => HTTPMethod::OPTIONS,
            &_ => HTTPMethod::GET,
        };
    }
}

#[derive(Debug, Clone)]
pub struct HTTPHeader(pub String, pub String);

#[derive(Debug)]
pub struct HTTPQuery(String, String);

#[derive(Debug)]
pub struct HTTPContext {
    pub method: HTTPMethod,
    pub http_version: String,
    pub headers: Vec<HTTPHeader>,
    pub path: String,
    pub queries: Vec<HTTPQuery>,
    pub body: Option<String>,
}

fn to_header(pair: Option<(&str, &str)>) -> Option<HTTPHeader> {
    return match pair {
        Some((key, value)) => Some(HTTPHeader(key.trim().to_string(), value.trim().to_string())),
        None => None,
    };
}

fn parse_headers(raw_headers: std::slice::Iter<'_, &str>) -> Vec<HTTPHeader> {
    raw_headers
        .map(|&raw_header| raw_header.split_once(":"))
        .map(to_header)
        .filter(|header| header.is_some())
        .map(|header| header.unwrap())
        .collect()
}

fn parse_queries(raw_query: Vec<&str>) -> Vec<HTTPQuery> {
    raw_query
        .iter()
        .map(|&q| q.split_once("="))
        .filter(|&x| x.is_some())
        .map(|x| x.unwrap())
        .map(|(key, value)| HTTPQuery(key.to_string(), value.to_string()))
        .collect()
}

fn parse_path(raw_path: &str) -> (&str, Vec<&str>) {
    let parts: Vec<_> = raw_path.split("?").collect();
    if parts.len() == 1 {
        return (parts[0], vec![]);
    }

    let queries: Vec<_> = parts[1].split("&").collect();
    return (parts[0], queries);
}

fn build_request(raw_headers: &String, raw_body: &String) -> Option<HTTPContext> {
    let request_lines: Vec<_> = raw_headers.split("\r\n").collect();
    let start = request_lines.first();
    if start.is_none() {
        return None;
    }
    let start_line = *start.unwrap();
    let start_parts: Vec<&str> = start_line.split(" ").collect();
    if start_parts.len() < 3 {
        return None;
    }
    let method = start_parts[0];
    let raw_path = start_parts[1];
    let (path, raw_queries) = parse_path(raw_path);
    let queries = parse_queries(raw_queries);
    let http_version = start_parts[2];
    let headers = parse_headers(request_lines[1..].iter());
    let mut body = None;
    if raw_body.trim().len() > 0 {
        body = Some(raw_body.trim().to_string());
    }
    return Some(HTTPContext {
        method: method.into(),
        http_version: http_version.to_string(),
        headers,
        queries,
        body,
        path: path.to_string(),
    });
}

fn get_content_length(header: &str) -> usize {
    return match header.split_once(":") {
        Some((_, content_length)) => content_length.trim().parse().unwrap_or_default(),
        None => 0,
    };
}

fn read_stream(stream: &mut TcpStream) -> Option<(String, String)> {
    let mut reader = BufReader::new(stream);
    let mut raw_headers = String::new();
    let mut content_length = 0;
    const MIN_LINE_LENGTH: usize = 3;
    loop {
        match reader.read_line(&mut raw_headers) {
            Ok(size) => {
                if size < MIN_LINE_LENGTH {
                    break;
                }
            }
            Err(_) => return None,
        }
    }
    for line in raw_headers.lines() {
        if line.contains("Content-Length") {
            content_length = get_content_length(line);
        }
    }

    let mut buffer = vec![0; content_length];

    match reader.read_exact(&mut buffer) {
        Ok(_) => {
            return Some((
                raw_headers.trim().to_string(),
                String::from_utf8_lossy(&buffer).to_string(),
            ))
        }
        Err(_) => return None,
    }
}

pub fn parse_stream(stream: &mut TcpStream) -> Result<HTTPContext, String> {
    let request_parts = read_stream(stream);
    if request_parts.is_none() {
        return Err("Failed to read lines".to_string());
    }
    let (raw_headers, raw_body) = request_parts.unwrap();
    match build_request(&raw_headers, &raw_body) {
        Some(context) => Ok(context),
        None => Err("Could not read request ".to_string()),
    }
}
