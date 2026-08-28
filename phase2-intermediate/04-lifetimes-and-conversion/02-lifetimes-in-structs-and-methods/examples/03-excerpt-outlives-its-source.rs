//! DELIBERATELY BROKEN — expected: E0597
//! Run `cargo run --example 03-excerpt-outlives-its-source --features broken`
//! and read the error.

struct Excerpt<'a> {
    text: &'a str,
}

fn main() {
    let excerpt;
    {
        let article = String::from("Ownership is central. Borrowing comes next.");
        excerpt = Excerpt { text: &article };
    }
    println!("first line: {}", excerpt.text);
}
