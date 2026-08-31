//! DELIBERATELY BROKEN — expected: E0369
//! Atomic types deliberately don't implement `PartialEq` — comparing them
//! directly would silently invite you to read two values that were never
//! actually observed together. `.load()` first, then compare numbers.
//!
//!     cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 12-atomics-no-direct-equality --features broken

use std::sync::atomic::AtomicUsize;

fn main() {
    let a = AtomicUsize::new(1);
    let b = AtomicUsize::new(1);
    if a == b {
        println!("equal");
    }
}
