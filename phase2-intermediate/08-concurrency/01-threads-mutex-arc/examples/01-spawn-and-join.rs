//! A real OS thread: `thread::spawn`, `JoinHandle`, and `.join()`.
//!
//!     cargo run -p p2-08-01-threads-mutex-arc --example 01-spawn-and-join

use std::thread;

fn main() {
    let handle = thread::spawn(|| {
        println!("hello from the spawned thread");
        2 + 2
    });

    let result = handle.join().unwrap(); // blocks until the thread finishes
    println!("the spawned thread returned: {result}");
}
