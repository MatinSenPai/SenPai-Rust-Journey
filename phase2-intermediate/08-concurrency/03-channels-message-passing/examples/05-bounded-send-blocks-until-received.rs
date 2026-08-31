//! `sync_channel(0)` is a rendezvous: `.send()` cannot return until a
//! `.recv()` is there to take the value — real backpressure, not just a
//! bigger queue. A second, ordinary channel acts as a "did send() return
//! yet?" signal so this proves it without guessing at timing.
//!
//!     cargo run -p p2-08-03-channels-message-passing --example 05-bounded-send-blocks-until-received

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    let (tx, rx) = mpsc::sync_channel::<i32>(0);
    let (signal_tx, signal_rx) = mpsc::channel::<()>();

    thread::spawn(move || {
        tx.send(1).unwrap(); // blocks here until main calls rx.recv()
        signal_tx.send(()).unwrap(); // only reachable once send() returns
    });

    thread::sleep(Duration::from_millis(200)); // plenty of time to reach send()
    println!("before recv(): signal so far = {:?}", signal_rx.try_recv());

    let value = rx.recv().unwrap();
    thread::sleep(Duration::from_millis(50)); // let the signal catch up
    println!(
        "after recv() got {value}: signal so far = {:?}",
        signal_rx.try_recv()
    );
}
