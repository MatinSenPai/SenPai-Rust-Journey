/// A source of greetings. Native `async fn` in a trait — no macro needed,
/// as long as nothing ever needs to store several `Greeter`s behind one
/// single type (see the README for when that changes).
///
/// `#[allow(async_fn_in_trait)]`: this trait stays inside this one crate, so
/// the lint's actual concern (an outside crate being unable to name the
/// `Future`'s `Send`-ness) does not apply — a separate nuance from this
/// lesson's own topic, object safety.
#[allow(async_fn_in_trait)]
pub trait Greeter {
    async fn greet(&self, name: &str) -> String;
}

pub struct Formal;

impl Greeter for Formal {
    /// TODO: return the exact string "Good day, {name}." — e.g. for
    /// name = "Sara", return "Good day, Sara.".
    async fn greet(&self, name: &str) -> String {
        todo!("return the exact string \"Good day, {name}.\"")
    }
}

/// Awaits `greeter.greet(name)` and returns the result. An ordinary generic
/// bounded function — static dispatch, no `dyn`, no `Box` needed here.
pub async fn greet_with<G: Greeter>(greeter: &G, name: &str) -> String {
    greeter.greet(name).await
}

/// CPU-heavy, deliberately-synchronous (non-async) work: sums `0..n` with
/// wrapping addition. Already implemented — the exercise below is what
/// calls it.
pub fn sum_range(n: u64) -> u64 {
    let mut total: u64 = 0;
    for i in 0..n {
        total = total.wrapping_add(i);
    }
    total
}

/// TODO: return the result of `sum_range(n)`, WITHOUT ever calling
/// `sum_range` directly inside this `async fn` — run it on tokio's
/// blocking thread pool via `spawn_blocking` instead, so it never stalls
/// the async runtime.
pub async fn run_cpu_work_off_the_runtime(n: u64) -> u64 {
    todo!("run sum_range(n) on tokio's blocking thread pool via spawn_blocking, then return its result")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[tokio::test]
    async fn formal_greets_by_name() {
        let result = Formal.greet("Sara").await;
        assert_eq!(result, "Good day, Sara.");
    }

    #[tokio::test]
    async fn greet_with_calls_the_greeter() {
        let result = greet_with(&Formal, "Reza").await;
        assert_eq!(result, "Good day, Reza.");
    }

    #[tokio::test]
    async fn cpu_work_returns_the_correct_sum() {
        let result = run_cpu_work_off_the_runtime(1_000).await;
        assert_eq!(result, sum_range(1_000));
    }

    #[tokio::test]
    async fn cpu_work_runs_without_blocking_a_concurrent_sleep() {
        let start = Instant::now();
        let cpu = tokio::spawn(run_cpu_work_off_the_runtime(50_000_000));

        tokio::time::sleep(Duration::from_millis(10)).await;
        let elapsed_at_wake = start.elapsed();

        cpu.await.unwrap();
        assert!(
            elapsed_at_wake < Duration::from_millis(200),
            "a concurrent 10ms sleep took {elapsed_at_wake:?} to complete — looks like \
             sum_range ran directly instead of through spawn_blocking"
        );
    }
}
