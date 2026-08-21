//! DELIBERATELY BROKEN — expected: E0308.
//!
//!     cargo run -p p1-01-04-functions-and-expressions --example 04-stray-semicolon

fn main() {
    println!("{}", tripled(5));
}

// One semicolon too many. The body is now a statement and the function is
// worth `()`, which is not a `u32`.
fn tripled(n: u32) -> u32 {
    n * 3;
}
