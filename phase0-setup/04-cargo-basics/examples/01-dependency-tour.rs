//! A whistle-stop tour of what a dependency buys you. This example uses the
//! `rand` crate declared in this lesson's `Cargo.toml`.
//!
//!     cargo run -p p0-04-cargo-basics --example 01-dependency-tour

fn main() {
    println!("a random number: {}", rand::random::<u8>());
    println!("a coin flip:     {}", rand::random::<bool>());
    println!();
    println!("Neither of those lines is in the standard library.");
    println!("They work because `Cargo.toml` says `rand.workspace = true`.");
}
