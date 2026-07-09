# 07.1 — Threads, `Mutex`, `Arc`

## A real OS thread

```rust
use std::thread;

let handle = thread::spawn(|| {
    2 + 2
});

let result = handle.join().unwrap(); // blocks until the thread finishes
```

`thread::spawn` hands your closure to the operating system, which runs it
on a genuinely separate OS thread — actual parallelism, not the cooperative,
single-core-at-a-time concurrency Python gives you by default. (Python's
threads exist, but the GIL — Global Interpreter Lock — means only one of
them runs Python bytecode at a time; you get real parallelism in Python
only via `multiprocessing`, at the cost of separate processes. Rust threads
are real OS threads from the start, no GIL to work around.)

`spawn` returns a `JoinHandle<T>`, where `T` is whatever the closure
returns. Calling `.join()` blocks the current thread until the spawned one
finishes, and gives you back a `Result<T, Box<dyn Any + Send>>` — `Ok(T)`
if the closure returned normally, `Err(...)` if it **panicked**. That's why
you see `.join().unwrap()` everywhere in examples: it's asserting "I expect
this thread to succeed, crash loudly if it didn't." Production code that
can tolerate a worker thread failing should `match` on the `Result` instead
of unwrapping it.

## Why closures passed to `spawn` must be `'static`

```rust
let data = vec![1, 2, 3];
let handle = thread::spawn(move || {
    println!("{data:?}"); // `data` is now OWNED by the closure
});
handle.join().unwrap();
```

The `move` keyword forces the closure to take ownership of everything it
references (`data`), instead of borrowing it. This is required, not
optional style: `spawn`'s closure must be `'static`, meaning it can't hold
a borrow of anything that might not outlive the thread. Consider why —
the function that called `spawn` could return and its stack frame (where a
borrowed `data` would live) could be destroyed *before* the spawned thread
gets around to using it. The compiler has no way to prove the parent will
wait, so it simply refuses to compile a `spawn` closure that borrows local
data. In Python, `threading.Thread(target=lambda: print(data))` just works
without a second thought, because the GIL and reference-counted garbage
collection quietly keep `data` alive as long as anything still points to
it. Rust makes that lifetime question a compile-time decision instead of a
runtime accident: `move` + ownership is how you answer it.

## Sharing data across threads: `Arc<Mutex<T>>`

Two *separate* problems show up once multiple threads need the same data:

1. **Ownership** — every value needs exactly one owner, but several
   threads want to hold onto the same value. `Arc<T>` (from module 05) is
   the thread-safe answer: clone the `Arc` once per thread, and each clone
   is an equally valid owner of the same underlying allocation.
2. **Mutation** — if any thread might *write* to the shared value, two
   threads writing at once is a data race. `Mutex<T>` ("mutual exclusion")
   wraps a value so only one thread can access it at a time:

```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));
let counter2 = Arc::clone(&counter);

thread::spawn(move || {
    let mut guard = counter2.lock().unwrap(); // blocks until the lock is free
    *guard += 1;
    // `guard` is dropped here, at end of scope — the lock is released automatically
}).join().unwrap();
```

`.lock()` returns a `Result<MutexGuard<T>, _>` (`Err` only if another
thread holding the lock panicked, "poisoning" it — `.unwrap()` is the
common case). The `MutexGuard` is a smart pointer: dereference it (`*guard`)
to read or write the inner value, and when it goes out of scope, `Drop`
releases the lock automatically — you almost never call `.unlock()`
by hand.

**Important distinction this lesson makes concrete:** threads alone don't
require `Mutex`. You only need it when threads share *mutable* state. If
every thread computes something independently and hands the result back
through its `JoinHandle`, there's nothing to lock — see the contrast
between the two functions below.

## Your task

Implement both functions in `src/lib.rs`:

- `sum_in_threads` — no shared mutable state, no `Mutex` needed.
- `count_matching_in_threads` — threads *do* share mutable state (a running
  count), so it needs `Arc<Mutex<i32>>`.

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`.
