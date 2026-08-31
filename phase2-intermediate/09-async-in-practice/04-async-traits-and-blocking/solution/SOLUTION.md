# Solution

```rust
#[async_trait]
impl Greeter for Formal {
    async fn greet(&self, name: &str) -> String {
        format!("Good day, {name}.")
    }
}

pub async fn run_cpu_work_off_the_runtime(n: u64) -> u64 {
    tokio::task::spawn_blocking(move || sum_range(n))
        .await
        .unwrap()
}
```

`Formal::greet` is exactly the spec, one `format!` call. `run_cpu_work_off_the_runtime` is the actual point of this half of the lesson: `sum_range(n)` is a plain, synchronous closure — nothing inside it is `async`, nothing in it ever yields — so handing it directly to `spawn_blocking` is what moves it off tokio's cooperative worker threads and onto the separate blocking pool. `spawn_blocking` returns the same kind of `JoinHandle` `tokio::spawn` does, so `.await` on it resolves to a `Result<u64, JoinError>`; `.unwrap()` here is safe because `sum_range` can't panic. `cpu_work_returns_the_correct_sum` checks the value is right; `cpu_work_runs_without_blocking_a_concurrent_sleep` checks the *point* — a concurrent 10ms sleep finishes on schedule even while a 50-million-iteration sum runs, which would not be true if `sum_range(n)` had been called directly inside the `async fn`.

## Build: a second `Greeter`, and `#[async_trait]` for `dyn`

```rust
#[async_trait]
pub trait Greeter {
    async fn greet(&self, name: &str) -> String;
}

pub struct Casual;

#[async_trait]
impl Greeter for Casual {
    async fn greet(&self, name: &str) -> String {
        format!("Hey {name}!")
    }
}
```

`Casual` is `Formal`'s twin, same shape, different string. The real work is the `#[async_trait]` on the trait itself and on **both** `impl`s — one macro, three sites — which is what turns `Greeter` from "native `async fn`, static dispatch only" into "boxable as `dyn Greeter`", exactly the trade this lesson's `## The concept` section walked through: one heap allocation per call, paid so `Box<dyn Greeter>` compiles at all.

```rust
pub async fn greet_all(greeters: &[Box<dyn Greeter>], name: &str) -> Vec<String> {
    let mut results = Vec::with_capacity(greeters.len());
    for greeter in greeters {
        results.push(greeter.greet(name).await);
    }
    results
}
```

`greet_all` is what that retrofit was for: one slice holding both `Formal` and `Casual` behind the same `Box<dyn Greeter>`, awaited one at a time, in order — `greet_all_collects_every_greeter_in_order` mixes both greeters in the same `Vec` and checks the results come back in that exact order.

## Challenge: fan every sum out to its own `spawn_blocking`

```rust
pub async fn sum_many_off_the_runtime(ns: Vec<u64>) -> Vec<u64> {
    let handles: Vec<_> = ns
        .into_iter()
        .map(|n| tokio::task::spawn_blocking(move || sum_range(n)))
        .collect();

    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    results
}
```

The exact spawn-everything-first-then-await shape [2.9.1](../01-spawn-joinset-structured-concurrency/README.md) taught for `tokio::spawn` — the `.map(...).collect()` pass fires off every `spawn_blocking` call before any `.await` happens, so all of them are already running on the blocking pool concurrently by the time the second loop starts collecting. `sum_many_matches_individual_sums` checks the values; `sum_many_runs_concurrently_not_sequentially` checks the *point* — five 30-million-iteration sums finish well under the time five sequential calls to `sum_range` would take, because the blocking pool actually runs them side by side.
