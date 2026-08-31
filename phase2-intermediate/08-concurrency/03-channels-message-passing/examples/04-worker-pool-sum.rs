//! The worked example this lesson builds toward: several worker threads,
//! each handed one chunk of the data, sending exactly one partial result
//! back to a single collecting thread — the shape mpsc's own name
//! describes: multi-producer, single-consumer.
//!
//!     cargo run -p p2-08-03-channels-message-passing --example 04-worker-pool-sum

use std::sync::mpsc;
use std::thread;

fn main() {
    let minutes_watched = vec![24, 24, 23, 45, 24, 24, 24, 22, 47, 24, 24, 21];
    let worker_count = 4;
    let chunk_size = minutes_watched.len() / worker_count;

    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::new();
    for chunk in minutes_watched.chunks(chunk_size) {
        let tx = tx.clone();
        let chunk = chunk.to_vec();
        handles.push(thread::spawn(move || {
            let partial: i32 = chunk.iter().sum();
            tx.send(partial).unwrap();
        }));
    }
    drop(tx); // the original — without this, the loop below never ends

    let mut partial_count = 0;
    let mut total = 0;
    for partial in rx {
        total += partial;
        partial_count += 1;
    }
    for handle in handles {
        handle.join().unwrap();
    }
    println!("{partial_count} workers reported in, grand total: {total}");
}
