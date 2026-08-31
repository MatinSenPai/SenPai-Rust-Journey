//! Recap of 2.8.6: spawn onto a `Vec<JoinHandle<T>>`, then `.await` each
//! handle in the order it was pushed. Results always come back in SPAWN
//! order — never in the order the tasks actually finished.
//! Run `cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 02-vec-joinhandle-spawn-order`

use std::time::Duration;

async fn fetch(id: u32, delay_ms: u64) -> String {
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    format!("item-{id} (waited {delay_ms}ms)")
}

#[tokio::main]
async fn main() {
    let requests = vec![(1, 150), (2, 10), (3, 80)];
    let handles: Vec<_> = requests
        .into_iter()
        .map(|(id, delay)| tokio::spawn(fetch(id, delay)))
        .collect();

    for handle in handles {
        println!("{}", handle.await.unwrap());
    }
}
