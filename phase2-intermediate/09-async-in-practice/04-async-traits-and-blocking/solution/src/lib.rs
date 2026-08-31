use async_trait::async_trait;

/// A source of greetings. Retrofitted with `#[async_trait]` (the BUILD
/// exercise) so several implementors can sit behind one `Box<dyn Greeter>`.
#[async_trait]
pub trait Greeter {
    async fn greet(&self, name: &str) -> String;
}

pub struct Formal;

#[async_trait]
impl Greeter for Formal {
    async fn greet(&self, name: &str) -> String {
        format!("Good day, {name}.")
    }
}

/// BUILD: a second `Greeter` shape.
pub struct Casual;

#[async_trait]
impl Greeter for Casual {
    async fn greet(&self, name: &str) -> String {
        format!("Hey {name}!")
    }
}

/// Awaits `greeter.greet(name)` and returns the result. An ordinary generic
/// bounded function — static dispatch, no `dyn`, no `Box` needed here.
pub async fn greet_with<G: Greeter>(greeter: &G, name: &str) -> String {
    greeter.greet(name).await
}

/// BUILD: awaits every greeter in order, returns the results in that order.
pub async fn greet_all(greeters: &[Box<dyn Greeter>], name: &str) -> Vec<String> {
    let mut results = Vec::with_capacity(greeters.len());
    for greeter in greeters {
        results.push(greeter.greet(name).await);
    }
    results
}

/// CPU-heavy, deliberately-synchronous (non-async) work: sums `0..n` with
/// wrapping addition.
pub fn sum_range(n: u64) -> u64 {
    let mut total: u64 = 0;
    for i in 0..n {
        total = total.wrapping_add(i);
    }
    total
}

/// Runs `sum_range(n)` on tokio's blocking thread pool via `spawn_blocking`,
/// so it never stalls the async runtime.
pub async fn run_cpu_work_off_the_runtime(n: u64) -> u64 {
    tokio::task::spawn_blocking(move || sum_range(n))
        .await
        .unwrap()
}

/// CHALLENGE: fans every `n` in `ns` out to its own `spawn_blocking` call —
/// all spawned in one pass, before any `.await` — then awaits them in
/// order.
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
    async fn casual_greets_by_name() {
        let result = Casual.greet("Sara").await;
        assert_eq!(result, "Hey Sara!");
    }

    #[tokio::test]
    async fn greet_with_calls_the_greeter() {
        let result = greet_with(&Formal, "Reza").await;
        assert_eq!(result, "Good day, Reza.");
    }

    #[tokio::test]
    async fn greet_all_collects_every_greeter_in_order() {
        let greeters: Vec<Box<dyn Greeter>> = vec![Box::new(Formal), Box::new(Casual)];
        let results = greet_all(&greeters, "Sara").await;
        assert_eq!(results, vec!["Good day, Sara.", "Hey Sara!"]);
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

    #[tokio::test]
    async fn sum_many_matches_individual_sums() {
        let results = sum_many_off_the_runtime(vec![10, 20, 30]).await;
        assert_eq!(results, vec![sum_range(10), sum_range(20), sum_range(30)]);
    }

    #[tokio::test]
    async fn sum_many_runs_concurrently_not_sequentially() {
        let start = Instant::now();
        // Five 30,000,000-item sums: noticeably serial if run one after
        // another, comfortably fast if they overlap on the blocking pool.
        sum_many_off_the_runtime(vec![30_000_000; 5]).await;
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_millis(800),
            "took {elapsed:?} — looks sequential, not concurrent"
        );
    }
}
