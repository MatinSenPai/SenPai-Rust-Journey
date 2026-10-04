//! DELIBERATELY BROKEN — expected: a run-time panic (`Lagged(3)` from `unwrap`)
//!
//! A broadcast subscriber that falls behind gets an `Err(Lagged(n))` item.
//! `unwrap()` turns a slow client into a crash.

use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let (tx, rx) = broadcast::channel::<u32>(2);
    for n in 1..=5 {
        tx.send(n).unwrap();
    }
    drop(tx); // no more messages: the stream ends once the room is drained
    let mut stream = BroadcastStream::new(rx);
    while let Some(item) = stream.next().await {
        println!("got {}", item.unwrap());
    }
}
