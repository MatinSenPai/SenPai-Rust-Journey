# Module 8 — Errors, tracing & testing at scale

The last module of Phase 3 is what makes everything before it operable: one
consistent shape for every error response, a way to follow one request
across every log line it touches, and tests that run against a real,
disposable database instead of trusting a mock. It closes with the one
`axum` capability every REST-only module so far has left out: a connection
that stays open.

1. [01 — Consistent error envelopes](01-consistent-error-envelopes/README.md)
2. [02 — Request tracing and correlation IDs](02-request-tracing-and-correlation-ids/README.md)
   — one ID that follows a request through every log line and every
   downstream call, moved forward from Phase 4 because you need it to
   debug everything else in this phase.
3. [03 — Integration tests with `testcontainers`](03-integration-tests-with-testcontainers/README.md)
4. [04 — Test data factories and fixtures](04-test-data-factories-and-fixtures/README.md)
   — building realistic test data on demand instead of hand-writing the
   same three structs in every test file.
5. [05 — WebSockets and SSE in `axum`](05-websockets-and-sse-in-axum/README.md)
   — the one HTTP shape this whole phase has skipped so far: a connection
   that pushes data instead of waiting to be asked.
