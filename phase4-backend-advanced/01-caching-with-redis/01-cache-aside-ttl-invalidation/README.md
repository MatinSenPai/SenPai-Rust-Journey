# 04.1.1 — Cache-aside, TTL, invalidation

Welcome to Phase 4. Every lesson from here on attaches a "system design"
idea to real, running code — this one is caching, probably the single
highest-leverage technique for making a backend fast and cheap to run.

## Why cache anything?

Say a Django view does `Post.objects.filter(published=True).order_by("-created_at")[:20]`
on every request to render a homepage. That query might take 40ms and hit
the database every single time, even though the underlying data only
changes a few times a minute. If the homepage gets 500 requests/second,
that's 500 queries/second against a database that's doing the *same work*
over and over, computing an answer that hasn't changed. A cache breaks that
loop: compute the expensive thing once, stash the answer somewhere very
fast to read (in this ecosystem, almost always [Redis](https://redis.io) —
an in-memory key-value store), and read the stashed answer on every
subsequent request until it's no longer trustworthy.

This is a **trade-off**, not a free lunch — you are deliberately choosing to
sometimes serve slightly stale data in exchange for speed and less load on
your source of truth. Nearly all of system design is making trade-offs like
this one deliberately, instead of by accident.

## The cache-aside pattern

"Cache-aside" (also called "lazy loading") is the pattern where your
application code — not the cache itself — is responsible for keeping the
cache in sync:

```
read path:
  1. ask the cache for `key`
  2. cache HIT  -> return the cached value, done (fast path)
  3. cache MISS -> compute the real value (DB query, API call, whatever)
                   -> store it in the cache
                   -> return it

write path (when the underlying data changes):
  1. write the real value to the source of truth (e.g. the database)
  2. invalidate (delete) the cache entry for that key, so the next read
     is forced to recompute a fresh value
```

The name contrasts with other strategies (write-through, where every write
also updates the cache immediately; read-through, where the cache itself
knows how to fetch on miss) that you'll encounter by name in real systems,
but cache-aside is the one you'll reach for by default because your
application already has the logic to compute the real value — the cache
doesn't need to know how.

## TTL: your safety net against invalidation bugs

"Cache invalidation is one of the two hard problems in computer science" is
a decades-old joke for a reason: it's very easy to write code that updates
the source of truth but forgets to invalidate (or invalidates the *wrong*
key for) the corresponding cache entry, leaving stale data served forever.

A **TTL** (time-to-live) is the safety net: every cache entry is stored with
an expiry time, after which it's treated as a miss even if it's still
technically present. You still invalidate explicitly on writes when you
can (that's the fast, correct path), but the TTL guarantees that even a
missed invalidation self-heals within, say, 60 seconds — bounded staleness
instead of unbounded staleness. Picking a TTL is itself a trade-off: too
short and you lose most of the caching benefit (constant recomputation);
too long and a missed invalidation stays wrong for a long time.

## Cache keys are part of your API

A cache is a flat namespace of `string -> string` (or `string -> bytes`).
Nothing stops two unrelated features from picking the same key and silently
corrupting each other's data — so key construction is a real design
decision, not an afterthought. The convention this lesson uses is a
`namespace:identifier` shape, e.g. `"user-profile:42"` or
`"homepage-feed:page-1"` — the namespace groups related entries (and makes
it easy to reason about "everything I need to invalidate when a user's
profile changes"), the identifier picks out the specific entity.

## Why a `Cache` trait instead of calling Redis directly everywhere

This lesson's `src/lib.rs` defines caching logic against a `Cache` trait,
not the concrete `redis` crate — this is **dependency inversion**: the
cache-aside logic (`get_or_set`) is written once, generically, against
"anything that can get/set/invalidate a string by key," and doesn't care
whether that's backed by real Redis or an in-memory `HashMap`. Two
consequences:

1. **Testability without infrastructure.** The interesting logic here isn't
   "how do I talk to Redis" (that's a few lines of `redis` crate calls) —
   it's "did I check the cache before recomputing, did I honor the TTL, did
   I invalidate correctly." `InMemoryCache` (a `HashMap`-backed
   implementation of the same `Cache` trait) lets every test in this lesson
   run against real cache-aside *logic*, deterministically, with zero
   external processes. This mirrors a very common real-world pattern:
   Django's cache framework ships a `LocMemCache` backend for exactly this
   reason, and most ORMs/HTTP clients have an equivalent "fake" for tests.
2. **This sandbox (and a future learner's laptop) can't assume Redis is
   running.** `redis-server` is installed here but not started, and
   `cargo test` must be green with zero external services. Tests that need
   a genuinely live Redis are marked `#[ignore]` (see below) — they exist
   so you *can* verify the real thing works, just not as part of the
   default test run.

## Your task

Open `src/lib.rs`. The `Cache` trait, `CacheError`, `SystemClock`, and
`FakeClock` are given (read them — `FakeClock` is worth understanding, it's
how the TTL tests avoid ever calling `sleep`). You implement:

1. `cache_key` — the namespace:identifier key-construction helper.
2. `InMemoryCache`'s three `Cache` methods (`get`, `set`, `invalidate`),
   including the TTL expiry check.
3. `get_or_set` — the cache-aside orchestration function itself.

`RedisCache` (the real backend) is given, fully implemented, so you can see
what production code calling the actual `redis` crate looks like. Its own
tests are `#[ignore]`d — run them for real practice with:

```sh
redis-server --daemonize yes   # or: redis-server & (any local Redis works)
cargo test -p p4-01-01-cache-aside-ttl-invalidation -- --ignored
```

## Checkpoint

`cargo test -p p4-01-01-cache-aside-ttl-invalidation`, then `CHECKPOINT.md`,
then `solution/SOLUTION.md`.
