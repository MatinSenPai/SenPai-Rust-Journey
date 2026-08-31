//! `tokio::time::timeout` is the built-in shorthand for exactly the pattern
//! in `02-fetch-with-timeout.rs`: race a future against a deadline, hand back
//! a `Result` (`Ok` = finished in time, `Err` = the deadline won) instead of
//! a hand-written `select!`.

use std::time::Duration;
use tokio::time::sleep;

async fn fetch(delay_ms: u64) -> String {
    sleep(Duration::from_millis(delay_ms)).await;
    format!("data after {delay_ms}ms")
}

#[tokio::main]
async fn main() {
    match tokio::time::timeout(Duration::from_millis(100), fetch(20)).await {
        Ok(data) => println!("fast fetch: got {data:?}"),
        Err(_) => println!("fast fetch: timed out"),
    }
    match tokio::time::timeout(Duration::from_millis(100), fetch(300)).await {
        Ok(data) => println!("slow fetch: got {data:?}"),
        Err(_) => println!("slow fetch: timed out"),
    }
}
