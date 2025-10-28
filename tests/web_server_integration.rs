use harbor::server::start;
use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::Once,
    thread,
    time::Duration,
};

static START_SERVER: Once = Once::new();

fn setup() {
    START_SERVER.call_once(|| {
        thread::spawn(|| {
            start("127.0.0.1:7878");
        });

        thread::sleep(Duration::from_secs(2));
    });
}

#[test]
fn test_http_get_root_returns_hello_html() {
    setup();

    let mut stream = TcpStream::connect("127.0.0.1:7878")
        .expect("Failed to connect to server. Make sure port 7878 is free.");

    stream
        .write_all(b"GET / HTTP/1.1\r\n\r\n")
        .expect("Failed to write HTTP request.");

    let mut buffer: Vec<u8> = Vec::new();

    stream
        .read_to_end(&mut buffer)
        .expect("Failed to read server response.");

    let response = String::from_utf8_lossy(&buffer);

    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "Response does not contain 'HTTP/1.1 200 OK'. Response: 
      {}",
        response
    );
    assert!(
        response.contains("<h1>Hello!</h1>"),
        "Response does not contain '<h1>Hello!</h1>'. Response: {}",
        response
    );
}

#[test]
fn test_http_get_sleep_returns_hello_html_after_delay() {
    setup();

    let mut stream = TcpStream::connect("127.0.0.1:7878")
        .expect("Failed to connect to server. Make sure port 7878 is free.");

    stream
        .write_all(b"GET /sleep HTTP/1.1\r\n\r\n")
        .expect("Failed to write HTTP request.");

    let mut buffer: Vec<u8> = Vec::new();

    stream
        .read_to_end(&mut buffer)
        .expect("Failed to read server response.");

    let response = String::from_utf8_lossy(&buffer);

    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "Response does not contain 'HTTP/1.1 200 OK'. Response: 
      {}",
        response
    );
    assert!(
        response.contains("<h1>Hello!</h1>"),
        "Response does not contain '<h1>Hello!</h1>'. Response: 
      {}",
        response
    );
}

#[test]
fn test_http_get_unknown_path_returns_404_not_found() {
    setup();

    let mut stream = TcpStream::connect("127.0.0.1:7878")
        .expect("Failed to connect to server. Make sure port 7878 is free.");

    stream
        .write_all(b"GET /unknown_path HTTP/1.1\r\n\r\n")
        .expect("Failed to write HTTP request.");

    let mut buffer: Vec<u8> = Vec::new();

    stream
        .read_to_end(&mut buffer)
        .expect("Failed to read server response.");

    let response = String::from_utf8_lossy(&buffer);

    assert!(
        response.contains("HTTP/1.1 404 NOT FOUND"),
        "Response does not contain 'HTTP/1.1 404 NOT FOUND'. Response: {}",
        response
    );

    assert!(
        response.contains("<h1>Oops!</h1>"),
        "Response does not contain '<h1>Oops!</h1>' (from 
    notFound.html). Response: {}",
        response
    );
}
