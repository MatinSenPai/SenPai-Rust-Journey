//! The binary half of this lesson's crate. `src/lib.rs` holds the functions
//! you write; this file is what actually runs when you say `cargo run`.

use p0_03_hello_rust::{describe_journey, progress_bar, shout};

fn main() {
    println!("{}", describe_journey("Rust", 1));
    println!("{}", shout("here we go"));
    println!("{}", progress_bar(1, 7));
}
