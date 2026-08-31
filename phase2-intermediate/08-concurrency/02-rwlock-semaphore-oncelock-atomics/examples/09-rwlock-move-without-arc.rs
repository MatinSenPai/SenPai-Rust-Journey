//! DELIBERATELY BROKEN — expected: E0382
//! `RwLock` needs the same `Arc` treatment `Mutex` did in 2.8.1 — a plain
//! `RwLock<T>` still has exactly one owner, and `move` can only give it away
//! once.
//!
//!     cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 09-rwlock-move-without-arc --features broken

use std::sync::RwLock;
use std::thread;

fn main() {
    let cache = RwLock::new(String::from("v1"));
    for _ in 0..3 {
        thread::spawn(move || {
            let guard = cache.read().unwrap();
            println!("{guard}");
        });
    }
}
