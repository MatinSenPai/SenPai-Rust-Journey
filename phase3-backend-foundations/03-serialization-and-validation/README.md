# Module 3 — Serialization & validation

`axum`'s `Json<T>` extractor (module 2) already parses request bodies for
you. This module goes one level deeper into *how* — `serde`'s derive
macros, and the separate, explicit validation pass `validator` adds on top
— so that by module 4, wiring a real create/update endpoint against
Postgres is just "the same two-pass shape you already know," not new
concepts on top of new concepts.

1. [01 — `serde_json` and `validator`](01-serde-json-and-validator/README.md)
   — deriving `Serialize`/`Deserialize`, `#[serde(...)]` field attributes,
   and `#[derive(Validate)]` field-level rules, kept deliberately separate
   from HTTP entirely (no `axum` in this lesson at all).

Module 4 picks this straight back up: the `CreateAnime`/`UpdateAnime`-style
structs you've already seen in module 2 get real `validator` rules, then
get persisted for real.
