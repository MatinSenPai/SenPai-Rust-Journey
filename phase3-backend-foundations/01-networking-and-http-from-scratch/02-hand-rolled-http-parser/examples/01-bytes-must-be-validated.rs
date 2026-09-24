//! `&[u8]` off a socket isn't `&str` until you check it: `std::str::from_utf8`
//! is the first thing any byte-oriented parser reaches for.
//!
//!     cargo run -p p3-01-02-hand-rolled-http-parser --example 01-bytes-must-be-validated

// A client (or an attacker) can send whatever bytes it wants — 0xff is not a
// valid start of any UTF-8 sequence. A function call keeps the compiler from
// spotting this as an obviously-invalid literal and warning about it — a
// real socket read has exactly this same "who knows what's actually in
// here" shape.
fn bytes_off_a_socket() -> Vec<u8> {
    vec![0x47, 0x45, 0x54, 0xff, 0xfe]
}

fn main() {
    let good: &[u8] = b"GET / HTTP/1.1";
    match std::str::from_utf8(good) {
        Ok(text) => println!("valid:   {text:?}"),
        Err(e) => println!("invalid: {e}"),
    }

    let bad = bytes_off_a_socket();
    match std::str::from_utf8(&bad) {
        Ok(text) => println!("valid:   {text:?}"),
        Err(e) => println!("invalid: {e}"),
    }
}
