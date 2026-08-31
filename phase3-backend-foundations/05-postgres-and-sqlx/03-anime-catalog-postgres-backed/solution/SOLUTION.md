# Solution

```rust
pub async fn create(&self, input: CreateAnime) -> Result<Anime, AnimeError> {
    validate_rating(input.rating)?;

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO p3_04_03_anime (title, status, rating) VALUES ($1, $2, $3) \
         RETURNING id",
    )
    .bind(&input.title)
    .bind(input.status.as_str())
    .bind(input.rating)
    .fetch_one(&self.pool)
    .await
    .map_err(|e| AnimeError::Storage(e.to_string()))?;

    Ok(Anime {
        id,
        title: input.title,
        status: input.status,
        rating: input.rating,
    })
}

pub async fn get(&self, id: i64) -> Result<Anime, AnimeError> {
    let row: Option<(i64, String, String, Option<i16>)> =
        sqlx::query_as("SELECT id, title, status, rating FROM p3_04_03_anime WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AnimeError::Storage(e.to_string()))?;

    let (id, title, status, rating) = row.ok_or(AnimeError::NotFound)?;
    let status = WatchStatus::parse(&status).map_err(AnimeError::Storage)?;
    Ok(Anime { id, title, status, rating })
}
```

`list` is the same shape as `get`, minus the `WHERE`, mapped over every
row with `.collect::<Result<Vec<_>, _>>()` to short-circuit on the first
`WatchStatus::parse` failure. `update` and `delete` both call `self.get(id)`
first to get `NotFound` handling for free, then run their own
`UPDATE`/`DELETE` statement.

## What changed vs. the in-memory version, concretely

Every method gained an `await` and a `.map_err(|e| AnimeError::Storage(e.to_string()))?`
— the price of talking to a real database instead of a `Mutex`, which
can't fail. `id` became `i64` (Postgres `BIGSERIAL` has no unsigned
counterpart). `WatchStatus` gained `as_str`/`parse` because Postgres has
no concept of a Rust enum — it's stored as `TEXT` and converted at the
boundary, the same "convert to a wire/storage representation, convert
back" shape as `serde`'s `Serialize`/`Deserialize`, just hand-written
instead of derived, since a `TEXT` column isn't JSON.

## Why `get` parses `WatchStatus` instead of trusting the column

`WatchStatus::parse` returns `Result<Self, String>` rather than assuming
every row's `status` column is one of the four strings `as_str()` writes.
In practice, since this crate is the only thing that ever writes to
`p3_04_03_anime`, that assumption always holds — but "the only writer
today" is a fragile guarantee to build unchecked code on top of (a future
migration, a manual `psql` fix, a second service sharing this table would
all be ways for an unexpected value to appear). Returning a `Result` and
converting it to `AnimeError::Storage` costs one line and turns "silent
wrong behavior or a panic deep in match ergonomics" into "a clean 500 with
a diagnosable message" if that assumption is ever wrong.

## Why every test carries `#[serial(p3_04_03_anime_db)]`

`tests/store_test.rs` and `tests/api_test.rs` are two separate test
binaries that `cargo test` can run concurrently with each other, and
*within* each file every test also runs concurrently by default —  but
all nine tests across both files share the one `p3_04_03_anime` table,
and `fresh_store`/`fresh_app` both `DELETE FROM p3_04_03_anime` before
each test. Without `#[serial]`, one test's delete can land while another
test (in either file) is mid-`INSERT`/`SELECT`, producing exactly the kind
of nondeterministic failure this repo's own test suite hit while this
lesson was being verified — passing reliably under `--test-threads=1`,
failing intermittently under `cargo test`'s default concurrency. Tagging
every test with the *same* `#[serial(p3_04_03_anime_db)]` string (not two
different tags, one per file) is what makes the exclusion span both
binaries: `serial_test` only prevents two tests carrying an identical tag
from overlapping, so a shared tag across files is required for a table
shared across files.

## The lost-update race the recall questions asks about

`AnimeStore::update` is a plain read-then-write: `self.get(id)` runs as
its own round trip, then a separate `UPDATE` statement runs later based on
what that read returned. Between those two steps, nothing stops a second,
concurrent `update` call from reading the same "before" state and writing
its own conflicting change — the second `UPDATE` can silently overwrite
the first one's write, because neither ever checked "is the row still in
the state I read it in." This is a real, honest gap in this lesson's code
(deliberately left as a discussion question, not silently fixed), and the
earlier lesson that shows the fix is
`phase4-backend-advanced/03-background-jobs-and-message-queues/01-postgres-skip-locked-toy-queue`'s
`FOR UPDATE SKIP LOCKED` pattern — either lock the row for the duration of
the read-modify-write (`SELECT ... FOR UPDATE` inside a transaction), or
make the `UPDATE` itself atomic and conditional (`UPDATE ... SET ... WHERE
id = $1 AND <some version/timestamp column matches what was read>`,
checking `rows_affected() == 0` to detect a lost race) — the same "make
the database enforce the invariant, don't trust a gap between two
round trips" discipline covered there.
