# 01 — Caching with Redis

Every backend eventually hits the same wall: a database query, an external
API call, or an expensive computation gets requested far more often than its
underlying data changes. Caching is the general fix — keep a cheap, fast copy
of an expensive result close at hand, and only pay the expensive price again
when you have to. This module builds the most common caching pattern used in
production backends (the one you'd reach for behind a Django view or a Rails
controller just as much as an axum handler): cache-aside, with a TTL and
explicit invalidation.

1. [Cache-aside, TTL, invalidation](01-cache-aside-ttl-invalidation/README.md)
