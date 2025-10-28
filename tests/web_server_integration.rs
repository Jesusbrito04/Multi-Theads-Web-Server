use harbor::server::start;
use std::{
    io::{Read, Write},
    net::TcpStream,
    thread,
    time::Duration,
};

#[test]
fn test_http_get_root_returns_hello_html() {
    thread::spawn(|| {
        start("127.0.0.1:7878");
    });

    thread::sleep(Duration::from_secs(1));

    let mut stream = TcpStream::connect("127.0.0.1:7878")
        .expect("Failed to connect to server. Make sure port 7878 is free.");

    stream.write_all(b"GET / HTTP/1.1\r\n\r\n")
        .expect("Failed to write HTTP request.");

    let mut buffer: Vec<u8> = Vec::new();

    stream.read_to_end(&mut buffer).expect("Failed to read server response.");

    let response = String::from_utf8_lossy(&buffer);

    assert!(response.contains("HTTP/1.1 200 OK"), "Response does not contain 'HTTP/1.1 200 OK'. Response: 
      {}", response);
    assert!(response.contains("<h1>Hello!</h1>"), "Response does not contain '<h1>Hello!</h1>'. Response: {}"
    , response);
}
