use std::{thread::sleep, time};

use response::HttpResponse;
use route::Router;

use crate::server::Server;

mod request;
mod response;
mod route;
mod server;

fn say_jung(_request: &mut route::HTTPRequest) -> HttpResponse {
        let mut response = HttpResponse::new();
        let sleep_duration = time::Duration::from_millis(10);
        sleep(sleep_duration);
        response.set_body("Jung jung");
        return response;
}

fn echo(request: &mut route::HTTPRequest) -> HttpResponse {
        let mut response = HttpResponse::new();
        let sleep_duration = time::Duration::from_millis(10);
        sleep(sleep_duration);
        match &request.context.body {
            Some(value) => response.set_body(&value),
            None => response.set_body("Unknown"),
        };
        return response;
}
fn get_router() -> Router {
    let mut router = Router::new();
    router.route(request::HTTPMethod::GET, "/", |_request| {
        let mut response = HttpResponse::new();
        let sleep_duration = time::Duration::from_millis(10);
        sleep(sleep_duration);
        response.set_body("Jung jung");
        return response;
    });

    router.route(request::HTTPMethod::GET, "/app", say_jung);
    router.route(request::HTTPMethod::POST, "/echo", echo);

    return router;
}

#[tokio::main]
async fn main() {
    let address = "127.0.0.1:4221";

    let mut  server = Server::from_address(address).unwrap();
    server.use_router(get_router());

    server.run_async(move || println!("Listening synchronously on address: {:?}", address)).await;
}
