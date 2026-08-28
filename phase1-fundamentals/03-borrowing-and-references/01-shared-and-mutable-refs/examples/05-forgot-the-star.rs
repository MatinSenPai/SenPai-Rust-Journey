//! DELIBERATELY BROKEN — expected: E0308.
//!
//!     cargo run -p p1-03-01-shared-and-mutable-refs --example 05-forgot-the-star --features broken
//!
//! Assigning to the arrow instead of to what it points at.

fn main() {
    let mut count = 10;

    reset(&mut count);

    println!("{count}");
}

fn reset(counter: &mut i32) {
    counter = 0;
}
