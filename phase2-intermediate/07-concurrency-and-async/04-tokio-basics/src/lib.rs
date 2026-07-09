use std::time::Duration;
use tokio::time::sleep;

/// Simulates an async I/O call (e.g. fetching something over the network)
/// by sleeping for `delay_ms`, then returns a label for `id`.
pub async fn fetch_simulated(id: u32, delay_ms: u64) -> String {
    todo!("sleep(Duration::from_millis(delay_ms)).await; then format!(\"item-{{id}}\")")
}

/// Fetches every id in `ids` CONCURRENTLY (not one after another) by
/// spawning one task per id, then returns the results in the same order
/// as `ids` (not necessarily the order they finished in).
pub async fn fetch_all_concurrently(ids: Vec<u32>, delay_ms: u64) -> Vec<String> {
    todo!(
        "map each id to tokio::spawn(fetch_simulated(id, delay_ms)), collect the JoinHandles, \
         then await each handle in order and unwrap() it into the result Vec"
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
    async fn fetches_all_in_original_order() {
        let results = fetch_all_concurrently(vec![3, 1, 2], 10).await;
        assert_eq!(results, vec!["item-3", "item-1", "item-2"]);
    }

    #[tokio::test]
    async fn concurrent_fetches_are_actually_concurrent() {
        let start = Instant::now();
        // 5 fetches at 50ms each: ~250ms sequential, ~50ms concurrent.
        fetch_all_concurrently(vec![1, 2, 3, 4, 5], 50).await;
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_millis(200),
            "took {elapsed:?} — looks sequential, not concurrent (expected well under 200ms)"
        );
    }
}
