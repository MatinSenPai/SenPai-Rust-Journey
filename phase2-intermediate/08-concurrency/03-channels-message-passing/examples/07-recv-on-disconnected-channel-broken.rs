//! DELIBERATELY BROKEN — expected: a run-time panic, "called
//! `Result::unwrap()` on an `Err` value: RecvError"
//!
//! Every `Sender` is dropped before anything is ever sent, so `rx.recv()`
//! returns `Err` immediately instead of blocking forever — `.unwrap()` on
//! that `Err` is what actually panics here.
//!
//!     cargo run -p p2-08-03-channels-message-passing --example 07-recv-on-disconnected-channel-broken --features broken

use std::sync::mpsc;

fn main() {
    let (tx, rx) = mpsc::channel::<i32>();
    drop(tx); // no Sender left, and nothing was ever sent

    let value = rx.recv().unwrap();
    println!("{value}");
}
