//! One thread computes something and sends the single result back through a
//! channel — `rx.recv()` blocks the calling thread until that value (or a
//! disconnect) arrives.
//!
//!     cargo run -p p2-08-03-channels-message-passing --example 01-one-shot-handoff

use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let result = 6 * 7;
        tx.send(result).unwrap();
    });

    let answer = rx.recv().unwrap();
    println!("main thread received: {answer}");
}
