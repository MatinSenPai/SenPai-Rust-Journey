# Solution

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
```

Each thread gets its own **owned** `Vec<i32>` chunk (via `.to_vec()`,
cloning the relevant slice) and computes a sum entirely independently —
nothing is shared *while the threads are running*, so there's nothing to
protect with a `Mutex`. The only "sharing" happens after every thread has
already finished, back on the main thread, via `.join()` collecting each
result — completely sequential and single-threaded at that point.

```rust
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

Note the last two lines are **not** collapsed into a single tail expression
`*counter.lock().unwrap()`. That version fails to compile with `error[E0597]:
borrowed value does not live long enough` — the compiler's temporary-lifetime
rules for a tail expression try to keep the `MutexGuard` temporary (which
borrows from `counter`) alive exactly as long as `counter` itself, and the
two conflict at the point where the whole block's locals get dropped.
Binding the dereferenced (and therefore `Copy`d, since it's an `i32`) value
to `final_count` *first* ends the guard's borrow immediately, before
`counter` is dropped, and only the plain `i32` survives as the return
value. This is a real, sharp interaction between `MutexGuard` and Rust's
tail-expression drop-order rules — not a hypothetical edge case.

Here, every thread needs to update the *same* counter, so it's wrapped in
`Mutex<i32>` (exclusive access, one thread at a time) inside `Arc` (so
every thread can independently *own* a handle to that same `Mutex` — plain
`Mutex<i32>` can't be moved into more than one thread's closure, since
`move` requires each closure to take full ownership of what it captures,
and only one closure can own a given value).

On recall question 2: without `Arc`, `let counter = Mutex::new(0);`
followed by trying to `move` `counter` into two different
`thread::spawn` closures is a **compile error** — the second `move`
closure would be attempting to move a value that was already moved into
the first one, the exact same "used after move" error from Phase 1's
ownership lessons, just now surfacing across a `thread::spawn` boundary
instead of a plain function call. `Arc::clone` sidesteps it by giving each
thread its own independent, cheap-to-clone *reference-counted handle* to
the same underlying `Mutex`, rather than trying to move the `Mutex` itself
more than once.
