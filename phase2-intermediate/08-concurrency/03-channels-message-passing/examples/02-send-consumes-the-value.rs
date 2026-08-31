//! `Sender::send` takes its argument BY VALUE — it moves the value into the
//! channel. Nothing usable is left behind at the send site; whoever calls
//! `.recv()` gets full, independent ownership back.
//!
//!     cargo run -p p2-08-03-channels-message-passing --example 02-send-consumes-the-value

use std::sync::mpsc;

fn main() {
    let (tx, rx) = mpsc::channel();
    let readings = vec![12, 47, 8, 33];

    tx.send(readings).unwrap(); // `readings` moves into the channel here
                                // `readings` cannot be named again from this point on.

    let mut received = rx.recv().unwrap();
    received.push(100); // owned outright — free to mutate
    println!("received, now owned and mutated: {received:?}");
}
