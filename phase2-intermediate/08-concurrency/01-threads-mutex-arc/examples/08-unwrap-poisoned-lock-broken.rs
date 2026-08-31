//! DELIBERATELY BROKEN — expected: a run-time panic (poisoned lock).
//! Naively `.unwrap()`-ing a poisoned lock just panics again, in whatever
//! thread is unlucky enough to call `.lock()` next.
//!
//!     cargo run -p p2-08-01-threads-mutex-arc --example 08-unwrap-poisoned-lock-broken --features broken

use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let poisoner = Arc::clone(&counter);

    let handle = thread::spawn(move || {
        let mut guard = poisoner.lock().unwrap();
        *guard += 1;
        panic!("simulated failure mid-update");
    });
    let _ = handle.join();

    let guard = counter.lock().unwrap(); // panics: the lock is poisoned
    println!("unreachable: {}", *guard);
}
