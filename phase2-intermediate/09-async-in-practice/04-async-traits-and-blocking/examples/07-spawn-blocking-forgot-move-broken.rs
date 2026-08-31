//! DELIBERATELY BROKEN — expected: E0373
//! Run `cargo run -p p2-09-04-async-traits-and-blocking --example 07-spawn-blocking-forgot-move-broken --features broken`
//! and read the error.
//!
//! `spawn_blocking`'s closure runs on a different thread, possibly long
//! after this function returns, so it must own everything it touches —
//! `'static`, the same requirement `tokio::spawn` has always had. Borrowing
//! `n` instead of moving it breaks that promise.

fn slow_sum(n: u64) -> u64 {
    let mut total: u64 = 0;
    for i in 0..n {
        total = total.wrapping_add(i);
    }
    total
}

#[tokio::main]
async fn main() {
    let n: u64 = 20_000_000;
    let handle = tokio::task::spawn_blocking(|| slow_sum(n));
    let result = handle.await.unwrap();
    println!("result: {result}");
}
