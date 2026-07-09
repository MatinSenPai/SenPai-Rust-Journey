use std::time::Duration;
use tokio::time::sleep;

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
        fetch_all_concurrently(vec![1, 2, 3, 4, 5], 50).await;
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_millis(200),
            "took {elapsed:?} — looks sequential, not concurrent (expected well under 200ms)"
        );
    }
}
