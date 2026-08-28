//! DELIBERATELY BROKEN — expected: E0061.
//!
//!     cargo run -p p1-01-04-functions-and-expressions --example 05-missing-argument

fn main() {
    // Rust has no default arguments. Two parameters means two arguments,
    // every time.
    println!("{}", area(3));
}

fn area(width: u32, height: u32) -> u32 {
    width * height
}
