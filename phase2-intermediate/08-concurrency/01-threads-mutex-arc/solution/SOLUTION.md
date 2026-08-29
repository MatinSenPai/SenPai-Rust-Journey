# Solution — 2.8.1 Threads, `Mutex`, `Arc`

```rust
pub fn sum_in_threads(nums: Vec<i32>, thread_count: usize) -> i32 {
    let thread_count = thread_count.max(1);
    let chunk_size = nums.len().div_ceil(thread_count).max(1);
    let chunks: Vec<Vec<i32>> = nums.chunks(chunk_size).map(|c| c.to_vec()).collect();

    let handles: Vec<_> = chunks
        .into_iter()
        .map(|chunk| thread::spawn(move || chunk.iter().sum::<i32>()))
        .collect();

    handles.into_iter().map(|h| h.join().unwrap()).sum()
}

pub fn count_matching_in_threads(items: Vec<i32>, predicate: fn(i32) -> bool) -> i32 {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();

    for item in items {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            if predicate(item) {
                *counter.lock().unwrap() += 1;
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let final_count = *counter.lock().unwrap();
    final_count
}
```

## `sum_in_threads` — no shared state, so no `Mutex`

`nums` is sliced into `thread_count` owned chunks with `.chunks(chunk_size).map(|c| c.to_vec())` — each chunk is cloned out, so every spawned closure can `move` its own piece without borrowing anything from the others. Each thread computes its own partial sum entirely independently; nothing is touched by more than one thread while they're running. The only place anything comes back together is on the *calling* thread, after every `handle.join()` — by then it's back to being completely sequential, so summing the partial results needs no synchronization at all.

`thread_count.max(1)` handles a `thread_count` of `0` (treated as `1`). `chunk_size.max(1)` handles the empty-`nums` edge case: without it, `chunk_size` would compute to `0` whenever `nums` is empty, and `.chunks(0)` panics with `chunk size must be non-zero` — exactly what `sums_an_empty_vec` checks. `sums_with_more_threads_than_elements` (more threads requested than elements) needs no special-casing at all: `.chunks()` just yields fewer, smaller chunks than `thread_count` when there isn't enough to go around.

## `count_matching_in_threads` — genuinely shared, mutating state

Here every thread might touch the *same* `i32`, live, while the others are still running — the shape "The concept" built `Arc<Mutex<T>>` for. `Arc::clone(&counter)` runs once per item, handing each spawned closure its own owning handle to the same underlying `Mutex`; `*counter.lock().unwrap() += 1` only runs when `predicate(item)` is `true`, and the lock guarantees that increment can never interleave with another thread's.

Notice the last two lines are **not** collapsed into a bare tail expression `*counter.lock().unwrap()`. That's 2.8.1's own `E0597` from "Errors you will meet": a `MutexGuard` built in the very last expression of a function borrows from `counter`, and the temporary's lifetime extension collides with `counter` itself being dropped at the same point. Binding to `final_count` first ends the guard's borrow immediately, and only the plain (`Copy`) `i32` survives to be returned.

## The bonus test: a shared `HashMap`, not just an `i32`

The solution's test suite adds one thing beyond what `src/lib.rs` asks for: `shared_hashmap_across_threads_gets_every_update`, an `Arc<Mutex<HashMap<&str, u32>>>` updated from six threads via `.entry(word).or_insert(0) += 1`. It's exactly the "Build" exercise's shape, written out — proof that nothing about `Arc<Mutex<T>>` is special-cased to a plain integer; `T` can be any type that needs exclusive, synchronized access, the entry API included. Run it a few times in a row, not just once — concurrent tests can pass by luck on a single run in a way single-threaded tests never do.

## What this lesson was really about

Two separate jobs, stacked: `Arc` gives every thread a real, equally-valid *owner* of the same allocation — the exact job 2.6.3's `Rc` already did, just with an atomic count instead of a plain one. `Mutex` gives exactly one thread at a time *exclusive* access to mutate what's inside — the exact job 2.6.5's `RefCell` already did, just enforced by an OS-level lock instead of a run-time borrow count. Neither type is doing anything conceptually new; both are the thread-safe siblings of tools you already had, reached for the moment more than one OS thread enters the picture.
