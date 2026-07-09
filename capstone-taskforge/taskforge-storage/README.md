# taskforge-storage

Two implementations of `taskforge-core`'s `JobStore` trait:

- **`InMemoryJobStore`** — no external dependencies, used throughout this
  workspace's test suites. Its own test module also proves the concurrency
  contract every `JobStore` implementation must uphold: `claim_next` never
  lets two concurrent callers claim the same job.
- **`PostgresJobStore`** — the real implementation, via `sqlx`. Migrations
  live in `migrations/` and run automatically via `PostgresJobStore::connect`.

## Why every query uses `sqlx::query`/`query_as`, never `query!`/`query_as!`

The `query!`/`query_as!` macros type-check your SQL against a **live
database at compile time** (or a committed `.sqlx` offline cache). That's a
genuinely nice feature in a normal project — but it would mean `cargo build
--workspace` fails for this entire curriculum repository unless a Postgres
instance is reachable at build time, for every learner, forever. Not
acceptable here. So this crate uses the runtime-checked API everywhere
(`sqlx::query(...).bind(...)`, `sqlx::query_as::<_, JobRow>(...)`) — SQL
mistakes surface as runtime errors (caught by tests) instead of compile
errors, which is the trade-off. `row.rs`'s `JobRow` (deriving
`sqlx::FromRow`) still gets you safe, typed column mapping — that derive
only maps columns to fields at runtime, it doesn't touch a database at
compile time at all.

## Running the tests

```sh
cargo test -p taskforge-storage
```

This runs everything **except** `postgres::tests::enqueue_and_claim_round_trip`,
which is marked `#[ignore]` because it needs a real Postgres. To run it for
real:

```sh
# from capstone-taskforge/
docker compose up -d postgres
DATABASE_URL=postgres://taskforge:taskforge@localhost:5432/taskforge \
  cargo test -p taskforge-storage -- --ignored
```

(No Docker? Any local Postgres works — just point `DATABASE_URL` at it;
`PostgresJobStore::connect` runs the migrations in `migrations/` for you.
This was in fact verified against a plain local `postgresql` service
during development, not just Docker — a `taskforge`/`taskforge` role and
database were created by hand and the ignored test passed against it, no
container involved.)
