//! TCP is a byte stream, not a sequence of messages: two separate `write`
//! calls on one side of a connection can arrive together in a single `read`
//! on the other. This runs against a real loopback socket to prove it.
//!
//!     cargo run -p p3-01-01-tcp-echo-server --example 01-two-writes-one-read

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;

    let reader = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        // Give both writes below time to land in the kernel's socket buffer
        // before we read anything at all.
        thread::sleep(Duration::from_millis(200));
        let mut buf = [0u8; 64];
        let n = stream.read(&mut buf).unwrap();
        println!(
            "one read() call got {n} bytes: {:?}",
            String::from_utf8_lossy(&buf[..n])
        );
    });

    let mut client = TcpStream::connect(addr)?;
    client.write_all(b"hello\n")?; // first write() call
    client.write_all(b"world\n")?; // second, separate write() call

    reader.join().unwrap();
    Ok(())
}
