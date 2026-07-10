# Checkpoint

1. `AnimeStore::update` and `AnimeStore::delete` both start by calling
   `self.get(id).await?`. What two things does that one call buy each of
   them, and what would you have to write by hand in each method if it
   didn't exist?
2. `WatchStatus::parse` returns `Result<Self, String>` rather than
   panicking or just defaulting to some variant on an unrecognized value.
   Under what circumstance could this function's `Err` branch actually be
   reached, given that `as_str()`'s four strings are the only values this
   crate ever writes to the `status` column? Is that circumstance
   realistic enough to justify the `Result` (rather than an `unwrap()` or
   an `unreachable!()`), in your opinion?
3. The in-memory version's `Anime.id` was `u64`; this lesson's is `i64`.
   Why doesn't Postgres's `BIGSERIAL` support an unsigned 64-bit type
   directly, and what would you have had to do in every query if you'd
   insisted on keeping `id: u64` here anyway?
4. Compare `AnimeStore::list` here to the in-memory version's `list`. Both
   sort by id, but do so differently — one in Rust after fetching, one
   inside the SQL query itself (`ORDER BY id`). Is there a meaningful
   reason to prefer sorting in the database over sorting in application
   code, beyond "less Rust code to write"?
5. If two concurrent requests called `update_anime` on the same `id` at
   nearly the same instant, could the second update ever silently overwrite
   the first one based on stale data it read before the first update
   committed? Walk through what `AnimeStore::update` actually does
   (read-then-write, not a single atomic SQL statement) to answer this —
   and if you think there's a real race here, what earlier lesson in this
   repo showed a pattern (`FOR UPDATE`, or a single atomic `UPDATE ...
   WHERE` with the old value checked) that would close it?
6. Every test in both `tests/store_test.rs` and `tests/api_test.rs`
   carries the *same* tag, `#[serial(p3_04_03_anime_db)]` — not two
   different tags, one per file. Why does that specific detail matter,
   given the two files are separate test binaries that `cargo test`
   compiles and could otherwise run concurrently with each other?
