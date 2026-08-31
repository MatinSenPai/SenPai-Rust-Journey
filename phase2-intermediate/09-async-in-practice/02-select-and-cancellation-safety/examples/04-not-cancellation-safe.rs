//! NOT cancellation-safe. The `in_flight` bookkeeping lives inside the same
//! future `select!` might drop, so losing the race leaves it permanently
//! wrong — no panic, no error, nothing to catch.

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

async fn tracked_fetch(in_flight: Arc<Mutex<u32>>, delay_ms: u64) -> String {
    *in_flight.lock().unwrap() += 1; // "a fetch just started"
    sleep(Duration::from_millis(delay_ms)).await; // the actual work
    *in_flight.lock().unwrap() -= 1; // "it finished" — never runs if dropped first
    "done".to_string()
}

#[tokio::main]
async fn main() {
    let in_flight = Arc::new(Mutex::new(0u32));
    let budget_ms = 20;
    let attempts = 5;

    for _ in 0..attempts {
        tokio::select! {
            _ = tracked_fetch(in_flight.clone(), 200) => {}
            _ = sleep(Duration::from_millis(budget_ms)) => {}
        }
    }

    println!("attempts made: {attempts}");
    println!("in_flight counter: {}", *in_flight.lock().unwrap());
}
