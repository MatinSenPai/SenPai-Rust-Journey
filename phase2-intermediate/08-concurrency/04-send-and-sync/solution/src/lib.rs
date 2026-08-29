//! Solution for 2.8.4 — `Send` and `Sync`.

use std::sync::{Arc, Mutex};
use std::thread;

/// Moves `label` into a new thread, appends the suffix `" (from thread)"`
/// to it there, and returns the result after joining.
///
/// # Examples
///
/// For `label` equal to `"senpai"`, returns `"senpai (from thread)"`.
pub fn label_from_thread(label: String) -> String {
    let handle = thread::spawn(move || format!("{label} (from thread)"));
    handle.join().unwrap()
}

/// A counter that can be safely incremented from many threads at once.
///
/// Each `.clone()` is a handle to the *same* shared count, not an
/// independent copy of it — incrementing through one clone is visible
/// through every other clone.
#[derive(Clone)]
pub struct SharedCounter {
    value: Arc<Mutex<i32>>,
}

impl SharedCounter {
    /// Starts a new counter at `start`.
    pub fn new(start: i32) -> Self {
        SharedCounter {
            value: Arc::new(Mutex::new(start)),
        }
    }

    /// Adds 1 to the counter. Safe to call from any thread holding a clone,
    /// even while another thread is doing the same thing at the same time.
    pub fn increment(&self) {
        *self.value.lock().unwrap() += 1;
    }

    /// The counter's current value.
    pub fn value(&self) -> i32 {
        *self.value.lock().unwrap()
    }
}

/// Spawns `thread_count` threads, each cloning `counter` and calling
/// `.increment()` exactly `increments_each` times, then waits for every
/// thread to finish before returning the counter's final value.
///
/// # Examples
///
/// For `thread_count` equal to `8` and `increments_each` equal to `500`,
/// the counter's value goes up by exactly `4000` — no increment is ever
/// lost, no matter how the threads interleave.
pub fn fan_out_increments(
    counter: &SharedCounter,
    thread_count: usize,
    increments_each: usize,
) -> i32 {
    let handles: Vec<_> = (0..thread_count)
        .map(|_| {
            let counter = counter.clone();
            thread::spawn(move || {
                for _ in 0..increments_each {
                    counter.increment();
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    counter.value()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_the_label_into_a_thread_and_appends_the_suffix() {
        assert_eq!(
            label_from_thread("senpai".to_string()),
            "senpai (from thread)"
        );
    }

    #[test]
    fn moves_a_different_label_just_as_well() {
        assert_eq!(
            label_from_thread("rustacean".to_string()),
            "rustacean (from thread)"
        );
    }

    #[test]
    fn a_fresh_counter_reads_back_its_starting_value() {
        assert_eq!(SharedCounter::new(0).value(), 0);
        assert_eq!(SharedCounter::new(5).value(), 5);
    }

    #[test]
    fn increment_adds_exactly_one() {
        let counter = SharedCounter::new(0);
        counter.increment();
        counter.increment();
        counter.increment();
        assert_eq!(counter.value(), 3);
    }

    #[test]
    fn a_clone_shares_the_same_underlying_count() {
        let original = SharedCounter::new(0);
        let clone = original.clone();

        clone.increment();

        assert_eq!(
            original.value(),
            1,
            "incrementing the clone should be visible on original"
        );
    }

    #[test]
    fn fan_out_increments_loses_nothing_across_many_threads() {
        let counter = SharedCounter::new(0);
        let final_value = fan_out_increments(&counter, 8, 500);

        assert_eq!(final_value, 4000);
        assert_eq!(counter.value(), 4000);
    }
}
