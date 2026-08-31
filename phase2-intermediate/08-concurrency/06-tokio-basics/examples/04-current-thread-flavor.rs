//! The same program as 03, but on tokio's OTHER runtime flavor: every
//! task now runs on a single operating-system thread, cooperatively.
//! Concurrent waiting still works — it never needed multiple OS threads.
//!
//!     cargo run -p p2-08-06-tokio-basics --example 04-current-thread-flavor

use std::time::{Duration, Instant};

async fn fetch(id: u32, delay_ms: u64) -> String {
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    format!("item-{id} (waited {delay_ms}ms)")
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let start = Instant::now();
    let one = tokio::spawn(fetch(1, 200));
    let two = tokio::spawn(fetch(2, 200));

    println!("{}", one.await.unwrap());
    println!("{}", two.await.unwrap());
    println!("total: {:?}", start.elapsed());
}
