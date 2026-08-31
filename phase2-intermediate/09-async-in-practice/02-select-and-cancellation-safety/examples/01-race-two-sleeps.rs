//! `select!` races several futures against each other on one task. Whichever
//! completes first wins; every other branch's future is dropped immediately,
//! wherever it happened to be.

use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    tokio::select! {
        _ = sleep(Duration::from_millis(200)) => {
            println!("the 200ms sleep won");
        }
        _ = sleep(Duration::from_millis(50)) => {
            println!("the 50ms sleep won");
        }
    }
    println!("select! returned - the 200ms sleep's future is already gone, not just paused");
}
