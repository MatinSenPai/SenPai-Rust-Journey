# Solution — 2.8.2 `RwLock`, `Semaphore`, `OnceLock`/`LazyLock`, atomics

```rust
impl LazyGreeting {
    pub fn new(name: &str) -> Self {
        LazyGreeting {
            name: name.to_string(),
            cell: OnceLock::new(),
        }
    }

    pub fn greeting(&self) -> &str {
        self.cell.get_or_init(|| format!("hello, {}", self.name))
    }
}

impl HitCounter {
    pub fn new() -> Self {
        HitCounter {
            count: AtomicUsize::new(0),
        }
    }

    pub fn hit(&self) -> usize {
        self.count.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn count(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }
}

impl ResourcePool {
    pub fn new(capacity: usize) -> Self {
        ResourcePool {
            available: AtomicUsize::new(capacity),
        }
    }

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

    pub fn release(&self) {
        self.available.fetch_add(1, Ordering::SeqCst);
    }

    pub fn available(&self) -> usize {
        self.available.load(Ordering::SeqCst)
    }
}
```

Nothing here needed anything beyond what "The concept" showed — the same three tools, aimed at three small, self-contained types.

## `LazyGreeting` — a `OnceLock` field, not a `static` one

Every earlier `OnceLock`/`LazyLock` example used a `static`, because a program-wide value only needs computing once, period. `LazyGreeting` is the other common shape: an `OnceLock` living *inside* a struct, so each instance gets its own lazily-computed value, independent of every other instance. `greeting()` is one line — `self.cell.get_or_init(|| format!("hello, {}", self.name))` — because that is the entire API: hand `get_or_init` a closure that builds the value, and it either runs that closure (the first call) or ignores it and hands back what is already there (every call after).

`greeting_is_computed_once_and_then_cached` proves this without needing a counter: it compares the raw pointer behind the returned `&str` across two calls. If `get_or_init` had recomputed anything, the second `format!` would have allocated a *new* `String` at a *different* address — the test would fail. Getting back the identical address is only possible if the second call never built anything at all.

## `HitCounter` — one `AtomicUsize`, no `Mutex` anywhere

`hit()` is `fetch_add(1, Ordering::SeqCst) + 1`: `fetch_add` itself returns the value *before* the add, so `+ 1` is what turns that into "the total that includes this hit," matching the spec. `count()` is a plain `load`. There is no guard, no blocking, no possibility of two threads corrupting each other's update — `fetch_add` is a single atomic hardware operation, not a read followed by a separate write.

`hit_counter_survives_many_threads_hitting_it_at_once` spawns 8 threads that each call `hit()` 500 times and asserts the final count is exactly 4000. This is not a "usually passes" test: every one of the 4000 increments is guaranteed to land, in some order, with nothing ever lost — the same property [the lock-free counter example](../README.md#atomics-a-single-small-value-no-lock-at-all) demonstrated.

## `ResourcePool` — the same compare-and-swap loop as `try_claim` in the lesson

`try_claim` reads the current `available` count, and if it is not already 0, tries to swap it for one less using `compare_exchange`. `Ok(_)` means the swap actually happened — nobody else changed the count between the `load` and the `compare_exchange`, so this thread genuinely claimed a resource. `Err(actual)` means another thread's claim or release landed first; `current` is updated to that real value and the loop tries again from there, rather than either giving up or corrupting the count. This is the exact shape [the one-shot flag example](../README.md#atomics-a-single-small-value-no-lock-at-all) used with `AtomicBool`, generalized from "was it `false`?" to "was it more than 0?".

`pool_never_lets_more_than_capacity_claims_stay_out_at_once` spawns 20 threads racing `try_claim()` against a pool of 2, and asserts — from inside every thread that actually claims one — that the number currently held never exceeds 2. Run this test (or the whole suite) several times in a row rather than trusting one green run; that is good practice for any test with real threads in it, and it is what makes this assertion worth having at all.

## What this lesson was really about

- **A `compare_exchange` loop is not a `Mutex` in disguise — it is genuinely lock-free.** No thread here ever blocks waiting for another one; a losing `try_claim()` just returns `false` immediately and the caller decides what to do next.
- **`OnceLock` is not only for `static`s.** A struct field wrapped in one gives *each instance* its own once-only computation, independent of every other instance — the `static` form from the lesson and this instance form solve the same problem at two different scopes.
- **Pointer equality is a legitimate way to prove "no work happened."** `greeting_is_computed_once_and_then_cached` never counts function calls; it only checks that the second call handed back the exact same memory, which is only possible if nothing was recomputed.
