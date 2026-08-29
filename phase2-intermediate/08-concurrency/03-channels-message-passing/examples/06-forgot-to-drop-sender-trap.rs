//! No compiler error, no panic — forgetting to drop the ORIGINAL `tx` (kept
//! around after handing out a `.clone()`) leaves `rx` waiting for a `Sender`
//! that will never send again. `recv_timeout` proves it is genuinely stuck,
//! instead of actually hanging this example forever.
//!
//!     cargo run -p p2-08-03-channels-message-passing --example 06-forgot-to-drop-sender-trap

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    let (tx, rx) = mpsc::channel::<i32>();
    let worker_tx = tx.clone();

    thread::spawn(move || worker_tx.send(1).unwrap())
        .join()
        .unwrap();

    println!("first recv: {:?}", rx.recv());
    // `tx`, the original, is still alive here — nobody dropped it.
    println!(
        "second recv: {:?}",
        rx.recv_timeout(Duration::from_millis(200))
    );
}
