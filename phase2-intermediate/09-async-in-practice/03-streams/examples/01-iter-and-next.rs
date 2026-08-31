//! `tokio_stream::iter` turns a plain iterator into the simplest possible
//! `Stream`. Driving it is the one idiom the rest of this lesson builds on.
//!
//!     cargo run -p p2-09-03-streams --example 01-iter-and-next

use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let mut numbers = tokio_stream::iter(vec![10, 20, 30]);

    while let Some(n) = numbers.next().await {
        println!("got {n}");
    }
    println!("stream exhausted");
}
