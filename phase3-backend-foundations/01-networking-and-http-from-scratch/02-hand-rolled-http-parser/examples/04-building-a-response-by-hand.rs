//! The mirror image of parsing: turn Rust data back into HTTP/1.1 wire
//! bytes, with a computed `Content-Length` included.
//!
//!     cargo run -p p3-01-02-hand-rolled-http-parser --example 04-building-a-response-by-hand

fn response_bytes(status: u16, reason: &str, headers: &[(&str, &str)], body: &str) -> Vec<u8> {
    let mut out = format!("HTTP/1.1 {status} {reason}\r\n");
    for (name, value) in headers {
        out.push_str(&format!("{name}: {value}\r\n"));
    }
    out.push_str(&format!("Content-Length: {}\r\n", body.len()));
    out.push_str("\r\n");
    out.push_str(body);
    out.into_bytes()
}

fn main() {
    let bytes = response_bytes(
        200,
        "OK",
        &[("Content-Type", "text/plain")],
        "Hello, world!\n",
    );
    print!("{}", String::from_utf8(bytes).unwrap());
}
