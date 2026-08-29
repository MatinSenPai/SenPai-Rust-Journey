# Solution

```rust
pub fn label_from_thread(label: String) -> String {
    let handle = thread::spawn(move || format!("{label} (from thread)"));
    handle.join().unwrap()
}

#[derive(Clone)]
pub struct SharedCounter {
    value: Arc<Mutex<i32>>,
}

impl SharedCounter {
    pub fn new(start: i32) -> Self {
        SharedCounter {
            value: Arc::new(Mutex::new(start)),
        }
    }

    pub fn increment(&self) {
        *self.value.lock().unwrap() += 1;
    }

    pub fn value(&self) -> i32 {
        *self.value.lock().unwrap()
    }
}

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
```

## `label_from_thread` — the same pattern as the lesson's first subsection

Nothing new here: `String` was already `Send`, so `move` carries it fully into the closure and `thread::spawn` accepts it without complaint. `.join().unwrap()` pulls out the value the closure returns — the `String` built with `format!`.

## `SharedCounter` — why its field is exactly `Arc<Mutex<i32>>`

Both halves were necessary, each for its own reason:

- **`Arc`**, because `SharedCounter` needs to be `Clone`, and every clone needs to see the *same* underlying count, not an independent copy. `#[derive(Clone)]` on `SharedCounter` itself gets this for free, precisely because `Arc<T>: Clone` means exactly that — cloning is a new handle, not a new allocation.
- **`Mutex`**, because `increment` has to write from behind `&self` — a shared reference. Without a real lock, two threads calling `.increment()` at the same time would create exactly the data race this entire lesson has been about; with `Mutex`, the compiler won't even let a type without that lock be written to from behind `&self`.

The `a_clone_shares_the_same_underlying_count` test proves exactly this: it increments through a clone, then reads through the original — that same test would also pass if the field were `Rc<RefCell<i32>>` (it's still single-threaded so far), but the moment `fan_out_increments` tried to spread this `counter` across real threads, it would stop compiling — exactly the `E0277` you saw in "Errors you will meet."

## `fan_out_increments` — why the final count never comes up short

Each thread gets a **clone** of `counter` — not a reference, the clone itself — because `spawn`'s closure has to be `'static` and can't hold a borrow of a `counter` that lives in the calling function. Cloning here is cheap: only `Arc`'s internal count goes up, not the data behind the `Mutex`.

The second loop — `for handle in handles { handle.join().unwrap(); }` — guarantees every thread's `increments_each` increments have genuinely finished before `counter.value()` is read. That's why `fan_out_increments_loses_nothing_across_many_threads` gets exactly the same number (`thread_count * increments_each`) every time it runs, no matter how the actual lock acquisitions interleave between threads from one run to the next — `Mutex`'s lock completes exactly one `.increment()` at a time, and none of them are ever lost.
