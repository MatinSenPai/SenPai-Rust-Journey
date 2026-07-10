# Module 2 — `axum` & REST API design

Module 1 built an HTTP server from raw bytes up. This module puts that
understanding to work with `axum`, the async web framework this whole
curriculum builds on from here forward — the same relationship Django has
to the raw WSGI protocol underneath it.

1. [01 — Routing, handlers, extractors](01-routing-handlers-extractors/README.md)
   — `Router`, async handlers, and the `Path`/`Json`/`State` extractors
   that replace manual request parsing.
2. [02 — Anime catalog CRUD (in-memory)](02-anime-catalog-crud-in-memory/README.md)
   — a full create/read/update/delete REST API over an in-memory store,
   the same shape you'll rebuild against real Postgres in module 4.

By the end of this module you'll be able to stand up a real REST API with
proper status codes and JSON error bodies — no database yet, that's next.
