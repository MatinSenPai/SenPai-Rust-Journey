//! Every status line has the same three fields — version, code, reason
//! phrase — the same shape 3.1.2 built for `200 OK`. Building a few more by
//! hand makes the difference between "similar" codes concrete: this prints
//! the actual wire text (`\r\n` shown, not a real line break) for status
//! codes people mix up.
//!
//!     cargo run -p p3-01-03-http-semantics-you-must-know --example 02-status-line-and-reason-phrases

fn status_line(code: u16, reason: &str) -> String {
    format!("HTTP/1.1 {code} {reason}\r\n")
}

/// Shows a wire string the way you'd want to read it in a terminal: `\r\n`
/// spelled out instead of the invisible bytes it actually is.
fn show(wire: &str) -> String {
    wire.replace("\r\n", "\\r\\n")
}

fn main() {
    let lines = [
        status_line(201, "Created"),
        status_line(204, "No Content"),
        status_line(301, "Moved Permanently"),
        status_line(302, "Found"),
        status_line(307, "Temporary Redirect"),
        status_line(308, "Permanent Redirect"),
        status_line(400, "Bad Request"),
        status_line(401, "Unauthorized"),
        status_line(403, "Forbidden"),
        status_line(409, "Conflict"),
        status_line(422, "Unprocessable Entity"),
        status_line(429, "Too Many Requests"),
        status_line(500, "Internal Server Error"),
        status_line(502, "Bad Gateway"),
        status_line(503, "Service Unavailable"),
    ];
    for line in &lines {
        println!("{}", show(line));
    }
}
