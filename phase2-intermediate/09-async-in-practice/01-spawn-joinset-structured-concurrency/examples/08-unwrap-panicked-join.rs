//! DELIBERATELY BROKEN — expected: a run-time panic when `.unwrap()`ing the
//! `Err(JoinError)` that comes back from a task that panicked.
//! Run `cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 08-unwrap-panicked-join --features broken`
//! and read the panic.

use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    let mut set: JoinSet<String> = JoinSet::new();
    set.spawn(async {
        panic!("simulated failure inside a spawned task");
    });

    let result = set.join_next().await.unwrap();
    println!("{}", result.unwrap());
}
