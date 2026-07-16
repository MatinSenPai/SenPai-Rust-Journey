# taskforge-api

The REST surface, built on `axum`.

| Route | Auth | Description |
|---|---|---|
| `GET /health` | no | liveness check |
| `GET /metrics` | no | Prometheus text exposition |
| `GET /api-docs/openapi.json` | no | OpenAPI 3.1 document (see below) |
| `POST /jobs` | bearer | enqueue a job — body `{"job_type": "...", "payload": {...}, "max_attempts": 5}` |
| `GET /jobs` | bearer | list jobs — `?job_type=...&limit=...` |
| `GET /jobs/{id}` | bearer | fetch one job |
| `POST /jobs/{id}/cancel` | bearer | cancel a `Pending`/`Retrying` job |

## OpenAPI

The full API describes itself. `utoipa` generates an OpenAPI 3.1 document at
compile time from the `#[utoipa::path(...)]` annotation on each handler and the
`#[derive(ToSchema)]` on each request/response type, and the router serves it
unauthenticated at `GET /api-docs/openapi.json`. Point any OpenAPI tool at it —
Swagger UI, Postman's importer, an SDK generator — to get an interactive,
always-in-sync API reference with zero hand-maintained spec file to drift.
`tests/api_test.rs` asserts the endpoint serves a valid document that includes
the `/jobs` paths, so the spec can't silently rot as handlers change.

Every error response is a consistent JSON envelope: `{"error": "message"}`,
with `JobError` (from `taskforge-core`) mapped onto the right HTTP status in
`src/error.rs` — see
[`phase3-backend-foundations/07-error-handling-and-testing-at-scale/
01-consistent-error-envelopes`](../../phase3-backend-foundations/07-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)
for the lesson this pattern comes from.

Auth is a single shared bearer token (`AppState::auth_token`) — enough for
an internal/admin API used by `taskforge-admin-bot` and `taskforge-cli`,
not a real multi-tenant auth scheme (see `src/auth.rs`'s doc comment for
the trade-off and where a real JWT-based scheme would go instead).

## Running the tests

```sh
cargo test -p taskforge-api
```

Every test in `tests/api_test.rs` runs the full router in-process (via
`tower::ServiceExt::oneshot` — no TCP port bound, no flakiness from port
conflicts) against `InMemoryJobStore`. No database, no Redis, nothing to
start first.

## Running it for real

The `build_router` function above is the library seam; `src/main.rs` is the
runnable service that wires config, a Postgres store, migrations, and graceful
shutdown around it. That binary is a **capstone exercise** — it ships with
`todo!()`s for you to complete (see the capstone README's "What you build").
The shape it builds toward:

```rust,ignore
let store: Arc<dyn JobStore> = Arc::new(PostgresJobStore::connect(&database_url).await?);
let app = taskforge_api::build_router(store, auth_token);
let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await?;
```

Once it's done, `docker compose up --build` from `capstone-taskforge/` runs it
alongside the worker, scheduler, and Postgres.
