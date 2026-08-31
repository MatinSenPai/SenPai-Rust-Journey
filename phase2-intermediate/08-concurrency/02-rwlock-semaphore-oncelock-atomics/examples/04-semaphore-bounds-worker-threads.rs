//! The real reason a semaphore exists: bounding how many threads may touch a
//! limited resource (here, a pretend connection pool with 2 slots) at once.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

struct Semaphore {
    available: Mutex<usize>,
    changed: Condvar,
}

impl Semaphore {
    fn new(permits: usize) -> Self {
        Semaphore {
            available: Mutex::new(permits),
            changed: Condvar::new(),
        }
    }

    fn acquire(&self) -> Permit<'_> {
        let mut available = self.available.lock().unwrap();
        available = self
            .changed
            .wait_while(available, |count| *count == 0)
            .unwrap();
        *available -= 1;
        Permit { semaphore: self }
    }
}

struct Permit<'a> {
    semaphore: &'a Semaphore,
}

impl Drop for Permit<'_> {
    fn drop(&mut self) {
        let mut available = self.semaphore.available.lock().unwrap();
        *available += 1;
        self.semaphore.changed.notify_one();
    }
}

fn main() {
    let semaphore = Arc::new(Semaphore::new(2));
    let in_flight = Arc::new(AtomicUsize::new(0));

    let handles: Vec<_> = (0..4)
        .map(|id| {
            let semaphore = Arc::clone(&semaphore);
            let in_flight = Arc::clone(&in_flight);
            thread::spawn(move || {
                let _permit = semaphore.acquire();
                let now = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
                assert!(now <= 2, "more than 2 workers held the resource at once");
                thread::sleep(Duration::from_millis(20));
                in_flight.fetch_sub(1, Ordering::SeqCst);
                id
            })
        })
        .collect();

    for handle in handles {
        let id = handle.join().unwrap();
        println!("worker {id} acquired and released the resource");
    }
    println!("never more than 2 workers held the resource at the same time");
}
