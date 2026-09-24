//! DELIBERATELY BROKEN — expected: never terminates. It never checks
//! `read_line`'s `Ok(0)` (EOF) case, so once the input is exhausted it spins
//! forever instead of stopping. Interrupt with Ctrl+C.
//!
//!     cargo run -p p3-01-01-tcp-echo-server --example 03-missing-eof-check-broken

use std::io::{BufRead, Cursor, Write};

fn main() {
    let mut reader = Cursor::new(b"hello\n".to_vec());
    let mut writer = std::io::stdout();
    let mut line = String::new();

    loop {
        line.clear();
        reader.read_line(&mut line).unwrap(); // bug: the Ok(0) EOF case is never checked
        writer.write_all(line.as_bytes()).unwrap();
    }
}
