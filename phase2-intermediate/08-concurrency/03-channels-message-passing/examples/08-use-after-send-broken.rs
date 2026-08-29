//! DELIBERATELY BROKEN — expected: E0382
//!
//! `.send()` moves its argument. Using `readings` again after handing it to
//! `tx.send()` is a use of a value Rust has already moved out from under
//! this binding.
//!
//!     cargo run -p p2-08-03-channels-message-passing --example 08-use-after-send-broken --features broken

use std::sync::mpsc;

fn main() {
    let (tx, rx) = mpsc::channel();
    let readings = vec![12, 47, 8, 33];

    tx.send(readings).unwrap();
    println!("still have it: {readings:?}"); // ERROR: used after move

    let received = rx.recv().unwrap();
    println!("received: {received:?}");
}
