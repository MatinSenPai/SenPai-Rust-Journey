//! Fixed. The `in_flight` bookkeeping moved outside the future `select!`
//! might drop. `run_one` itself is never raced against anything — it always
//! runs to completion, so its increment and decrement are always paired.

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

async fn fetch(delay_ms: u64) -> String {
    sleep(Duration::from_millis(delay_ms)).await;
    "done".to_string()
}

async fn run_one(in_flight: Arc<Mutex<u32>>, delay_ms: u64, budget_ms: u64) -> Option<String> {
    *in_flight.lock().unwrap() += 1; // outside the raced future: always runs
    let result = tokio::select! {
        data = fetch(delay_ms) => Some(data),
        _ = sleep(Duration::from_millis(budget_ms)) => None,
    };
    *in_flight.lock().unwrap() -= 1; // select! has already returned by this line
    result
}

#[tokio::main]
async fn main() {
    let in_flight = Arc::new(Mutex::new(0u32));
    let budget_ms = 20;
    let attempts = 5;

    for _ in 0..attempts {
        run_one(in_flight.clone(), 200, budget_ms).await;
    }

    println!("attempts made: {attempts}");
    println!("in_flight counter: {}", *in_flight.lock().unwrap());
}
