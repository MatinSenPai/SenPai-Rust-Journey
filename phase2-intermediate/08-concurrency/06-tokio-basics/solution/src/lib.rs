use std::time::Duration;
use tokio::time::sleep;

/// Simulates one async I/O call: waits for `delay_ms` milliseconds, then
/// returns the exact string `"item-{id}"` — e.g. `id = 7` becomes
/// `"item-7"`.
pub async fn fetch_simulated(id: u32, delay_ms: u64) -> String {
    sleep(Duration::from_millis(delay_ms)).await;
    format!("item-{id}")
}

/// Fetches every id in `ids` CONCURRENTLY (not one after another): every
/// `fetch_simulated` call's sleep must run at the same time, not in
/// sequence. Returns the results in a `Vec` ordered the same as `ids` —
/// NOT the order the fetches actually finished in.
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

/// BUILD exercise: like `fetch_all_concurrently`, but every request carries
/// its own `(id, delay_ms)` instead of sharing one delay. Still concurrent,
/// still returns results in the same order as `requests`.
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

/// CHALLENGE: exactly two ids, concurrent, via `tokio::join!` instead of
/// `tokio::spawn` — no task is ever handed to the runtime.
pub async fn fetch_two_with_join(id_a: u32, id_b: u32, delay_ms: u64) -> (String, String) {
    tokio::join!(
        fetch_simulated(id_a, delay_ms),
        fetch_simulated(id_b, delay_ms)
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

    #[tokio::test]
    async fn custom_delays_keep_request_order() {
        let results = fetch_with_custom_delays(vec![(9, 30), (1, 10), (5, 20)]).await;
        assert_eq!(results, vec!["item-9", "item-1", "item-5"]);
    }

    #[tokio::test]
    async fn custom_delays_run_concurrently_not_sequentially() {
        let start = Instant::now();
        // sequential would be 80+80+240 = 400ms; concurrent is ~240ms.
        // 340ms sits comfortably between the two either way.
        fetch_with_custom_delays(vec![(1, 80), (2, 80), (3, 240)]).await;
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_millis(340),
            "took {elapsed:?} — looks sequential, not concurrent (expected well under 340ms)"
        );
    }

    #[tokio::test]
    async fn join_fetches_both_concurrently() {
        let start = Instant::now();
        // sequential would be 150+150 = 300ms; concurrent is ~150ms.
        let (a, b) = fetch_two_with_join(1, 2, 150).await;
        let elapsed = start.elapsed();

        assert_eq!(a, "item-1");
        assert_eq!(b, "item-2");
        assert!(
            elapsed < Duration::from_millis(250),
            "took {elapsed:?} — looks sequential, not concurrent (expected well under 250ms)"
        );
    }
}
