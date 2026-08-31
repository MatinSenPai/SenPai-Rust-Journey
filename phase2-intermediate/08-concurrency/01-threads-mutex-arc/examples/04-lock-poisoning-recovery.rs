//! A panic while holding the lock poisons the `Mutex` — and recovering from
//! that poison, on purpose, is a legitimate choice.
//!
//!     cargo run -p p2-08-01-threads-mutex-arc --example 04-lock-poisoning-recovery

use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let poisoner = Arc::clone(&counter);

    // This thread panics WHILE holding the lock.
    let handle = thread::spawn(move || {
        let mut guard = poisoner.lock().unwrap();
        *guard += 1;
        panic!("simulated failure mid-update");
    });
    let _ = handle.join(); // Err — the thread panicked; we don't propagate it

    let value = match counter.lock() {
        Ok(guard) => *guard,
        Err(poisoned) => *poisoned.into_inner(),
    };
    println!("value after recovery: {value}");
}
