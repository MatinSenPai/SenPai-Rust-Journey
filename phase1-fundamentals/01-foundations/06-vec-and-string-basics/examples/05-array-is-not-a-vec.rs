//! DELIBERATELY BROKEN — expected: E0308.
//!
//!     cargo run -p p1-01-06-vec-and-string-basics --example 05-array-is-not-a-vec --features broken

fn main() {
    let fixed = [1, 2, 3];

    // They print the same and hold the same numbers, but they are different
    // types living in different places.
    let growable: Vec<i32> = fixed;

    println!("{growable:?}");
}
