# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can apply the vocabulary to a decision,
not recite the three definitions.

1. Explain, in one or two sentences each, what a forward proxy protects and
   what a reverse proxy protects. Then explain why "load balancer" and
   "reverse proxy" aren't actually synonyms, even though every load balancer
   discussed in this repo is a reverse proxy.
2. `phase4-backend-advanced/02-rate-limiting-and-backpressure/01-token-bucket-and-tower-limit`
   implements rate limiting inside the application (a `TokenBucket`, or
   `tower::limit::RateLimitLayer` on the service's own `Router`) rather than
   at an API gateway sitting in front of it. Describe one concrete way that
   choice would break, or behave unexpectedly, if that service were run as
   three replicas behind a load balancer — and describe one concrete
   advantage the in-app approach has that a generic gateway couldn't
   replicate without extra work.
3. Round-robin, least-connections, and consistent hashing. For each, describe
   a request pattern (not from this lesson) where it would clearly be the
   wrong choice — i.e., name the specific way each algorithm fails, not just
   what it does.
4. A load balancer is distributing traffic across three replicas of a service
   that accepts WebSocket connections (Lesson 2). Explain, specifically, why
   plain round-robin is actively broken for this traffic in a way it isn't
   for a stateless REST API, and what the load balancer needs to do instead.
5. Someone on your team wants to add JWT auth checking, per-client rate
   limits, and version-based routing (`/v1/*` vs `/v2/*`) all at the same
   nginx instance that's currently "just" load balancing three replicas of
   one service. Using this lesson's vocabulary, explain what that nginx
   instance is now functioning as, and why that's a meaningfully different
   role than what it was doing before, even though it's still the same
   process.
