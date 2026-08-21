//! `Copy` is not a faster clone. It is a different promise.
//!
//!     cargo run -p p1-02-03-clone-and-copy --example 02-copy-types

fn main() {
    // For a `Copy` type, assignment duplicates the value and both bindings
    // stay usable. No method call, no `.clone()`, nothing written by you.
    let a = 5_i32;
    let b = a;
    println!("a: {a}, b: {b}");

    // `.clone()` also works on them, because every `Copy` type is also
    // `Clone`. It is exactly the same operation with a longer name, and
    // clippy will tell you to drop it.
    let c = a.clone();
    println!("c: {c}");

    // What makes a type `Copy`: duplicating its bytes duplicates the whole
    // value, and there is nothing to clean up afterwards.
    println!();
    println!("Copy:     i32 u8 usize f64 bool char, &T, arrays and tuples of those");
    println!("not Copy: String, Vec<T>, and anything holding one");

    // A reference is `Copy` even when what it points at is not. Copying the
    // arrow does not copy the thing at the end of it.
    let owned = String::from("hello");
    let first = &owned;
    let second = first;
    println!();
    println!("both refs work: {first} / {second}");

    // `Clone` without `Copy` is the common case: duplication is possible but
    // costs something, so you have to ask for it by name.
    let text = String::from("hello");
    let duplicate = text.clone();
    println!("cloned:   {text} / {duplicate}");

    // `Copy` without `Clone` does not exist. Every `Copy` type is `Clone`
    // too — the language requires it.
    println!();
    println!("Copy implies Clone. The reverse is not true.");
}
