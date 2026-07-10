# Solution

```rust
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

pub async fn insert_widget(pool: &PgPool, name: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("INSERT INTO p3_04_02_widgets (name) VALUES ($1) RETURNING id")
        .bind(name)
        .fetch_one(pool)
        .await
}

pub async fn count_widgets(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM p3_04_02_widgets")
        .fetch_one(pool)
        .await
}
```

## Why the test manually resets state first

This lesson's database (`p3_04_02_migrations`) is **not** wiped between
test runs — unlike an in-memory `HashMap` that's fresh every `cargo test`
invocation, Postgres state persists across runs. `_sqlx_migrations`
recording migration `0001` as already applied from a *previous* test run
would make a second run's assertion ("the table exists and is empty right
after migrating") meaningless — the table might already have rows from a
prior `inserted_widgets_are_counted` run. Dropping the table and deleting
its tracking row first gives the test a genuinely clean slate to migrate
into, so "migrating creates an empty table" is actually being tested, not
just assumed.

## Why this lesson gets its own database

Every other Postgres lesson in this repo shares the `taskforge` database
and avoids collisions by prefixing *table* names
(`p3_04_02_widgets`, `p3_04_03_anime`, `taskforge-storage`'s `jobs`). That
trick doesn't extend to `sqlx::migrate!`'s own bookkeeping table,
`_sqlx_migrations` — it isn't namespaced per crate, it's one table per
*database*, keyed only by a migration's numeric version. This lesson's
`0001_create_widgets.sql` and `taskforge-storage`'s own
`0001_create_jobs.sql` are both "version 1"; running both crates'
`sqlx::migrate!` against the same database would make the second one to
run see version 1 already recorded with a *different* file's checksum and
fail with `VersionMismatch` — and this lesson's own
`migrating_creates_the_widgets_table` test, which deletes the version-1
tracking row to get a clean slate, would silently corrupt whichever other
crate's migration that row actually belonged to. This isn't a hypothetical
edge case dreamed up for the README — it's a real collision this repo's
own test suite hit while these two lessons were both being built against
the same shared database, which is exactly why this lesson was moved to
its own dedicated `p3_04_02_migrations` database instead.

## Why every test needs `#[serial(p3_04_02_migrations_db)]`

`cargo test`'s default behavior — run every `#[test]` function
concurrently, on separate threads — is safe for the vast majority of this
repo's tests precisely because most of them touch nothing but their own
freshly-constructed in-memory state. This lesson is the exception: all
three tests hit the *same* live Postgres database, and
`migrating_creates_the_widgets_table` actively drops and recreates the one
table the other two depend on. Without `#[serial]`, this repo's own test
suite hit exactly this race while being built — `inserted_widgets_are_counted`
failed nondeterministically, only under default parallel execution, never
under `--test-threads=1` — because a concurrently-running
`migrating_creates_the_widgets_table` could drop the table between its
`before` and `after` counts. `#[serial(p3_04_02_migrations_db)]` forces
every test carrying that same tag to run one at a time, in some order,
never interleaved — turning "maybe flaky depending on thread scheduling"
into "deterministic every time," without needing `--test-threads=1` (which
would also unnecessarily serialize every *other*, unrelated test in the
same binary, if there were any).

## Adding `0002` vs. editing `0001` after the fact

Adding `0002_add_widget_price.sql` and running `run_migrations` against a
database that already has `0001` applied: exactly the intended workflow —
`0001` is skipped (already recorded), `0002` runs and gets recorded too.
This is the normal, safe way schemas evolve over time.

Editing the *already-applied* `0001_create_widgets.sql` file, however, is
a real footgun: `sqlx` records a checksum of each migration file's content
alongside its version. On the next `run_migrations` call against a
database that already applied the original `0001`, `sqlx` detects the
checksum mismatch and refuses to proceed with a `VersionMismatch` /
checksum error — deliberately, because silently re-running (or silently
ignoring) a changed migration against a database that already has the *old*
schema applied is exactly how migration history and actual database state
drift apart. The correct fix for "I need to change something I already
migrated" is always a *new* migration file that alters the existing
schema, never editing history in place — the same discipline Django
teaches by making you `makemigrations` again rather than hand-editing an
already-applied migration file.

## Why compile-time embedding matters for deployment

Because the migration SQL is baked into the compiled binary
(`sqlx::migrate!` reads the files at compile time via a proc macro), a
deployed release artifact is fully self-contained — copy one binary to a
server, run it, and it can migrate its own database on startup with no
separate `migrations/` directory needing to travel alongside it, no risk
of the binary and its migration files drifting apart because someone
forgot to copy one of them. Contrast with reading `.sql` files from disk
at runtime, which would require shipping and correctly locating a whole
extra directory next to (or bundled into) every deployment.

## vs. Django's `django_migrations`

Structurally identical idea: a tracking table recording which migrations
have been applied, checked before running new ones. The real difference is
where the migration *content* comes from. Django's `makemigrations` diffs
your model classes against the last-known schema state and *generates*
migration Python/SQL for you; `sqlx` has no models to diff against — you
hand-write every migration's SQL yourself, and `sqlx::migrate!` only
handles applying/tracking, not generating. This is a direct consequence of
Rust/`sqlx` not having an ORM's runtime model registry to diff against in
the first place — the tradeoff for `sqlx`'s "you write real SQL, get real
type-checking around it" philosophy over Django ORM's "the framework
writes the SQL for you."

## `query_scalar` vs. `query_as`

`query_scalar` is the right choice specifically when a query returns
**exactly one column** per row — it decodes that single column directly
into a plain Rust value (`i64` here), skipping the intermediate step of
decoding into a row/struct first. `query_as` is for queries returning
**multiple columns** that map onto a struct (or tuple) implementing
`FromRow` — exactly what `taskforge-storage`'s `JobRow` does for its
multi-column `jobs` table rows. Reaching for `query_scalar` on a
multi-column query wouldn't compile (nothing to decode `SELECT id, name,
created_at` into a single scalar), and reaching for `query_as::<_, i64>`
on `SELECT COUNT(*)` would technically work but is needlessly indirect
compared to the purpose-built `query_scalar`.
