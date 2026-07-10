# Checkpoint

1. `many_concurrent_queries_share_a_small_pool_safely` fires 20 concurrent
   queries at a pool capped at `max_connections(5)`. Walk through what
   actually happens to queries 6 through 20 while the first 5 are still
   running — are they rejected, dropped, or something else?
2. `connect_pool` calls `.connect(database_url).await` rather than some
   lazier "build a pool object but don't touch the network yet" API. What's
   the practical benefit of that, demonstrated by which test in this
   lesson?
3. If you raised `max_connections` from 5 to 500 in a real production
   service, what's the actual risk — where does the ceiling really come
   from if not "however high you want to set this number"?
4. `acquire_timeout` is set to 3 seconds here. What would you actually
   observe (in this lesson's own tests) if you removed it entirely — which
   specific test would change behavior, and how?
5. `PgPool` is cheap to `.clone()` (it wraps an `Arc` internally). In the
   concurrent-queries test, each spawned task gets its own `pool.clone()`
   rather than sharing one `&PgPool` reference across tasks. Why does
   `tokio::spawn`'s `'static` requirement force that, and what earlier
   lesson in this curriculum covered exactly this kind of `Arc`-cloning
   pattern for sharing state across tasks?
