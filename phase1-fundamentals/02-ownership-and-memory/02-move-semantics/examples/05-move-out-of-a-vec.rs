//! DELIBERATELY BROKEN — expected: E0507.
//!
//!     cargo run -p p1-02-02-move-semantics --example 05-move-out-of-a-vec --features broken

fn main() {
    let lines = vec![String::from("alpha"), String::from("beta")];

    // Taking element zero out would leave a hole in the Vec, and the Vec has
    // no way to represent a hole.
    let first = lines[0];

    println!("{first}");
    println!("{lines:?}");
}
