# Solution

```rust
pub async fn fetch_simulated(id: u32, delay_ms: u64) -> String {
    sleep(Duration::from_millis(delay_ms)).await;
    format!("item-{id}")
}

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

The one structural move everything depends on: **spawn every task first, in a single pass** (the `.map(...).collect()`), and *only then*, in a separate loop, `.await` each handle. Writing `tokio::spawn(fetch_simulated(id, delay_ms)).await` inside that same single loop instead would await each spawn the instant it happened, fully finishing before the next one even starts — functionally sequential again, just with pointless spawning overhead on top. Spawning all of them up front is exactly what lets every `sleep` count down alongside the others — by the time the first `.await` in the second loop starts waiting, all five tasks are already, independently, counting down their own timers.

`handle.await.unwrap()`: `.await`ing the handle `tokio::spawn` returns resolves to a `Result` that is only `Err` if the spawned task panicked (or was cancelled) — same shape, same reason, as the `Result` `thread::spawn`'s `.join()` gave you back in [2.8.1](../../01-threads-mutex-arc/README.md), just async instead of blocking.

## Warm-up question 1, revisited

Try the sequential rewrite — `for id in ids { results.push(fetch_simulated(id, delay_ms).await); }`, no `tokio::spawn` at all — and it still compiles, still uses `async`/`.await` throughout, but `concurrent_fetches_are_actually_concurrent` fails against it: instead of well under 150ms, it takes around 200ms. This is the sharpest point in the whole lesson: **writing `async fn` and using `.await`, by itself, makes nothing concurrent.** Concurrency comes specifically from `tokio::spawn` (or a combinator like `tokio::join!`, below) — something that hands more than one `Future` to the runtime to make progress on independently. `.await` alone only ever means "pause here until this one finishes" — which, as you just saw, can describe perfectly sequential code just as well.

## Build: per-request delays

```rust
pub async fn fetch_with_custom_delays(requests: Vec<(u32, u64)>) -> Vec<String> {
    let handles: Vec<_> = requests
        .into_iter()
        .map(|(id, delay_ms)| tokio::spawn(fetch_simulated(id, delay_ms)))
        .collect();

    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    results
}
```

The exact same pattern — `.map` just destructures a tuple this time, so each request carries its own delay along with it. `custom_delays_run_concurrently_not_sequentially` proves it with numbers: three requests at 80, 80, and 240ms, run sequentially, would add up to 400ms; run concurrently, the whole thing takes only as long as the slowest one — around 240ms.

## Challenge: the same result, with no spawning at all

```rust
pub async fn fetch_two_with_join(id_a: u32, id_b: u32, delay_ms: u64) -> (String, String) {
    tokio::join!(
        fetch_simulated(id_a, delay_ms),
        fetch_simulated(id_b, delay_ms)
    )
}
```

The timing is identical — `join_fetches_both_concurrently` confirms it. But the mechanism underneath is completely different. `tokio::join!` never spawns a new task; neither `Future` is ever handed to the runtime — both stay inside the one calling task, and `join!` manually interleaves between their `.await` points itself: drive one until its first point of waiting, switch to the other, and so on until both are done. That is exactly why this version never needs `Send + 'static` ([2.8.4](../../04-send-and-sync/README.md)): that bound only matters for something the runtime might move between worker threads — and something that is never handed to the runtime in the first place is never moved either.
