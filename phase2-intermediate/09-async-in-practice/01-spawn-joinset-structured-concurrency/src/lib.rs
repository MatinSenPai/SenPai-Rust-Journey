use std::time::Duration;
use tokio::task::JoinSet;
use tokio::time::sleep;

/// Simulates one async I/O call: waits for `delay_ms` milliseconds, then
/// returns exactly the string `"item-{id}"` — e.g. `id = 7` becomes
/// `"item-7"`.
pub async fn fetch_simulated(id: u32, delay_ms: u64) -> String {
    todo!("sleep for delay_ms milliseconds to simulate I/O, then return the string \"item-{id}\"")
}

/// Spawns one task per `(id, delay_ms)` pair in `requests` onto a
/// `JoinSet`, each task calling `fetch_simulated(id, delay_ms)`. Returns
/// every result in the order the tasks actually FINISH — not the order
/// `requests` lists them in.
pub async fn fetch_all_via_joinset(requests: Vec<(u32, u64)>) -> Vec<String> {
    todo!(
        "spawn one task per request onto a JoinSet; collect results with a join_next loop, \
         in the order they finish, not the order requests lists them in"
    )
}

/// Spawns one task per id in `ids` onto a `JoinSet`. A task whose id
/// appears in `panics_at` panics instead of finishing normally; every
/// other task finishes immediately, successfully. Returns
/// `(successes, panics)`: how many tasks came back `Ok` from
/// `join_next`, and how many came back `Err`.
pub async fn count_task_outcomes(ids: Vec<u32>, panics_at: Vec<u32>) -> (usize, usize) {
    todo!(
        "spawn one task per id onto a JoinSet, panicking when the id is in panics_at; loop \
         join_next and count how many results are Ok versus Err"
    )
}

/// Spawns one task per id in `ids` onto a `JoinSet`: a task whose id is in
/// `fail_ids` panics immediately; every other task calls
/// `fetch_simulated(id, delay_ms)`. Returns the FIRST successful result
/// found (`Some`), or `None` if every task panicked — and, the moment a
/// successful result is found, aborts every task still running.
pub async fn first_ok_of(ids: Vec<u32>, delay_ms: u64, fail_ids: Vec<u32>) -> Option<String> {
    todo!(
        "spawn one task per id, panicking for ids in fail_ids and calling fetch_simulated for \
         the rest; return the first Ok result from a join_next loop, then abort_all the rest"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn fetches_a_single_item() {
        let result = fetch_simulated(1, 10).await;
        assert_eq!(result, "item-1");
    }

    #[tokio::test]
    async fn joinset_returns_results_in_completion_order() {
        let results = fetch_all_via_joinset(vec![(1, 80), (2, 10), (3, 50)]).await;
        assert_eq!(results, vec!["item-2", "item-3", "item-1"]);
    }

    #[tokio::test]
    async fn joinset_fetches_are_actually_concurrent() {
        let start = Instant::now();
        // 5 requests at 40ms each: ~200ms sequential, ~40ms concurrent.
        let requests: Vec<_> = (1..=5).map(|id| (id, 40)).collect();
        fetch_all_via_joinset(requests).await;
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_millis(150),
            "took {elapsed:?} — looks sequential, not concurrent (expected well under 150ms)"
        );
    }

    #[tokio::test]
    async fn counts_successes_and_panics_separately() {
        let outcomes = count_task_outcomes(vec![1, 2, 3, 4, 5], vec![2, 4]).await;
        assert_eq!(outcomes, (3, 2));
    }

    #[tokio::test]
    async fn counts_all_panics() {
        let outcomes = count_task_outcomes(vec![1, 2, 3], vec![1, 2, 3]).await;
        assert_eq!(outcomes, (0, 3));
    }

    #[tokio::test]
    async fn first_ok_of_skips_the_failing_ids() {
        let result = first_ok_of(vec![1, 2, 3], 10, vec![1, 3]).await;
        assert_eq!(result, Some("item-2".to_string()));
    }

    #[tokio::test]
    async fn first_ok_of_returns_none_when_everything_panics() {
        let result = first_ok_of(vec![1, 2, 3], 10, vec![1, 2, 3]).await;
        assert_eq!(result, None);
    }
}
