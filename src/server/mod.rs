use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use crate::ThreadPool;

/// Entry point of the web server.
///
/// Bind a TCP Listener to the address, creates a thread pool, and enters
/// a loop to handler incoming connections.
pub fn start(address: &str) {
    let listener = TcpListener::bind(address).unwrap();
    let pool = match ThreadPool::build(4) {
        Ok(threads) => threads,
        Err(error) => {
            eprintln!("{:?}", error);
            return;
        }
    };

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

/// Handles a single TCP connection.
///
/// Read the first line of the HTTP request to determine the endpoint.
/// Respons with the content of `hello.html` for the root path `/` and
/// `notFound.html` for any other path.
///
/// Simulates a delay for the `/sleep` path.
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "notFound.html"),
    };
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap()
}
