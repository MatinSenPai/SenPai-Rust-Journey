//! DELIBERATELY BROKEN — expected: E0507.
//!
//!     cargo run -p p1-03-01-shared-and-mutable-refs --example 06-taking-through-a-shared-ref --features broken
//!
//! A borrow lets you look. It does not let you take.

fn main() {
    let owned = String::from("hello");

    println!("{}", stolen(&owned));
}

fn stolen(text: &String) -> String {
    let mine = *text;
    mine
}
