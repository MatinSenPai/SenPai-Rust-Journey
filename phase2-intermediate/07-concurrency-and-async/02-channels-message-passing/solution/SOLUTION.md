# Solution

```rust
pub fn compute_async_sum(nums: Vec<i32>) -> i32 {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let sum: i32 = nums.iter().sum();
        tx.send(sum).unwrap();
    });
    rx.recv().unwrap()
}
```

A single producer, a single message, then done — `rx.recv()` blocks until
that one value arrives (or returns `Err` if the sender was dropped without
sending, which can't happen here since the spawned thread always sends
before it ends).

```rust
pub fn collect_from_producers(producer_count: usize, values_per_producer: usize) -> Vec<i32> {
    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::new();
    for i in 0..producer_count {
        let tx = tx.clone();
        handles.push(thread::spawn(move || {
            let start = (i * values_per_producer) as i32;
            for v in start..start + values_per_producer as i32 {
                tx.send(v).unwrap();
            }
        }));
    }
    drop(tx);
    let mut collected: Vec<i32> = rx.iter().collect();
    for handle in handles {
        handle.join().unwrap();
    }
    collected.sort();
    collected
}
```

`drop(tx)` right after the spawn loop is the line most likely to be
missing if this hangs on you: `rx.iter()` (equivalent to `for v in rx`)
only stops once *every* `Sender`, including every `.clone()`, has been
dropped. Each producer thread's own `tx` clone is dropped automatically
when that thread's closure ends — but the *original* `tx` you cloned from
is still alive in the parent thread's scope until you explicitly `drop`
it (or it goes out of scope naturally, which here would be too late —
after the `rx.iter().collect()` call that's waiting on it).

On checkpoint question 4: `count_matching_in_threads` (last lesson) needed
every thread to update **the same single number**, live, as they went —
that's shared, mutating state, which is `Mutex`'s job. `collect_from_producers`
needs every thread to hand over a **stream of independent values** with no
ongoing shared state between them — nothing is being mutated in place,
values are just being moved from many places to one — which is exactly
what a channel models directly.
