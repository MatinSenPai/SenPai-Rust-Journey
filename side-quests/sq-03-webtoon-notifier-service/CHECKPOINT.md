# Checkpoint

1. `check_all_webtoons` is a plain `async fn`, completely separate from the
   `tokio::spawn` + `tokio::time::interval` loop in `spawn_notifier`. Why
   does that separation matter for testing — what would `notifier_test.rs`
   have had to do differently (and worse) if the "did anything change"
   logic lived directly inside the `loop { interval.tick().await; ... }`
   block instead?
2. `ChapterChecker` uses `#[async_trait]`, but the cache-aside lesson's
   `Cache` trait (`phase4-backend-advanced/01-caching-with-redis/01-cache-aside-ttl-invalidation`)
   deliberately used plain `async fn` in the trait instead. What's
   different about how `ChapterChecker` gets used (specifically, in
   `AppState`) that makes `#[async_trait]` necessary here?
3. `check_all_webtoons` only counts a checker's answer as "new" when the
   reported chapter is *strictly greater* than the stored one, not merely
   `Some(_)`. Walk through what would go wrong if it accepted any
   `Some(chapter)` as a new release, using `SameChapterChecker` from
   `notifier_test.rs` as a concrete example.
4. `WebtoonStore` is in-memory — everything you're following is gone the
   moment the process restarts. The Phase 3 README blurb name-drops
   Postgres for this project. What's the smallest change you'd make to
   `WebtoonStore` to back it with a real database (hint: look at how
   `04-postgres-and-sqlx/03-anime-catalog-postgres-backed` — once you reach
   it — rebuilds `AnimeStore` against `sqlx`) without changing anything in
   `lib.rs` outside the store itself?
5. `RandomChapterChecker` is what `main.rs` actually runs with, but every
   test uses one of the deterministic checkers instead. Why would a test
   suite built entirely on `RandomChapterChecker` be a bad idea, even
   though it's the "more realistic" one?
