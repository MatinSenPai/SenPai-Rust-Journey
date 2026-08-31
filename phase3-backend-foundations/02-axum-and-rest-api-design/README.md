# Module 2 — `axum` & REST API design

Module 1 built an HTTP server from raw bytes up. This module puts that
understanding to work with `axum`, the async web framework this whole
curriculum builds on from here forward — the same relationship Django has
to the raw WSGI protocol underneath it.

1. [01 — Routing, handlers, extractors](01-routing-handlers-extractors/README.md)
   — `Router`, async handlers, and the `Path`/`Json`/`State` extractors
   that replace manual request parsing.
2. [02 — Writing your own extractor](02-writing-your-own-extractor/README.md)
   — implementing `FromRequestParts` by hand, for the day the built-in
   extractors don't cover what a handler actually needs.
3. [03 — Anime catalog CRUD (in-memory)](03-anime-catalog-crud-in-memory/README.md)
   — a full create/read/update/delete REST API over an in-memory store,
   the same shape you'll rebuild against real Postgres in module 5.
4. [04 — `tower::Service` / `Layer`: middleware by hand](04-tower-service-and-layer-middleware/README.md)
   — what a `Layer` actually is underneath `.layer(...)`, built from
   scratch before the next lesson hands you a ready-made one.
5. [05 — CORS and frontend integration](05-cors-and-frontend-integration/README.md)
   — the browser-enforced same-origin policy, preflight `OPTIONS`
   requests, and a `CorsLayer` configured for dev vs. prod — all tested
   without a browser via `oneshot`.

By the end of this module you'll be able to stand up a real REST API with
proper status codes, JSON error bodies, and your own middleware — no
database yet, that's module 5.
