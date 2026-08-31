//! DELIBERATELY BROKEN — expected: E0499
//! Run `cargo run -p p2-09-02-select-and-cancellation-safety --example
//! 09-double-borrow-broken --features broken` and read the error.

use std::time::Duration;
use tokio::time::sleep;

async fn add_one(total: &mut u32) {
    sleep(Duration::from_millis(10)).await;
    *total += 1;
}

async fn add_two(total: &mut u32) {
    sleep(Duration::from_millis(20)).await;
    *total += 2;
}

#[tokio::main]
async fn main() {
    let mut total = 0u32;
    tokio::select! {
        _ = add_one(&mut total) => {}
        _ = add_two(&mut total) => {}
    }
    println!("total = {total}");
}
