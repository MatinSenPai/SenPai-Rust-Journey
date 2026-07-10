# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can explain the design precisely, not
whether a test passes.

1. Sketch the `idempotency_keys` table design from this lesson. Why does
   the correctness of the whole scheme depend on `key` being an actual
   database-enforced `PRIMARY KEY`/`UNIQUE` constraint, rather than the
   application checking "does this key exist?" and then inserting as two
   separate steps?
2. Two retries of the same `POST /jobs` request, carrying the same
   `Idempotency-Key`, arrive at two different `taskforge-api` replicas at
   nearly the same instant. Walk through what happens to each one under
   the design in this lesson, and explain specifically why running two
   replicas doesn't reintroduce the double-enqueue bug.
3. Explain the difference between "insert a placeholder row first, then
   fill in the real response after doing the work" versus "do the work and
   the insert in one transaction." What failure mode does the placeholder
   approach handle that the single-transaction approach doesn't?
4. A junior engineer proposes replacing `taskforge-storage`'s
   `FOR UPDATE SKIP LOCKED` with a Redis `SETNX`-based lock, arguing "Redis
   is faster." Give the honest correctness objection to a naive TTL-based
   Redis lock that a Postgres row lock doesn't have.
5. When would reaching for Zookeeper or etcd actually be worth its
   operational cost, versus sticking with a lock in whatever database
   already holds the data? Give a concrete scenario where the answer tips
   toward Zookeeper/etcd.
