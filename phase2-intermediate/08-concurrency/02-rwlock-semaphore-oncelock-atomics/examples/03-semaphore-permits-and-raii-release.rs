//! `std::sync` has no `Semaphore` type — this builds a minimal counting one
//! from `Mutex` + `Condvar`, with an RAII guard (like `MutexGuard`) so a
//! permit is always returned, even if the holder panics.

use std::sync::{Condvar, Mutex};

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

    /// Blocks until a permit is free, then takes one.
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

/// Returned by [`Semaphore::acquire`]. Returns its permit when dropped.
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
    let sem = Semaphore::new(2);
    {
        let _a = sem.acquire();
        let _b = sem.acquire();
        println!(
            "acquired 2 permits, available: {}",
            *sem.available.lock().unwrap()
        );
    } // both permits released here, in reverse order, as `_b` then `_a` drop

    println!(
        "after scope ends, available: {}",
        *sem.available.lock().unwrap()
    );
}
