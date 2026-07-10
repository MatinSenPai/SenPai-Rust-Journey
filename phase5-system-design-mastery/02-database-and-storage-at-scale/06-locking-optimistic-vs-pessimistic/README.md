# 02.6 — Locking: optimistic vs. pessimistic

No code in this lesson. This module's lesson 02 (transactions and
isolation levels) already flagged that `AnimeStore::update` has a real gap
between its read and its write. This lesson is that gap's full treatment —
it's not a hypothetical example, it's a genuine bug this repo's own test
suite design left in deliberately, and it's the sharpest, most concrete
worked example this whole module has for locking.

## The bug: `AnimeStore::update`'s lost-update race

`phase3-backend-foundations/04-postgres-and-sqlx/03-anime-catalog-postgres-backed`'s
`AnimeStore::update` is a **plain read-then-write**:

```rust
pub async fn update(&self, id: i64, input: UpdateAnime) -> Result<Anime, AnimeError> {
    // 1. self.get(id).await? — a full round trip, its own SELECT
    // 2. merge input's Some(...) fields over the fetched current values
    // 3. a *separate* UPDATE statement, based on the merged values
}
```

(See the solution's shape in `solution/SOLUTION.md`.) Between step 1 and
step 3, nothing stops a second, concurrent call to `update` on the same
`id` from also reading the row (getting the same "before" state), also
merging its own change on top, and also writing — and whichever `UPDATE`
commits *second* wins, silently overwriting the first one's change, with
neither transaction ever checking "is the row still in the state I read it
in." This is exactly what `solution/SOLUTION.md`'s closing section, "The
lost-update race `CHECKPOINT.md` asks about," names directly: *"the second
`UPDATE` can silently overwrite the first one's write, because neither
ever checked 'is the row still in the state I read it in.'"* It's flagged
there as a deliberate, honest gap — not silently patched, a discussion
question instead.

**Concretely**: two requests both `PATCH /anime/42` at nearly the same
instant. Request A wants to bump the rating from 7 to 9. Request B — maybe
the user double-clicked, or a second device syncing — wants to change the
title. Both call `self.get(42)` and both see `{title: "Frieren", status:
Watching, rating: 7}`. A merges its change and writes `{title: "Frieren",
status: Watching, rating: 9}`. B, using the *stale* row it read (rating
still 7, because it read before A's write committed), merges its own title
change and writes `{title: "Frieren S2", status: Watching, rating: 7}` —
silently reverting A's rating bump back to 7, even though B's request
never mentioned rating at all. Neither request errored. Neither user was
told anything was wrong. The rating change is just gone.

## Fix 1: pessimistic locking (`SELECT ... FOR UPDATE`)

**Pessimistic locking** assumes conflict is likely enough to guard against
up front: lock the row for the entire read-modify-write, so no other
transaction can even *read* a lockable version of it until you're done.
Applied to `AnimeStore::update`, wrapped in an explicit transaction:

```sql
BEGIN;
SELECT id, title, status, rating FROM p3_04_03_anime WHERE id = $1 FOR UPDATE;
-- (merge in Rust, using this locked read)
UPDATE p3_04_03_anime SET title = $1, status = $2, rating = $3 WHERE id = $4;
COMMIT;
```

`FOR UPDATE` on the initial `SELECT` takes a row lock immediately — a
concurrent transaction's own `SELECT ... FOR UPDATE` (or `UPDATE`) on the
same row *blocks*, waiting until the first transaction commits or rolls
back, rather than reading a value that's about to become stale. Request B
above would simply wait for A's transaction to finish, then read A's
*already-committed* rating of 9 as its own "current" state — B's merge
would correctly preserve rating 9 while changing only the title. No lost
update, because B literally could not read until A was done.

This is the exact same primitive as `phase4-backend-advanced/03-background-jobs-and-message-queues/01-postgres-skip-locked-toy-queue`'s
`FOR UPDATE SKIP LOCKED` — both take a row lock via `FOR UPDATE` — but a
meaningfully different *variant*. The toy queue's `SKIP LOCKED` means a
second worker racing for the same row doesn't wait at all — it just moves
on to a different row, because for a job queue, "claim *some* available
job" and "claim *this specific* job" are equivalent; any pending job will
do. `AnimeStore::update`'s fix needs the opposite behavior: request B
specifically wants *this* row, id 42, updated correctly — skipping to a
different row isn't an option, so it has to block and wait its turn
instead. Same lock primitive, opposite policy for what a loser does: skip
to something else (queue claiming) vs. wait in line (updating a specific
record).

## Fix 2: optimistic locking (a version column)

**Optimistic locking** assumes conflict is *rare* and cheap to detect
after the fact rather than guard against up front: add a `version`
(or `updated_at`) column, read it along with everything else, and make the
`UPDATE` itself conditional on that version still matching what you read:

```sql
-- read: SELECT id, title, status, rating, version FROM p3_04_03_anime WHERE id = $1;
UPDATE p3_04_03_anime
SET title = $1, status = $2, rating = $3, version = version + 1
WHERE id = $4 AND version = $5;  -- $5 = the version read above
```

No lock is ever held across the read and the write — every transaction
reads and writes freely, no blocking. The safety comes entirely from the
`WHERE ... AND version = $5` clause: if another transaction committed a
change (and bumped `version`) in between this transaction's read and
write, the `UPDATE` matches zero rows (`rows_affected() == 0`), because
the `version` in the `WHERE` clause no longer matches what's actually in
the table. The caller checks that row count and, on zero, knows it lost a
race — and either retries (re-read the now-current row, re-merge, try
again) or surfaces a "this was changed by someone else, please retry"
error to the user, rather than silently clobbering their change. Applied
to the A/B scenario above: B's `UPDATE ... WHERE id = 42 AND version = 1`
(the version it read) matches zero rows, because A's commit already bumped
the version to 2 — B detects the conflict and can retry, re-reading the
now-current row (title unchanged, rating now 9) and re-applying just its
title change on top, correctly preserving A's rating bump this time.

## Which one for the anime catalog, and why

For `AnimeStore::update` specifically, **optimistic locking is the better
default**. The reasoning is about *contention*, not correctness — both
fixes are equally correct; the question is which one is cheaper given how
often conflicts actually happen here. An anime watchlist entry is,
realistically, edited by exactly one user, from at most one or two devices,
at a time — the odds of two concurrent `PATCH /anime/42` requests actually
overlapping are low. Pessimistic locking pays its cost (every request
takes a lock, however briefly, and any request unlucky enough to overlap
another has to sit and wait) on *every single update*, whether or not a
conflict would have happened. Optimistic locking pays essentially no cost
in the common, uncontended case (just one extra column in the `SELECT`/
`WHERE`) and only pays a retry cost in the rare case a conflict actually
occurs — which is exactly the shape you want when conflicts are rare:
cheap in the common case, correct (never silently wrong) in the rare
conflict case.

The calculus flips for a genuinely high-contention resource — which is
exactly `taskforge-storage`'s `claim_next`: *many* workers are constantly
racing for the *same small set* of pending jobs, all the time, by design.
Optimistic locking there would mean most `claim_next` calls fail their
version check and have to retry, over and over, under exactly the load
where you'd most want low latency — a bad fit. Pessimistic locking (`FOR
UPDATE`, with the `SKIP LOCKED` variant specifically) is the right call
there precisely *because* contention is the normal, expected case, not the
rare exception. The general rule this pair of examples illustrates:
optimistic locking wins when conflicts are rare and the cost of a lock
held "just in case" would be paid far more often than it's needed;
pessimistic locking wins when conflicts are the norm and paying the lock's
cost up front is cheaper than repeatedly detecting and retrying a
near-guaranteed conflict.

## Checkpoint

No `cargo test` for this lesson — go straight to `CHECKPOINT.md`.
