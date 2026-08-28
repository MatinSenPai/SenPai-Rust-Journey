//! A standalone preview of the crate you're about to build in `src/lib.rs`
//! — a local copy, not an import (see 02 for why that distinction matters).
//! `main` calls the private-looking `round1` directly: both live in this
//! same file, so nothing stops it. That is the exact trick a unit test
//! relies on.

fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

fn celsius_to_fahrenheit(c: f64) -> f64 {
    round1(c * 9.0 / 5.0 + 32.0)
}

fn main() {
    println!("0C  -> {}F", celsius_to_fahrenheit(0.0));
    println!("37C -> {}F", celsius_to_fahrenheit(37.0));
    println!("round1(3.14159) called directly: {}", round1(3.14159));
}
