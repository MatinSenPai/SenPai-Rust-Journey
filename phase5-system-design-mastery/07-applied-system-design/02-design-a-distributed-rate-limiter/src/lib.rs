//! Design a distributed rate limiter — takes the in-process token bucket
//! from `phase4-backend-advanced/02-rate-limiting-and-backpressure` and
//! side-quest 4's aggregator API, and fixes the gap both of those lessons'
//! own recall questions already named: an in-process limiter only limits *that
//! one process*. See `README.md` for the full requirements walkthrough
//! before touching any `todo!()` here.

use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum RateLimiterError {
    #[error("rate limiter backend error: {0}")]
    Backend(String),
}

/// A fixed-window counter, shared across every process talking to the same
/// Redis — the `key` is usually something like `format!("ratelimit:{user_id}")`,
/// scoping the limit per caller.
pub struct RedisRateLimiter {
    client: redis::Client,
    limit: u32,
    window: Duration,
}

impl RedisRateLimiter {
    /// `addr` is a Redis connection URL (e.g. `"redis://127.0.0.1/"`).
    /// `limit` is the maximum number of calls allowed per `window`.
    pub fn new(addr: &str, limit: u32, window: Duration) -> Result<Self, RateLimiterError> {
        let client =
            redis::Client::open(addr).map_err(|e| RateLimiterError::Backend(e.to_string()))?;
        Ok(Self {
            client,
            limit,
            window,
        })
    }

    async fn connection(&self) -> Result<redis::aio::MultiplexedConnection, RateLimiterError> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RateLimiterError::Backend(e.to_string()))
    }

    /// Returns `Ok(true)` if this call is allowed under `key`'s current
    /// window, `Ok(false)` if `key` has already hit `limit` for this
    /// window. See `README.md` for why this is one atomic Lua script
    /// rather than a separate `GET`/`INCR`/`EXPIRE` sequence of commands.
    pub async fn check(&self, key: &str) -> Result<bool, RateLimiterError> {
        todo!(
            "build a redis::Script::new(...) wrapping this Lua source: \
             \"local current = redis.call('INCR', KEYS[1]) if current == 1 then \
             redis.call('EXPIRE', KEYS[1], ARGV[1]) end return current\"; get a connection via \
             self.connection().await?; run script.key(key).arg(self.window.as_secs())\
             .invoke_async::<i64>(&mut conn).await, mapping redis::RedisError to \
             RateLimiterError::Backend(e.to_string()); return Ok(count <= self.limit as i64): {{}}"
        )
    }
}
