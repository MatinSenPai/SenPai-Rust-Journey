# 04.3 — Anime catalog, Postgres-backed

The `Anime`/`WatchStatus`/`CreateAnime`/`UpdateAnime`/`AnimeError` domain
from `phase3-backend-foundations/02-axum-and-rest-api-design/02-anime-catalog-crud-in-memory`,
rebuilt so `AnimeStore` talks to real Postgres instead of a
`Mutex<HashMap<...>>`. Read that lesson again first if it's been a while —
this one is deliberately structured so you can diff the two side by side
and see exactly what persistence costs you in code, and what it doesn't.

## What's identical to the in-memory version

The public shape: `Anime`, `CreateAnime`, `UpdateAnime`, `AnimeError`,
every handler signature, and `app()`'s routing are unchanged. A caller of
this crate's axum app — a frontend, a test, `curl` — cannot tell whether
`AnimeStore` is backed by a `HashMap` or Postgres just from the HTTP
surface. That's the entire point of keeping the store's interface stable
across both lessons: the storage layer is an implementation detail behind
`AnimeStore`'s methods, not something that leaks into the API contract.

## What actually changes

- **Every `AnimeStore` method is now `async`** and returns errors that can
  originate from Postgres (`AnimeError::Storage(String)`, wrapping
  whatever `sqlx::Error` occurred), not just domain errors like
  `InvalidRating`.
- **`id` is `i64`, not `u64`.** Postgres has no native unsigned integer
  type — `BIGSERIAL` is a signed 64-bit auto-incrementing column — so this
  lesson's `Anime` uses `i64` throughout rather than fighting the database
  with casts at every query.
- **`WatchStatus` needs explicit `as_str`/`parse` conversions.** The
  in-memory version stored `WatchStatus` directly as a Rust enum; Postgres
  has no idea what a Rust enum is, so this lesson stores it as a `TEXT`
  column and converts at the boundary — `as_str()` going in,
  `WatchStatus::parse()` coming back out. (A real project reaching for
  more type safety here might use a Postgres native `ENUM` type instead of
  `TEXT`, at the cost of needing a migration every time a variant is
  added — `TEXT` is the simpler, more common default.)
- **`update` and `delete` call `self.get(id)` first.** Rather than writing
  a `SELECT`-then-check-existence dance twice, both methods reuse `get`
  to fetch (and validate the existence of) the current row — free
  `NotFound` handling, and one less place a bug could hide.

## Setup

Same shared `taskforge` database as modules 4.1 and 4.2:

```sh
DATABASE_URL=postgres://taskforge:taskforge@localhost:5432/taskforge \
  cargo test -p p3-04-03-anime-catalog-postgres-backed -- --ignored
```

`tests/store_test.rs` and `tests/api_test.rs` each `CREATE TABLE IF NOT
EXISTS` and `DELETE FROM p3_04_03_anime` at the start, so they're safe to
run repeatedly against the same shared database — but only because every
test in both files also carries `#[serial(p3_04_03_anime_db)]` (from the
`serial_test` crate). Without it, `cargo test`'s default concurrent
execution could let one test's `DELETE FROM p3_04_03_anime` fire while
another test (in either file) is mid-query against rows it just created,
since all nine tests share this one table.

## Your task

Open `src/lib.rs`. Implement `AnimeStore::create`, `get`, `list`,
`update`, and `delete`, then the five handlers
(`create_anime`/`list_anime`/`get_anime`/`update_anime`/`delete_anime`)
and `app()`.

## Next

```sh
DATABASE_URL=postgres://taskforge:taskforge@localhost:5432/taskforge \
  cargo test -p p3-04-03-anime-catalog-postgres-backed -- --ignored
```

Then `solution/SOLUTION.md` — but only after a real attempt.
