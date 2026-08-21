//! DELIBERATELY BROKEN — expected: E0308
//!
//!     cargo run -p p1-04-03-building-and-transforming-strings --example 06-adding-two-strings --features broken
//!
//! `+` on a `String` wants a `&str` on its right-hand side, not a second
//! `String`. The error says so, and offers the one-character fix.

fn main() {
    let first = "report".to_string();
    let second = "-2026".to_string();
    let joined = first + second;
    println!("{joined}");
}
