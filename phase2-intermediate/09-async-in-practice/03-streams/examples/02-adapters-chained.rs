//! `StreamExt` adapters chain exactly like `Iterator` adapters do — lazy,
//! one item at a time, only doing work once something drives the chain
//! with `.next().await`.
//!
//!     cargo run -p p2-09-03-streams --example 02-adapters-chained

use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let mut doubled_evens = tokio_stream::iter(1..=10)
        .filter(|n| n % 2 == 0)
        .map(|n| n * 2);

    while let Some(n) = doubled_evens.next().await {
        println!("{n}");
    }
}
