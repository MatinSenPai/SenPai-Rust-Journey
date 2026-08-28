# Solution

## `cache_key`

```rust
pub fn cache_key(namespace: &str, id: &str) -> String {
    format!("{namespace}:{id}")
}
```

Nothing fancier than string formatting — the value here isn't the code, it's
the *discipline* of always going through this function instead of hand-rolling
key strings at every call site, so every cache key in the codebase has the
same shape and namespaces never collide by accident.

## `InMemoryCache`'s `Cache` impl

```rust
async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
    match self.store.get(key) {
        Some((value, expires_at)) if self.clock.now() < *expires_at => Ok(Some(value.clone())),
        _ => Ok(None),
    }
}

async fn set(&mut self, key: &str, value: String, ttl: Duration) -> Result<(), CacheError> {
    self.store
        .insert(key.to_string(), (value, self.clock.now() + ttl));
    Ok(())
}

async fn invalidate(&mut self, key: &str) -> Result<(), CacheError> {
    self.store.remove(key);
    Ok(())
}
```

The match guard (`if self.clock.now() < *expires_at`) is the entire TTL
mechanism: an entry is a hit only if it exists *and* the clock hasn't passed
its stored expiry. `_ => Ok(None)` deliberately covers two different cases
with one arm — "never was set" and "was set but has expired" — because from
`get_or_set`'s point of view they're the same thing: a miss, recompute.

Note `set` doesn't check whether `key` already exists; `HashMap::insert`
overwrites unconditionally, which is exactly cache-aside's write behavior —
a fresh `set` always wins.

## `get_or_set`

```rust
pub async fn get_or_set<C, F, Fut>(
    cache: &mut C,
    key: &str,
    ttl: Duration,
    compute: F,
) -> Result<String, CacheError>
where
    C: Cache,
    F: FnOnce() -> Fut,
    Fut: Future<Output = String>,
{
    if let Some(value) = cache.get(key).await? {
        return Ok(value);
    }
    let value = compute().await;
    cache.set(key, value.clone(), ttl).await?;
    Ok(value)
}
```

This is the whole cache-aside pattern in five lines: check, early-return on
hit, otherwise compute-store-return. `compute: F` is generic over
`FnOnce() -> Fut` rather than accepting an already-evaluated `String` — that
laziness is what makes caching actually save work. If the caller had to
compute the value before calling `get_or_set`, they'd pay the expensive cost
on *every* call regardless of whether it turned out to be a hit, defeating
the entire purpose. The `compute_only_runs_once_on_repeated_hits` test
verifies this directly by counting calls with an `Rc<Cell<i32>>`.

## Worked answers

**1. Why lazy `compute`?** Covered above — laziness is the mechanism by
which a hit avoids paying the expensive cost at all. An eager API
(`get_or_set_eager(cache, key, ttl, value: String)`) would still let you
*store* efficiently, but the caller would already have done the expensive
work by the time they called it, cache hit or not.

**2. Why `&self` on `get`, leaving expired entries un-swept?** Removing an
expired entry on read would require `&mut self`, which would force every
caller of `get` (a read!) to hold an exclusive borrow — awkward, and
unnecessary for correctness since the expiry check already makes an
expired entry behave like a miss. The trade-off is memory: an entry nobody
reads again past its expiry sits in the `HashMap` forever, never freed.
Real caches (including Redis) do run a background sweep ("active expiry")
precisely to bound memory usage under exactly this scenario — write-heavy,
read-rarely-after-expiry workloads would otherwise leak memory unboundedly
under a lazy-only scheme.

**3. Cache stampede.** With `get_or_set` as written, N concurrent misses on
the same key all independently call `compute` and all independently call
`set` — the cache buys you nothing during that window, and your database
(or whatever `compute` calls) takes N times the load right when a popular
key expires, which is exactly the worst possible moment. The standard fix
is a per-key "in-flight" lock: the first caller to see a miss marks the key
as "being computed" (e.g. a `HashMap<String, tokio::sync::Mutex<()>>` or a
dedicated single-flight structure), other concurrent callers for the same
key wait on that lock instead of also calling `compute`, and whoever wins
the race populates the cache for everyone else to read.

**4. Why `Cell<Instant>` in `FakeClock`?** `Clock::now(&self)` takes a
shared reference, on purpose — a clock you can read from many places at
once (like a cache checked from many call sites) shouldn't need exclusive
access just to tell the time. But `advance` still needs to *mutate* the
stored value. `Cell<T>` is exactly "interior mutability for `Copy` types
accessed from one thread": it lets `now(&self)` (shared) coexist with
`advance(&self)` (still shared, but mutating through the cell) without a
`Mutex` or `RefCell`'s runtime borrow-checking, at the cost of not being
`Sync` — which is fine here, this lesson's futures were deliberately not
required to be `Send`/`Sync` (see the `Cache` trait's doc comment).

**5. TTL alone, no invalidation.** Imagine a user edits their profile bio
and the cached profile page has a 5-minute TTL with no explicit
invalidation on save. The user hits "save," is redirected to their own
profile, and sees their *old* bio for up to 5 minutes — on their own page,
right after editing it. That's a bad, confusing experience precisely
because it's self-inflicted and immediately visible, not some rare edge
case. Explicit invalidation on write (`cache.invalidate(&cache_key("user-profile", &user_id))`
right after the database write succeeds) fixes exactly this; the TTL stays
as the safety net for cases where invalidation was missed or failed.
