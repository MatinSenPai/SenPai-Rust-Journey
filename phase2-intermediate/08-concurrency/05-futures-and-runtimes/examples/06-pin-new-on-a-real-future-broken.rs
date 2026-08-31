//! DELIBERATELY BROKEN — expected: E0277
//! Run `cargo run --example 06-pin-new-on-a-real-future-broken --features broken`
//! and read the error.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Waker};

async fn add_one(x: u32) -> u32 {
    x + 1
}

fn main() {
    let mut future = add_one(41);
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);

    let result = Pin::new(&mut future).poll(&mut cx);
    println!("{result:?}");
}
