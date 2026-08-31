//! Same `slow_sum`, same `ticker`, same single-worker `current_thread`
//! runtime as `03` — the only change is `spawn_blocking`. It hands
//! `slow_sum` to tokio's separate blocking thread pool instead of running it
//! on the worker thread, so `ticker`'s sleeps keep firing while the heavy
//! work runs elsewhere. How many ticks land before "heavy result" varies
//! run to run — real wall-clock timing between two threads, not a
//! guarantee — but at least one always does, which never happens in `03`.
//!
//!     cargo run -p p2-09-04-async-traits-and-blocking --example 04-spawn-blocking-fixes-it

use std::time::{Duration, Instant};

fn slow_sum(n: u64) -> u64 {
    let mut total: u64 = 0;
    for i in 0..n {
        total = total.wrapping_add(i);
    }
    total
}

async fn ticker() {
    for tick in 1..=3 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        println!("tick {tick}");
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let start = Instant::now();
    let ticks = tokio::spawn(ticker());

    let heavy = tokio::task::spawn_blocking(|| slow_sum(20_000_000))
        .await
        .unwrap();
    println!("heavy result: {heavy}");

    ticks.await.unwrap();
    println!("elapsed: {:?}", start.elapsed());
}
