//! DELIBERATELY BROKEN — expected: E0308
//! Run `cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 07-forgot-await-on-join-next --features broken`
//! and read the error.
//!
//! `.join_next()` returns a `Future`, not an `Option` — you still have to
//! `.await` it, same as any other async call. Forgetting to is an easy typo
//! once `while let Some(...) = ...` starts feeling like a normal iterator.

use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    let mut set: JoinSet<u32> = JoinSet::new();
    set.spawn(async { 1 + 1 });

    while let Some(result) = set.join_next() {
        println!("{result:?}");
    }
}
