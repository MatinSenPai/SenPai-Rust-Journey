# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can explain the mechanism behind the
`EXPLAIN ANALYZE` numbers you already saw, not just restate them.

1. `phase3-backend-foundations/05-database-design-and-query-performance/01-indexing-explain-analyze-n-plus-1`
   showed `EXPLAIN ANALYZE` output going from a `Seq Scan` (118.501ms) to
   an `Index Scan` (0.121ms) after adding a B-tree index on
   `comments.post_id`. Explain, structurally, why a B-tree's shape (many
   keys per page, shallow tree) produces that specific `O(n)` → `O(log n)`
   improvement.

2. Explain what happens, step by step, when a B-tree-indexed column is
   updated (`UPDATE comments SET post_id = 7 WHERE id = 42`), and why that
   makes B-trees comparatively expensive under heavy write load. Now
   explain what happens when an LSM-tree-backed store handles the same
   kind of write, and why that path is cheaper.

3. LSM-trees make reads more expensive than a B-tree's single tree-walk.
   Describe specifically what a read has to do in an LSM-tree (memtable,
   SSTables, compaction) that a B-tree read doesn't, and why compaction
   falling behind under sustained write load causes read latency to
   degrade.

4. Using the read/write ratio framework from this lesson, explain why
   Postgres defaults to B-tree indexes rather than an LSM-tree structure,
   and name one real system (from this lesson) that made the opposite
   default choice, plus the kind of workload that justifies it.

5. `phase3-backend-foundations/05-database-design-and-query-performance/01-indexing-explain-analyze-n-plus-1`'s
   `posts`/`comments` schema and `capstone-taskforge`'s `jobs` table both
   run on Postgres with B-tree indexes today. Compare their read/write
   ratios and explain why one of them sits much closer to "would
   theoretically benefit from an LSM-backed store at high enough scale"
   than the other, using specifics from `taskforge-storage/src/postgres.rs`'s
   `claim_next` to make your case.

6. `capstone-taskforge/docs/adr/0002-postgres-backed-queue.md` names
   "sustained throughput Postgres can't handle" as a trigger for
   revisiting the storage choice. Connect that trigger to this lesson's
   B-tree/LSM trade-off in your own words — what would actually be
   happening, mechanically, inside Postgres if TaskForge's job-claim rate
   grew by two or three orders of magnitude, that an LSM-backed
   alternative would avoid?
