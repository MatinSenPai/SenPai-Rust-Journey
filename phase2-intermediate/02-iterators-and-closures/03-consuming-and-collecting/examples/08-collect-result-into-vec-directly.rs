//! DELIBERATELY BROKEN — expected: E0277
//!
//! `.parse::<i32>()` produces `Result<i32, ParseIntError>`, not `i32` — so
//! this iterator's items are `Result<i32, ParseIntError>`. `Vec<i32>` only
//! knows how to be built from an iterator of `i32` directly; it has no idea
//! what to do with the `Result` wrapper. Asking for `Result<Vec<i32>, _>`
//! instead is the fix — see the lesson body.
//!
//!     cargo run -p p2-02-03-consuming-and-collecting --example 08-collect-result-into-vec-directly --features broken

fn main() {
    let inputs = ["1", "2", "x"];
    let parsed: Vec<i32> = inputs.iter().map(|s| s.parse::<i32>()).collect();
    println!("{parsed:?}");
}
