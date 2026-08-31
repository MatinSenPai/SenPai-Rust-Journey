# Solution

```rust
async fn fetch(delay_ms: u64) -> String {
    sleep(Duration::from_millis(delay_ms)).await;
    format!("data after {delay_ms}ms")
}

pub async fn fetch_with_deadline(delay_ms: u64, budget_ms: u64) -> Option<String> {
    tokio::select! {
        data = fetch(delay_ms) => Some(data),
        _ = sleep(Duration::from_millis(budget_ms)) => None,
    }
}

pub async fn run_until_cancelled(token: CancellationToken, tick_ms: u64) -> u32 {
    let mut ticks = 0;
    loop {
        tokio::select! {
            _ = token.cancelled() => return ticks,
            _ = sleep(Duration::from_millis(tick_ms)) => ticks += 1,
        }
    }
}
```

`fetch_with_deadline` is exactly the pattern "The concept" built up step by step - one branch is the fetch, the other plays the deadline, and the winner decides whether the result is `Some` or `None`.

`run_until_cancelled` has a subtler point: why `return ticks` instead of `break`? Because this `select!` sits inside a `loop` that has no value of its own - a plain `break` would only exit the loop, not the function. `return ticks` exits the loop and returns the value from the function itself, in one move. A second point `ticks_stop_after_cancellation` leans on: a tick only counts if the `sleep` branch genuinely wins - if the token gets cancelled right in the middle of a `sleep`, that `sleep` gets dropped without ever bumping `ticks`, because the `cancelled` branch won, not the tick branch.

## Build: retry until cancelled or successful

```rust
pub async fn fetch_with_retries(
    token: CancellationToken,
    delay_ms: u64,
    budget_ms: u64,
    max_attempts: u32,
) -> Option<String> {
    for _ in 0..max_attempts {
        tokio::select! {
            _ = token.cancelled() => return None,
            result = fetch_with_deadline(delay_ms, budget_ms) => {
                if let Some(data) = result {
                    return Some(data);
                }
            }
        }
    }
    None
}
```

Here `fetch_with_deadline` itself - already a `select!` on its own - sits as a branch of an outer `select!`. That's entirely legal: any future can be a branch, no matter how many layers of `select!` it's built from itself. If `token` gets cancelled mid-attempt, the `token.cancelled()` branch wins and the whole in-flight `fetch_with_deadline` - both of its own inner branches included - gets dropped that same instant; `fetch_with_retries` returns `None` immediately, even mid-attempt.

## Challenge: whichever task finishes first

```rust
pub async fn fetch_first_of(delays: Vec<u64>, budget_ms: u64) -> Option<String> {
    let mut set = JoinSet::new();
    for delay_ms in delays {
        set.spawn(fetch(delay_ms));
    }

    tokio::select! {
        Some(Ok(data)) = set.join_next() => Some(data),
        _ = sleep(Duration::from_millis(budget_ms)) => None,
    }
}
```

Every delay becomes its own fully independent task - all of them counting down at once, from the very start. The first branch of `select!` is a refutable pattern: `Some(Ok(data)) = set.join_next()`. `set.join_next()` is itself an async method that hands back *one* finished task each time it's called - so this single branch, every time `select!` polls it again, is looking for the next task to become ready; whichever task had the shortest delay of them all always arrives first. If no task finishes before `budget_ms`, the `sleep` branch wins and `None` comes back - the remaining still-running tasks are left exactly where they were, mid-work (precisely the cancellation-safety pattern this whole lesson is built on, this time applied to whole tasks instead of a single future).
