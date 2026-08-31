//! DELIBERATELY BROKEN — expected: E0308
//! Run `cargo run -p p2-09-02-select-and-cancellation-safety --example
//! 10-mismatched-types-broken --features broken` and read the error.

use std::time::Duration;
use tokio::time::sleep;

async fn fetch() -> &'static str {
    sleep(Duration::from_millis(10)).await;
    "ok"
}

#[tokio::main]
async fn main() {
    let result = tokio::select! {
        data = fetch() => data,
        _ = sleep(Duration::from_millis(50)) => 404,
    };
    println!("{result:?}");
}
