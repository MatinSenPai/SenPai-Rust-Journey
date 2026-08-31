//! DELIBERATELY BROKEN — expected: E0277
//! Run `cargo run -p p2-09-03-streams --example 06-for-loop-on-stream --features broken`
//! and read the error.
//!
//! A `Stream` is not an `Iterator` — plain `for` never drives it, even
//! with `StreamExt` in scope.

#[tokio::main]
async fn main() {
    let numbers = tokio_stream::iter(vec![1, 2, 3]);
    for n in numbers {
        println!("{n}");
    }
}
