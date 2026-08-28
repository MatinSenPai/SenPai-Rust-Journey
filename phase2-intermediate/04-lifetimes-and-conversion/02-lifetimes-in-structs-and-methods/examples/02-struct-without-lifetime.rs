//! DELIBERATELY BROKEN — expected: E0106
//! Run `cargo run --example 02-struct-without-lifetime --features broken`
//! and read the error.

struct Excerpt {
    text: &str,
}

fn main() {
    let article = String::from("Ownership is central. Borrowing comes next.");
    let excerpt = Excerpt { text: &article };
    println!("{}", excerpt.text);
}
