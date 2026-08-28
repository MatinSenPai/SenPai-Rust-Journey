# 03.2 — CDNs & latency numbers every engineer should know

No code in this lesson. This one is about building intuition for *why*
certain designs matter, not just recognizing their names. Two ideas: what a
CDN is actually solving, and a set of orders-of-magnitude you should be
able to reason with on a whiteboard without looking them up. The second
idea makes the N+1 problem — which you already found and fixed elsewhere
in this repo — make sense as a *latency* problem, not just a "too many
queries" problem.

## What a CDN actually does

A **CDN (Content Delivery Network)** is a set of servers ("edge nodes" or
"points of presence") distributed geographically around the world, each
holding a cached copy of content that would otherwise have to be fetched
from one central **origin server**. When a client requests something, it's
routed (usually via DNS) to whichever edge node is physically closest to
it, and if that edge node already has the content cached, it serves it
directly — no round trip to the origin at all.

The reason this helps is almost entirely about *distance*, not about the
origin server being slow. Light in fiber travels at roughly two-thirds the
speed of light in a vacuum — call it about 200,000 km/s. A request from a
user in Tokyo to an origin server in Virginia covers roughly 11,000 km each
way; even in a hypothetical world with zero processing time anywhere along
the path, the physics alone costs on the order of 100ms round trip. No
amount of origin-server optimization — a faster CPU, a better query plan,
more RAM — touches that cost, because it isn't a compute cost, it's a
distance cost. A CDN's answer is to move the *data* physically closer to
the *user*, so that Tokyo request instead travels a few km to a local edge
node and back.

**Where this applies well:** static or semi-static content — images,
videos, CSS/JS bundles, a blog post's rendered HTML, anything that's
identical for every requester (or nearly so) and doesn't change every
second. This is the overwhelming majority of what a CDN is built for, and
it's a close cousin of caching in general — same core idea (serve a
precomputed answer instead of recomputing), just at a different point in
the network topology: your `RedisCache` from the Phase 4 lesson caches
*computation* close to your application server; a CDN caches *content*
close to the *user*, often thousands of kilometers from your application
server entirely.

**Where it applies badly, or needs real design work:** personalized or
highly dynamic API responses. `GET /api/users/42/dashboard`, unique per
user and changing on every request, gains nothing from being cached at an
edge node in Tokyo — every user's dashboard is a different cache key, and
most of them are read exactly once before changing again, so the hit rate
would be close to zero. This doesn't mean CDNs are useless for dynamic
content, but it does mean the *cache key design* becomes the entire
problem: you can sometimes get real value by caching a personalized
response keyed on `(user_id, response_version)` with a short TTL, or by
separating a page into a cacheable static shell plus a small dynamic
fragment fetched separately (this is the idea behind "edge-side includes"
and modern edge-rendering approaches) — but none of that is "just turn the
CDN on," it's real design work, the same category of decision as picking a
cache key namespace in the Phase 4 lesson (`cache_key("user-profile", "42")`
already had to reckon with exactly this: a key that's too coarse mixes
unrelated data, a key that's too fine-grained never gets reused).

## Latency numbers every backend engineer should know

These aren't numbers to memorize to the digit — nobody should ever say
"main memory access is 100.7ns" with a straight face. What's worth having
in your head is the **order of magnitude**, and specifically the *ratios*
between them, because those ratios are what make certain designs
obviously right or obviously wrong before you've written a line of code.
This list (approximate, rounded, the same rough shape popularized as
"latency numbers every programmer should know") is worth being able to
reproduce from memory within a factor of 2-3x:

| Operation                                      | Rough latency  |
|-------------------------------------------------|----------------|
| L1 cache reference                               | ~1 ns          |
| Main memory (RAM) reference                      | ~100 ns        |
| SSD random read                                  | ~100 µs (100,000 ns) |
| Round trip within the same datacenter            | ~500 µs        |
| Round trip, same continent (e.g. coast to coast) | ~10-40 ms      |
| Round trip, cross-continent (e.g. California to Netherlands) | ~150 ms |

The ratios matter more than the absolute numbers: main memory is roughly
**100x** slower than L1 cache. An SSD read is roughly **1,000x** slower
than main memory. A same-datacenter network round trip is roughly **1,000x**
slower again than an SSD read, and a cross-continent round trip is roughly
**300x** slower still. Stack those up and a cross-continent network round
trip is on the order of **100 million times** slower than an L1 cache
reference. Nothing about how clever your code is changes that ratio — it's
a property of physics and network topology, the same fact that makes CDNs
work in the previous section.

The practical habit this list is meant to install: **every network round
trip is expensive relative to almost everything else your code does**,
even a round trip to a server in the same building. A query that takes
0.1ms of actual database work but requires a network hop to reach the
database is, from the caller's perspective, dominated by the *round trip*,
not the work. This is easy to forget because a single round trip feels
free when you're testing locally (the "hop" is `localhost`, effectively
zero distance) — the cost only becomes visible when you either add real
network distance, or add many round trips in a row.

## Why the N+1 problem is really a latency-numbers problem

`phase3-backend-foundations/05-database-design-and-query-performance/01-indexing-explain-analyze-n-plus-1`
already had you build (deliberately, as the exercise) the N+1 anti-pattern
in `BlogStore::posts_with_comments_naive`: one query to list posts, then
one *more* query per post to fetch that post's comments. That lesson's
README already points out the headline fact — "at 10,000 posts, that's
10,001 round trips to the database versus 1" — but it's worth making the
*mechanism* explicit using the latency numbers above, because "10,001
queries" sounds bad in a way that's easy to wave off ("well, each query is
fast") without actually being alarming until you attach real numbers to
it.

Say each individual query genuinely is fast — the actual work Postgres
does to answer `SELECT * FROM comments WHERE post_id = 42` with an index
in place, per the indexing lesson's `EXPLAIN ANALYZE` example, is well
under a millisecond of real database work. The naive access pattern's cost
isn't in that per-query work — it's in the **round trip** surrounding
every single one of those queries. Even for a database sitting in the same
datacenter as the application server (the best case — a truly local
`localhost` Postgres in dev is faster still, which is exactly why this bug
is invisible locally), that's ~500µs paid *per query*, on top of whatever
the query itself costs.

At **N = 1,000 posts**, `posts_with_comments_naive` issues 1,001 round
trips (1 for `list_posts`, 1,000 more for `comments_for_post`, one per
post) — this is exactly `BlogStore`'s `query_count()` counter proving the
point, the same number you'd read off a real APM trace or Django's debug
toolbar SQL panel. Even at a generous ~0.5-1ms per round trip (same-datacenter
network hop plus a small, real slice of connection-pool and query-parsing
overhead — a bit worse than the bare-minimum ~500µs number above, because
a real round trip is never *just* the network), that's roughly
**1,001 × ~1ms ≈ 1 second of wall-clock time spent purely on round
trips** — genuinely noticeable to a user, and the number only gets worse
as either the post count grows or the database gets physically farther
from the application server (imagine that same access pattern hitting a
read replica in a different region: swap in the ~150ms cross-continent
number instead of ~1ms, and 1,001 round trips becomes nearly **150
seconds** — the pattern that was merely slow becomes completely unusable
the instant it crosses a real network distance).

`posts_with_comments_batched`'s single `LEFT JOIN` query fixes this not by
making Postgres compute anything faster — the total *rows examined* is
similar either way — but by paying the round-trip cost exactly **once**
instead of N+1 times. That's the whole lesson compressed into one
sentence: **the N+1 problem is expensive because of round trips, not
because of query cost**, and now you have the actual numbers to say
precisely how expensive, instead of just "it's slow, don't do that."

## Next

No `cargo test` for this lesson — it is a reading lesson.
