# Module 3 — Serialization & validation

`axum`'s `Json<T>` extractor (module 2) already parses request bodies for
you. This module goes one level deeper into *how* — `serde`'s derive
macros and attributes, the separate, explicit validation pass `validator`
adds on top, and the two things every real API eventually needs: a
machine-readable contract for its shape, and a plan for changing that shape
without breaking every existing caller.

1. [01 — Serde in depth](01-serde-depth/README.md)
   — `rename_all`, `flatten`, `skip_serializing_if`, a hand-written
   `Serialize` impl, and `untagged` enums: the attributes real APIs
   actually reach for, kept deliberately separate from HTTP entirely.
2. [02 — Validation](02-validation/README.md)
   — `#[derive(Validate)]` field-level rules and `ValidationErrors`, on top
   of the structs the previous lesson just built.
3. [03 — API contracts and OpenAPI](03-api-contracts-and-openapi/README.md)
   — generating a real OpenAPI document from your own types with
   `utoipa`, instead of hand-writing one that quietly drifts out of sync.
4. [04 — API versioning and evolution](04-api-versioning-and-evolution/README.md)
   — what actually breaks a caller, and the real strategies — URL, header,
   additive-only — for changing an API without breaking one.

Module 5 picks this straight back up: the `CreateAnime`/`UpdateAnime`-style
structs you've already seen in module 2 get real `validator` rules, then
get persisted for real.
