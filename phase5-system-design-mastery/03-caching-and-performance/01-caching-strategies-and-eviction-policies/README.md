# 03.1 — Caching strategies & eviction policies

No code in this lesson. You've already built a real cache in this repo —
twice, in fact. What you built is one specific point in a much larger
design space: one write strategy (cache-aside) and one eviction policy
(TTL-only). This lesson maps the rest of that space and, for every point
in it, asks the same question a system design interview asks: what does
this trade away, and when would you actually pick it?

## What you already built

`phase4-backend-advanced/01-caching-with-redis/01-cache-aside-ttl-invalidation`'s
`Cache` trait, implemented by `InMemoryCache` and `RedisCache`, plus the
`get_or_set` orchestration function, is **cache-aside**: your application
code checks the cache, and on a miss, your application code computes the
real value and writes it into the cache. The cache is a passive, dumb
key-value store — it never initiates anything. `side-quests/sq-04-anime-manga-aggregator-api`'s
`TtlCache` + `get_or_fetch` is the exact same pattern, restated with a
`Mutex<HashMap<...>>` instead of Redis. Both are also **TTL-only eviction**:
the only way an entry ever leaves either cache is that its stored expiry
`Instant` has passed (`InMemoryCache::get`/`TtlCache::get` treat "present
but expired" and "absent" identically). Neither cache ever removes an
entry because the cache is "full," because it isn't — a `HashMap` (or
Redis) just keeps growing until something else decides to bound it. That's
a real, honest gap in both, not an oversight to be embarrassed about — it's
the natural starting point, and the rest of this lesson is what you'd add
if you needed to.

## The write-strategy menu

Cache-aside is one of (at least) four common answers to "who's responsible
for keeping the cache and the source of truth in sync, and when."

### Cache-aside (what you built)

```
read:  check cache -> miss -> compute from source of truth -> populate cache
write: write to source of truth -> invalidate (delete) the cache entry
```

The application owns both the read-miss path and the write-invalidation
path. Nothing happens automatically — every method (`InMemoryCache::get`,
`RedisCache::set`, etc.) does exactly what you called it to do and nothing
more.

- **Staleness risk**: bounded by TTL, but a missed invalidation between the
  write and the next TTL expiry serves genuinely wrong data.
- **Write latency**: fast — a write only ever touches the source of truth
  (plus a cheap `DEL`, not a re-`SET`).
- **Complexity**: lowest of the four — one trait, two methods used on the
  read path, one on the write path. This is exactly why the Phase 4 lesson
  picked it as the default to teach first.
- **Durability risk**: none beyond the source of truth's own guarantees —
  the cache is never the only place data lives.

### Write-through

```
write: write to cache AND source of truth, synchronously, as one logical operation
read:  check cache -> should basically always hit, since every write updated it
```

Every write updates both places before returning to the caller. Contrast
with cache-aside's write path, which only *deletes* the cache entry and
lets the next read recompute it — write-through instead recomputes (or is
handed) the new value immediately and pushes it into the cache itself.

**What it would take to add to your code:** `InMemoryCache`'s `set` already
writes to the `HashMap` — the missing half is a caller that also writes to
Postgres (or whatever source of truth) in the same operation, and only
returns success once *both* writes succeed. Today, nothing in the
cache-aside lesson calls `Cache::set` from a write path at all — writes
call `invalidate`. Write-through would mean introducing a new function,
something like `write_through(cache: &mut C, store: &Store, key, value)`,
that does `store.save(&value).await?` then `cache.set(key, value, ttl).await?`
— and deciding what happens if the second call fails after the first
succeeded (usually: log it and let the TTL self-heal, same safety net as
cache-aside already leans on).

- **Staleness risk**: lowest of the four — the cache is never observably
  behind the source of truth from a client's point of view (barring a mid-write
  crash between the two writes).
- **Write latency**: worse than cache-aside — every write now pays the cost
  of two operations instead of one, and waits for both.
- **Complexity**: a bit more than cache-aside — the write path has two
  failure modes to reason about instead of one.
- **Durability risk**: none — same as cache-aside, the source of truth is
  always written synchronously.

### Write-behind (write-back)

```
write: write to cache immediately, return to caller -> flush to source of truth asynchronously (batched, delayed, or on a timer)
read:  check cache -> hits reflect writes the source of truth may not have yet
```

The opposite trade from write-through: the write path returns as soon as
the cache is updated, and the source of truth catches up later, often
batched (accumulate 100 writes, flush them as one transaction) which can
be a genuine throughput win under heavy write load.

**What it would take to add to your code:** `InMemoryCache::set` already
does the "write to cache" half. The new piece is a background task —
something like a `tokio::spawn`ed loop that periodically drains recently-set
keys and persists them to Postgres — plus a decision about what happens if
the process crashes with writes sitting in the cache that never made it to
the source of truth. That last question is the entire reason this strategy
is used far less often than the other three: it's a genuine durability
trade, not just a performance knob.

- **Staleness risk**: inverted from the other strategies — it's the
  *source of truth* that's stale relative to the cache, briefly, which
  matters a lot if anything else reads directly from the source of truth
  (a second service, a nightly analytics job) without going through this
  cache.
- **Write latency**: best of the four — the caller only waits on the cache
  write, which is why this is the strategy of choice for write-heavy
  workloads (e.g. metrics/counters, high-frequency updates).
- **Complexity**: highest — you now need a background flush mechanism, and
  it needs its own failure handling.
- **Durability risk**: real and the whole point to weigh — if the cache
  (an in-memory `HashMap`, or Redis without persistence enabled) crashes
  before a flush, those writes are gone. This is a strict step up in risk
  from every other strategy on this list, which is exactly why it's not
  the default anywhere in this repo's capstone: `taskforge-storage` writes
  synchronously to Postgres for the same reason the CAP-theorem lesson's
  `PostgresJobStore` is CP-leaning — a job silently vanishing because a
  cache crashed is worse than a slower write.

### Read-through

```
read:  ask the cache for `key` -> cache itself fetches from the source of truth on a miss, populates itself, returns the value
write: usually paired with write-through, so the cache is always current
```

The difference from cache-aside is *who* is responsible for the miss path.
In cache-aside, `get_or_set` — your application code — calls `compute()`
and then `cache.set(...)` explicitly; the cache itself has no idea a miss
even happened, it just returned `None`. In read-through, the cache
implementation itself knows how to compute the real value on a miss — the
caller only ever calls one method (`get`) and never sees a miss as a
distinct code path. This is common in managed caching layers (some CDNs,
some ORM-level second-level caches) where the caching layer is handed a
"loader function" once, up front, rather than being a bare key-value store.

**What it would take to add to your code:** the `Cache` trait's `get`
would need to take (or be constructed with) something like the `compute:
F` closure that `get_or_set` currently takes as a *parameter on every
call*. Concretely, `InMemoryCache` would need a stored
`loader: Box<dyn Fn(&str) -> Fut>` field, and `get` itself would call it on
a miss instead of returning `Ok(None)` and making the caller decide. This
is a bigger structural change than write-through or write-behind — it
changes the shape of the trait, not just how callers use it — which is a
reasonable reason to leave it out of an intro lesson.

- **Staleness risk**: same as cache-aside, since it's usually paired with
  TTL or write-through.
- **Write latency**: not directly affected — this is a read-path pattern.
- **Complexity**: moderate — pushes complexity *into* the cache
  implementation, which simplifies every caller (no `get_or_set`
  boilerplate needed anywhere), a real ergonomic win at scale if you have
  many call sites.
- **Durability risk**: none — same as cache-aside.

### Summary table

| Strategy      | Staleness risk        | Write latency | Complexity | Durability risk |
|---------------|------------------------|----------------|------------|------------------|
| Cache-aside   | bounded by TTL/invalidation bugs | low  | lowest  | none |
| Write-through | lowest                 | highest        | low-medium | none |
| Write-behind  | source of truth is briefly stale | lowest | highest | real — crash loses unflushed writes |
| Read-through  | same as cache-aside    | n/a (read-path)| moderate   | none |

## Eviction policies

TTL answers "when does an entry stop being trustworthy." Eviction policies
answer a different question: "the cache is full — which entry do I throw
out to make room for a new one?" Your `InMemoryCache` and `TtlCache` never
face this question because neither has a capacity bound — a `HashMap`
happily grows forever. A real production cache (Redis with `maxmemory` set,
an in-process LRU cache, a CDN edge node with finite disk) always does,
which is why eviction policy is a separate decision from TTL, not a
replacement for it — production systems commonly run both at once (TTL
bounds *staleness*, eviction bounds *memory*).

- **LRU (Least Recently Used)** — evict whichever entry hasn't been
  *read* in the longest time. The good default for most workloads, because
  most real access patterns have temporal locality: something read
  recently is disproportionately likely to be read again soon (a user
  browsing their own recent posts, a product page that just went viral).
  Redis's own `maxmemory-policy allkeys-lru` is this.
- **LFU (Least Frequently Used)** — evict whichever entry has been read
  the *fewest total times*, regardless of how recently. Better than LRU
  when you have a small set of consistently "hot" items and a much larger
  set of items each read once and never again — LRU can get tricked by a
  burst of one-time reads (a scraper, a batch job) evicting genuinely hot
  entries just because they happen to be "less recent" at that instant;
  LFU wouldn't, because the hot entries' *count* still dwarfs the
  one-timers'. The cost: LFU needs to track a running count per entry
  forever (or with decay, to avoid an old count staying artificially high
  after an item stops being popular), which is more bookkeeping than LRU's
  "just remember order."
- **FIFO (First In, First Out)** — evict whichever entry was *inserted*
  first, full stop, ignoring whether it's been read since. Cheapest to
  implement (a queue), but ignores usage entirely — a FIFO cache can evict
  something read a thousand times a second, right after it was first
  inserted, just because it's chronologically oldest. Rarely the right
  default for a general-purpose cache; occasionally right for something
  like a fixed-size ring buffer of "last N events" where recency of
  *insertion*, not usage, is literally the point.
- **TTL-based (what you built)** — evict based on a per-entry expiry
  time, independent of how full the cache is or how often an entry is
  read. This is the right default when staleness, not memory pressure, is
  the thing you're actually defending against — which is exactly the case
  the Phase 4 lesson was teaching: a `RedisCache`-backed homepage feed
  isn't at risk of running out of memory over one key, it's at risk of
  serving a feed from 20 minutes ago after the underlying posts changed.

When would you reach for each as the *primary* policy?

- Small number of genuinely hot keys, general-purpose workload → **LRU**,
  by far the most common production default.
- Access pattern has a long tail of one-off reads that would otherwise
  evict your hot set under LRU → **LFU**.
- You specifically care about insertion order and usage is irrelevant (a
  bounded event log, a "last N" buffer) → **FIFO**.
- The risk you're managing is staleness, not memory → **TTL**, often
  layered under whichever of the above bounds memory.

### The classic LRU implementation, briefly

Worth knowing the shape even though you won't implement it here: the
standard `O(1)`-get, `O(1)`-put LRU cache is a `HashMap<Key, *Node>` for
lookup combined with a doubly linked list threading all entries in
recency order (most-recently-used at the head, least-recently-used at the
tail). A `get` looks the node up in the map (`O(1)`), then unlinks it and
re-splices it at the head of the list (`O(1)`, because it's a *doubly*
linked list — you can unlink a node given only a pointer to it, no
traversal needed). A `put` that exceeds capacity just pops the tail node
and removes its key from the map. The reason this needs a linked list at
all rather than, say, sorting by a timestamp on every access: a
`HashMap` alone gives you `O(1)` lookup but no notion of order, and
resorting on every `get` would make reads `O(n log n)` instead of `O(1)` —
which would defeat the entire point of a cache. This is a genuinely fun
Phase 2-level data-structures exercise (self-referential-ish structures
like a hand-rolled doubly linked list are exactly the kind of thing Rust's
ownership model makes annoying — you'll reach for `Rc<RefCell<...>>` or
indices into a `Vec` instead of real pointers) — worth doing once for its
own sake, but it's not part of this lesson; the point here is knowing the
shape of the answer, not implementing it.

## Next

No `cargo test` for this lesson — it is a reading lesson.
