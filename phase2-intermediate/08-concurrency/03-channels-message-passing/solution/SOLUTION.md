# Solution — 2.8.3 channels and message passing

```rust
pub fn sum_via_channel(nums: Vec<i32>) -> i32 {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let total: i32 = nums.iter().sum();
        tx.send(total).unwrap();
    });
    rx.recv().unwrap()
}
```

One thread, one message, then done. `nums` moves into the spawned closure (`move ||`), the closure sums it and sends the single total back, and `rx.recv()` blocks the calling thread until that value (or a disconnect) arrives. Since the spawned thread always sends before it ends, `recv()` never actually sees the disconnected case here.

```rust
pub fn collect_from_workers(worker_count: usize, values_per_worker: usize) -> Vec<i32> {
    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::new();

    for i in 0..worker_count {
        let tx = tx.clone();
        handles.push(thread::spawn(move || {
            let start = (i * values_per_worker) as i32;
            let end = start + values_per_worker as i32;
            for value in start..end {
                tx.send(value).unwrap();
            }
        }));
    }
    drop(tx); // without this, rx.iter() below would wait forever

    let mut collected: Vec<i32> = rx.iter().collect();
    for handle in handles {
        handle.join().unwrap();
    }

    collected.sort();
    collected
}
```

`drop(tx)` right after the spawn loop is the line most likely to be missing if this hangs on you: `rx.iter()` (equivalent to `for v in rx`) only stops once *every* `Sender`, including every `.clone()`, has been dropped. Each worker's own `tx` clone drops itself automatically when that worker's closure ends — but the *original* `tx` you cloned from is still alive in the outer scope until you explicitly `drop` it (or it goes out of scope naturally, which here would be too late — that would happen only after the `rx.iter().collect()` call that is waiting on it).

Each worker sends `values_per_worker` individual numbers, not one summary value — that's what makes iterating `rx` directly the natural fit here, rather than calling `.recv()` a fixed number of times: the collecting side doesn't need to know in advance how many messages are coming, only that the channel will tell it when to stop.

`worker_count == 0` and `values_per_worker == 0` both fall out for free: with zero workers, the loop never runs, `drop(tx)` drops the only `Sender` immediately, and `rx.iter()` yields nothing. With zero values per worker, every worker's range is empty (`start..start`), so every worker sends nothing and finishes immediately — same empty result.

## What this lesson was really about

- **Sending moves ownership.** Neither function ever needed to share `nums` or the numbers being sent — each value has exactly one owner at a time, first the sender, then the channel, then the receiver.
- **`Sender::clone()` is what "multi-producer" means in practice.** `collect_from_workers` clones `tx` once per worker; every clone feeds the one `Receiver` in `rx`.
- **The `Receiver`'s iterator does the "when do I stop?" bookkeeping for you** — but only once every `Sender`, including the original you cloned from, is actually gone. That's the detail this exercise is really testing.
- **Neither function needed a `Mutex`.** Compare this to [2.8.1](../../01-threads-mutex-arc/README.md)'s `count_matching_in_threads`, which needed every thread to update one shared, mutating counter live — that's shared state, `Mutex`'s job. Here, values are just moving from many places to one, with nothing shared in between — exactly what a channel models directly.
