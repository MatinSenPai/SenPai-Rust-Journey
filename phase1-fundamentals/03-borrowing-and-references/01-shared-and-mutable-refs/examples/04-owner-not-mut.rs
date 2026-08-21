//! DELIBERATELY BROKEN — expected: E0596.
//!
//!     cargo run -p p1-03-01-shared-and-mutable-refs --example 04-owner-not-mut --features broken
//!
//! You cannot lend out a permission you do not have yourself.

fn main() {
    let greeting = String::from("hello");

    add_exclamation(&mut greeting);

    println!("{greeting}");
}

fn add_exclamation(text: &mut String) {
    text.push('!');
}
