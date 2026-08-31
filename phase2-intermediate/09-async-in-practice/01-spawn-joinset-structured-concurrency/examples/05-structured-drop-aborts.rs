//! Structured concurrency, made observable: dropping a `JoinSet` aborts
//! every task still inside it — the sleeping task below never gets to run
//! its post-sleep `println!`.
//! Run `cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 05-structured-drop-aborts`

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    let ran = Arc::new(AtomicBool::new(false));
    {
        let flag = Arc::clone(&ran);
        let mut set = JoinSet::new();
        set.spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            flag.store(true, Ordering::SeqCst);
            println!("structured task: finished sleeping"); // never prints
        });
        // `set` drops right here — well before the 100ms sleep is over.
    }

    tokio::time::sleep(Duration::from_millis(200)).await;
    println!("structured task ran: {}", ran.load(Ordering::SeqCst));
}
