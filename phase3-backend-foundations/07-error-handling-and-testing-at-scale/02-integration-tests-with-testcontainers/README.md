# 07.2 — Integration tests with `testcontainers`

Every database-touching lesson so far in this repo tests against a shared,
long-lived Postgres instance (`postgres://taskforge:taskforge@localhost:5432/taskforge`),
with each test cleaning up its own lesson-scoped table first. That's fast
and simple, but it means every test implicitly depends on that one specific
database already existing and being reachable — nobody's SQL, migrations,
or schema assumptions get checked against a genuinely fresh, from-scratch
database unless someone remembers to set one up by hand. `testcontainers`
fixes that: each test boots its own real, disposable Postgres in a Docker
container, runs against it, and the container is destroyed the moment the
test finishes — no shared state, no manual setup, no "works on my machine
because I already have the right tables."

## Why not just use the in-memory test doubles everywhere?

Every earlier lesson's tests (the anime catalog CRUD, the toy job queue,
`taskforge-storage`'s `InMemoryJobStore`) prove your *business logic* is
correct against a `Mutex<HashMap<...>>` stand-in — fast, and exactly what
you want for the majority of your test suite. But none of those tests ever
send real SQL to a real Postgres, which means none of them can catch a
typo in a column name, a migration that doesn't actually produce the
schema you assumed, or a query that works fine on your local Postgres
version but not the one production runs. Real backend projects run **both**
kinds of tests: fast, numerous, in-memory unit tests for logic, and a
smaller number of slower, real-infrastructure integration tests to catch
exactly the class of bug the in-memory tests structurally cannot.

## What `testcontainers` actually does

```rust
let container = Postgres::default().start().await?;
let host = container.get_host().await?;
let port = container.get_host_port_ipv4(5432).await?;
let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");
```

`Postgres::default().start()` pulls (if needed) and runs a real Postgres
Docker image, waits until it's actually ready to accept connections, and
hands back a `ContainerAsync` handle. Postgres inside the container always
listens on port 5432, but Docker maps that to some **other**, dynamically
chosen port on the host (so multiple tests running concurrently, each with
their own container, never collide on port numbers) — `get_host_port_ipv4(5432)`
is how you find out what that mapped port actually is. The container is
tied to the `ContainerAsync` handle's lifetime: when it drops (at the end
of the test), `testcontainers` tears the container down automatically —
there's no leftover container to clean up by hand, and no state that could
possibly leak into the next test run.

## Why every test in `tests/integration_test.rs` is `#[ignore]`d

Booting a real Docker container needs a running Docker daemon. This
sandbox has the Docker **CLI** installed but no daemon running — a real,
common situation (a CI runner without Docker-in-Docker enabled, a
developer's machine with Docker Desktop closed) that this repo handles the
same way it's already handled missing live Postgres/Redis elsewhere:
`#[ignore]` the tests that need it, so `cargo test` alone always passes
with zero infrastructure, and anyone who *does* have Docker running can
opt in with `cargo test -- --ignored`. Crucially, `#[ignore]`d tests still
**compile** as part of `cargo test`/`cargo check --all-targets` — this
lesson's `testcontainers`/`sqlx` API usage is fully type-checked even in an
environment that can never actually run these tests, which is exactly why
this lesson's `solution/` could be built and verified correctly in this
sandbox despite Docker being unavailable.

## Reading `WidgetRepository`

Same `Widget` shape as the previous lesson's in-memory `WidgetStore`
(`id`, `name`, `quantity`), but every method here is a real, runtime-checked
`sqlx::query_as` call against Postgres — `#[derive(sqlx::FromRow)]` on
`Widget` is what lets `query_as` decode a returned row directly into it,
matching columns to fields by name.

## Your task

Open `src/lib.rs`. Implement `run_schema`, `WidgetRepository::create`,
`WidgetRepository::get`, and `WidgetRepository::list`.

## Next

`cargo test -p p3-07-02-integration-tests-with-testcontainers` (passes
trivially — every Docker-dependent test is `#[ignore]`d). If you have
Docker running locally: `cargo test -p p3-07-02-integration-tests-with-testcontainers -- --ignored`.
Then `solution/SOLUTION.md` — but only after a real attempt.
