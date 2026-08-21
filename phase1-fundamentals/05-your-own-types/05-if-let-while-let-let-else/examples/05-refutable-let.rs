//! DELIBERATELY BROKEN — expected: E0005
//!
//!     cargo run -p p1-05-05-if-let-while-let-let-else --example 05-refutable-let --features broken
//!
//! A plain `let` must match every time. `Some(score)` does not.

fn main() {
    let rating: Option<u8> = Some(9);
    let Some(score) = rating;
    println!("score: {score}");
}
