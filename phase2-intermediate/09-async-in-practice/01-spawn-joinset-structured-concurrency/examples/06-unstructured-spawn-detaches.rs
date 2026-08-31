//! Contrast with `05`: dropping a bare `JoinHandle` does NOT touch its
//! task. `tokio::spawn` on its own is unstructured — the task keeps running
//! fully detached from whoever spawned it.
//! Run `cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 06-unstructured-spawn-detaches`

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let ran = Arc::new(AtomicBool::new(false));
    {
        let flag = Arc::clone(&ran);
        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            flag.store(true, Ordering::SeqCst);
            println!("unstructured task: finished sleeping");
        });
        drop(handle); // dropping the HANDLE does not cancel the TASK
    }

    tokio::time::sleep(Duration::from_millis(200)).await;
    println!("unstructured task ran: {}", ran.load(Ordering::SeqCst));
}
