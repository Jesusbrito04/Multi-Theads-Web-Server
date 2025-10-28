//! A simple multi-threaded web server.
//!
//! Listens on `127.0.0.1:7878` and serves content from `hello.hmtl`
//! or `notFound.html` using a thread pool.
use harbor::server::start;

fn main() {
    start("127.0.0.1:7878");
}
