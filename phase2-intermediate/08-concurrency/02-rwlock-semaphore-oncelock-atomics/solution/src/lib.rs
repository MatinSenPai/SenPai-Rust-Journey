//! Reference solution for 2.8.2 — `RwLock`, a hand-built `Semaphore`,
//! `OnceLock`/`LazyLock`, and atomics.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

/// A greeting that's only built the first time it's asked for.
pub struct LazyGreeting {
    name: String,
    cell: OnceLock<String>,
}

impl LazyGreeting {
    /// Stores `name` for later. Builds nothing yet — [`Self::greeting`]
    /// does that, once, on its first call.
    pub fn new(name: &str) -> Self {
        LazyGreeting {
            name: name.to_string(),
            cell: OnceLock::new(),
        }
    }

    /// Returns `"hello, {name}"`, using the name passed to [`Self::new`].
    /// Computed on the first call only; every later call returns that exact
    /// same `String`, without recomputing it.
    pub fn greeting(&self) -> &str {
        self.cell.get_or_init(|| format!("hello, {}", self.name))
    }
}

/// A counter safe to share and update from many threads at once, with no
/// lock at all.
pub struct HitCounter {
    count: AtomicUsize,
}

impl HitCounter {
    /// A fresh counter starting at 0.
    pub fn new() -> Self {
        HitCounter {
            count: AtomicUsize::new(0),
        }
    }

    /// Adds 1 to the counter and returns the new total — the count that
    /// includes this hit.
    pub fn hit(&self) -> usize {
        self.count.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// The current total, unchanged.
    pub fn count(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }
}

/// A pool of `capacity` interchangeable resources, bounded with atomics
/// instead of a lock — nobody ever blocks waiting for one.
pub struct ResourcePool {
    available: AtomicUsize,
}

impl ResourcePool {
    /// A pool with `capacity` resources, all free.
    pub fn new(capacity: usize) -> Self {
        ResourcePool {
            available: AtomicUsize::new(capacity),
        }
    }

    /// If at least one resource is free, claims it (the free count drops by
    /// 1) and returns `true`. If none are free, changes nothing and returns
    /// `false`. Never blocks.
    pub fn try_claim(&self) -> bool {
        let mut current = self.available.load(Ordering::SeqCst);
        loop {
            if current == 0 {
                return false;
            }
            match self.available.compare_exchange(
                current,
                current - 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }

    /// Returns one resource to the pool — the free count goes up by 1.
    pub fn release(&self) {
        self.available.fetch_add(1, Ordering::SeqCst);
    }

    /// How many resources are currently free.
    pub fn available(&self) -> usize {
        self.available.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn greeting_uses_the_stored_name() {
        let g = LazyGreeting::new("Matin");
        assert_eq!(g.greeting(), "hello, Matin");
    }

    #[test]
    fn greeting_is_computed_once_and_then_cached() {
        let g = LazyGreeting::new("Matin");
        let first = g.greeting().as_ptr();
        let second = g.greeting().as_ptr();
        assert_eq!(
            first, second,
            "the second call should reuse the cached String"
        );
    }

    #[test]
    fn hit_counter_starts_at_zero() {
        let counter = HitCounter::new();
        assert_eq!(counter.count(), 0);
    }

    #[test]
    fn hit_returns_the_total_after_incrementing() {
        let counter = HitCounter::new();
        assert_eq!(counter.hit(), 1);
        assert_eq!(counter.hit(), 2);
        assert_eq!(counter.hit(), 3);
        assert_eq!(counter.count(), 3);
    }

    #[test]
    fn hit_counter_survives_many_threads_hitting_it_at_once() {
        let counter = Arc::new(HitCounter::new());
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let counter = Arc::clone(&counter);
                thread::spawn(move || {
                    for _ in 0..500 {
                        counter.hit();
                    }
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
        assert_eq!(counter.count(), 4000);
    }

    #[test]
    fn pool_starts_with_every_resource_free() {
        let pool = ResourcePool::new(3);
        assert_eq!(pool.available(), 3);
    }

    #[test]
    fn try_claim_fails_once_nothing_is_left() {
        let pool = ResourcePool::new(1);
        assert!(pool.try_claim());
        assert!(!pool.try_claim());
        assert_eq!(pool.available(), 0);
    }

    #[test]
    fn release_gives_a_resource_back() {
        let pool = ResourcePool::new(1);
        assert!(pool.try_claim());
        pool.release();
        assert_eq!(pool.available(), 1);
        assert!(pool.try_claim());
    }

    #[test]
    fn pool_never_lets_more_than_capacity_claims_stay_out_at_once() {
        let pool = Arc::new(ResourcePool::new(2));
        let in_use = Arc::new(AtomicUsize::new(0));
        let handles: Vec<_> = (0..20)
            .map(|_| {
                let pool = Arc::clone(&pool);
                let in_use = Arc::clone(&in_use);
                thread::spawn(move || {
                    if pool.try_claim() {
                        let now = in_use.fetch_add(1, Ordering::SeqCst) + 1;
                        assert!(
                            now <= 2,
                            "more than capacity resources were claimed at once"
                        );
                        in_use.fetch_sub(1, Ordering::SeqCst);
                        pool.release();
                    }
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
    }
}
