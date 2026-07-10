# 04.1 — Connecting and pooling

Django's `DATABASES` setting quietly manages a connection (or, with
`CONN_MAX_AGE`, a small reused pool) for you. This lesson builds that
management by hand with `sqlx`, so "connection pool" stops being a setting
you copy from a tutorial and becomes something you understand.

## Why a pool, not one connection

A naive approach — open a fresh TCP connection to Postgres for every
request, close it when done — works, but every connection costs a real
TCP handshake, TLS negotiation (if enabled), and Postgres-side
authentication round-trip before your query even runs. Under real traffic
that overhead dominates. The opposite naive approach — one single shared
connection for the whole process — falls over immediately once two async
tasks try to query concurrently (Postgres connections aren't safe to
multiplex like that).

A **pool** is the middle ground: open `N` real connections once, up front,
and hand them out to whichever task needs one for the duration of a single
query, then take them back. `sqlx::PgPool` is `Clone` (cheaply — it's an
`Arc` internally) and `Send + Sync`, so you build it once in `main` (or
once per test) and pass clones (or a shared reference) to everywhere that
needs to query.

```rust
let pool = PgPoolOptions::new()
    .max_connections(5)
    .acquire_timeout(Duration::from_secs(3))
    .connect(database_url)
    .await?;
```

- `max_connections(5)` — the hard ceiling. Once 5 queries are in flight
  simultaneously, a 6th caller waits for one to free up rather than opening
  a 6th real connection. Tuning this is a real production decision: too low
  and requests queue under load; too high and you can exhaust Postgres's
  own `max_connections` setting across all your app's replicas combined.
- `acquire_timeout(Duration::from_secs(3))` — bounds how long any caller
  will wait for a connection (whether that's "wait for the initial connect
  to succeed" or "wait for a busy pool to free one up") before giving up
  with an error, instead of hanging forever against an unreachable or
  overloaded database. This is what makes `a_bad_url_fails_to_connect_instead_of_hanging`'s
  test name true — without it, a bad connection attempt can hang far
  longer than any test (or, worse, any production request) should wait.

## `connect()` fails fast

Unlike some lazy-connection designs, `PgPoolOptions::connect(...)` eagerly
establishes at least one real connection before returning — a bad
`database_url` (wrong host, wrong password, database doesn't exist) fails
right there, with a clear `sqlx::Error`, rather than surfacing later as a
confusing failure on your first real query.

## Testing without hardcoding a connection string

Every test reads `DATABASE_URL` from the environment (`std::env::var`,
panicking with a clear message if it's unset) rather than hardcoding a
connection string — the same idea as Django's `settings.py` reading
`DATABASE_URL` from the environment instead of committing credentials to
source control. Run the live-database tests with:

```sh
DATABASE_URL=postgres://taskforge:taskforge@localhost:5432/taskforge \
  cargo test -p p3-04-01-connecting-and-pooling -- --ignored
```

## Your task

Open `src/lib.rs`. Implement `connect_pool` and `health_check`.

## Checkpoint

`cargo test -p p3-04-01-connecting-and-pooling` (passes trivially — no
`DATABASE_URL` needed since every DB-touching test is `#[ignore]`d), then
run the ignored tests with `DATABASE_URL` set as shown above, then
`CHECKPOINT.md`, then `solution/SOLUTION.md`.
