use harbor::server::handle_connection;
use std::net::TcpListener;
use threadpool::ThreadPool;

pub fn start(addr: &str) {
    if let Ok(listener) = TcpListener::bind(addr) {
        let pool = ThreadPool::new(3);

        for stream in listener.incoming() {
            let stream = stream.unwrap();

            pool.execute(move || {
                if let Err(err) = handle_connection(stream) {
                    eprintln!("Error al manejar la conexión: {}", err);
                }
            });
        }
    }
}

fn main() {
    start("127.0.0.1:7879");
}
