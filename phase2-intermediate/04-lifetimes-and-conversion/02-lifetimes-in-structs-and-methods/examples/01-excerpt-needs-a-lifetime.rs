//! `Excerpt` borrows a slice of a string it does not own. The struct
//! itself has to say, in its own type, how long that borrow is good for.
//!
//! Run:
//! `cargo run -p p2-04-02-lifetimes-in-structs-and-methods --example 01-excerpt-needs-a-lifetime`

struct Excerpt<'a> {
    text: &'a str,
}

fn main() {
    let article = String::from("Ownership is central. Borrowing comes next.");
    let excerpt = Excerpt { text: &article };

    println!("excerpt: {}", excerpt.text);
    println!("source:  {article}");
}
