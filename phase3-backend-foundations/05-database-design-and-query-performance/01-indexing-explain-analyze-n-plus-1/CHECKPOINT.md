# Checkpoint

1. `EXPLAIN ANALYZE` on the unindexed `comments` table showed `Rows Removed
   by Filter: 999952` on a `Seq Scan`. In your own words, why does adding an
   index change that from "read practically the whole table" to "walk a
   tree straight to the matching rows" — what is a B-tree actually storing
   that a bare table doesn't?
2. Indexing isn't free. What does an index cost you on every `INSERT`,
   `UPDATE`, and `DELETE` against the indexed column, and why is "just
   index every column" a real anti-pattern rather than a safe default?
3. `posts_with_comments_naive` and `posts_with_comments_batched` return
   identical results (`naive_and_batched_agree_on_the_result` proves it) but
   very different `query_count()`s. Walk through, in your own words, exactly
   which line(s) in each method are responsible for the query count
   difference at 5 posts.
4. `group_joined_rows` is tested with plain `#[test]`, no `#[tokio::test]`,
   no database, no `#[ignore]`. Why was it possible to write the "fix the
   N+1" logic as a pure function at all — what had to be true about the
   *shape* of the problem for the regrouping step to not need any I/O?
5. This lesson uses `sqlx::query`/`query_as` (runtime-checked) rather than
   `sqlx::query!`/`query_as!` (compile-time-checked against a live
   database). What would break about `cargo check -p
   p3-05-01-indexing-explain-analyze-n-plus-1` in this sandbox — or in CI —
   if this crate used the `!` macros instead, and why does that matter for
   a single crate inside a much larger workspace?
6. In Django terms: which of this lesson's two methods is what
   `Post.objects.prefetch_related("comments")` does under the hood, and
   which is what happens if you forget to call `prefetch_related` at all
   and just iterate `post.comments.all()` in a template or serializer?
