//! A chunked body has no `Content-Length` up front — the sender doesn't
//! know the total length yet, maybe because it is still generating the
//! body. Instead: a hex length, that many bytes of data, `\r\n`, repeated,
//! ending in a zero-length chunk. This builds one by hand and shows the
//! raw bytes it actually puts on the wire.
//!
//!     cargo run -p p3-01-03-http-semantics-you-must-know --example 05-chunked-encoding-by-hand

fn push_chunk(data: &str, out: &mut String) {
    out.push_str(&format!("{:X}\r\n", data.len()));
    out.push_str(data);
    out.push_str("\r\n");
}

/// Shows a wire string the way you'd want to read it in a terminal: `\r\n`
/// spelled out, one line per real line, instead of the invisible bytes it
/// actually is.
fn show(wire: &str) -> String {
    wire.replace("\r\n", "\\r\\n\n")
}

fn main() {
    let mut body = String::new();
    push_chunk("Wiki", &mut body);
    push_chunk("pedia", &mut body);
    push_chunk(" in\r\n\r\nchunks.", &mut body);
    body.push_str("0\r\n\r\n"); // the terminating zero-length chunk

    print!("{}", show(&body));
    println!("total wire bytes: {}", body.len());
}
