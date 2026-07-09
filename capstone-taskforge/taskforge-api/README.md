# taskforge-api

The REST surface, built on `axum`.

| Route | Auth | Description |
|---|---|---|
| `GET /health` | no | liveness check |
| `GET /metrics` | no | Prometheus text exposition |
| `POST /jobs` | bearer | enqueue a job — body `{"job_type": "...", "payload": {...}, "max_attempts": 5}` |
| `GET /jobs` | bearer | list jobs — `?job_type=...&limit=...` |
| `GET /jobs/{id}` | bearer | fetch one job |
| `POST /jobs/{id}/cancel` | bearer | cancel a `Pending`/`Retrying` job |

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

```rust,ignore
let store: Arc<dyn JobStore> = Arc::new(PostgresJobStore::connect(&database_url).await?);
let app = taskforge_api::build_router(store, auth_token);
let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
axum::serve(listener, app).await?;
```
