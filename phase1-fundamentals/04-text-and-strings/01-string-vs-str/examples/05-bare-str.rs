//! DELIBERATELY BROKEN — expected: E0277
//!
//!     cargo run -p p1-04-01-string-vs-str --example 05-bare-str --features broken
//!
//! What happens when you write `str` without the `&` in front of it.

fn byte_length(text: str) -> usize {
    text.len()
}

fn main() {
    println!("{}", byte_length(*"سلام"));
}
