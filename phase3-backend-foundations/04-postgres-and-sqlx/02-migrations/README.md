# 04.2 — Migrations

Django's `makemigrations`/`migrate` tracks schema changes as an ordered
sequence of versioned files, applies whichever ones a given database
hasn't seen yet, and records what's been applied so it never re-runs a
migration twice. `sqlx::migrate!` does the same job, minus the
auto-generation from model diffs — you write the SQL by hand, `sqlx`
handles ordering, tracking, and safe re-application.

## Reading `migrations/0001_create_widgets.sql`

```sql
CREATE TABLE p3_04_02_widgets (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

Filename convention: a numeric (or timestamp) prefix that determines
apply-order, then a short description — `sqlx` doesn't care about the
exact scheme as long as filenames sort into the order you want them
applied. Real projects usually use `sqlx migrate add <name>` (the `sqlx-cli`
tool) to generate a correctly-named, timestamped file rather than writing
the filename by hand.

## What `sqlx::migrate!("./migrations").run(pool)` actually does

1. Reads every `.sql` file in `./migrations/` at **compile time** (the path
   is resolved relative to the crate's `Cargo.toml`, and the SQL text gets
   embedded into your binary — no separate files need to ship with a
   deployed build).
2. At runtime, creates a `_sqlx_migrations` bookkeeping table in your
   database if it doesn't already exist (columns: version, description,
   a checksum of the file's contents, when it was applied).
3. For each migration file, in order: if its version isn't already
   recorded in `_sqlx_migrations`, runs it inside a transaction and
   records it; if it *is* already recorded, skips it entirely.

That third point is why `running_migrations_twice_is_a_safe_no_op` passes:
calling `run_migrations` again after the first successful run does nothing
on the second call — every migration is already accounted for — rather
than erroring with "relation already exists."

## Why every test carries `#[serial(p3_04_02_migrations_db)]`

`cargo test` runs different `#[test]` functions **concurrently** by
default (on separate OS threads), and all three tests here share one real
Postgres database — `migrating_creates_the_widgets_table` actively `DROP`s
the table the other two tests read from. Without `#[serial]` (from the
`serial_test` crate), that's a genuine race: if the drop happens to land
between another test's own queries, that test sees a table disappear out
from under it and fails — not because your `run_migrations`/
`insert_widget`/`count_widgets` logic is wrong, but purely because two
tests touched the same live database at the same instant. `#[serial(p3_04_02_migrations_db)]`
tells the test harness "never run any test carrying this same tag
concurrently with another" — the tests still run in one `cargo test`
invocation, just one at a time instead of interleaved, which is enough to
make three tests sharing one mutable Postgres database deterministic. (An
in-memory `Mutex<HashMap<...>>` test suite never needs this: each test
gets its own store, so there's nothing to race over — this is specifically
a live-database-integration-test problem.)

## Why the SQL itself doesn't need `IF NOT EXISTS`

A common instinct coming from ad-hoc SQL scripts is to defensively write
`CREATE TABLE IF NOT EXISTS` everywhere. Here, that's redundant: the
`_sqlx_migrations` tracking table is *itself* the idempotency guarantee —
`sqlx` won't attempt to re-run a migration it already has a checksum
record for, so the underlying SQL can (and, for clearer errors on genuine
double-application bugs, arguably should) assume a clean slate.

## Why this lesson uses its own database, not the shared `taskforge` one

Every other Postgres lesson in this module reuses the same
`taskforge` database and just prefixes its *data* table names
(`p3_04_02_widgets`, `p3_04_03_anime`) to avoid collisions — that works
because each lesson's tables are its own business. `_sqlx_migrations`,
the bookkeeping table `sqlx::migrate!` creates, is different: it isn't
prefixed per-crate, it's one shared table per *database*, keyed only by a
migration's numeric version. This lesson's `0001_create_widgets.sql` and
`capstone-taskforge/taskforge-storage`'s own `0001_create_jobs.sql` are
both "version 1" — if both ever ran their `sqlx::migrate!` against the
same database, the second one to run would find version 1 already
recorded with a *different* file's checksum and fail with a
`VersionMismatch` error, and worse, running this lesson's tests could
silently overwrite the capstone's own migration tracking. (This is not a
hypothetical — it's a real collision this repo hit while being built,
which is exactly why this lesson gets an isolated database instead.) Set
up a dedicated database once:

```sh
sudo -u postgres psql -c "CREATE DATABASE p3_04_02_migrations OWNER taskforge;"
```

## Your task

Open `src/lib.rs`. Implement `run_migrations`, `insert_widget`, and
`count_widgets`.

## Checkpoint

```sh
DATABASE_URL=postgres://taskforge:taskforge@localhost:5432/p3_04_02_migrations \
  cargo test -p p3-04-02-migrations -- --ignored
```

Then `CHECKPOINT.md`, then `solution/SOLUTION.md`.
