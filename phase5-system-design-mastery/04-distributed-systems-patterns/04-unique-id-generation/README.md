# 04.4 — Unique ID generation

No code in this lesson. Two ID strategies already show up side by side in
this repo, unremarked-on until now: `BIGSERIAL` auto-increment in every
`phase3-backend-foundations/04-postgres-and-sqlx/*` lesson, and `Uuid` in
`capstone-taskforge`. That wasn't an accident on either side — this lesson
explains why each choice fit its context, and what a third option
(Snowflake-style IDs) trades off against both.

## Why auto-increment breaks down with multiple writers

`BIGSERIAL` (used throughout `phase3-backend-foundations/04-postgres-and-sqlx`
— e.g. `id BIGSERIAL PRIMARY KEY` in the widgets and anime-catalog
migrations) works by having Postgres maintain a single sequence counter:
every `INSERT` grabs the next integer, atomically, from that one sequence.
That atomicity is exactly the problem the instant "one sequence" stops
being true. Two scenarios where it breaks:

- **Sharding.** If you split a table across two independent Postgres
  instances (shard A and shard B, each with its *own* sequence), both
  start counting from 1 independently. Shard A hands out `id=501` to one
  row; shard B, with no way to know what shard A has already handed out,
  also hands out `id=501` to a completely different row. The IDs are only
  unique *within* a shard, not globally — the instant you need to
  reference a row by ID across shards (a foreign key, a URL, a cache key),
  you have a collision.
- **Multiple writers to what's supposed to be one logical dataset**, even
  without sharding — e.g. an offline mobile client generating a row while
  disconnected, meant to sync later, or two independent services each
  trying to mint IDs for what's conceptually the same sequence without a
  single database arbitrating. Auto-increment assumes exactly one
  authority handing out numbers in order; anything that breaks that
  assumption breaks the guarantee.

Notice this is a single-writer-vs-multi-writer question, not a
"Postgres vs. something else" question — a single Postgres instance's
`BIGSERIAL` is perfectly fine at practically any realistic scale for a
system with one write path. The problem only shows up once you need more
than one authority minting IDs.

## UUIDs: no coordination, but a real index-locality cost

A UUID (v4, the random variant) needs zero coordination between writers —
any process, anywhere, can generate one with essentially no chance of
collision (122 random bits), with no shared counter, no network call, no
database round-trip. That's exactly why `capstone-taskforge` uses them:
`taskforge-core/src/job.rs`'s `JobId` wraps a `Uuid` directly
(`pub struct JobId(pub Uuid)`, generated via `Uuid::new_v4()` in
`JobId::new()`), and the `jobs` table's migration declares
`id UUID PRIMARY KEY`. Jobs in TaskForge can be created by *any* replica
of `taskforge-api` — potentially many, running concurrently behind a load
balancer (the horizontal-scaling setup the earlier CAP-and-scaling lesson
described) — and none of them coordinate with each other before minting a
job's ID. A `BIGSERIAL`-style single sequence would force every replica to
funnel through one authority (the database sequence) just to get an ID,
which is a fine cost for a single-writer teaching example but a real
coordination bottleneck once you're deliberately running N stateless
replicas specifically so no single instance is a bottleneck.

The real cost, worth naming rather than glossing over, especially given
the B-tree-vs-LSM material this repo covers elsewhere
(`phase5-system-design-mastery/02-database-and-storage-at-scale/05-indexing-deep-dive-btree-vs-lsm`):
**random UUIDs are bad for B-tree index locality.** A B-tree primary key
index wants new keys to arrive in roughly sorted order — that way new rows
land near the "right edge" of the tree, in pages that are already hot in
memory and don't require rewriting pages scattered all over the table.
`BIGSERIAL` gives you that for free: every new ID is strictly larger than
the last, so inserts are always append-like. A random UUID, by design, has
no relationship to insertion order at all — the next UUID `Uuid::new_v4()`
generates is equally likely to sort anywhere in the keyspace. That means
every insert potentially touches a *different*, effectively random page of
the primary key's B-tree index, which increases page splits, hurts buffer
cache hit rates, and can bloat the index over time compared to a
monotonic key — a real, measurable cost at high insert volume, not a
theoretical one. (TaskForge's insert volume doesn't come close to where
this bites in practice — it's a real cost that exists, not a mistake in
this specific system's design.)

## Snowflake-style IDs: sortable, compact, but need machine coordination

A Snowflake-style ID (the pattern Twitter popularized) is a single 64-bit
integer built out of three parts packed into the bits: a timestamp (often
milliseconds since some custom epoch), a machine/shard ID, and a sequence
number that increments within the same millisecond on the same machine.
Roughly: `[41 bits timestamp][10 bits machine id][12 bits sequence]` (exact
bit widths vary by implementation).

This gets you the coordination-free property UUIDs have — any machine
with its own assigned machine ID can mint IDs independently, no shared
counter, no per-ID network round-trip — *and* fixes UUID's index-locality
problem, because the timestamp occupies the high-order bits: IDs generated
later are numerically larger, so they still sort (and insert into a
B-tree) in roughly creation order, the same append-friendly pattern
`BIGSERIAL` gives you. They're also more compact than a UUID (8 bytes as a
plain integer vs. 16 bytes, and human-eyeballable/sortable-by-time without
decoding anything, unlike a UUID's opaque random bits).

The cost Snowflake IDs add back in: **machine-ID assignment needs
coordination.** Every ID-generating process needs a machine ID that's
guaranteed unique among all currently-running generators, or two machines
with the same ID can mint colliding IDs within the same millisecond. That
assignment problem is exactly a distributed-coordination problem again —
solved in practice with something like a Zookeeper/etcd-backed
registration step at process startup (this lesson's own module,
`02-idempotency-and-distributed-locking`, covers exactly that class of
tool), a fixed config-assigned ID per deployment slot, or a cloud
provider's instance metadata. It's not *no* coordination — it's
coordination moved from "every single ID" (a sequence) to "once per
process lifetime" (a machine-ID assignment), which is a much smaller and
less frequent coordination cost, but not zero.

## Why TaskForge chose UUIDs and the anime-catalog lessons chose BIGSERIAL

Put side by side, the choice in each part of this repo tracks the actual
constraint, not a blanket "UUIDs are more modern" preference:

- **`phase3-backend-foundations/04-postgres-and-sqlx`'s anime-catalog and
  widgets lessons use `BIGSERIAL`** because they're single-writer teaching
  examples — one process, one Postgres instance, no horizontal scaling in
  scope for those lessons. `BIGSERIAL` is simpler (no extra dependency,
  human-readable sequential IDs, best-case index locality), and simplicity
  mattered more than a scaling property those lessons deliberately don't
  need yet.
- **`capstone-taskforge` uses UUIDs (`JobId(Uuid)`, `Uuid::new_v4()`)**
  because jobs get created by potentially many `taskforge-api` replicas
  concurrently — there's no single auto-increment sequence to coordinate
  through without reintroducing exactly the kind of shared bottleneck
  running stateless replicas was supposed to eliminate. TaskForge doesn't
  use Snowflake-style IDs either, even though they'd fix the B-tree
  locality cost UUIDs pay — because that fix isn't free (it needs a
  machine-ID assignment scheme this repo hasn't built), and TaskForge's
  actual job-insert volume doesn't come close to where UUID index bloat is
  a proven problem worth that added complexity. Reach for Snowflake IDs
  when you've *measured* that cost mattering, the same "don't reach for
  it preemptively" principle the sharding lesson makes about sharding
  itself.

## Next

No `cargo test` for this lesson — it is a reading lesson.
