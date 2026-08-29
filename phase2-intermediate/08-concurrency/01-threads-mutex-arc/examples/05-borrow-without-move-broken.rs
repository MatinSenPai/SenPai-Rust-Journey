//! DELIBERATELY BROKEN — expected: E0373.
//!
//!     cargo run -p p2-08-01-threads-mutex-arc --example 05-borrow-without-move-broken --features broken

use std::thread;

fn main() {
    let data = vec![1, 2, 3];

    let handle = thread::spawn(|| {
        println!("{data:?}");
    });

    handle.join().unwrap();
}
