//! DELIBERATELY BROKEN — expected: E0308
//!
//!     cargo run -p p1-04-01-string-vs-str --example 04-string-where-str-wanted --features broken
//!
//! Two mistakes, one in each direction. Read both errors before fixing either.

fn byte_length(text: &str) -> usize {
    text.len()
}

fn main() {
    let owned = String::from("سلام");

    // Downwards: an owner where a view was asked for.
    println!("{}", byte_length(owned));

    // Upwards: a view where an owner was asked for.
    let copy: String = "سلام";
    println!("{copy}");
}
