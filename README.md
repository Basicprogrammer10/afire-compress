# afire-compress
🦄 afire middleware to compress outgoing HTTP traffic

Supports `Gzip`, `Deflate`, and `Brotli`.
Make sure this is the first middleware added to a server.

## Example

```rust
use afire::{Method, Response, Server};
use afire_compress::{Compress, CompressType};

fn main() {
    let mut server = Server::new("localhost", 8080);

    // Add Compressor
    Compress::new().attach(&mut server);

    server.route(Method::GET, "/", |_| Response::new().text("Hello World"));

    server.start().unwrap();
}
```
