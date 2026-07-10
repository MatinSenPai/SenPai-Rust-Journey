# 02 — Rate limiting & backpressure

Caching (module 01) protects your *database* from redundant work. Rate
limiting protects your *service* from too much work, period — a client (or a
bug, or an attacker) hammering an endpoint faster than you can safely serve
it. Backpressure is the related idea one layer down: what a system does when
it's asked to accept more than it can currently handle — reject early,
queue with a bound, or slow the caller down — instead of the naive failure
mode of just falling over (unbounded queues, exhausted memory, cascading
timeouts).

1. [Token bucket and `tower::limit`](01-token-bucket-and-tower-limit/README.md)
