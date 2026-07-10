# Solution

```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::NotFound(message) => (StatusCode::NOT_FOUND, "not_found", message),
            ApiError::Validation(message) => {
                (StatusCode::BAD_REQUEST, "validation_failed", message)
            }
            ApiError::Internal(message) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", message)
            }
        };

        (
            status,
            Json(ErrorBody {
                error: ErrorDetail {
                    code: code.to_string(),
                    message,
                },
            }),
        )
            .into_response()
    }
}
```

```rust
impl From<validator::ValidationErrors> for ApiError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let mut messages: Vec<String> = errors
            .field_errors()
            .iter()
            .flat_map(|(field, field_errors)| {
                field_errors.iter().map(move |error| {
                    let message = error
                        .message
                        .clone()
                        .unwrap_or_else(|| error.code.clone())
                        .to_string();
                    format!("{field}: {message}")
                })
            })
            .collect();
        messages.sort();
        ApiError::validation(messages.join("; "))
    }
}
```

`WidgetStore::create` and `WidgetStore::get` are the same lock-then-mutate
shape you've now written several times (`AnimeStore` in the previous
module, `InMemoryQueue` in Phase 4's toy queue lesson): take the `Mutex`,
do the minimal work under the lock, return a clone so nothing outside the
store keeps holding a reference into it.

## Why matching `self` by value, not `&self`

`into_response(self)` takes `self` by value (that's `IntoResponse`'s
signature — it consumes the error to produce a `Response`), so matching on
`self` directly and destructuring each variant's inner `String` moves it out
for free: `ApiError::NotFound(message) => ...` binds `message: String`, no
`.clone()` needed. This is a small but real efficiency win over a design
that took `&self` and had to `.to_string()` a `Display` impl or `.clone()`
the field — worth noticing because "does this design require an allocation
it doesn't need to" is a habit worth having, not because this one clone
would ever matter at this scale.

## `code` vs. `message`, and why both exist

This is the crux of the lesson, so it's worth stating plainly: **`code` is
a contract, `message` is a courtesy.** A frontend, a mobile client, or
another backend service consuming this API should only ever branch on
`code` — string-matching on `message` is fragile by construction, because
`message` is meant to be freely rewordable (clearer phrasing, a typo fix,
localization down the line) without that being a breaking API change. If
`code` didn't exist and everyone matched on `message` instead (which is
exactly what happens in codebases that skip this lesson's discipline),
every wording improvement becomes a silent breaking change for every
consumer that happened to grep the old string. Two fields instead of one
is a small design cost that buys real API stability.

## Why one enum instead of a separate domain error + conversion

Compare this lesson to `taskforge-api::error`: there, `taskforge_core::
JobError` is a *storage-agnostic domain error* (it knows nothing about
HTTP), and `ApiError` is a separate *transport error* that `JobError`
converts into via `From`. That separation earns its keep in TaskForge
because `taskforge_core` is a library other things depend on — a worker
process, a CLI, a test harness — none of which should have to import
`axum` just to get `JobError`. This lesson's `WidgetStore` has exactly one
consumer (this crate's own HTTP handlers), so collapsing the domain error
and the transport error into a single `ApiError` is the right call, not a
shortcut: introducing a second enum with a `From` impl that's only ever
called from one place would be ceremony without payoff. The lesson: the
"one error type" vs. "domain error + boundary conversion" choice is a real
architectural decision, not a stylistic default — and it should track
whether your core logic actually has more than one consumer, the same
"ports and adapters" reasoning from earlier in Phase 3's axum module (keep
the store's logic decoupled from HTTP *when something other than HTTP might
call it*).

## What the envelope-shape test catches that the per-case tests don't

`create_with_empty_name_returns_400_with_a_validation_envelope` and
`get_missing_widget_returns_404_with_a_not_found_envelope` each hardcode
which top-level keys they expect for *one* error case. If a future change
added a third top-level key to `ErrorDetail` (say, a `details: Vec<String>`
field added only to the `Validation` variant's JSON, forgotten on the
`NotFound` variant), both of those tests would keep passing — neither one
asserts "and nothing else." `every_error_response_shares_the_same_envelope_
shape` is the test that would actually catch that: it collects the *exact*
key set from two structurally different error responses and asserts they're
identical. That's the difference between testing "this one case looks
right" and testing "the invariant this whole lesson exists to establish
still holds" — the second kind of test is what actually earns you the right
to say "every error from this API looks the same" with confidence, rather
than just believing it because the two examples you happened to write
agree.
