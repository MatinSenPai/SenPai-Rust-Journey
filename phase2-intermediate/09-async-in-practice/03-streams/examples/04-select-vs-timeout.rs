//! `select!` (2.9.2) races a stream's `.next().await` against any other
//! future exactly like it races two plain futures — here, a timeout.
//!
//!     cargo run -p p2-09-03-streams --example 04-select-vs-timeout

use std::time::Duration;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let ticks = tokio_stream::iter([30u64, 30, 30]).then(|delay_ms| async move {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        delay_ms
    });
    tokio::pin!(ticks);

    loop {
        tokio::select! {
            item = ticks.next() => match item {
                Some(delay_ms) => println!("tick (waited {delay_ms}ms)"),
                None => {
                    println!("stream ended");
                    break;
                }
            },
            _ = tokio::time::sleep(Duration::from_millis(200)) => {
                println!("timed out waiting for the next tick");
                break;
            }
        }
    }
}
