//! `tokio::join!`: wait on a fixed, known-up-front set of futures, all on
//! the SAME task.
//! Run `cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 01-join-two-futures`

use std::time::{Duration, Instant};

async fn fetch(id: u32, delay_ms: u64) -> String {
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    format!("item-{id} (waited {delay_ms}ms)")
}

#[tokio::main]
async fn main() {
    let start = Instant::now();
    let (a, b) = tokio::join!(fetch(1, 150), fetch(2, 50));
    println!("{a}");
    println!("{b}");
    println!("total: {:?}", start.elapsed());
}
