//! DELIBERATELY BROKEN — expected: E0599
//! Run `cargo run -p p2-09-03-streams --example 05-forgot-streamext --features broken`
//! and read the error.
//!
//! Unlike `Iterator`, `StreamExt` is not in the prelude — no `use`, no
//! `.next()`.

#[tokio::main]
async fn main() {
    let mut numbers = tokio_stream::iter(vec![1, 2, 3]);
    while let Some(n) = numbers.next().await {
        println!("{n}");
    }
}
