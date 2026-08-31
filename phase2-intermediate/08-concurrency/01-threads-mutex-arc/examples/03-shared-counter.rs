//! Ten threads, one counter: `Arc<Mutex<i32>>`.
//!
//!     cargo run -p p2-08-01-threads-mutex-arc --example 03-shared-counter

use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            let mut guard = counter.lock().unwrap();
            *guard += 1;
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("final count: {}", *counter.lock().unwrap());
}
