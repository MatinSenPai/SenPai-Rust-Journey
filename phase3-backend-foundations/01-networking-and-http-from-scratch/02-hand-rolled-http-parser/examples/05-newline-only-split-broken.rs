//! DELIBERATELY BROKEN — expected: a run-time panic (stray `\r` baked into
//! the parsed version string).
//! Splits on `\n` alone instead of `\r\n` — a real, easy mistake to make by
//! hand, and exactly why the concept section calls this out.
//!
//!     cargo run -p p3-01-02-hand-rolled-http-parser --example 05-newline-only-split-broken --features broken

fn request_line_version(raw: &str) -> String {
    // BUG: should split on "\r\n", not bare "\n" — see "The concept" above.
    let request_line = raw.split('\n').next().unwrap();
    let mut tokens = request_line.split(' ');
    let _method = tokens.next().unwrap();
    let _target = tokens.next().unwrap();
    tokens.next().unwrap().to_string()
}

fn main() {
    let raw = "GET / HTTP/1.1\r\nHost: localhost:7879\r\n\r\n";
    let version = request_line_version(raw);
    assert_eq!(
        version, "HTTP/1.1",
        "the version should not carry a stray \\r"
    );
    println!("version parsed cleanly: {version:?}");
}
