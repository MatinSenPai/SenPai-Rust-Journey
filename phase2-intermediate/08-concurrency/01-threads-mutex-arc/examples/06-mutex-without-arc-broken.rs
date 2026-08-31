//! DELIBERATELY BROKEN — expected: E0382.
//!
//!     cargo run -p p2-08-01-threads-mutex-arc --example 06-mutex-without-arc-broken --features broken

use std::sync::Mutex;
use std::thread;

fn main() {
    let counter = Mutex::new(0);

    thread::spawn(move || {
        *counter.lock().unwrap() += 1;
    });

    thread::spawn(move || {
        *counter.lock().unwrap() += 1;
    });
}
