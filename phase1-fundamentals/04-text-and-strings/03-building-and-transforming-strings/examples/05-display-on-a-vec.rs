//! DELIBERATELY BROKEN — expected: E0277
//!
//!     cargo run -p p1-04-03-building-and-transforming-strings --example 05-display-on-a-vec --features broken
//!
//! `{}` asks a value for its human-readable form. A `Vec` does not have one.

fn main() {
    let words = vec!["one", "two"];
    println!("{}", words);
}
