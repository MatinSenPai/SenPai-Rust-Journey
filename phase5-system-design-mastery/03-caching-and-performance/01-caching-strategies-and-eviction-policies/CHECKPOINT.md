# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can explain each idea using a real example
from this repo, not whether a test passes.

1. `InMemoryCache::set` and `TtlCache::set` both only ever write to the
   in-memory store. Describe, concretely, what would have to change about
   the *calling code* (not necessarily the trait itself) to turn either
   one into a write-through cache. What new failure mode does that
   introduce that cache-aside's write path (`invalidate`, not `set`) never
   has to deal with?
2. Write-behind offers the lowest write latency of the four strategies but
   is described in this lesson as carrying "real durability risk." Walk
   through the exact failure sequence — what has to happen, in what order —
   for a write-behind cache to lose data that a cache-aside or
   write-through cache never would have lost.
3. `InMemoryCache` and `TtlCache` both use TTL-only eviction and no
   capacity bound. Give a concrete scenario where that becomes a real
   production problem (not a hypothetical — describe the actual resource
   that runs out and why), and explain whether adding an LRU policy on top
   of the existing TTL would fix that problem, replace TTL entirely, or
   need to coexist with it.
4. A cache is serving traffic for two very different key populations at
   once: a small set of celebrity user profiles read constantly, and a
   huge, ever-changing set of one-off search-result pages each read
   exactly once by whoever triggered that search. Explain why LRU is a
   worse fit here than LFU, specifically — what request pattern causes LRU
   to evict a profile it shouldn't.
5. Explain why a plain `HashMap<Key, Value>` can't implement `O(1)` LRU
   eviction on its own, and what the doubly linked list specifically adds
   that the hashmap can't provide by itself. (You don't need to write the
   code — describe what a `get` and an eviction do to the list, in words.)
6. Cache-aside and read-through both end up with the same data sitting in
   the same cache after a miss. What's the actual difference between them,
   and where does the responsibility for "what to do on a miss" live in
   each? Use `get_or_set` from the Phase 4 lesson as your cache-aside
   example when you answer.
