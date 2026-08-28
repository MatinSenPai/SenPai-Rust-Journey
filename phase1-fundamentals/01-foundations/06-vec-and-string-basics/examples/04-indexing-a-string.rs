//! DELIBERATELY BROKEN — expected: E0277.
//!
//!     cargo run -p p1-01-06-vec-and-string-basics --example 04-indexing-a-string --features broken

fn main() {
    let greeting = String::from("hello");

    // Works in Python, works in C, does not work here. A String is bytes, and
    // one byte is not always one character.
    let first = greeting[0];

    println!("{first}");
}
