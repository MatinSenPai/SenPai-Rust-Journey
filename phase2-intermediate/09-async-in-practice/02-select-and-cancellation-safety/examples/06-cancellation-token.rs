//! `CancellationToken` (from the separate `tokio-util` crate) lets one part
//! of a program cooperatively ask a running task to stop.

use std::time::Duration;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

async fn worker(token: CancellationToken) -> u32 {
    let mut ticks = 0;
    loop {
        tokio::select! {
            _ = token.cancelled() => {
                println!("worker: cancelled, stopping");
                break;
            }
            _ = sleep(Duration::from_millis(40)) => {
                ticks += 1;
                println!("worker: tick {ticks}");
            }
        }
    }
    ticks
}

#[tokio::main]
async fn main() {
    let token = CancellationToken::new();
    let handle = tokio::spawn(worker(token.clone()));

    sleep(Duration::from_millis(140)).await;
    println!("controller: asking the worker to stop");
    token.cancel();

    let ticks = handle.await.unwrap();
    println!("worker completed {ticks} ticks before stopping");
}
