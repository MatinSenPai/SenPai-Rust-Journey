//! DELIBERATELY BROKEN — expected: E0204.
//!
//!     cargo run -p p1-02-03-clone-and-copy --example 04-copy-needs-no-heap --features broken
//!
//! `struct` gets its own lesson in 1.5.1. All you need here is that this
//! declares a type made of two fields.

#[derive(Clone, Copy)]
struct Reading {
    value: i32,
    label: String,
}

fn main() {
    let first = Reading {
        value: 1,
        label: String::from("start"),
    };
    let second = first;
    println!("{} {}", second.value, second.label);
}
