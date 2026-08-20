# 04.1 — CAP theorem and consistency models

No code in this lesson.
`phase4-backend-advanced/06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking`
already covered the CAP theorem itself, with `taskforge-storage`'s
`PostgresJobStore` as the CP-leaning worked example. Go reread that
section if it's been a while — this lesson assumes you have it fresh and
does not re-derive it. What that lesson glossed over, on purpose, is what
the "C" in CAP actually *means* precisely, and that gap matters: most
systems that market themselves as "consistent" are not promising what CAP's
"C" promises.

## CAP's "C" is linearizability, and that's a much stronger claim than it sounds

CAP's Consistency is a specific, formal guarantee called
**linearizability**: every operation appears to take effect atomically at
some single instant between when it was called and when it returned, and
all nodes agree on that same total order. Concretely — if client A writes
`x = 5` and gets a success response, then client B (who starts reading
*after* A's write returned) is guaranteed to see `5`, not some older value,
no matter which node B talks to. There's no window where "the write
happened" but some node in the system hasn't heard about it yet from B's
perspective.

That's a much stricter bar than plain English "consistency" suggests, and
it's why CAP is easy to misapply in casual conversation: a system can
provide a real, useful, well-defined consistency guarantee — and still not
be "C" in the CAP sense, because CAP's "C" specifically means
linearizability, not "eventually correct" or "consistent most of the time."
The four models below are the actual menu of guarantees systems provide;
only the first one satisfies CAP's "C" as CAP defines it.

## The consistency models, concretely

**Strong consistency (linearizability).** Every read sees the most recent
write, full stop, as described above. This is what `taskforge-storage`'s
`PostgresJobStore` gives you: a single-primary Postgres instance is the
sole source of truth, so a `claim_next` call always sees every
`enqueue`/`mark_succeeded`/`mark_failed` that committed before it, with no
ambiguity about ordering. This is also the *expensive* option — it's
exactly the property that forces the CP choice during a partition,
because guaranteeing "no stale reads, ever" means refusing to answer at
all when you can't be sure you have the latest write.

**Eventual consistency.** Writes propagate to all nodes *eventually*, but
there's no bound on how long "eventually" takes, and a read immediately
after a write on a different node can return stale data. This is the
classic AP-leaning tradeoff the earlier lesson mentioned but didn't spell
out: **DynamoDB and Cassandra are the canonical real-world example.**
Both accept writes on any replica during normal operation and reconcile
divergent copies later (Cassandra via read-repair and hinted handoff,
DynamoDB via a similar background-reconciliation model) — the design
priority is "never refuse a write," at the cost of a client being able to
observe an old value shortly after a newer one was accepted elsewhere.
Concrete example: you increment a "like" counter on one DynamoDB replica;
a read against a *different* replica half a second later might still
return the pre-increment count. For a like counter, that's a fine trade —
nobody's harmed by a momentarily-stale count. For `taskforge-storage`,
that same laxity would mean two workers could each believe they'd
successfully claimed the same job, because "claimed" wouldn't be globally
agreed upon the instant it happened — which is exactly the double-run risk
the earlier lesson said TaskForge was built to avoid.

**Causal consistency.** Weaker than strong, stronger than plain eventual:
operations that are *causally related* (B happened because of A — B read
a value A wrote, or B was issued by the same client right after A) are
seen by every node in that same order. Operations with no causal
relationship (concurrent, independent writes from unrelated clients) can
be seen in different orders by different nodes, and that's fine. Concrete
example: you comment on a post, then someone else replies to your
comment. Causal consistency guarantees no reader ever sees the reply
before the comment it's replying to (that ordering is causally required
to make sense) — but it does *not* guarantee that two completely
unrelated comments on the same post appear in the same order to every
reader, because there's no causal link between them to preserve.

**Read-your-writes consistency.** The narrowest, cheapest guarantee: a
client is guaranteed to see its *own* writes on any subsequent read *it*
makes, even if other clients might briefly see stale data. Concrete
example: you update your profile bio, then reload your own profile page
and see the new bio immediately — even though the underlying store might
still be propagating that write to other read replicas that a *different*
user's request could land on. A common cheap implementation: route a
client's reads to the same replica it just wrote to (sticky sessions), or
have the client pass along the write's timestamp/version and have reads
refuse to serve anything older than it.

## Where these sit relative to each other

Strong consistency implies causal consistency implies read-your-writes
consistency — each is a strictly weaker promise than the one before it,
and a system built for a stronger guarantee automatically satisfies the
weaker ones. Plain eventual consistency is the odd one out: it makes no
ordering promise at all, causal or otherwise, which is what makes it the
cheapest to implement and the easiest to scale horizontally without
coordination — and also why it's the model DynamoDB and Cassandra default
to, since horizontal write scaling with zero cross-node coordination is
the entire point of those systems.

None of this is "the strong model is better." `taskforge-storage` needed
linearizability because two workers disagreeing about who owns a job is a
correctness bug, not a UX nitpick. A social feed's like counter, a
comment thread's read path, a user's own-profile reload — each of those
reaches for a progressively weaker (and progressively cheaper, more
available) model because the cost of a stale read in each case is
progressively closer to "nobody notices."

## Next

No `cargo test` for this lesson — it is a reading lesson.
