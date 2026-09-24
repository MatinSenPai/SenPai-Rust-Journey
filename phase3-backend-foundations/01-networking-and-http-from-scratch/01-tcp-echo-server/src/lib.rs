use std::io::{self, BufRead, Write};
use std::net::{TcpListener, TcpStream};

/// Reads lines from `reader` until EOF, writing each line straight back to
/// `writer` as it's read. Returns the total number of bytes read.
///
/// Generic over `R: BufRead` and `W: Write` (not concrete `TcpStream`s) on
/// purpose: a real `TcpStream` implements both, but so does an in-memory
/// `Cursor<&[u8]>` / `Vec<u8>`. That's what lets `tests/echo_test.rs` drive
/// this function without ever opening a real socket.
///
/// `R: BufRead` rather than plain `R: Read` because reading a *line* at a
/// time needs `read_line`, which only `BufRead` provides — `Read` alone
/// only gives you raw, unbuffered byte reads.
pub fn run_echo<R: BufRead, W: Write>(mut reader: R, mut writer: W) -> io::Result<usize> {
    todo!(
        "read lines from `reader` one at a time until EOF, writing each line straight back out \
         to `writer` as it arrives, and return the total number of bytes read"
    )
}

/// Accepts exactly one connection on `listener`, echoes everything sent on
/// it back to the sender until the sender closes their write half (EOF),
/// then returns the number of bytes echoed.
///
/// A real long-running server loops on `listener.incoming()` forever and
/// spawns a thread per connection (see `main.rs`) — `serve_once` handles
/// exactly one connection, which is all a test needs and keeps thread
/// spawning (which isn't itself meaningfully testable) out of this function.
pub fn serve_once(listener: &TcpListener) -> io::Result<usize> {
    todo!(
        "accept one connection from `listener`, get an independent reader and writer handle to \
         it, and run `run_echo` over the pair"
    )
}
