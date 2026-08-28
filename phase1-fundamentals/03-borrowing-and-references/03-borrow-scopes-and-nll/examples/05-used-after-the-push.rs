//! DELIBERATELY BROKEN — expected: E0502
//! Run `cargo run -p p1-03-03-borrow-scopes-and-nll \
//!   --example 05-used-after-the-push --features broken` and read the error.
//!
//! This is examples/01-one-line-moved.rs with one line moved down.

fn main() {
    let mut names = vec![String::from("Matin")];

    let peek = &names;

    names.push(String::from("Sora"));
    println!("before the push: {} name(s)", peek.len());
    println!("after the push:  {} name(s)", names.len());
}
