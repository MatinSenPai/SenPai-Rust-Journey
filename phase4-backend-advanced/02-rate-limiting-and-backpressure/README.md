# Module 2 — Rate limiting & backpressure

A service that accepts everything fails for everyone. This module covers the algorithms that decide who gets in, how to enforce one limit across many instances, and what a system should do when it is simply full: push back, shed load, or let the queue grow until it hurts.

1. [01 — Rate-limiting algorithms (token bucket, tower::limit)](01-token-bucket-and-tower-limit/README.md)
   — Token bucket, leaky bucket and windows, and what tower already gives you.
2. [02 — Distributed rate limiting with Redis and Lua](02-distributed-rate-limiting-redis-lua/README.md)
   — One limit shared across many instances, made atomic with a Lua script.
3. [03 — Backpressure, load shedding and queue depth](03-backpressure-and-load-shedding/README.md)
   — What a full system should do: push back, shed, or bound the queue.
