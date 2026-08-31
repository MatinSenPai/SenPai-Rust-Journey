# Solution

```rust
pub async fn fetch_simulated(id: u32, delay_ms: u64) -> String {
    sleep(Duration::from_millis(delay_ms)).await;
    format!("item-{id}")
}

pub async fn fetch_all_via_joinset(requests: Vec<(u32, u64)>) -> Vec<String> {
    let mut set = JoinSet::new();
    for (id, delay_ms) in requests {
        set.spawn(fetch_simulated(id, delay_ms));
    }

    let mut results = Vec::new();
    while let Some(result) = set.join_next().await {
        results.push(result.unwrap());
    }
    results
}
```

The spawn loop and the collection loop are two separate passes, same structural move as [2.8.6](../../../08-concurrency/06-tokio-basics/README.md)'s `fetch_all_concurrently`: every request is handed to the `JoinSet` first, so every `sleep` starts counting down at roughly the same moment, before any collecting happens at all. The difference from 2.8.6 is entirely in the second loop — `while let Some(result) = set.join_next().await` pulls back whichever task happens to be done, not the one that's next in some list, which is exactly why `joinset_returns_results_in_completion_order` sees `["item-2", "item-3", "item-1"]` for delays of `[80, 10, 50]` — sorted by delay, not by the order `requests` listed them in.

`result.unwrap()`: `.join_next()` hands back `Option<Result<T, JoinError>>`; the `Option` is exhausted by the `while let`, and `.unwrap()` here assumes the task didn't panic — reasonable for this function, since nothing inside `fetch_simulated` ever does.

## Build: counting successes and panics

```rust
pub async fn count_task_outcomes(ids: Vec<u32>, panics_at: Vec<u32>) -> (usize, usize) {
    let mut set = JoinSet::new();
    for id in ids {
        let should_panic = panics_at.contains(&id);
        set.spawn(async move {
            if should_panic {
                panic!("task {id} was told to panic");
            }
        });
    }

    let mut successes = 0;
    let mut panics = 0;
    while let Some(result) = set.join_next().await {
        match result {
            Ok(()) => successes += 1,
            Err(_) => panics += 1,
        }
    }
    (successes, panics)
}
```

`should_panic` is computed *before* the `async move` block, from the plain `Vec<u32>` — `panics_at.contains(&id)` needs `&panics_at`, and the task itself only ever needs to know the one `bool` it captured, not the whole list. Counting doesn't care what order `join_next` hands tasks back in, only how many of each kind arrive — so `counts_successes_and_panics_separately` and `counts_all_panics` both pass regardless of scheduling.

## Challenge: first success wins, cancel the rest

```rust
pub async fn first_ok_of(ids: Vec<u32>, delay_ms: u64, fail_ids: Vec<u32>) -> Option<String> {
    let mut set = JoinSet::new();
    for id in ids {
        let fails = fail_ids.contains(&id);
        set.spawn(async move {
            if fails {
                panic!("task {id} was told to fail");
            }
            fetch_simulated(id, delay_ms).await
        });
    }

    while let Some(result) = set.join_next().await {
        if let Ok(value) = result {
            set.abort_all();
            return Some(value);
        }
    }
    None
}
```

Every `Err` from `join_next` is simply skipped by the `while let` loop — the moment an `Ok(value)` shows up, `set.abort_all()` cancels whatever is still running (structured concurrency, triggered on purpose instead of by a scope closing) and the function returns immediately, without waiting for the rest. `first_ok_of_skips_the_failing_ids` gives ids `[1, 2, 3]` with `[1, 3]` failing, so only task 2 can ever produce an `Ok` — the test doesn't need to know completion order to know the answer has to be `"item-2"`. `first_ok_of_returns_none_when_everything_panics` covers the other end: every id fails, `join_next` only ever hands back `Err`, and the loop runs out with nothing to return.
