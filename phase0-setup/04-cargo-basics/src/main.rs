//! The runnable half of this lesson. `cargo run` executes this file.

use p0_04_cargo_basics::{format_greeting, pick_encouragement};

fn main() {
    // `std::env::args()` hands you the command line. Entry 0 is always the
    // program's own path, so entry 1 is the first real argument you typed.
    // The "there might not be one" part is `Option`, which is Phase 1
    // material — read this as "the name if one was given, otherwise World".
    let name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "World".to_string());

    println!("{}", format_greeting(&name, 2));
    println!();
    println!("{}", pick_encouragement());
}
