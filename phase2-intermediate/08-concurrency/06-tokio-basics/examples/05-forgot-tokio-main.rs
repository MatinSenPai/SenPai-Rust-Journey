//! DELIBERATELY BROKEN — expected: E0752
//! Run `cargo run -p p2-08-06-tokio-basics --example 05-forgot-tokio-main --features broken`
//! and read the error.
//!
//! `async fn main` on its own is not a real entry point — nothing has
//! built a runtime to drive it. That is the one job `#[tokio::main]` does.

use std::time::Duration;

async fn main() {
    tokio::time::sleep(Duration::from_millis(10)).await;
    println!("done");
}
