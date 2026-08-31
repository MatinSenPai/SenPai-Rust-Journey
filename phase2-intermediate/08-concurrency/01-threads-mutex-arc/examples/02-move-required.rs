//! The closure passed to `spawn` must own what it uses — `move` is how.
//!
//!     cargo run -p p2-08-01-threads-mutex-arc --example 02-move-required

use std::thread;

fn main() {
    let data = vec![1, 2, 3];

    let handle = thread::spawn(move || {
        println!("the spawned thread owns: {data:?}");
    });

    handle.join().unwrap();
}
