//! DELIBERATELY BROKEN — expected: E0499.
//!
//!     cargo run -p p3-01-01-tcp-echo-server --example 02-double-mut-borrow-broken --features broken

use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

fn run_echo<R: BufRead, W: Write>(_reader: R, _writer: W) -> io::Result<usize> {
    Ok(0)
}

fn handle(mut stream: TcpStream) -> io::Result<usize> {
    let reader = BufReader::new(&mut stream);
    let writer = &mut stream;
    run_echo(reader, writer)
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let (stream, _addr) = listener.accept()?;
    handle(stream)?;
    Ok(())
}
