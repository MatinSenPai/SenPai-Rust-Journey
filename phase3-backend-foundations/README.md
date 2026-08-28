# Phase 3 — Backend Foundations

This is where the Django background pays off — and gets challenged. You
already know what a REST API, a database, and auth are *conceptually*; this
phase rebuilds that intuition on top of what a framework is actually doing
underneath, starting with raw TCP before you ever import `axum`.

1. **Networking & HTTP from scratch**
   - [01 — TCP echo server](01-networking-and-http-from-scratch/01-tcp-echo-server/README.md)
   - [02 — Hand-rolled HTTP parser](01-networking-and-http-from-scratch/02-hand-rolled-http-parser/README.md)
2. **`axum` & REST API design**
   - [01 — Routing, handlers, extractors](02-axum-and-rest-api-design/01-routing-handlers-extractors/README.md)
   - [02 — Anime catalog CRUD (in-memory)](02-axum-and-rest-api-design/02-anime-catalog-crud-in-memory/README.md)
3. **Serialization & validation**
   - [01 — `serde_json` and `validator`](03-serialization-and-validation/01-serde-json-and-validator/README.md)
4. **PostgreSQL & `sqlx`**
   - [01 — Connecting and pooling](04-postgres-and-sqlx/01-connecting-and-pooling/README.md)
   - [02 — Migrations](04-postgres-and-sqlx/02-migrations/README.md)
   - [03 — Anime catalog, Postgres-backed](04-postgres-and-sqlx/03-anime-catalog-postgres-backed/README.md)
5. **Database design & query performance**
   - [01 — Indexing, `EXPLAIN ANALYZE`, the N+1 problem](05-database-design-and-query-performance/01-indexing-explain-analyze-n-plus-1/README.md)
6. **Auth & security**
   - [01 — Password hashing with `argon2`](06-auth-and-security/01-password-hashing-argon2/README.md)
   - [02 — JWTs and `tower` middleware](06-auth-and-security/02-jwt-and-tower-middleware/README.md)
7. **Error handling & testing at scale**
   - [01 — Consistent error envelopes](07-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)
   - [02 — Integration tests with `testcontainers`](07-error-handling-and-testing-at-scale/02-integration-tests-with-testcontainers/README.md)

**Motivational recall questions:** [Side-quest 3 — Webtoon Notification Service](../side-quests/sq-03-webtoon-notifier-service/README.md)
— `axum` + Postgres + a scheduled job, previewing Phase 4's background jobs.

**Requires:** PostgreSQL installed locally (or via Docker) starting at
module 4 — that lesson's `README.md` covers setup, nothing earlier needs it.

When Phase 3 is fully checked off in [`PROGRESS.md`](../PROGRESS.md), move on
to [Phase 4](../phase4-backend-advanced/README.md).
