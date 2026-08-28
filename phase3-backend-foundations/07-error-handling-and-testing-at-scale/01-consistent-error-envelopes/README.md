# 07.1 — Consistent error envelopes

## The problem: every handler invents its own error shape

Go back through Phase 3's earlier lessons and look at what an error response
actually looked like. `02-axum-and-rest-api-design`'s anime catalog answers a
404 with `{"error": "anime not found"}`. That's fine in isolation — but
picture the same handler file six months and thirty endpoints later, written
by four different people. One handler returns `{"error": "not found"}`.
Another, added under deadline pressure, returns `{"message": "Not Found"}`
because that's what `axum`'s default 404 fallback produces. A third returns
`{"errors": ["bad rating"]}` — plural, because that one handler happens to
validate two fields. Nothing is *wrong* with any single one of these in
isolation. Collectively, they're a nightmare for anyone consuming the API:
a frontend can't write one error-handling function, it has to special-case
per endpoint. An API client SDK generator can't infer a schema. A support
engineer grepping logs can't `jq '.error.code'` uniformly.

This is not a hypothetical. It is one of the most common sources of
"papercut" bugs and wasted debugging time in real backend codebases — and
unlike a lot of engineering problems, it has a genuinely cheap fix: **decide
the shape once, build it in one place, and make it structurally impossible
for a handler to bypass it.**

## The shape

```json
{
  "error": {
    "code": "not_found",
    "message": "widget 42 not found"
  }
}
```

Two fields, deliberately different jobs:

- **`code`** is machine-readable and stable. A frontend can
  `if (error.code === "validation_failed") { highlightFields(...) }` without
  parsing English out of a sentence that might get reworded next sprint.
  Codes are part of your API's contract in a way free-text messages never
  should be.
- **`message`** is for humans — logs, an API explorer, a stack trace a
  backend engineer is staring at during an incident. It can change wording
  freely without breaking anyone's `if` statement, because nobody should be
  matching on it.

Real systems (Stripe, GitHub, most well-designed public APIs) converge on
almost exactly this shape for exactly this reason. This lesson builds a
small, honest version of it.

## One enum, one `IntoResponse` impl

The mechanism is the same one `capstone-taskforge/taskforge-api/src/
error.rs` uses for the whole TaskForge API — read that file, this lesson's
`ApiError` is a close cousin, not a copy:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // match self -> (StatusCode, code, message) -> one Json<ErrorBody>
    }
}
```

`thiserror::Error` gives `ApiError` a `Display` impl (via `#[error("...")]`)
and makes it a real `std::error::Error` — useful for logging with `tracing`,
or for `?`-propagating out of any function that returns
`Result<_, ApiError>`. But `Display` only answers "what string represents
this error" — it says nothing about HTTP. That's a completely separate
question, answered by a completely separate trait: `axum::response::
IntoResponse`. Implementing it once for `ApiError` is what lets *every*
handler in this crate write `Result<T, ApiError>` and never construct a
`StatusCode` or `Json<ErrorBody>` by hand. The mapping from "what went
wrong" to "what the client sees" lives in exactly one `match`, in one
`impl`, in one file — not scattered across every handler that can fail.

This is the same discipline you've now seen from three different angles:
`Result<T, DomainError>` in Phase 1-2's pure logic, `tonic::Status` in Phase
4's gRPC lesson (if you've gotten there), and now `ApiError` here. Different
transport, same idea: **make failure a value, and centralize the
translation from that value to whatever the caller sees.**

## Validation errors, specifically

`CreateWidget` derives `validator::Validate` (the same crate and pattern as
`phase3-backend-foundations/03-serialization-and-validation/01-serde-json-
and-validator`):

```rust
#[derive(Debug, Deserialize, Validate)]
pub struct CreateWidget {
    #[validate(length(min = 1, max = 100, message = "name must be 1-100 characters"))]
    pub name: String,
    #[validate(range(min = 1, max = 100_000, message = "quantity must be between 1 and 100000"))]
    pub quantity: u32,
}
```

`input.validate()` returns `Result<(), validator::ValidationErrors>` on
failure — a structured bag of per-field errors, not a `String`. Without a
`From<ValidationErrors> for ApiError` impl, every handler that validates
input would need its own `.map_err(|e| ...)` boilerplate to turn that bag
into an `ApiError::Validation`. With the impl, `input.validate()?` just
works inside any function returning `Result<_, ApiError>` — the same
`From<domain error> for ApiError` pattern `taskforge-api::error` uses to
convert `taskforge_core::JobError` at the boundary.

## Reading the starter

`src/lib.rs` has one small in-memory resource — `Widget` — behind a
`WidgetStore` (a `HashMap` guarded by a `Mutex`, same shape as `02-axum-and-
rest-api-design/02-anime-catalog-crud-in-memory`'s `AnimeStore`), and two
handlers:

- `create_widget` — `POST /widgets`, can fail with `ApiError::Validation`
  (400) if the name is empty or the quantity is out of range.
- `get_widget` — `GET /widgets/{id}`, can fail with `ApiError::NotFound`
  (404) if the id doesn't exist.

Every fallible function in this crate — `WidgetStore::create`,
`WidgetStore::get`, both handlers — returns `Result<_, ApiError>` directly.
There is no separate domain error type being converted at the HTTP boundary
this time (contrast with `taskforge-api`'s `JobError` → `ApiError`
conversion) — for a crate this small, `ApiError` *is* the domain error type,
which is a perfectly reasonable choice for a small service. The interesting
architectural question — "should HTTP status codes leak into your core
business logic, or should there be a separate domain error type translated
at the boundary?" — is exactly the kind of thing the recall questions asks you to
think through.

## Your task

Open `src/lib.rs`. Implement the `todo!()`s:

- `impl IntoResponse for ApiError` — the heart of this lesson: match on
  `self`, decide the `(StatusCode, code, message)` triple per variant,
  return one `Json<ErrorBody>`.
- `impl From<ValidationErrors> for ApiError` — flatten `validator`'s
  per-field error map into one readable message.
- `WidgetStore::create` / `WidgetStore::get` — the pure storage logic.
- `create_widget` / `get_widget` — thin `axum` handlers wrapping the store.
- `app` — the two-route table.

## Try it for real

```sh
cargo run -p p3-07-01-consistent-error-envelopes &
curl -X POST -H 'content-type: application/json' \
  -d '{"name":"gizmo","quantity":12}' http://127.0.0.1:3002/widgets
curl http://127.0.0.1:3002/widgets/1
curl http://127.0.0.1:3002/widgets/999   # 404, structured error body
curl -X POST -H 'content-type: application/json' \
  -d '{"name":"","quantity":12}' http://127.0.0.1:3002/widgets   # 400, structured error body
```

## Next

`cargo test -p p3-07-01-consistent-error-envelopes`, then the recall questions,
then `solution/SOLUTION.md`.
