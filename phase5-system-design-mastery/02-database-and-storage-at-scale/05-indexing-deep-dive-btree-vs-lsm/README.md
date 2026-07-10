# 02.5 — Indexing deep dive: B-tree vs. LSM-tree

No code in this lesson. `phase3-backend-foundations/05-database-design-and-query-performance/01-indexing-explain-analyze-n-plus-1`
already had you build a B-tree index and read real `EXPLAIN ANALYZE`
output showing the `Seq Scan` → `Index Scan` shift it produces. This
lesson explains *why* a B-tree is what Postgres reaches for by default,
what the alternative (an LSM-tree) actually is, and — the part that
lesson didn't need to cover — what each one trades away.

## B-trees: what Postgres uses, and why

A **B-tree** (specifically, Postgres uses a B+-tree variant) keeps keys in
sorted order across a tree of fixed-size pages, each page holding many
keys, with the tree kept shallow (typically 3-4 levels even for millions
of rows) by giving each node many children. `CREATE INDEX idx_comments_post_id
ON comments(post_id)` — exactly what
`phase3-backend-foundations/05-database-design-and-query-performance/01-indexing-explain-analyze-n-plus-1`
has you run — builds one of these. The property that made the lesson's
`EXPLAIN ANALYZE` output go from `Seq Scan` (`O(n)`) to `Index Scan`
(`O(log n)`, about 21 comparisons at 2 million rows instead of up to 2
million) is a direct consequence of this shape: walking from the root to a
leaf touches only a handful of pages, each narrowing the search
substantially, because each page holds many keys.

The part that lesson didn't need to dwell on: an **update** to a B-tree
index happens **in place**. When you `UPDATE comments SET post_id = 7
WHERE id = 42`, Postgres has to find the old index entry (`post_id`'s old
value, pointing at row 42), remove it, and insert a new entry (`post_id`'s
new value, pointing at row 42) — a random-access write into whatever page
that new key belongs on, wherever it happens to be in the tree. If that
page isn't already in memory, it's a random disk read followed by a random
disk write. For a **read-heavy** workload with a comparatively low rate of
writes, this cost is easily amortized (pages you access get cached; writes
are infrequent enough that the occasional random I/O doesn't dominate).
That's exactly the access pattern of `posts`/`comments`: many more reads
(rendering a blog post and its comments) than writes (someone posting a
new comment), which is precisely why a B-tree is the right default for
that table shape, and why Postgres — a general-purpose relational database
whose most common workloads skew read-heavy — makes B-tree its default
index type without you having to ask for it.

## LSM-trees: the write-optimized alternative

A **Log-Structured Merge-tree** (LSM-tree) — what RocksDB, Cassandra,
LevelDB, and many other write-optimized stores use internally — takes a
completely different approach: never update anything in place. Writes
land first in an in-memory buffer (a **memtable**), and once that buffer
fills up, it's flushed to disk as a new, immutable, sorted file (an
**SSTable**). A read has to potentially check the memtable *and* several
SSTables (newest first, since a later write to the same key supersedes an
earlier one) — more work per read than a B-tree's single tree-walk. In the
background, a **compaction** process periodically merges multiple SSTables
together, discarding superseded/deleted entries and reducing the number of
files a read has to check.

The trade this makes is the mirror image of a B-tree's: writes become
**append-only, sequential** disk I/O (write to the end of the current
SSTable, or just to memory until it flushes) instead of random in-place
writes — sequential I/O is dramatically faster than random I/O on both
spinning disks and, to a lesser but still real degree, SSDs. This is why
LSM-trees are the structure of choice for write-heavy workloads: sensor
telemetry, event logging, Cassandra's wide-column model built for high
sustained write throughput across a cluster. The cost shows up on the read
side (checking multiple SSTables instead of one tree) and in the
background compaction work itself, which consumes real CPU and I/O
competing with foreground traffic — a badly-tuned LSM store under heavy,
sustained write load can suffer from "compaction can't keep up," where
SSTables pile up faster than they're merged and read latency degrades.

## The trade-off, side by side

| | B-tree | LSM-tree |
|---|---|---|
| Write pattern | In-place, random I/O | Append-only, sequential I/O |
| Read pattern | One tree-walk | Check memtable + multiple SSTables |
| Best for | Read-heavy, moderate write rate | Write-heavy, high sustained throughput |
| Background cost | None special | Compaction (real, ongoing CPU/I/O) |
| Used by (default) | Postgres, MySQL (InnoDB) | RocksDB, Cassandra, LevelDB |

Neither is "better" in the abstract — they're optimized for opposite ends
of the read/write ratio, which is exactly why a general-purpose relational
database defaults to B-tree (most applications, including every one in
this repo, are read-heavier than they are write-heavy) while
purpose-built, write-throughput-first stores default to LSM.

## Applying this to tables you've already built

`phase3-backend-foundations/05-database-design-and-query-performance/01-indexing-explain-analyze-n-plus-1`'s
`posts`/`comments` schema is a textbook read-heavy access pattern: a blog
post gets written once and read (rendered, along with its comments) many,
many times after that. A B-tree index on `comments.post_id` is exactly
right here — the lesson's own `EXPLAIN ANALYZE` numbers (`Seq Scan`'s
118ms down to `Index Scan`'s 0.121ms) are the payoff of optimizing for the
read path, and the occasional `INSERT` of a new comment paying a small,
in-place index-update cost is a complete non-issue at that read:write
ratio. If that same schema were instead a write-heavy firehose — imagine
logging every page view as a row instead of storing blog comments,
millions of inserts per hour, read only occasionally for analytics — an
LSM-backed store would be the better fit for the *storage engine*, even
though the *relational modeling* (rows with columns, `WHERE post_id = 42`)
might look identical on paper.

`capstone-taskforge`'s `jobs` table sits somewhere more interesting: it's
not read-*rarely*, but its write pattern is heavier and more contentious
than a blog's — every `claim_next` is an `UPDATE`, every `mark_succeeded`/
`mark_failed` is an `UPDATE`, and a busy queue can be doing many status
transitions per second, all against index-covered columns (`status`,
`next_attempt_at` — see the `WHERE status = 'pending' OR (status =
'retrying' AND next_attempt_at <= now())` clause in
`taskforge-storage/src/postgres.rs`'s `claim_next`). This is *still* a
reasonable case for Postgres's default B-tree — `docs/adr/0002-postgres-backed-queue.md`
already states the throughput ceiling explicitly ("thousands of jobs/second
on modest hardware," past which "a dedicated broker becomes justified") —
but it's a useful example of a workload that's genuinely closer to the
LSM side of the spectrum than a typical CRUD table is, which is exactly
why that ADR names "sustained throughput Postgres can't handle" as one of
its three explicit triggers for revisiting the storage choice later. If
TaskForge's job-claim rate ever grew by two or three orders of magnitude,
an LSM-backed queue implementation (or a purpose-built broker, covered in
lesson 07) would be trading Postgres's read/join convenience for exactly
the write-throughput profile this lesson describes.

## Checkpoint

No `cargo test` for this lesson — go straight to `CHECKPOINT.md`.
