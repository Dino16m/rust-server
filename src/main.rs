use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn flush_stream(stream: &mut TcpStream) {
    match stream.flush() {
        Ok(_) => println!("Flushed stream"),
        Err(_) => eprintln!("An error occured when flushing stream"),
    }
}

fn read_request(stream: &mut TcpStream) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(content) => {
            println!("Read content from stream: {:?}", content)
        }
        Err(e) => eprintln!("Failed to read to stream: {e}"),
    }
}

fn main() {
    let address = "127.0.0.1:4221";
    println!("Listening on address: {:?}", address);

    let listener = TcpListener::bind(address).unwrap();

    for raw_stream in listener.incoming() {
        match raw_stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                read_request(&mut stream);
                let body = "hello";
                let status = "200 OK";
                let status_line = format!("HTTP/1.1 {}\r\n", status);
                stream.write(status_line.as_bytes()).unwrap();

                let response: String = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
                stream.write(response.as_bytes()).unwrap();

                flush_stream(&mut stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
