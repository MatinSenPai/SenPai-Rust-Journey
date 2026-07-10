# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can explain each idea using a real example
from this repo, not whether a test passes.

1. `taskforge-storage`'s `PostgresJobStore` is CP-leaning. Describe a
   concrete scenario (a specific network partition) where that choice means
   a client gets an error instead of a possibly-stale answer. What would an
   AP-leaning version of the same job store do differently in that same
   scenario, and what could go wrong if it did?
2. Explain, specifically, why `taskforge-api` can be run as multiple
   replicas behind a load balancer with zero code changes, while a
   hypothetical version that cached job state in a `HashMap` field on
   `AppState` could not. What exactly would go wrong, and under what
   conditions would you actually notice it (i.e., what request pattern
   would expose the bug)?
3. Round-robin, least-connections, and consistent hashing are three load
   balancing strategies. Pick one system from earlier in this repo (the
   anime/manga aggregator side-quest is a good candidate) and explain which
   strategy you'd choose if it ran as 3 replicas, and why the other two
   would perform worse for that specific system.
4. `POST /jobs` in `taskforge-api` isn't idempotent today. Walk through the
   exact sequence of events (client action → network event → server action)
   that causes a job to be enqueued twice, and then describe, in your own
   words, the minimal change to `taskforge-core`/`taskforge-storage` that
   would fix it (you don't need to write the code — just the shape of the
   fix: what new data would need to be stored, and what check would need to
   happen before enqueueing).
5. `SELECT ... FOR UPDATE SKIP LOCKED` is described in this lesson as "a
   distributed lock." Someone unfamiliar with the term objects: "that's just
   a database query, not a *lock*." Explain why they're wrong — specifically,
   what property does `FOR UPDATE SKIP LOCKED` provide that a `std::sync::Mutex`
   in your worker process's memory could never provide, no matter how
   correctly you used it?
