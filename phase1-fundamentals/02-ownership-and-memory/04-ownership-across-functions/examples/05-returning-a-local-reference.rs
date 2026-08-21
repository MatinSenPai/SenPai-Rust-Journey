//! DELIBERATELY BROKEN — expected: E0106.
//!
//!     cargo run -p p1-02-04-ownership-across-functions --example 05-returning-a-local-reference --features broken

fn main() {
    println!("{}", make_greeting());
}

/// A first attempt at "return it without giving ownership away". The String
/// is made here and dies here, so a reference to it would outlive it.
fn make_greeting() -> &String {
    let greeting = String::from("hello");
    &greeting
}
