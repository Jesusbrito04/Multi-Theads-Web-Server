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
    let listener = match TcpListener::bind(address) {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("Failed to bind to address: {}. Error: {}", address, err);
            return;
        }
    };

    let pool = match ThreadPool::build(4) {
        Ok(threads) => threads,
        Err(error) => {
            eprintln!("You cannot create a thread pool of size zero: {:?}", error);
            return;
        }
    };

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("{}", err);
                continue;
            }
        };
        pool.execute(move|| {
           match handle_connection(stream) {
                Ok(_) => Ok("Connection handled successfully".to_string()),
                Err(e) => Err(format!("Error handling connection: {}", e)),
           }
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
fn handle_connection(mut stream: TcpStream) -> Result<(), String> {
    let buf_reader = BufReader::new(&stream);

    if let Some(request_line) = buf_reader.lines().next() {
        let request_line = match request_line {
            Ok(request) => request,
            Err(err) => {
                eprintln!("No text UTF-8 valid: {}", err);
                return Err(err.to_string());
            }
        };

        let (status_line, filename) = match &request_line[..] {
            "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
            "GET /sleep HTTP/1.1" => {
                thread::sleep(Duration::from_secs(5));
                ("HTTP/1.1 200 OK", "hello.html")
            }
            _ => ("HTTP/1.1 404 NOT FOUND", "notFound.html"),
        };

        let contents = match fs::read_to_string(filename) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("This file isn't avalible {}", err);
                return Err(err.to_string());
            }
        };

        let length = contents.len();

        let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

        match stream.write_all(response.as_bytes()) {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("{}", err);
                return Err(err.to_string());
            }
        }
    };
    Ok(())
}
