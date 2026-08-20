use std::io::{self, BufRead, Write};
use std::net::{TcpListener, TcpStream};

/// See `../README.md` for the full walkthrough. Short version: read a line
/// at a time until EOF (`read_line` returning `Ok(0)`), writing each line
/// straight back out, tallying total bytes read.
pub fn run_echo<R: BufRead, W: Write>(mut reader: R, mut writer: W) -> io::Result<usize> {
    let mut total = 0usize;
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            break; // EOF: the peer closed their write half.
        }
        writer.write_all(line.as_bytes())?;
        writer.flush()?;
        total += bytes_read;
    }

    Ok(total)
}

/// Accept exactly one connection and echo it via `run_echo`. `try_clone`
/// gives us two independent handles to the same underlying socket — one we
/// read from, one we write to — sidestepping the "can't borrow the same
/// `TcpStream` as both `&mut R` and `&mut W` at once" problem (see
/// recall questions.md question 3).
pub fn serve_once(listener: &TcpListener) -> io::Result<usize> {
    let (stream, _addr) = listener.accept()?;
    let writer: TcpStream = stream.try_clone()?;
    let reader = io::BufReader::new(stream);
    run_echo(reader, writer)
}
