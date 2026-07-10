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
        "loop: line.clear(); let n = reader.read_line(&mut line)?; if n == 0, break (EOF); \
         otherwise writer.write_all(line.as_bytes())? + writer.flush()?, and add n to a running total. \
         Return Ok(total) once the loop breaks."
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
        "let (stream, _addr) = listener.accept()?; clone it with stream.try_clone()? to get a \
         second handle to the same connection (one for reading, one for writing — see \
         CHECKPOINT.md question 3 for why you can't just reuse one &mut stream for both); wrap \
         the reading half in io::BufReader::new(..); then return run_echo(reader, writer)."
    )
}
