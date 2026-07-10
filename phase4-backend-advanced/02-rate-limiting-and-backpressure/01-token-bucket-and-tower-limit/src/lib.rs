//! A dependency-free token bucket rate limiter — see `README.md` for the
//! algorithm and how it relates to `tower::limit::RateLimitLayer`.

use std::time::Instant;

/// A token bucket: `capacity` tokens max, refilling at `refill_rate` tokens
/// per second, starting full. `tokens` is `f64` (not an integer) so that a
/// slow refill rate (e.g. 0.5 tokens/sec) still accumulates fractional
/// progress between calls instead of losing it to integer truncation.
pub struct TokenBucket {
    capacity: f64,
    refill_rate: f64,
    tokens: f64,
    last_refill: Instant,
}

impl TokenBucket {
    /// Creates a bucket that starts full (`tokens == capacity`), as of
    /// `now`. `capacity` is the max burst size; `refill_rate` is tokens
    /// added per second of elapsed time.
    pub fn new(capacity: u32, refill_rate: f64, now: Instant) -> Self {
        todo!(
            "Self {{ capacity: capacity as f64, refill_rate, tokens: capacity as f64, last_refill: now }}"
        )
    }

    /// Advances `tokens` by however many seconds have elapsed since
    /// `last_refill`, times `refill_rate`, capped at `capacity` (the bucket
    /// never overflows). Updates `last_refill` to `now` regardless of
    /// whether any tokens were actually added, so the next call measures
    /// elapsed time from here, not from some earlier call.
    fn refill(&mut self, now: Instant) {
        todo!(
            "let elapsed = now.saturating_duration_since(self.last_refill).as_secs_f64(); \
             self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity); \
             self.last_refill = now;"
        )
    }

    /// Refills based on elapsed time, then attempts to take one token. On
    /// success, `tokens` decreases by 1 and this returns `true`. On
    /// failure (bucket empty), `tokens` is left unchanged and this returns
    /// `false` — a rejected request that the caller should treat as
    /// "try again later," not retry in a tight loop.
    pub fn try_acquire(&mut self, now: Instant) -> bool {
        todo!(
            "self.refill(now); if self.tokens >= 1.0 {{ self.tokens -= 1.0; true }} else {{ false }}"
        )
    }

    /// The current token balance, mostly useful for tests and
    /// observability (e.g. exposing "tokens remaining" as a gauge metric —
    /// see phase4-backend-advanced/05-observability/02-metrics-and-prometheus).
    pub fn tokens(&self) -> f64 {
        self.tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn starts_full_and_allows_capacity_requests_immediately() {
        let now = Instant::now();
        let mut bucket = TokenBucket::new(3, 1.0, now);

        assert!(bucket.try_acquire(now));
        assert!(bucket.try_acquire(now));
        assert!(bucket.try_acquire(now));
    }

    #[test]
    fn rejects_once_the_bucket_is_empty() {
        let now = Instant::now();
        let mut bucket = TokenBucket::new(2, 1.0, now);

        assert!(bucket.try_acquire(now));
        assert!(bucket.try_acquire(now));
        assert!(!bucket.try_acquire(now), "bucket should be empty by now");
    }

    #[test]
    fn refills_over_time_and_allows_another_request() {
        let now = Instant::now();
        let mut bucket = TokenBucket::new(1, 1.0, now); // 1 token/sec refill

        assert!(bucket.try_acquire(now));
        assert!(!bucket.try_acquire(now), "no tokens left yet");

        let later = now + Duration::from_millis(500);
        assert!(
            !bucket.try_acquire(later),
            "only 0.5 tokens should have refilled"
        );

        let much_later = now + Duration::from_secs(1);
        assert!(
            bucket.try_acquire(much_later),
            "a full token should have refilled after 1s"
        );
    }

    #[test]
    fn never_exceeds_capacity_even_after_a_long_idle_period() {
        let now = Instant::now();
        let mut bucket = TokenBucket::new(2, 100.0, now); // fast refill

        // Let a huge amount of (simulated) time pass with no requests.
        let much_later = now + Duration::from_secs(3600);
        bucket.try_acquire(much_later);

        // Capacity is 2; even though refill_rate * elapsed is enormous,
        // the balance after one acquire should be capped at capacity - 1.
        assert!(bucket.tokens() <= 1.0);
    }

    #[test]
    fn sustained_rate_matches_refill_rate_over_a_long_window() {
        let now = Instant::now();
        let mut bucket = TokenBucket::new(1, 2.0, now); // 2 tokens/sec, burst 1

        let mut accepted = 0;
        let mut t = now;
        // Simulate 5 seconds at 10 attempts/sec = 50 attempts. At 2
        // tokens/sec sustained, roughly 10 (+/- the initial burst of 1)
        // should be accepted.
        for _ in 0..50 {
            if bucket.try_acquire(t) {
                accepted += 1;
            }
            t += Duration::from_millis(100);
        }

        assert!(
            (9..=12).contains(&accepted),
            "expected roughly 10 accepted requests over 5 simulated seconds at 2/sec, got {accepted}"
        );
    }

    #[test]
    fn does_not_go_negative_on_repeated_rejections() {
        let now = Instant::now();
        let mut bucket = TokenBucket::new(1, 0.0, now); // never refills

        assert!(bucket.try_acquire(now));
        for _ in 0..5 {
            assert!(!bucket.try_acquire(now));
        }
        assert_eq!(bucket.tokens(), 0.0);
    }
}
