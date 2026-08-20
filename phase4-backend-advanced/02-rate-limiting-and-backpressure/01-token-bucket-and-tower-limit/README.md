# 04.2.1 — Token bucket and `tower::limit`

Module 01 made reads cheap by caching. This lesson is about protecting a
service from being asked to do too much *work* in the first place — rate
limiting: rejecting or delaying requests once a client (or a bug, or an
attacker) exceeds an agreed-upon rate, instead of letting every request
through and hoping the downstream system (a database, a third-party API with
its own rate limit, your own CPU) keeps up.

## The token bucket algorithm

Picture a bucket that holds up to `capacity` tokens. It starts full. Every
request that wants to proceed must first take one token out of the bucket;
if the bucket is empty, the request is rejected (or made to wait, depending
on the variant). Meanwhile, tokens trickle back into the bucket continuously
at `refill_rate` tokens per second, capped at `capacity` (the bucket can't
overflow).

```
capacity = 5, refill_rate = 1 token/sec

t=0.0s  bucket: [#####] (5/5)  try_acquire -> true   (4/5)
t=0.0s  bucket: [####.] (4/5)  try_acquire -> true   (3/5)
t=0.0s  ... three more instantly ...                  (0/5)
t=0.0s  bucket: [.....] (0/5)  try_acquire -> false  (still 0/5, rejected)
t=2.5s  bucket: [##...] (2/5)  (2.5s * 1/sec = 2.5 tokens refilled)
        try_acquire -> true                            (1.5/5)
```

Two properties fall out of this design, both deliberate:

- **Bursts are allowed, up to `capacity`.** A client that's been idle can
  fire off `capacity` requests instantly — the bucket doesn't force a rigid
  "exactly one request every N milliseconds" cadence, which would be overly
  strict for bursty-but-generally-well-behaved traffic (a page load firing
  five API calls at once, say).
- **Sustained rate is capped at `refill_rate`, forever.** However bursty the
  traffic, it can never sustain more than `refill_rate` requests/second
  averaged over time, because that's the fastest the bucket can refill.

This is *the* standard rate-limiting algorithm — you'll find it (or its close
cousin, the leaky bucket) inside AWS API Gateway, Stripe's API limiter,
nginx's `limit_req`, and, in this ecosystem, `tower::limit::RateLimitLayer`.

## Backpressure: what to do when the bucket is empty

Rate limiting is one concrete instance of a bigger idea: **backpressure** —
what a system does when it's asked to accept more than it can currently
handle. There are three broad answers, and picking one deliberately (rather
than defaulting into whichever your framework does) is a real system-design
decision:

1. **Reject immediately** (this lesson's `try_acquire` — `bool`, no
   waiting). Simple, and the caller finds out instantly that it needs to
   back off. Right choice when "try again shortly" is an acceptable answer
   (most public APIs return `429 Too Many Requests` this way).
2. **Queue with a bound.** Accept the request but hold it until a token
   frees up, up to some maximum wait or maximum queue depth — beyond that
   bound, reject. This trades latency for a smoother client experience, at
   the cost of holding resources (a task, a connection) longer.
3. **Unbounded queueing (the failure mode to avoid).** Accepting everything
   and just letting the queue grow — memory grows without bound, latency
   grows without bound, and eventually the process falls over anyway, just
   later and messier than an immediate, honest rejection would have been.

## Why a struct you implement, not just reaching for `tower` directly

The point of this exercise is the *algorithm* — refill math, capping at
capacity, the accept/reject decision — not the crate. `TokenBucket` here is
plain, dependency-free, synchronous Rust: a `capacity`, a `refill_rate`, a
running `tokens` balance, and a `last_refill` timestamp. `try_acquire` takes
`now: Instant` as an explicit parameter (rather than calling
`Instant::now()` internally) for the same reason the caching lesson used a
`FakeClock`: tests need to assert "no tokens available yet" and "tokens
refilled after 2.5 simulated seconds" without an actual `std::thread::sleep`
making the test suite slow (or, at short enough durations, flaky).

In a real axum/tower service, you would not hand-roll this — you'd reach for
[`tower::limit::RateLimitLayer`](https://docs.rs/tower/latest/tower/limit/struct.RateLimitLayer.html),
a `Layer` you stack onto your router (`Router::layer(RateLimitLayer::new(num, per))`)
that applies the same token-bucket-style logic to every request automatically,
integrated with tower's `Service`/backpressure model (a saturated
`RateLimitLayer` makes `poll_ready` return `Pending` rather than immediately
erroring, which is the "queue with a bound" strategy above, bounded by
tower's own buffering). Understanding the algorithm this lesson builds by
hand is what makes `RateLimitLayer`'s two constructor arguments
(`num_requests`, `per: Duration`) legible instead of magic.

## Your task

Open `src/lib.rs`. Implement:

1. `TokenBucket::new` — starts the bucket full (`tokens == capacity`).
2. `TokenBucket::refill` (private) — advances `tokens` based on elapsed time
   since `last_refill`, capped at `capacity`, and updates `last_refill`.
3. `TokenBucket::try_acquire` — calls `refill`, then either takes one token
   and returns `true`, or returns `false` without changing `tokens`.

## Next

`cargo test -p p4-02-01-token-bucket-and-tower-limit`, then the recall questions,
then `solution/SOLUTION.md`.
