//! DELIBERATELY BROKEN — expected: E0277
//! Run `cargo run -p p3-07-02-sessions-vs-jwt --example 05-clock-with-cell-not-sync-broken --features broken`
//! and read the error.

use std::cell::Cell;
use std::sync::Arc;
use std::thread;

struct ManualClock(Cell<u64>);

impl ManualClock {
    fn now(&self) -> u64 {
        self.0.get()
    }
}

fn main() {
    let clock = Arc::new(ManualClock(Cell::new(1_000)));
    let for_handler = Arc::clone(&clock);
    // A handler runs on another thread and reads the clock.
    let t = thread::spawn(move || for_handler.now());
    clock.0.set(2_000);
    println!("{}", t.join().unwrap());
}
