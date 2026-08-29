//! DELIBERATELY BROKEN — expected: E0594
//! A `RwLockReadGuard` only ever hands out `&T` — writing through it is
//! exactly as illegal as writing through any other shared reference.
//!
//!     cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 10-write-through-read-guard --features broken

use std::sync::RwLock;

fn main() {
    let counter = RwLock::new(0);
    let guard = counter.read().unwrap();
    *guard += 1;
    println!("{guard}");
}
