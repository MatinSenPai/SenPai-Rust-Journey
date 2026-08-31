use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio_stream::{Stream, StreamExt};

/// Sums every item of a stream built from `values`, driven by hand.
pub async fn sum_stream(values: Vec<i32>) -> i32 {
    let mut stream = tokio_stream::iter(values);
    let mut total = 0;
    while let Some(value) = stream.next().await {
        total += value;
    }
    total
}

/// Keeps only even numbers, then only the first `limit` of those.
pub async fn first_n_even(values: Vec<i32>, limit: usize) -> Vec<i32> {
    let mut stream = tokio_stream::iter(values)
        .filter(|n| n % 2 == 0)
        .take(limit);

    let mut kept = Vec::new();
    while let Some(value) = stream.next().await {
        kept.push(value);
    }
    kept
}

/// BUILD exercise: sleeps for each of `delays_ms`, one at a time, in real
/// time, and returns them all in order once the stream ends.
pub async fn ticks_after(delays_ms: Vec<u64>) -> Vec<u64> {
    let stream = tokio_stream::iter(delays_ms).then(|delay_ms| async move {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        delay_ms
    });
    tokio::pin!(stream);

    let mut ticks = Vec::new();
    while let Some(delay_ms) = stream.next().await {
        ticks.push(delay_ms);
    }
    ticks
}

/// CHALLENGE: a hand-rolled `Stream` — the `Stream` equivalent of 2.8.5's
/// hand-rolled `Countdown` `Future`. No adapters, no `tokio_stream::iter`;
/// `poll_next` is written by hand, exactly like `Countdown::poll` was.
pub struct CountdownStream {
    remaining: u32,
}

impl CountdownStream {
    pub fn new(from: u32) -> Self {
        CountdownStream { remaining: from }
    }
}

impl Stream for CountdownStream {
    type Item = u32;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<u32>> {
        if self.remaining == 0 {
            Poll::Ready(None)
        } else {
            self.remaining -= 1;
            Poll::Ready(Some(self.remaining))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn sums_an_empty_stream_to_zero() {
        assert_eq!(sum_stream(vec![]).await, 0);
    }

    #[tokio::test]
    async fn sums_every_item() {
        assert_eq!(sum_stream(vec![1, 2, 3, 4]).await, 10);
    }

    #[tokio::test]
    async fn keeps_only_the_first_n_even_numbers_in_order() {
        let result = first_n_even(vec![1, 2, 3, 4, 5, 6, 7, 8], 3).await;
        assert_eq!(result, vec![2, 4, 6]);
    }

    #[tokio::test]
    async fn returns_fewer_than_limit_if_not_enough_even_numbers_exist() {
        let result = first_n_even(vec![1, 3, 5, 6], 5).await;
        assert_eq!(result, vec![6]);
    }

    #[tokio::test]
    async fn ticks_after_returns_every_delay_in_order() {
        let result = ticks_after(vec![10, 10, 10]).await;
        assert_eq!(result, vec![10, 10, 10]);
    }

    #[tokio::test]
    async fn ticks_after_actually_takes_real_time() {
        let start = Instant::now();
        // sequential sleeps: 5 * 20ms = ~100ms. An instant, non-sleeping
        // stub would finish in well under 10ms.
        ticks_after(vec![20, 20, 20, 20, 20]).await;
        let elapsed = start.elapsed();

        assert!(
            elapsed >= Duration::from_millis(90),
            "took {elapsed:?} — looks like it never actually slept (expected at least 90ms)"
        );
    }

    #[tokio::test]
    async fn countdown_stream_counts_down_to_none() {
        let mut stream = CountdownStream::new(3);
        assert_eq!(stream.next().await, Some(2));
        assert_eq!(stream.next().await, Some(1));
        assert_eq!(stream.next().await, Some(0));
        assert_eq!(stream.next().await, None);
    }

    #[tokio::test]
    async fn countdown_stream_from_zero_ends_immediately() {
        let mut stream = CountdownStream::new(0);
        assert_eq!(stream.next().await, None);
    }
}
