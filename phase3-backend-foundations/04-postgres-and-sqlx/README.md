# Module 4 — PostgreSQL & `sqlx`

Every lesson before this one used an in-memory `Mutex<HashMap<...>>` store —
correct, fast, and gone the instant the process exits. This module swaps
that for real Postgres via `sqlx`, an async, compile-time-optional SQL
toolkit (not a full ORM like Django's — you write real SQL, `sqlx` gives you
type-safe binding/decoding around it).

**Setup:** you need a local Postgres reachable via a connection string. This
sandbox already has Postgres 16 installed; start it with `sudo service
postgresql start` if it isn't running (`sudo service postgresql status` to
check). Every lesson here uses `postgres://taskforge:taskforge@localhost:5432/taskforge`
as its default `DATABASE_URL` — the same role/database the capstone's
`taskforge-storage` crate already uses, created once and reused everywhere
in this repo so you don't need to set up a new database per lesson. If you
don't have that role yet: `sudo -u postgres psql -c "CREATE ROLE taskforge
LOGIN PASSWORD 'taskforge' CREATEDB;" && sudo -u postgres createdb -O
taskforge taskforge`.

Every test that needs a live database is `#[ignore]`-gated (see
`phase4-backend-advanced/03-background-jobs-and-message-queues/01-postgres-skip-locked-toy-queue/`
for why this repo does that everywhere) — `cargo test` alone always passes
with zero infrastructure; `cargo test -- --ignored` (with `DATABASE_URL`
exported) actually exercises Postgres.

1. [01 — Connecting and pooling](01-connecting-and-pooling/README.md)
2. [02 — Migrations](02-migrations/README.md)
3. [03 — Anime catalog, Postgres-backed](03-anime-catalog-postgres-backed/README.md)
   — the exact same `Anime`/`WatchStatus` domain from module 2's in-memory
   CRUD lesson, rebuilt against a real database.
4. [04 — Transactions](04-transactions/README.md) — the double-write
   problem, `pool.begin()`/`tx.commit()`, rollback-on-drop (vs. Django's
   `transaction.atomic()`), and reversible `.up.sql`/`.down.sql`
   migrations with `sqlx migrate revert`.
