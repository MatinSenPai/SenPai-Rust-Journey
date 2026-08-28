//! DELIBERATELY BROKEN — expected: E0425
//!
//!     cargo run -p p1-04-03-building-and-transforming-strings --example 07-a-name-that-is-not-there --features broken
//!
//! An inline `{name}` in a format string is a real variable lookup, so a typo
//! inside the braces is a real name error.

fn main() {
    let name = "ماتین";
    println!("hello {nmae}");
    println!("{name}");
}
