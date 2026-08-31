//! The lesson's worked example: fan out one task per request through a
//! `JoinSet`, then collect results with `.join_next().await` — in the order
//! the tasks actually finish, not the order we spawned them.
//! Run `cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 03-joinset-completion-order`

use std::time::Duration;
use tokio::task::JoinSet;

async fn fetch(id: u32, delay_ms: u64) -> String {
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    format!("item-{id} (waited {delay_ms}ms)")
}

#[tokio::main]
async fn main() {
    let requests = vec![(1, 150), (2, 10), (3, 80)];
    let mut set = JoinSet::new();
    for (id, delay) in requests {
        set.spawn(fetch(id, delay));
    }

    println!("{} tasks outstanding", set.len());
    while let Some(result) = set.join_next().await {
        println!("finished: {}", result.unwrap());
    }
    println!("{} tasks outstanding", set.len());
}
