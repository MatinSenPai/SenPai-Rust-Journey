//! A stream whose items really do arrive over time: each delay in the
//! source list becomes one real `tokio::time::sleep`, run one at a time,
//! before that item is yielded.
//!
//!     cargo run -p p2-09-03-streams --example 03-time-spaced-stream

use std::time::{Duration, Instant};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let start = Instant::now();
    let ticks = tokio_stream::iter([40u64, 40, 40]).then(|delay_ms| async move {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        delay_ms
    });
    tokio::pin!(ticks);

    while let Some(delay_ms) = ticks.next().await {
        println!(
            "tick (waited {delay_ms}ms) — elapsed so far: {:?}",
            start.elapsed()
        );
    }
}
