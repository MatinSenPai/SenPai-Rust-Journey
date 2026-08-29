//! DELIBERATELY BROKEN — expected: a run-time panic, on the second `.unwrap()`
//! `OnceLock::set` only succeeds the first time; every later call finds the
//! slot already full and returns `Err` instead of overwriting it.
//!
//!     cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 11-oncelock-set-twice-panics --features broken

use std::sync::OnceLock;

fn main() {
    let config: OnceLock<String> = OnceLock::new();
    config.set(String::from("first")).unwrap();
    config.set(String::from("second")).unwrap();
}
