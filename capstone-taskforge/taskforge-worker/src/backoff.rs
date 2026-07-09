use std::time::Duration;

/// See `../docs/adr/0004-worker-failure-handling.md` for why exponential
/// backoff *with jitter*, not just exponential backoff.
#[derive(Debug, Clone)]
pub struct BackoffConfig {
    pub base: Duration,
    pub max: Duration,
    /// Fraction of the computed delay to randomly add or subtract, e.g.
    /// `0.2` means ±20%.
    pub jitter_fraction: f64,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        BackoffConfig {
            base: Duration::from_secs(1),
            max: Duration::from_secs(300),
            jitter_fraction: 0.2,
        }
    }
}

/// Computes the delay before retry number `attempt` (1-indexed — the first
/// retry is `attempt = 1`): `base * 2^(attempt - 1)`, capped at `max`, then
/// jittered by up to `±jitter_fraction`.
pub fn compute_backoff(attempt: u32, config: &BackoffConfig) -> Duration {
    let exponent = attempt.saturating_sub(1).min(20); // avoid absurd overflow on huge attempt counts
    let unjittered = config
        .base
        .checked_mul(1u32.checked_shl(exponent).unwrap_or(u32::MAX))
        .unwrap_or(config.max)
        .min(config.max);

    let jitter_range = unjittered.as_secs_f64() * config.jitter_fraction;
    let jitter = rand::random::<f64>() * 2.0 * jitter_range - jitter_range;
    let jittered_secs = (unjittered.as_secs_f64() + jitter).max(0.0);

    Duration::from_secs_f64(jittered_secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grows_exponentially_before_capping() {
        let config = BackoffConfig {
            base: Duration::from_secs(1),
            max: Duration::from_secs(1000),
            jitter_fraction: 0.0, // disable jitter to test the curve precisely
        };
        assert_eq!(compute_backoff(1, &config), Duration::from_secs(1));
        assert_eq!(compute_backoff(2, &config), Duration::from_secs(2));
        assert_eq!(compute_backoff(3, &config), Duration::from_secs(4));
        assert_eq!(compute_backoff(4, &config), Duration::from_secs(8));
    }

    #[test]
    fn never_exceeds_max() {
        let config = BackoffConfig {
            base: Duration::from_secs(1),
            max: Duration::from_secs(30),
            jitter_fraction: 0.0,
        };
        assert_eq!(compute_backoff(10, &config), Duration::from_secs(30));
        assert_eq!(compute_backoff(100, &config), Duration::from_secs(30));
    }

    #[test]
    fn jitter_stays_within_bounds() {
        let config = BackoffConfig {
            base: Duration::from_secs(10),
            max: Duration::from_secs(1000),
            jitter_fraction: 0.2,
        };
        for _ in 0..100 {
            let delay = compute_backoff(1, &config).as_secs_f64();
            assert!(
                (8.0..=12.0).contains(&delay),
                "delay {delay} out of ±20% bounds"
            );
        }
    }
}
