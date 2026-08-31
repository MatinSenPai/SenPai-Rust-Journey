use std::time::Duration;
use tokio::time::sleep;

/// Simulates one async I/O call: waits for `delay_ms` milliseconds, then
/// returns the exact string `"item-{id}"` — e.g. `id = 7` becomes
/// `"item-7"`.
pub async fn fetch_simulated(id: u32, delay_ms: u64) -> String {
    todo!("sleep for delay_ms milliseconds to simulate I/O, then return the string \"item-{id}\"")
}

/// Fetches every id in `ids` CONCURRENTLY (not one after another): every
/// `fetch_simulated` call's sleep must run at the same time, not in
/// sequence. Returns the results in a `Vec` ordered the same as `ids` —
/// NOT the order the fetches actually finished in.
pub async fn fetch_all_concurrently(ids: Vec<u32>, delay_ms: u64) -> Vec<String> {
    todo!(
        "spawn one task per id so every delay runs at the same time, wait for every task \
         to finish, then return the results in the same order as ids"
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
        // 5 fetches at 40ms each: ~200ms sequential, ~40ms concurrent.
        fetch_all_concurrently(vec![1, 2, 3, 4, 5], 40).await;
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_millis(150),
            "took {elapsed:?} — looks sequential, not concurrent (expected well under 150ms)"
        );
    }
}
