//! The canonical use of `select!`: race a real operation against a deadline.

use std::time::Duration;
use tokio::time::sleep;

async fn fetch(delay_ms: u64) -> String {
    sleep(Duration::from_millis(delay_ms)).await;
    format!("data after {delay_ms}ms")
}

async fn fetch_with_budget(delay_ms: u64, budget_ms: u64) -> Option<String> {
    tokio::select! {
        data = fetch(delay_ms) => Some(data),
        _ = sleep(Duration::from_millis(budget_ms)) => None,
    }
}

#[tokio::main]
async fn main() {
    match fetch_with_budget(20, 100).await {
        Some(data) => println!("fast fetch: got {data:?}"),
        None => println!("fast fetch: timed out"),
    }
    match fetch_with_budget(300, 100).await {
        Some(data) => println!("slow fetch: got {data:?}"),
        None => println!("slow fetch: timed out"),
    }
}
