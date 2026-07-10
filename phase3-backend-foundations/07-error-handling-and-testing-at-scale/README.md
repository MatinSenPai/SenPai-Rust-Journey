# Module 7 — Error handling & testing at scale

Two ideas that only really bite once a codebase gets bigger than one
lesson: every error response needs to look the same across an entire API
(not "whatever `format!()` a handler happened to write"), and in-memory
test doubles — used everywhere else in this repo for fast, infra-free tests
— eventually need to be checked against the real thing at least somewhere.

1. [01 — Consistent error envelopes](01-consistent-error-envelopes/README.md)
2. [02 — Integration tests with `testcontainers`](02-integration-tests-with-testcontainers/README.md)
