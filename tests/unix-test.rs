#![cfg(unix)]

extern crate tiny_http;

use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
};

#[allow(dead_code)]
mod support;

#[test]
fn unix_basic_handling() {
    let server = tiny_http::Server::http_unix(Path::new("/tmp/tiny-http-test.sock")).unwrap();
    let path: PathBuf = server
        .server_addr()
        .to_unix()
        .unwrap()
        .as_pathname()
        .unwrap()
        .into();
    let mut client = UnixStream::connect(path).unwrap();

    write!(
        client,
        "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
    )
    .unwrap();

    let request = server.recv().unwrap();
    assert!(*request.method() == tiny_http::Method::Get);
    //assert!(request.url() == "/");
    request
        .respond(tiny_http::Response::from_string("hello world".to_owned()))
        .unwrap();

    server.try_recv().unwrap();

    let mut content = String::new();
    client.read_to_string(&mut content).unwrap();
    assert!(content.ends_with("hello world"));
}

#[test]
fn unix_stream_basic_handling() {
    let addr = "/tmp/tiny-http-test.sock";
    if std::path::Path::new(addr).exists() {
        std::fs::remove_file(addr).unwrap();
    }
    let listener = std::os::unix::net::UnixListener::bind(addr).unwrap();
    let stream = UnixStream::connect(addr).unwrap();
    let server = tiny_http::Server::from_unix_stream(stream).unwrap();

    let client_handle = std::thread::spawn(move || {
        let mut client = listener.accept().unwrap().0;

        write!(
            client,
            "GET /ok HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
        )
        .unwrap();

        let mut content = String::new();
        client.read_to_string(&mut content).unwrap();
        assert!(content.ends_with("hello world"));
    });

    let server_handle = std::thread::spawn(move || {
        let request = server.recv().unwrap();
        assert!(*request.method() == tiny_http::Method::Get);
        assert!(request.url() == "/ok");
        request
            .respond(tiny_http::Response::from_string("hello world".to_owned()))
            .unwrap();

        server.try_recv().unwrap();
    });

    client_handle.join().unwrap();
    server_handle.join().unwrap();
}

