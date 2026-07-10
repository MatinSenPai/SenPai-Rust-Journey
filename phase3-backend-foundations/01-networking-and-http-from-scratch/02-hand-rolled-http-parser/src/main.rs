// Provided for you — no `todo!()`s here, only in `lib.rs`. This wires the
// parser into an actual server: accept a connection, read the raw bytes,
// hand them to `parse_request`, decide on a response, write it back.
use std::io::{BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

use p3_01_02_hand_rolled_http_parser::{parse_request, HttpResponse};

fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:7879";
    let listener = TcpListener::bind(addr)?;
    println!("hand-rolled http server listening on {addr} — try: curl -v http://{addr}/");

    for incoming in listener.incoming() {
        let stream = match incoming {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("failed to accept connection: {e}");
                continue;
            }
        };
        if let Err(e) = handle_connection(stream) {
            eprintln!("connection error: {e}");
        }
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> std::io::Result<()> {
    // A real server reads until it finds the blank-line terminator (or
    // Content-Length bytes, for a body). To keep this lesson focused on
    // parsing rather than framing, we just read whatever's immediately
    // available into a generous fixed buffer — plenty for a `curl` GET.
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut buf = [0u8; 8192];
    let bytes_read = reader.read(&mut buf)?;

    let response = match parse_request(&buf[..bytes_read]) {
        Ok(request) => {
            println!("{} {}", request.method, request.path);
            if request.path == "/" {
                HttpResponse::ok("Hello from a hand-rolled HTTP server!\n")
            } else {
                HttpResponse::not_found()
            }
        }
        Err(e) => {
            eprintln!("failed to parse request: {e}");
            HttpResponse::new(400, "Bad Request", format!("Bad Request: {e}\n"))
        }
    };

    stream.write_all(&response.to_bytes())?;
    stream.flush()
}
