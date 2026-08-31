//! Spawn two tasks, let their sleeps run concurrently, then await both
//! handles in turn and report what each found — and how long the whole
//! thing actually took. Only the joining code ever prints, so the output
//! order below is deterministic even though the two tasks race each other.
//!
//!     cargo run -p p2-08-06-tokio-basics --example 03-spawn-two-tasks

use std::time::{Duration, Instant};

async fn fetch(id: u32, delay_ms: u64) -> String {
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    format!("item-{id} (waited {delay_ms}ms)")
}

#[tokio::main]
async fn main() {
    let start = Instant::now();
    let one = tokio::spawn(fetch(1, 200));
    let two = tokio::spawn(fetch(2, 200));

    println!("{}", one.await.unwrap());
    println!("{}", two.await.unwrap());
    println!("total: {:?}", start.elapsed());
}
