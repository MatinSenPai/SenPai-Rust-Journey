# Solution

```rust
pub async fn run_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS p3_07_02_widgets (\
            id BIGSERIAL PRIMARY KEY, name TEXT NOT NULL, quantity INT NOT NULL)",
    )
    .execute(pool)
    .await
    .map(|_| ())
}

pub async fn create(&self, name: &str, quantity: i32) -> Result<Widget, sqlx::Error> {
    sqlx::query_as(
        "INSERT INTO p3_07_02_widgets (name, quantity) VALUES ($1, $2) \
         RETURNING id, name, quantity",
    )
    .bind(name)
    .bind(quantity)
    .fetch_one(&self.pool)
    .await
}

pub async fn get(&self, id: i64) -> Result<Option<Widget>, sqlx::Error> {
    sqlx::query_as("SELECT id, name, quantity FROM p3_07_02_widgets WHERE id = $1")
        .bind(id)
        .fetch_optional(&self.pool)
        .await
}

pub async fn list(&self) -> Result<Vec<Widget>, sqlx::Error> {
    sqlx::query_as("SELECT id, name, quantity FROM p3_07_02_widgets ORDER BY id")
        .fetch_all(&self.pool)
        .await
}
```

Every method is a single `sqlx::query`/`query_as` call — the interesting
design decisions in this lesson are all in the test setup, not the
repository itself.

## Why `_container`, not `_`

`ContainerAsync<Postgres>`'s `Drop` implementation is what actually tears
the Docker container down. Binding it as `_` (rather than `_container`)
drops it **immediately** — right there on the `let` statement — because
`_` never binds a value at all, it's pattern-matched and discarded on the
spot. `_container` (or any real identifier prefixed with `_` to silence
the "unused variable" warning) still holds the value for the rest of the
scope, so the container stays alive and running for the whole test
function, only actually stopping when the test returns and the binding
finally goes out of scope. Writing `let (pool, _) = ...` would boot a
container, immediately kill it, and then try to run every subsequent query
against a pool pointing at a database that no longer exists — every query
would fail with a connection error, not because the SQL was wrong, but
because the infrastructure underneath it was gone before the test even
got to it.

## Why no cleanup query is needed

Every earlier Postgres lesson in this repo (module 4, the toy job queue)
shares one long-lived `taskforge` database across every test run, forever —
so each of those tests has to explicitly `DROP TABLE`/`DELETE FROM` first
to guarantee a clean starting point, because leftover rows from a previous
run are still sitting there. Here, `Postgres::default().start()` creates a
**brand-new container** — a fresh Postgres server with no data at all —
for every single test function. There is no "previous run's leftover
rows" to clean up, because there is no previous run as far as this
container is concerned; it didn't exist a moment ago and won't exist a
moment after the test ends. This is the actual tradeoff `testcontainers`
makes: slower (booting a real container costs real time — likely a few
hundred milliseconds to low seconds per test) in exchange for perfect
isolation, versus the shared-database lessons' fast-but-must-clean-up-by-hand
approach.

## Why a dynamic host port, not always 5432

If every container bound host port 5432 directly, only one container could
ever be running at a time — a second `Postgres::default().start()` call
(from a second test running concurrently, which `cargo test`'s default
parallel test execution does routinely) would fail outright, because
something is already listening on host port 5432. Docker's port mapping
(container's internal 5432 mapped to some free, dynamically-chosen port on
the host) means `cargo test` can run many of these tests in parallel, each
with its own container, each on its own distinct host port, with zero
coordination between them. `get_host_port_ipv4(5432)` is how each test
discovers *its own* container's actual mapped port after the fact, since
it isn't knowable in advance.

## What `#[ignore]` does and doesn't skip

`#[ignore]` only affects whether a test **runs** by default — `cargo test`
skips executing it (reporting `ignored`), while `cargo test -- --ignored`
runs exactly the ignored ones. It has zero effect on compilation: every
test function, `#[ignore]`d or not, is still part of the compiled test
binary, meaning every line of `use testcontainers_modules::...`, every
`.await` call, every type this lesson's code relies on, gets fully
type-checked by `cargo test`/`cargo check --all-targets` regardless of
whether Docker exists anywhere in the environment. That's precisely why
`cargo clippy --all-targets -- -D warnings` and `cargo test` (which
compiles, then skips the ignored tests) both succeeded when this solution
was verified in this sandbox — the *code* was fully checked; only the
*execution* of the four Docker-dependent tests was skipped.

## Would `WidgetStore` need to change first?

Yes. `consistent-error-envelopes`'s `WidgetStore` is a `Mutex<HashMap<...>>`
with synchronous (non-`async`) methods — there's no `PgPool`, no `sqlx`
anywhere in it, by design, since that lesson is about error handling, not
persistence. Bolting `testcontainers` onto it directly wouldn't work: you'd
have a real Postgres container running, and a repository type that has no
way to talk to it at all. The actual fix mirrors this lesson's own
`WidgetRepository` — and, at a larger scale, the pattern
`docs/adr/0001-architecture-overview.md` documents for the capstone: define
a trait (say, `WidgetRepository`) that both an in-memory implementation
*and* a `sqlx`-backed implementation satisfy, have `consistent-error-envelopes`'s
handlers depend on `Arc<dyn WidgetRepository>` instead of the concrete
`WidgetStore`, and only *then* would a `testcontainers`-backed integration
test suite (exercising the new Postgres-backed implementation specifically)
make sense to add alongside the existing fast in-memory tests.
