//! DELIBERATELY BROKEN — expected: E0308
//!
//!     cargo run -p p1-04-02-utf8-bytes-chars-graphemes --example 06-a-char-is-not-a-string --features broken
//!
//! `.chars()` hands you `char`s. Double quotes make a `&str`. Read the error.

fn main() {
    let word = "سلام";
    let mut seen = 0;

    for letter in word.chars() {
        if letter == "س" {
            seen += 1;
        }
    }

    println!("the letter appears {seen} times");
}
