//! The exact split points a hand-rolled request parser needs: `\r\n`
//! between lines, a single space inside the request line, `:` inside each
//! header line.
//!
//!     cargo run -p p3-01-02-hand-rolled-http-parser --example 02-splitting-the-wire-format

fn main() {
    let raw = "GET /anime?status=watching HTTP/1.1\r\nHost: localhost:7879\r\nAccept: */*\r\n\r\n";

    let mut lines = raw.split("\r\n");
    let request_line = lines.next().unwrap();
    println!("request line: {request_line:?}");

    let mut tokens = request_line.split(' ');
    println!("method:  {:?}", tokens.next());
    println!("target:  {:?}", tokens.next());
    println!("version: {:?}", tokens.next());

    for line in lines {
        if line.is_empty() {
            println!("(blank line — headers end here)");
            break;
        }
        let (name, value) = line.split_once(':').unwrap();
        println!("header:  {:?} = {:?}", name.trim(), value.trim());
    }
}
