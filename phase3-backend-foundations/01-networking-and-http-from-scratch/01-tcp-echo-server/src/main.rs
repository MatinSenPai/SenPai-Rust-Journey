// This file is provided for you — no `todo!()`s here, only in `lib.rs`.
//
// A real server can't just handle one connection and stop (that's what
// `serve_once` is for, in the tests) — it has to accept connections
// forever, and it shouldn't let one slow client block every other client.
// The classic fix, before `tokio`'s async model shows up next module, is
// one OS thread per connection.
use std::io::BufReader;
use std::net::TcpListener;
use std::thread;

use p3_01_01_tcp_echo_server::run_echo;

fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:7878";
    let listener = TcpListener::bind(addr)?;
    println!("tcp echo server listening on {addr} — try: nc {addr}");

    for incoming in listener.incoming() {
        let stream = match incoming {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("failed to accept connection: {e}");
                continue;
            }
        };

        thread::spawn(move || {
            let peer = stream.peer_addr().ok();
            let writer = match stream.try_clone() {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("failed to clone stream for {peer:?}: {e}");
                    return;
                }
            };
            let reader = BufReader::new(stream);

            match run_echo(reader, writer) {
                Ok(bytes) => println!("connection {peer:?} closed after echoing {bytes} bytes"),
                Err(e) => eprintln!("connection {peer:?} errored: {e}"),
            }
        });
    }

    Ok(())
}
