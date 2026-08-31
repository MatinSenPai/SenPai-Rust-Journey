//! `slow_sum` is real, synchronous CPU work — no `.await` anywhere inside
//! it. Calling it directly from an `async fn` never yields, so it stalls
//! whatever worker thread it lands on. `current_thread` makes this obvious:
//! there is only one worker, and `ticker` (already spawned, already
//! supposed to be ticking every 50ms) cannot run a single step until
//! `slow_sum` finally returns.
//!
//!     cargo run -p p2-09-04-async-traits-and-blocking --example 03-blocking-call-stalls-the-runtime

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

    println!("heavy result: {}", slow_sum(20_000_000));

    ticks.await.unwrap();
    println!("elapsed: {:?}", start.elapsed());
}
