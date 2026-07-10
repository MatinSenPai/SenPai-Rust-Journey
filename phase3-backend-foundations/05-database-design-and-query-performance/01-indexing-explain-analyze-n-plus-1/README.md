# 05.1 — Indexing, `EXPLAIN ANALYZE`, and the N+1 problem

Two ideas, both about the same underlying fact: **the database does the
minimum work you told it to, not the minimum work you meant.** Without an
index, "find this row" means "look at every row." Without a batched query,
"give me these posts and their comments" means "ask once per post." Neither
mistake shows up on your laptop with 12 rows of seed data. Both show up in
production at 2am with 2 million rows, and by then it's an incident, not a
code review comment.

## Part 1 — what an index actually is

A table with no index is a stack of rows in no particular order as far as a
query is concerned. `SELECT * FROM posts WHERE title = 'Frieren'` with no
index on `title` means Postgres reads **every row**, checks whether its
`title` matches, and moves on — a **sequential scan** (`Seq Scan`), cost
`O(n)` in the number of rows. Doesn't matter if you want one row back or
all of them; the *scan* itself always visits every row.

`CREATE INDEX idx_posts_title ON posts(title);` builds a separate data
structure — a **B-tree** by default in Postgres — that keeps title values in
sorted order alongside a pointer back to each row. Looking something up in
a sorted B-tree is `O(log n)`: at 2 million rows, `O(log n)` is about 21
comparisons; `O(n)` is up to 2 million. That gap is the entire reason
indexes exist, and it only widens as tables grow — this is the same
intuition as a phone book (sorted, binary-searchable) versus a stack of
unsorted receipts.

The trade-off, because there always is one: an index isn't free. Every
`INSERT`/`UPDATE`/`DELETE` on an indexed column now also has to update the
index's B-tree, and the index itself takes disk space. Indexing every
column "just in case" is a real anti-pattern, not a safety net — index the
columns you actually filter, join, or sort on (usually foreign keys and
whatever appears in a `WHERE`/`ORDER BY`), not everything.

## Part 2 — reading `EXPLAIN ANALYZE`

`EXPLAIN ANALYZE <query>` actually *runs* the query and shows you the real
plan Postgres chose plus real timings (`EXPLAIN` alone shows the planned
cost *without* running it — useful when running the query for real would be
expensive or destructive). The output below is **illustrative** — hand-written
to show the shape you'll see, not copy-pasted from a real run — but it
matches what you'll get from a genuinely unindexed vs. indexed `comments`
table at meaningful scale.

**Before an index** on `comments.post_id`, finding one post's comments:

```
EXPLAIN ANALYZE SELECT * FROM comments WHERE post_id = 42;

                                            QUERY PLAN
----------------------------------------------------------------------------------------------
 Seq Scan on comments  (cost=0.00..21390.00 rows=48 width=72) (actual time=0.031..118.442 rows=48 loops=1)
   Filter: (post_id = 42)
   Rows Removed by Filter: 999952
 Planning Time: 0.112 ms
 Execution Time: 118.501 ms
```

- `Seq Scan on comments` — a full table scan, exactly as described above.
- `Rows Removed by Filter: 999952` — it read essentially the whole
  million-row table just to throw away everything that wasn't `post_id =
  42`.
- `actual time=0.031..118.442` — first row took 0.031ms, *all* rows took
  118.442ms. That's real, user-facing latency for one query.

**After** `CREATE INDEX idx_comments_post_id ON comments(post_id);`, the
same query:

```
EXPLAIN ANALYZE SELECT * FROM comments WHERE post_id = 42;

                                                  QUERY PLAN
---------------------------------------------------------------------------------------------------------
 Index Scan using idx_comments_post_id on comments  (cost=0.29..8.61 rows=48 width=72) (actual time=0.021..0.089 rows=48 loops=1)
   Index Cond: (post_id = 42)
 Planning Time: 0.098 ms
 Execution Time: 0.121 ms
```

- `Index Scan using idx_comments_post_id` — Postgres walked the B-tree
  straight to the matching rows instead of reading the whole table.
- `cost=0.29..8.61` versus `cost=0.00..21390.00` — the planner's own cost
  estimate dropped by roughly three orders of magnitude.
- `Execution Time: 0.121 ms` versus `118.501 ms` — roughly 1000x faster, on
  a table that will only get bigger.

The general reading habit: look for `Seq Scan` on a large table inside a
query that only wanted a few rows back — that's your signal an index is
missing. `cost=X..Y` is the planner's *estimate* (startup cost..total cost,
in arbitrary units, not milliseconds); `actual time=X..Y` and `rows=N` (once
you use `ANALYZE`) are what really happened. When they disagree wildly,
that's often a sign Postgres's table statistics are stale (`ANALYZE
<table>;` refreshes them) rather than the query itself being wrong.

## Part 3 — the N+1 problem

You've hit this exact shape before, just from the other side of an ORM.
Django's classic trap:

```python
posts = Post.objects.all()               # 1 query
for post in posts:
    print(post.comments.all())           # 1 query PER post — N more queries
```

`select_related`/`prefetch_related` exist specifically to collapse that into
one or two queries. sqlx has no ORM to hide this behind, which is a feature
for this lesson: the N+1 shape is right there in the code you write, not
buried in lazy-loading magic. `src/lib.rs`'s `BlogStore` gives you both
versions of the exact same query, over a tiny `posts`/`comments` schema:

```rust
// The trap: 1 query for posts, then 1 more query PER post for its comments.
pub async fn posts_with_comments_naive(&self) -> Result<Vec<PostWithComments>, sqlx::Error> {
    // list_posts() -> 1 query
    // for each post: comments_for_post(post.id) -> 1 query each
}

// The fix: exactly 1 query, via a LEFT JOIN, regardless of how many posts there are.
pub async fn posts_with_comments_batched(&self) -> Result<Vec<PostWithComments>, sqlx::Error> {
    // one LEFT JOIN query, then group_joined_rows() reassembles it in Rust
}
```

At 2 posts, `posts_with_comments_naive` costs 3 queries and
`posts_with_comments_batched` costs 1 — not a big deal. At 10,000 posts,
that's 10,001 round trips to the database versus 1. Every one of those round
trips pays real network latency even when the query itself is instant, which
is exactly why this bug is so easy to miss locally (fast local Postgres,
tiny seed data, nobody notices 51 queries instead of 1) and so painful in
production (real network hops, real row counts).

## Walking the starter code

- `Post` / `Comment` / `PostWithComments` — plain data, no surprises.
- `JoinedRow` — one row of the *flattened* `LEFT JOIN` result: one row per
  `(post, comment)` pair, with `comment_id`/`comment_body` both `None` for a
  post that has zero comments. That's the entire reason it's a `LEFT JOIN`
  and not a plain `JOIN` — a plain `JOIN` would make a commentless post
  disappear from the result set instead of appearing once with nulls.
- `group_joined_rows` — **pure, synchronous, no database at all.** Takes the
  flattened rows and regroups them into one `PostWithComments` per post.
  This is the part of "fix the N+1" that's actually worth unit testing
  without infrastructure, and it's what backs every test that runs by
  default with plain `cargo test`.
- `BlogStore` — the Postgres-backed piece: `connect`, `create_post`,
  `create_comment`, `list_posts`, `comments_for_post` are given, fully
  implemented, each incrementing `query_count` so tests can *prove* exactly
  how many round trips a given access pattern makes — the same number
  you'd read off Django's debug toolbar SQL panel.
- Every query here uses `sqlx::query`/`sqlx::query_as` (the runtime-checked
  API, `bind`-ing parameters and mapping rows into tuples by hand) rather
  than the `query!`/`query_as!` macros. The macro versions type-check your
  SQL against a real database *at compile time*, which would mean this
  crate could not `cargo check`/`cargo clippy` without a live Postgres
  connection available during every build — unacceptable for a lesson (and
  a workspace) that needs to compile without infrastructure. The
  runtime-checked API defers all of that to when the query actually runs,
  which is exactly why `#[ignore]` is the right tool: everything compiles
  and lints cleanly always, only the database-touching tests need
  `--ignored` and a live Postgres.

## Your task

Open `src/lib.rs`. Two `todo!()`s:

1. **`group_joined_rows`** — the pure regrouping logic. No database
   involved; `cargo test -p p3-05-01-indexing-explain-analyze-n-plus-1`
   exercises it directly.
2. **`BlogStore::posts_with_comments_naive`** — write the N+1 loop yourself:
   call `list_posts`, then `comments_for_post` once per post. You're not
   fixing the N+1 here — you're *building* it, on purpose, so the fix in
   `posts_with_comments_batched` (already given) means something concrete
   when you compare `query_count()` between the two.

## Seeing it for real (optional but worth doing once)

```sh
sudo service postgresql start   # if not already running
psql -U taskforge -h localhost -d taskforge -c "
  CREATE TABLE IF NOT EXISTS demo_posts (id serial primary key, title text);
  CREATE TABLE IF NOT EXISTS demo_comments (id serial primary key, post_id int, body text);
  INSERT INTO demo_posts (title) SELECT 'post ' || g FROM generate_series(1, 5000) g;
  INSERT INTO demo_comments (post_id, body) SELECT (g % 5000) + 1, 'comment ' || g FROM generate_series(1, 50000) g;
  EXPLAIN ANALYZE SELECT * FROM demo_comments WHERE post_id = 42;
  CREATE INDEX idx_demo_comments_post_id ON demo_comments(post_id);
  EXPLAIN ANALYZE SELECT * FROM demo_comments WHERE post_id = 42;
"
```

Compare the two `EXPLAIN ANALYZE` outputs yourself — you should see the same
`Seq Scan` → `Index Scan` shift as the illustrative example above, on real
data you generated.

Then run this lesson's own `#[ignore]`d tests against real Postgres:

```sh
DATABASE_URL=postgres://taskforge:taskforge@localhost:5432/taskforge \
  cargo test -p p3-05-01-indexing-explain-analyze-n-plus-1 -- --ignored --test-threads=1
```

## Checkpoint

`cargo test -p p3-05-01-indexing-explain-analyze-n-plus-1`, then
`CHECKPOINT.md`, then `solution/SOLUTION.md`.
