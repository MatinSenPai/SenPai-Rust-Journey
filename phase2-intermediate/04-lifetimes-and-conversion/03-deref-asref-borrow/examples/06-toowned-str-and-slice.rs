//! `cargo run -p p2-04-03-deref-asref-borrow --example 06-toowned-str-and-slice`
//!
//! `&str` is `Copy`, so `.clone()` on one just copies the reference — the
//! compiler's own `noop_method_call` lint says so. `.to_owned()` is the
//! actual tool: it builds a real, independent `String`.

fn main() {
    let borrowed: &str = "Frieren";

    let still_borrowed: &str = borrowed.clone();
    println!("still borrowed: {still_borrowed}");

    let owned: String = borrowed.to_owned();
    println!("owned: {owned}");

    let numbers: &[i32] = &[1, 2, 3];
    let owned_numbers: Vec<i32> = numbers.to_owned();
    println!("owned_numbers: {owned_numbers:?}");
}
