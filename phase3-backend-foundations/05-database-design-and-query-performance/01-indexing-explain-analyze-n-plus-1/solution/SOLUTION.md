# Solution

```rust
pub fn group_joined_rows(rows: Vec<JoinedRow>) -> Vec<PostWithComments> {
    let mut posts: Vec<PostWithComments> = Vec::new();
    let mut index_of: HashMap<i64, usize> = HashMap::new();

    for row in rows {
        let index = *index_of.entry(row.post_id).or_insert_with(|| {
            posts.push(PostWithComments {
                post: Post { id: row.post_id, title: row.post_title.clone() },
                comments: Vec::new(),
            });
            posts.len() - 1
        });

        if let Some(comment_id) = row.comment_id {
            posts[index].comments.push(Comment {
                id: comment_id,
                post_id: row.post_id,
                body: row.comment_body.unwrap_or_default(),
            });
        }
    }

    posts
}
```

The `HashMap<i64, usize>` is doing one job: remembering "I already created a
`PostWithComments` for this `post_id`, and it lives at index N in `posts`,"
so a post whose rows aren't contiguous (or a post that appears many times,
once per comment) only gets pushed into `posts` once.
`index_of.entry(row.post_id).or_insert_with(...)` is the idiomatic way to
express "look this up, and if it's not there, run this closure to produce
and remember a value" in one call — the closure both pushes the new
`PostWithComments` and returns its index, so `entry`/`or_insert_with`
threads that index straight back out. `comment_id` being `None` (the
commentless-post case, thanks to `LEFT JOIN`) simply skips the `if let
Some`, leaving that post's `comments` at whatever it already was — usually
still empty.

```rust
pub async fn posts_with_comments_naive(&self) -> Result<Vec<PostWithComments>, sqlx::Error> {
    let posts = self.list_posts().await?;
    let mut result = Vec::with_capacity(posts.len());
    for post in posts {
        let comments = self.comments_for_post(post.id).await?;
        result.push(PostWithComments { post, comments });
    }
    Ok(result)
}
```

This is the N+1 written out plainly: `list_posts().await?` is query #1, and
then the `for post in posts` loop calls `comments_for_post(post.id).await?`
— one full round trip to Postgres — **per post**. At 2 posts that's 3
queries total; at 10,000 posts it's 10,001. Nothing here is a bug in the
sense of "wrong code" — every individual query is correct SQL, correctly
executed, and the test suite's `naive_and_batched_agree_on_the_result`
proves the two methods return identical data. The bug is entirely about
**how many round trips** it took to get there, which is exactly why this
class of problem is so easy to miss in code review: nothing in the diff
looks wrong, it's the *shape* of the access pattern that's the problem, and
that only becomes visible once you count queries or watch latency at scale.

## Why the fix is a `LEFT JOIN` plus a pure regroup, not "just await less"

`posts_with_comments_batched` doesn't avoid the loop by being clever about
`await` — it avoids the loop by asking Postgres a fundamentally different
question. Instead of "give me post A's comments, then post B's comments,
...", it asks "give me every `(post, comment)` pairing in one shot" via
`LEFT JOIN`, and does the *grouping* — the part that used to be implicit in
"which loop iteration are we on" — explicitly, in Rust, after the data is
already in memory. That's the same trade every ORM's `prefetch_related`
makes under the hood: one bigger, flatter query plus client-side
reassembly, instead of many small, shaped-like-the-object-graph queries.

`group_joined_rows` being a plain, synchronous function (no `async`, no
`sqlx::Error`, no `&self`) is what makes it unit-testable without
Postgres — it's pure data transformation, the same "keep I/O at the edges,
pure logic in the middle" discipline as `InMemoryQueue` vs. `PostgresQueue`
in the toy job queue lesson, or `AnimeStore`'s CRUD methods vs. its axum
handlers. The three non-`#[ignore]`d tests in this crate all exercise
`group_joined_rows` directly with hand-built `JoinedRow` vectors — no
`#[tokio::test]`, no connection string, no `sudo service postgresql start`
— because the hard-to-get-right part of "fix the N+1" was never "can you
write a `LEFT JOIN`," it was "can you correctly reassemble flattened rows
back into a tree," and that part never needed a database in the first
place.

## Why `query_count` matters more than "it looks faster"

`AtomicU64` plus `fetch_add(1, Ordering::SeqCst)` on every query-issuing
method turns "the batched version *feels* faster" into something you can
assert on in a test:
`naive_makes_one_query_per_post_plus_one` and
`batched_makes_exactly_one_query_regardless_of_post_count` pin down the
exact query counts (3 vs. 1 at 2 posts; N+1 vs. 1 at N posts) the same way
you'd verify an N+1 fix by watching Django's debug toolbar SQL panel go
from 51 queries to 2. `Ordering::SeqCst` is the simplest (and here,
sufficient) memory ordering — this counter's only job is bookkeeping for
tests, not coordinating real concurrent access the way `InMemoryQueue`'s
`Mutex` does, so there's no correctness reason to reach for a weaker
ordering.

## On the checkpoint questions

**Q1 (what a B-tree buys you):** A bare table has no ordering a query can
exploit — Postgres has to check every row because nothing tells it where a
match *could* be short of looking. A B-tree index stores the indexed
column's values in sorted order (as a balanced tree, so lookups, inserts,
and range scans are all `O(log n)`), each pointing back to its row. "Find
`post_id = 42`" becomes "binary-search down the tree" instead of "scan
everything," which is the entire `Seq Scan` → `Index Scan` difference in
the README's `EXPLAIN ANALYZE` output.

**Q2 (indexes aren't free):** Every `INSERT`/`UPDATE`/`DELETE` touching an
indexed column has to also update that column's B-tree(s) — more indexes
means more write-time work and more disk space, even though reads get
faster. "Index everything" trades write throughput and storage for read
speed on columns that may never actually be filtered/joined/sorted on —
the right call is indexing the columns your real query patterns actually
need (foreign keys like `comments.post_id`, whatever shows up in a `WHERE`
or `ORDER BY`), not blanket coverage.

**Q3 (where the query-count difference comes from):** `list_posts` inside
`posts_with_comments_naive` fires 1 query; the `for post in posts` loop
then calls `comments_for_post` once per post, firing 1 more query each — 5
posts means `1 + 5 = 6` queries total for the naive path. `
posts_with_comments_batched` fires exactly 1 query (the `LEFT JOIN`) no
matter how many posts exist, because `group_joined_rows` — the part that
handles "many posts, many comments" — does its work entirely in memory
after that single query returns.

**Q4 (why the fix could be pure):** The N+1 problem is fundamentally an
access-pattern problem, not a data-transformation problem — once you have
*all* the rows in memory (from one `LEFT JOIN`), turning a flat list into a
grouped tree is ordinary in-memory work with no further need to talk to
anything external. The I/O boundary is "fetch the flattened rows"; what
happens to those rows afterward doesn't care where they came from, which is
exactly what let `group_joined_rows` be tested with hand-built `JoinedRow`
vectors instead of a real database.

**Q5 (why runtime-checked, not `query!`/`query_as!`):** The `!` macros
connect to a real database *at compile time* to type-check your SQL against
the live schema (or a `.sqlx` offline cache you'd have to keep in sync).
Using them here would mean `cargo check -p
p3-05-01-indexing-explain-analyze-n-plus-1` — and by extension `cargo build
--workspace` for the *entire* repo — would fail without a running,
correctly-migrated Postgres available during every build, in every
sandbox, on every contributor's machine, in CI. One lesson needing infra
just to type-check would threaten the whole workspace's build; the
runtime-checked API (`sqlx::query`/`query_as`, binding parameters and
mapping rows by hand) defers all of that to when the query actually
executes, which is precisely what makes `#[ignore]`-gating the
database-touching tests sufficient instead of necessary everywhere.

**Q6 (mapping back to Django):** `posts_with_comments_batched` is what
`Post.objects.prefetch_related("comments")` does for you automatically —
one extra query, results stitched back onto their parent objects before you
ever touch them. `posts_with_comments_naive` is what happens if you *skip*
`prefetch_related` and just iterate `post.comments.all()` inside a loop, a
template, or a serializer — each access lazily fires its own query, and
nothing in the Python syntax warns you that you just went from 1 query to
N+1.
