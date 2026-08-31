//! DELIBERATELY BROKEN — expected: E0597.
//! A `MutexGuard` is a borrow, exactly like `Ref`/`RefMut` from 2.6.5 — a
//! bare tail expression that dereferences one, at the very end of a block
//! that owns the `Mutex` only locally, hits the same "does this live long
//! enough" question any other reference would.
//!
//!     cargo run -p p2-08-01-threads-mutex-arc --example 07-bare-tail-guard-broken --features broken

use std::sync::{Arc, Mutex};

fn current_value() -> i32 {
    let counter = Arc::new(Mutex::new(5));
    *counter.lock().unwrap()
}

fn main() {
    println!("{}", current_value());
}
