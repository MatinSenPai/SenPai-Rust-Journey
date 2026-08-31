//! A spawned task's failure shows up as `Err(JoinError)` from
//! `.join_next()` — here because it panicked. (A cancelled task, [2.9.2],
//! produces the same `Err(JoinError)` shape, just for a different reason.)
//! Run `cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 04-joinset-task-panics`
//!
//! Three tasks below never `.await` anything, so they all finish on their
//! very first poll — which one of the three `join_next()` hands back first
//! is not guaranteed by anything this lesson promised. Expect one panic
//! message on stderr, and three "finished: ..." lines on stdout in SOME
//! order.

use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    let mut set = JoinSet::new();
    set.spawn(async { 1 + 1 });
    set.spawn(async {
        panic!("simulated failure inside a spawned task");
    });
    set.spawn(async { 2 + 2 });

    while let Some(result) = set.join_next().await {
        match result {
            Ok(value) => println!("finished: ok({value})"),
            Err(err) => println!("finished: panicked = {}", err.is_panic()),
        }
    }
}
