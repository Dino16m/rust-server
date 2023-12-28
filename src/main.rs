use std::{thread::sleep, time};

use response::HttpResponse;
use route::Router;

use crate::server::Server;
mod request;
mod response;
mod route;
mod server;

fn get_router() -> Router {
    let mut router = Router::new();
    router.route(request::HTTPMethod::GET, "/", |_request| {
        let mut response = HttpResponse::new();
        let sleep_duration = time::Duration::from_millis(10);
        sleep(sleep_duration);
        response.set_body("Jung jung");
        return response;
    });

    return router;
}

fn main() {
    let address = "127.0.0.1:4221";

    let mut server = Server::from_address(address).unwrap();
    server.use_router(get_router());

    server.run(move || println!("Listening on address: {:?}", address))
}
