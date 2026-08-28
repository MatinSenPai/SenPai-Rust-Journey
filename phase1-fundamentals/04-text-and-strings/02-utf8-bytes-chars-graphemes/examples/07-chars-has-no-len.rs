//! DELIBERATELY BROKEN — expected: E0599
//!
//!     cargo run -p p1-04-02-utf8-bytes-chars-graphemes --example 07-chars-has-no-len --features broken
//!
//! `.chars()` is a walk, not a collection. Read what the compiler offers you
//! instead — and then think about whether you want it.

fn main() {
    let word = "سلام";
    println!("letters: {}", word.chars().len());
}
