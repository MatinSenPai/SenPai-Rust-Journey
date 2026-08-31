use std::time::Duration;
use tokio::task::JoinSet;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

/// Races a simulated fetch against a deadline.
///
/// The simulated fetch waits `delay_ms` milliseconds, then "arrives" with
/// the exact string `format!("data after {delay_ms}ms")`. Race it against a
/// timer of `budget_ms` milliseconds:
///
/// - if the fetch finishes first, return `Some` of that exact string.
/// - if the deadline wins first, return `None`.
pub async fn fetch_with_deadline(delay_ms: u64, budget_ms: u64) -> Option<String> {
    tokio::select! {
        data = fetch(delay_ms) => Some(data),
        _ = sleep(Duration::from_millis(budget_ms)) => None,
    }
}

async fn fetch(delay_ms: u64) -> String {
    sleep(Duration::from_millis(delay_ms)).await;
    format!("data after {delay_ms}ms")
}

/// Runs a cooperative loop that "ticks" every `tick_ms` milliseconds, until
/// `token` is cancelled.
///
/// Each iteration races one tick (a `tick_ms` sleep) against
/// `token.cancelled()`. A tick counts only if its sleep finishes before
/// cancellation is observed; a tick that is still sleeping when the token is
/// cancelled does not get counted. Returns the total number of completed
/// ticks once the loop stops.
pub async fn run_until_cancelled(token: CancellationToken, tick_ms: u64) -> u32 {
    let mut ticks = 0;
    loop {
        tokio::select! {
            _ = token.cancelled() => return ticks,
            _ = sleep(Duration::from_millis(tick_ms)) => ticks += 1,
        }
    }
}

/// BUILD exercise: retries a simulated fetch up to `max_attempts` times,
/// each attempt racing its own `budget_ms` deadline, stopping early with
/// `None` the moment `token` is cancelled. Returns `Some` of the first
/// attempt that beats its deadline.
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

/// CHALLENGE: spawns every delay in `delays` as its own task (2.9.1's
/// `JoinSet`), then races "the first task to finish" against one shared
/// `budget_ms` timeout. Returns the first fetch to complete if any beats the
/// deadline, `None` if the deadline wins against all of them.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn fetch_wins_before_deadline() {
        let result = fetch_with_deadline(10, 200).await;
        assert_eq!(result, Some("data after 10ms".to_string()));
    }

    #[tokio::test]
    async fn deadline_wins_before_fetch() {
        let result = fetch_with_deadline(300, 20).await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn zero_ticks_if_cancelled_immediately() {
        let token = CancellationToken::new();
        let handle = tokio::spawn(run_until_cancelled(token.clone(), 1000));

        sleep(Duration::from_millis(5)).await;
        token.cancel();

        let ticks = handle.await.unwrap();
        assert_eq!(ticks, 0);
    }

    #[tokio::test]
    async fn ticks_stop_after_cancellation() {
        let token = CancellationToken::new();
        let handle = tokio::spawn(run_until_cancelled(token.clone(), 30));

        sleep(Duration::from_millis(100)).await; // let 2-3 ticks happen
        token.cancel();

        let ticks = handle.await.unwrap();
        assert!(
            (2..=3).contains(&ticks),
            "expected 2 or 3 completed ticks before cancellation, got {ticks}"
        );
    }

    #[tokio::test]
    async fn retries_return_first_success() {
        let token = CancellationToken::new();
        let result = fetch_with_retries(token, 10, 50, 3).await;
        assert_eq!(result, Some("data after 10ms".to_string()));
    }

    #[tokio::test]
    async fn retries_give_up_after_max_attempts() {
        let token = CancellationToken::new();
        // Every attempt times out (delay > budget), so all 3 attempts fail.
        let start = Instant::now();
        let result = fetch_with_retries(token, 200, 20, 3).await;
        let elapsed = start.elapsed();

        assert_eq!(result, None);
        // 3 attempts at a 20ms budget each: well over 60ms, well under 200ms
        // (which is what a single un-timed-out fetch alone would take).
        assert!(elapsed >= Duration::from_millis(60));
        assert!(elapsed < Duration::from_millis(180));
    }

    #[tokio::test]
    async fn retries_stop_immediately_when_cancelled() {
        let token = CancellationToken::new();
        token.cancel();
        let result = fetch_with_retries(token, 10, 200, 5).await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn first_of_returns_the_fastest() {
        let result = fetch_first_of(vec![200, 10, 100], 500).await;
        assert_eq!(result, Some("data after 10ms".to_string()));
    }

    #[tokio::test]
    async fn first_of_times_out_if_all_are_slow() {
        let result = fetch_first_of(vec![300, 400], 20).await;
        assert_eq!(result, None);
    }
}
