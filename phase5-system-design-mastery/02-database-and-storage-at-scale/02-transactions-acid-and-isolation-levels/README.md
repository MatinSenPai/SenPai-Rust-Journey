# 02.2 — Transactions, ACID, and isolation levels

No code in this lesson. You've been relying on ACID guarantees every time
you've run a query against Postgres in this repo without ever having to
name them — this lesson is about naming them precisely enough to reason
about what a database *does* and *doesn't* protect you from, because "it's
ACID, so it's safe" is not actually a specific enough claim to build a
correct system on.

## ACID, one letter at a time, with a concrete example each

**Atomicity** — a transaction's writes either *all* happen or *none* of
them do; there's no state where half of a multi-statement transaction
committed and the other half didn't. Concrete example: imagine a hypothetical
`transfer_funds` that debits account A and credits account B as two
separate `UPDATE` statements. If the process crashes after the debit but
before the credit, atomicity is what guarantees the debit never happened
either — wrapping both statements in `BEGIN; ... COMMIT;` means Postgres
either applies both or, on crash/rollback, neither. Without atomicity,
that crash would silently destroy money.

**Consistency** — every transaction takes the database from one valid state
to another valid state, where "valid" means every constraint you've
declared (`NOT NULL`, `CHECK`, foreign keys, uniqueness) holds, both before
and after. Concrete example: `phase3-backend-foundations/04-postgres-and-sqlx/03-anime-catalog-postgres-backed`'s
`validate_rating` plus the database's own column types mean a committed
transaction can never leave an `anime` row with a rating of `15` sitting in
the table — either the whole insert is rejected, or it never runs. This is
"consistency" in the ACID sense specifically — a narrower claim than the
C in CAP theorem (which is about *all nodes agreeing*, a distributed
property); ACID's C is about a *single* database never violating its own
declared invariants, even mid-transaction.

**Isolation** — concurrent transactions behave (from each transaction's own
point of view) as if they ran one at a time, even though the database is
actually interleaving their work for performance. Concrete example: two
`AnimeStore::update` calls running "at the same time" shouldn't be able to
see each other's uncommitted, half-written intermediate state — the
question of *how strictly* that's enforced is the whole subject of
isolation levels, below, and it's the letter where things get genuinely
subtle (and where `AnimeStore::update`'s real bug lives — more on that in
lesson 06 of this module).

**Durability** — once a transaction commits, its writes survive a crash,
full stop. Concrete example: `taskforge-storage`'s `PostgresJobStore::enqueue`
returns success to the caller, the process then immediately crashes (power
loss, OOM kill) — durability is the guarantee that the job is still in the
`jobs` table when Postgres comes back up, because Postgres wrote it to its
write-ahead log (WAL) on disk *before* reporting the `INSERT` as committed,
not just to an in-memory buffer that a crash would lose.

## The isolation levels, and what each one actually prevents

Isolation is the one letter that's a *spectrum*, not a binary. The SQL
standard defines four levels, each permitting fewer anomalies than the
last (at the cost of more locking/blocking or transaction retries):

| Level | Dirty read | Non-repeatable read | Phantom read |
|---|---|---|---|
| Read Uncommitted | Possible | Possible | Possible |
| Read Committed | Prevented | Possible | Possible |
| Repeatable Read | Prevented | Prevented | Possible (standard) / Prevented (Postgres) |
| Serializable | Prevented | Prevented | Prevented |

Three anomalies, defined concretely:

- **Dirty read** — transaction B reads a row transaction A has written but
  not yet committed. If A then rolls back, B just read data that, as far
  as the database is concerned, never happened. Example: A starts
  `UPDATE jobs SET status = 'succeeded' WHERE id = $1` but hasn't committed
  yet; under Read Uncommitted, B's concurrent `SELECT` could see
  `status = 'succeeded'` and act on it, then A rolls back (say, a later
  statement in A's transaction fails) and the job was never actually
  succeeded. Postgres does not implement Read Uncommitted's weak behavior
  at all — even its "Read Uncommitted" setting behaves like Read
  Committed — so this specific anomaly is not actually reachable in
  Postgres regardless of which level you ask for.
- **Non-repeatable read** — transaction B reads the same row twice within
  one transaction and gets two different values, because transaction A
  committed a change to that row in between B's two reads. Example: B
  reads a job's `status` as `Running` at the start of a long-running
  transaction, does some other work, reads the same job's `status` again
  later in the *same* transaction, and now sees `Succeeded` because A
  committed in the meantime — B's own transaction is internally
  inconsistent about what it saw.
- **Phantom read** — transaction B runs the same *range* query twice within
  one transaction and gets a different *set of rows* the second time,
  because transaction A committed an `INSERT`/`DELETE` that changed which
  rows match the query's `WHERE` clause. Example: B runs
  `SELECT * FROM jobs WHERE status = 'pending'` twice in one transaction; A
  commits a new pending job in between; B's second query sees a row (a
  "phantom") that wasn't there the first time. Note the distinction from a
  non-repeatable read: that's the same *row* changing value; a phantom is
  the *set of rows* changing membership.

Postgres's default is **Read Committed** — each individual statement sees a
fresh snapshot of committed data (so dirty reads are impossible), but two
statements *within the same transaction* can each see different committed
states if another transaction committed in between. Repeatable Read (which
Postgres implements via MVCC snapshotting, stricter than the SQL standard
technically requires — Postgres's Repeatable Read actually does prevent
phantom reads too, unlike the bare standard) takes one consistent snapshot
for the whole transaction. Serializable adds detection-and-abort: Postgres
tracks read/write dependencies between concurrent transactions and forces
one to abort with a serialization failure (that the caller must retry) if
letting both commit could produce a result impossible under any true
one-at-a-time ordering.

## Why `claim_next`'s `FOR UPDATE SKIP LOCKED` doesn't need Serializable

`taskforge-storage/src/postgres.rs`'s `claim_next` runs entirely inside
Postgres's default Read Committed level:

```sql
WITH claimed AS (
    SELECT id FROM jobs
    WHERE status = 'pending'
       OR (status = 'retrying' AND next_attempt_at <= now())
    ORDER BY created_at
    FOR UPDATE SKIP LOCKED
    LIMIT 1
)
UPDATE jobs SET status = 'running', updated_at = now()
WHERE id IN (SELECT id FROM claimed)
RETURNING *
```

It's worth being precise about *why* this is safe at Read Committed,
because "it's a queue claim, surely that needs the strongest isolation
level" is a reasonable but wrong instinct. The anomalies Serializable
guards against (non-repeatable reads, phantom reads across multiple
statements in one transaction) are about a transaction seeing an
inconsistent *view* across several separate reads. `claim_next` doesn't do
that — it's a *single* SQL statement (the CTE and the `UPDATE` are one
statement, not two round trips), and `FOR UPDATE` inside that single
statement takes a row lock at the moment of selection, not at the start of
some longer transaction that later has to stay consistent with a stale
snapshot. `SKIP LOCKED` then means a second worker's concurrent
`claim_next` simply never sees the row A already locked as a candidate at
all — there's no "phantom" to observe because the row that's unavailable
just doesn't appear in B's candidate set, full stop, not "appears then
mysteriously vanishes." The correctness property `claim_next` needs —
"no two workers ever claim the same row" — is provided entirely by the row
lock plus the atomicity of doing the select-and-update as one statement,
not by isolation level. Bumping this to Serializable would buy nothing
here and would cost real throughput: Serializable transactions that
conflict get aborted and must be retried by the *caller* (application-level
retry logic you'd have to write), whereas `SKIP LOCKED` gets equivalent
"never double-claim" safety by simply routing a losing worker to a
*different* row instead of forcing anyone to retry at all — strictly
better for a queue's throughput under contention.

Contrast this with a case that *would* need something stronger:
`phase3-backend-foundations/04-postgres-and-sqlx/03-anime-catalog-postgres-backed`'s
`AnimeStore::update` does `self.get(id)` as one round trip, then a separate
`UPDATE` later — two statements, with a gap between them where another
transaction can commit a conflicting write. That gap is exactly the shape
Repeatable Read or Serializable (or, more cheaply, an explicit
`SELECT ... FOR UPDATE` on the first read) exists to close. Lesson 06 of
this module covers that race and its fixes in depth.

## Next

No `cargo test` for this lesson — it is a reading lesson.
