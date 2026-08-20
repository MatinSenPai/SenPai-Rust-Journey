# Solution

```rust
pub async fn fetch_all_concurrently(ids: Vec<u32>, delay_ms: u64) -> Vec<String> {
    let handles: Vec<_> = ids
        .into_iter()
        .map(|id| tokio::spawn(fetch_simulated(id, delay_ms)))
        .collect();

    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    results
}
```

The key structural move: **spawn every task first, in one pass** (the
`.map(...).collect()`), *then* `.await` each `JoinHandle` in a second,
separate loop. If you instead wrote `tokio::spawn(...).await` inside a
single loop, each spawn would be immediately awaited before the next one
even starts — functionally sequential again, just with extra task-spawning
overhead for no benefit. Spawning all of them upfront is what actually lets
every `fetch_simulated` call's `sleep` run concurrently — by the time the
first `.await` in the second loop starts waiting, all five tasks are
already independently counting down their own timers.

`handle.await.unwrap()`: a `JoinHandle<T>`'s `.await` resolves to
`Result<T, JoinError>` — `Err` specifically when the spawned task panicked
(or was cancelled), same shape and same reasoning as a thread's
`JoinHandle::join()` from lesson 1, just async instead of blocking.

On recall question 1: a sequential rewrite (`for id in ids { results.push(fetch_simulated(id, delay_ms).await); }`,
no `tokio::spawn`) still compiles and still uses `async`/`.await`
throughout — but `concurrent_fetches_are_actually_concurrent` would fail
against it, taking ~250ms instead of under 200ms. This is the sharpest
lesson here: **writing `async fn` and using `.await` does not, by itself,
make anything concurrent.** Concurrency comes specifically from `tokio::spawn`
(or equivalent combinators like `join!`/`try_join!`), which hand multiple
`Future`s to the runtime to make progress on independently. `.await` alone
just means "pause here until this one Future finishes" — perfectly capable
of describing purely sequential code, same as it does here.
