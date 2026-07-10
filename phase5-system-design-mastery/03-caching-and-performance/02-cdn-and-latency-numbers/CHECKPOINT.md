# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can explain each idea using a real example
from this repo, not whether a test passes.

1. Explain, using the speed-of-light-in-fiber argument from this lesson,
   why a CDN edge node in Tokyo serving a Tokyo user is faster than an
   origin server in Virginia serving that same user, *even if* the origin
   server has a faster CPU and a warmer application-level cache than the
   edge node. What kind of cost is the CDN reducing, and why can no amount
   of server-side optimization touch it?
2. `GET /api/users/42/dashboard` (personalized, changes often) and
   `GET /static/logo.png` (identical for everyone, rarely changes) are
   both HTTP GETs. Explain why a CDN helps enormously for one and barely
   at all for the other by default, and describe one concrete change to
   how the dashboard endpoint is served that would let a CDN help there
   too.
3. Put these in order from fastest to slowest, and give the rough
   order-of-magnitude gap between adjacent items: SSD random read, L1
   cache reference, cross-continent network round trip, main memory
   reference, same-datacenter network round trip.
4. `BlogStore::posts_with_comments_naive` (from the Phase 3 indexing/N+1
   lesson) issues 1,001 round trips for 1,000 posts, each one individually
   fast. Explain specifically why the *individual query speed* is the
   wrong thing to look at here, and what number you should look at
   instead. Then explain why the exact same code would look fine in local
   testing but be a real production incident.
5. `posts_with_comments_batched` fixes the N+1 problem with a single
   `LEFT JOIN` instead of many small queries. In your own words: is the
   fix "Postgres does less total work," "the network is crossed fewer
   times," or both? Justify your answer using the `EXPLAIN ANALYZE`
   numbers from the indexing lesson and the round-trip-count argument from
   this one.
6. Suppose `BlogStore` connected to a read replica in a different region
   instead of a local database, and someone re-introduced
   `posts_with_comments_naive` on a page with 1,000 posts. Using the
   latency numbers from this lesson, roughly how much wall-clock time
   would that one page load cost purely in round trips, and how does that
   compare to the same code running against a same-datacenter database?
