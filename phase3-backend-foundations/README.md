# Phase 3 — Backend Foundations

This is where the Django background pays off — and gets challenged. You
already know what a REST API, a database, and auth are *conceptually*; this
phase rebuilds that intuition on top of what a framework is actually doing
underneath, starting with raw TCP before you ever import `axum`.

Thirty-three lessons in eight modules, in the order they are meant to be
read.

## 1. [Networking & HTTP from scratch](01-networking-and-http-from-scratch/README.md)

Before `axum`, see what it's built on.

1. [TCP echo server](01-networking-and-http-from-scratch/01-tcp-echo-server/README.md)
2. [Hand-rolled HTTP parser](01-networking-and-http-from-scratch/02-hand-rolled-http-parser/README.md)
3. [HTTP semantics you must know](01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.md)

## 2. [`axum` & REST API design](02-axum-and-rest-api-design/README.md)

The framework this curriculum builds on from here forward.

1. [Routing, handlers, extractors](02-axum-and-rest-api-design/01-routing-handlers-extractors/README.md)
2. [Writing your own extractor (`FromRequestParts`)](02-axum-and-rest-api-design/02-writing-your-own-extractor/README.md)
3. [Anime catalog CRUD (in-memory)](02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.md)
4. [`tower::Service` / `Layer`: middleware by hand](02-axum-and-rest-api-design/04-tower-service-and-layer-middleware/README.md)
5. [CORS and frontend integration](02-axum-and-rest-api-design/05-cors-and-frontend-integration/README.md)

## 3. [Serialization & validation](03-serialization-and-validation/README.md)

What `Json<T>` is actually doing, one layer deeper.

1. [Serde in depth](03-serialization-and-validation/01-serde-depth/README.md)
2. [Validation](03-serialization-and-validation/02-validation/README.md)
3. [API contracts and OpenAPI (`utoipa`)](03-serialization-and-validation/03-api-contracts-and-openapi/README.md)
4. [API versioning and evolution](03-serialization-and-validation/04-api-versioning-and-evolution/README.md)

## 4. [Configuration & app structure](04-configuration-and-app-structure/README.md)

The plumbing a real deployment needs before it ever touches a database.

1. [12-factor config and secrets](04-configuration-and-app-structure/01-config-and-secrets/README.md)
2. [Application state and dependency wiring](04-configuration-and-app-structure/02-app-state-and-dependency-wiring/README.md)
3. [Graceful shutdown, health and readiness](04-configuration-and-app-structure/03-graceful-shutdown-health-readiness/README.md)

## 5. [PostgreSQL & `sqlx`](05-postgres-and-sqlx/README.md)

A database that survives a restart.

1. [Connecting and pooling](05-postgres-and-sqlx/01-connecting-and-pooling/README.md)
2. [Migrations](05-postgres-and-sqlx/02-migrations/README.md)
3. [Anime catalog, Postgres-backed](05-postgres-and-sqlx/03-anime-catalog-postgres-backed/README.md)
4. [Transactions](05-postgres-and-sqlx/04-transactions/README.md)
5. [The repository pattern](05-postgres-and-sqlx/05-repository-pattern/README.md)

## 6. [Database design & query performance](06-database-design-and-query-performance/README.md)

What changes when data volume becomes real.

1. [Indexing, `EXPLAIN ANALYZE`, the N+1 problem](06-database-design-and-query-performance/01-indexing-explain-analyze-n-plus-1/README.md)
2. [Pagination: offset vs. keyset](06-database-design-and-query-performance/02-pagination/README.md)
3. [Schema design for a real service](06-database-design-and-query-performance/03-schema-design-for-a-real-service/README.md)

## 7. [Auth & security](07-auth-and-security/README.md)

Everything Django's `User` model gave you for free, built by hand.

1. [Password hashing with `argon2`](07-auth-and-security/01-password-hashing-argon2/README.md)
2. [Sessions vs. JWT: the real trade-off](07-auth-and-security/02-sessions-vs-jwt/README.md)
3. [JWTs and `tower` middleware](07-auth-and-security/03-jwt-and-tower-middleware/README.md)
4. [Refresh-token rotation and revocation](07-auth-and-security/04-refresh-token-rotation-and-revocation/README.md)
5. [Modelling RBAC and permissions](07-auth-and-security/05-modelling-rbac-and-permissions/README.md)

## 8. [Errors, tracing & testing at scale](08-error-handling-and-testing-at-scale/README.md)

What makes everything before this operable.

1. [Consistent error envelopes](08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)
2. [Request tracing and correlation IDs](08-error-handling-and-testing-at-scale/02-request-tracing-and-correlation-ids/README.md)
3. [Integration tests with `testcontainers`](08-error-handling-and-testing-at-scale/03-integration-tests-with-testcontainers/README.md)
4. [Test data factories and fixtures](08-error-handling-and-testing-at-scale/04-test-data-factories-and-fixtures/README.md)
5. [WebSockets and SSE in `axum`](08-error-handling-and-testing-at-scale/05-websockets-and-sse-in-axum/README.md)

---

**Motivational recall questions:** [Side-quest 3 — Webtoon Notification Service](../side-quests/sq-03-webtoon-notifier-service/README.md)
— `axum` + Postgres + a scheduled job, previewing Phase 4's background jobs.

**Requires:** PostgreSQL installed locally (or via Docker) starting at
module 5 — that lesson's `README.md` covers setup, nothing earlier needs it.

When Phase 3 is fully checked off in [`PROGRESS.md`](../PROGRESS.md), move on
to [Phase 4](../phase4-backend-advanced/README.md).
