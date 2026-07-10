# Checkpoint

1. `get_or_set` takes `compute: F` as `FnOnce() -> Fut` rather than, say,
   just an already-computed `String`. Why does it matter that `compute` is
   lazy (only actually runs if there's a cache miss) instead of the caller
   computing the value up front and passing it in? What would you lose if
   `get_or_set` always evaluated `compute` before checking the cache?
2. `InMemoryCache::get` takes `&self`, not `&mut self`, even though an
   expired entry is still sitting in `self.store` after a miss (it isn't
   removed). Real Redis behaves similarly — an expired key isn't
   necessarily purged from memory the instant it expires. What's the
   trade-off here (bookkeeping cost vs. memory usage), and can you think of
   a reason a real cache might want to actively sweep expired entries
   anyway rather than only doing this "lazy" check-on-read?
3. Walk through what happens if two concurrent requests both get a cache
   miss for the same key at the same time (e.g. a sudden burst of traffic
   right when a popular entry's TTL expires): both call `compute`, both
   pay the full expensive cost, both write to the cache. This is called a
   "cache stampede" or "thundering herd." `get_or_set` as implemented here
   doesn't protect against it — what would you need to add (think about
   what a `Mutex` per key, or a "who's already computing this key" tracking
   structure, would buy you)?
4. Why does `FakeClock` use `Cell<Instant>` instead of a plain
   `Instant` field you could just reassign with `self.now = ...`? (Hint:
   look at the `Clock` trait's `now(&self)` signature — not `&mut self`.)
5. In the checkpoint for this lesson's README you read that TTL is a
   "safety net" against invalidation bugs, not a replacement for explicit
   invalidation. Give a concrete example (from this repo or a real app)
   where relying on TTL *alone*, with no explicit invalidation on writes,
   would produce a noticeably bad user experience.
