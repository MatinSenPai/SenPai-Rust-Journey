//! DELIBERATELY BROKEN — expected: E0308
//!
//!     cargo run -p p1-05-05-if-let-while-let-let-else --example 06-else-must-diverge --features broken
//!
//! The `else` of a `let ... else` has one job: leave. This one falls out of
//! the bottom, and then `score` would have no value.

fn main() {
    let rating: Option<u8> = None;
    let Some(score) = rating else {
        println!("no rating yet");
    };
    println!("score: {score}");
}
