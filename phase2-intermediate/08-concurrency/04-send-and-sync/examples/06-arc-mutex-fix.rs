//! The fix for both broken examples above: `Arc<Mutex<i32>>` instead of
//! `Rc<RefCell<i32>>` or `Arc<RefCell<i32>>`. `Mutex<T>` is `Sync` whenever
//! `T` is `Send` — its lock enforces exclusion at run time with a
//! thread-safe primitive, unlike `RefCell`'s plain borrow counters.

use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let shared = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();

    for _ in 0..4 {
        let counter = Arc::clone(&shared);
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                *counter.lock().unwrap() += 1;
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("final count: {}", *shared.lock().unwrap());
}
