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
    todo!(
        "race a delay_ms sleep-then-format future against a budget_ms sleep with select!; \
         Some(the formatted string) if the fetch wins, None if the deadline wins"
    )
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
    todo!(
        "loop: select! between token.cancelled() (stop and return the count) and a tick_ms \
         sleep (increment the count and keep looping)"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

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
}
