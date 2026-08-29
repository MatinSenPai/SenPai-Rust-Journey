//! `Sender<T>` implements `Clone` — that is the "multi-producer" half of
//! mpsc. Each clone feeds the exact same `Receiver`; the two threads below
//! never touch each other's memory, they just each own a handle to send
//! through.
//!
//!     cargo run -p p2-08-03-channels-message-passing --example 03-clone-sender-multiple-producers

use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();
    let tx_a = tx.clone();
    let tx_b = tx.clone();
    drop(tx); // only the two clones below will ever send

    thread::spawn(move || tx_a.send("thread A").unwrap());
    thread::spawn(move || tx_b.send("thread B").unwrap());

    let mut received: Vec<&str> = vec![rx.recv().unwrap(), rx.recv().unwrap()];
    received.sort();
    println!("received, sorted for a stable printout: {received:?}");
}
