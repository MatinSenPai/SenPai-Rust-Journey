# Module 1 — Caching

The cheapest request is the one you never make. This module takes caching from the single Redis call to the design problems around it: which strategy fits which data, how a cache goes stale, what happens when a hot key expires under load, and why an in-process cache in front of Redis is often the right first move.

1. [01 — Redis from Rust](01-redis-from-rust/README.md)
   — The redis crate, connection handling, and the handful of commands a cache actually uses.
2. [02 — Cache strategies and invalidation](02-cache-aside-ttl-invalidation/README.md)
   — Cache-aside, write-through, TTLs, and the invalidation problem.
3. [03 — Stampede protection, single-flight and TTL jitter](03-stampede-protection-and-single-flight/README.md)
   — What happens when a hot key expires under load, and three ways to stop it.
4. [04 — In-process caching (moka) and the two-tier pattern](04-in-process-caching-and-two-tier/README.md)
   — A cache in your own process in front of Redis, and the consistency price of having two.
