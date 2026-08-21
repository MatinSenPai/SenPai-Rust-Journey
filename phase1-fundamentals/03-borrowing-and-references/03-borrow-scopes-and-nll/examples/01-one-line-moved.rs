//! 1.3.3 — the same three statements, and why the order of two of them
//! decides whether this file compiles at all.
//!
//! `cargo run -p p1-03-03-borrow-scopes-and-nll --example 01-one-line-moved`

fn main() {
    let mut names = vec![String::from("Matin")];

    // The borrow starts here...
    let peek = &names;
    // ...and is finished on the next line, because nothing reads `peek` after
    // it. That is the whole of today's lesson.
    println!("before the push: {} name(s)", peek.len());

    names.push(String::from("Sora"));
    println!("after the push:  {} name(s)", names.len());

    println!();
    println!("Now move the `peek.len()` line below the push and rebuild.");
    println!("The result is examples/05-used-after-the-push.rs.");
}
