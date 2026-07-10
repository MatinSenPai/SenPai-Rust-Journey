use std::io::{Cursor, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

use p3_01_01_tcp_echo_server::{run_echo, serve_once};

#[test]
fn echoes_each_line_back() {
    let input = Cursor::new(b"hello\nworld\n".to_vec());
    let mut output = Vec::new();

    let bytes = run_echo(input, &mut output).unwrap();

    assert_eq!(bytes, 12);
    assert_eq!(output, b"hello\nworld\n");
}

#[test]
fn returns_zero_for_empty_input() {
    let input = Cursor::new(Vec::new());
    let mut output = Vec::new();

    let bytes = run_echo(input, &mut output).unwrap();

    assert_eq!(bytes, 0);
    assert!(output.is_empty());
}

#[test]
fn echoes_a_final_line_with_no_trailing_newline() {
    let input = Cursor::new(b"no newline at the end".to_vec());
    let mut output = Vec::new();

    run_echo(input, &mut output).unwrap();

    assert_eq!(output, b"no newline at the end");
}

/// A genuine end-to-end test: a real `TcpListener`, a real `TcpStream`
/// client connecting over localhost, no mocking anywhere. This is the kind
/// of test that's hard to write against a full web framework (too much
/// machinery in the way) but easy against raw sockets — worth noticing.
#[test]
fn serves_one_real_tcp_connection_end_to_end() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap(); // port 0 = "OS, pick one for me"
    let addr = listener.local_addr().unwrap();

    let handle = thread::spawn(move || serve_once(&listener).unwrap());

    let mut client = TcpStream::connect(addr).unwrap();
    client.write_all(b"ping\n").unwrap();
    // Signal "I'm done sending" so the server's read loop sees EOF and
    // returns, instead of blocking forever waiting for more input.
    client.shutdown(Shutdown::Write).unwrap();

    let mut response = String::new();
    client.read_to_string(&mut response).unwrap();
    assert_eq!(response, "ping\n");

    let bytes_echoed = handle.join().unwrap();
    assert_eq!(bytes_echoed, 5);
}
