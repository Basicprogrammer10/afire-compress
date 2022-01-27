use afire::{Method, Response, Server};
use afire_compress::{Compress, CompressType};

fn main() {
    let mut server = Server::new("localhost", 8080);

    Compress::new()
        .threshold(0)
        .compression(CompressType::Gzip(6))
        .attach(&mut server);

    server.route(Method::GET, "/", |_| Response::new().text("Hello World"));

    server.start().unwrap();
}
