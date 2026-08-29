//! `#[tokio::main]` turns this into a real entry point: it generates an
//! ordinary, synchronous `fn main`, which builds a `tokio` runtime and
//! uses it to drive this `async fn main` body to completion.
//!
//!     cargo run -p p2-08-06-tokio-basics --example 01-tokio-main-sugar

use std::time::Duration;

#[tokio::main]
async fn main() {
    tokio::time::sleep(Duration::from_millis(50)).await;
    println!("done after a real 50ms sleep");
}
