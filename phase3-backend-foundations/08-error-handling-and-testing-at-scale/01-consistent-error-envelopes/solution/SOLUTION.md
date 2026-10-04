# Solution — 3.8.1 Consistent error envelopes

The full code is `solution/src/lib.rs`; it passes every test in `solution/tests/`, including `conflict_test.rs` for the Build rung and `problem_test.rs` for the Challenge.

## `status`, `code` and `message`

```rust
pub fn status(&self) -> StatusCode {
    match self {
        ApiError::NotFound(_) => StatusCode::NOT_FOUND,
        ApiError::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
        ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
        ApiError::UnsupportedMediaType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ApiError::InvalidBody(_) | ApiError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
        ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
```

`code` is the same shape with strings. Both are matches with **no wildcard**, so a new variant is a compile error until each one answers (`E0004`, shown in the lesson). `InvalidBody` and `Validation` share `422` and still have different codes: the status is the HTTP class of the problem, the code is the exact kind.

```rust
pub fn message(&self) -> String {
    match self {
        ApiError::NotFound(m) | ApiError::BadRequest(m)
        | ApiError::UnsupportedMediaType(m) | ApiError::InvalidBody(m) => m.clone(),
        ApiError::MethodNotAllowed => "method not allowed for this route".to_string(),
        ApiError::Validation(_) => "the request body has invalid fields".to_string(),
        ApiError::Internal(_) => "something went wrong on our side".to_string(),
    }
}
```

The `Internal(_)` arm is the whole security story: the variant owns the server's text, and this function never returns it. `an_internal_message_never_contains_the_inner_text` plants a password in the text and asserts it is absent.

## `IntoResponse`

```rust
fn into_response(self) -> Response {
    if let ApiError::Internal(detail) = &self {
        eprintln!("internal error: {detail}");
    }
    let (status, code, message) = (self.status(), self.code(), self.message());
    let fields = match self { ApiError::Validation(f) => f, _ => Vec::new() };
    let body = ErrorBody { error: ErrorDetail { code, message, fields } };
    (status, Json(body)).into_response()
}
```

The three questions are answered by the methods, and the impl only assembles. The log line comes first, while `self` is still whole: the `match self` that moves the field list out comes last. `ErrorDetail` has `#[serde(skip_serializing_if = "Vec::is_empty")]` on `fields`, so only a validation failure that has entries carries the key. `Json` sets `Content-Type: application/json`, which is what the tests check on every failure. (An empty `Validation(vec![])` would omit `fields` too; the code never builds one.)

## Rejections

```rust
impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        let message = rejection.body_text();
        match rejection.status() {
            StatusCode::UNSUPPORTED_MEDIA_TYPE => ApiError::UnsupportedMediaType(message),
            StatusCode::UNPROCESSABLE_ENTITY => ApiError::InvalidBody(message),
            _ => ApiError::BadRequest(message),
        }
    }
}
```

`body_text()` is the text `axum` itself would have sent, so the message stays as informative as before. Matching on `status()` and not on the rejection's variants keeps working if `axum` adds a variant. The `_` wildcard here is deliberate and safe: a rejection we did not foresee is a request problem, so `400`. `PathRejection` is the same with one twist: a `500` from `axum` means the *router* is misconfigured (the path parameters are missing, for example because `Path` is used in a handler that no route with a path parameter leads to), which is never the client's fault, so it becomes `Internal`.

## `validate_new_show`

```rust
let mut fields = Vec::new();
if !(1..=100).contains(&input.title.chars().count()) {
    fields.push(FieldError { field: "title", code: "length",
        message: "title must be 1 to 100 characters".to_string() });
}
if !(1..=2000).contains(&input.episodes) {
    fields.push(FieldError { field: "episodes", code: "range",
        message: "episodes must be 1 to 2000".to_string() });
}
if fields.is_empty() { Ok(()) } else { Err(ApiError::Validation(fields)) }
```

Both checks always run, so every broken rule is reported together, with `title` first because it is pushed first. `chars().count()`, not `len()`: `len()` counts bytes, and a Persian title of 100 letters is 200 bytes (`title_length_counts_characters_not_bytes`).

## Build: `Conflict`

Adding `ApiError::Conflict(String)` with `409` and `"conflict"`, and a `message` arm that returns the text, is four small edits, and the compiler lists every one. The check belongs in the store, under the same lock as the insert:

```rust
let mut inner = self.inner.lock().unwrap();
if inner.items.values().any(|s| s.title == input.title) {
    return Err(ApiError::Conflict(format!("a show titled \"{}\" already exists", input.title)));
}
inner.next_id += 1;
```

If the handler asked the store "does this title exist?" and then separately inserted, two simultaneous requests could both see "no" and both insert. One lock, one check-and-insert, closes that. The id is incremented only after the check, so a rejected request does not use one up (`a_conflict_does_not_use_up_an_id`). `insert` now returns `Result<Show, ApiError>` and `create_show` uses `?`, the same pattern as `validate_new_show(&input)?` just above it. A scan of every value is O(n), the honest cost of "unique title" in a `HashMap` keyed by id; a database would use a unique index (the Postgres module, 3.5.3 and later).

## Challenge: `problem_response`

```rust
pub fn problem_response(&self, instance: &str) -> Response {
    let status = self.status();
    let mut body = serde_json::json!({
        "type": format!("https://api.example.com/problems/{}", self.code()),
        "title": status.canonical_reason().unwrap_or("Error"),
        "status": status.as_u16(), "detail": self.message(), "instance": instance,
    });
    if let ApiError::Validation(fields) = self { body["errors"] = serde_json::json!(fields); }
    (status, [(header::CONTENT_TYPE, "application/problem+json")], body.to_string()).into_response()
}
```

It reuses `status()`, `code()` and `message()` unchanged. That is the point of the lesson: because those three questions are answered in one place, a second wire format is one function, not a change to every handler. `Json(...)` would force `application/json`, so the body is a string with an explicit header. `title` is the status's reason phrase, as the RFC wants it stable across occurrences (the specific text goes in `detail`). A `500` still carries only the fixed `message()`, so the problem leaks nothing either (`an_internal_problem_leaks_nothing`).

## What the whole-stack tests catch that the unit tests do not

`tests/error_test.rs` proves each `ApiError` produces the right envelope. It cannot tell you that a request really *reaches* `ApiError`: a bad `{id}` goes through `axum`'s `Path` extractor first, an unknown route through the router first. `every_error_has_exactly_one_top_level_key_and_a_code_and_message` in `tests/api_test.rs` is the test that earns the lesson's claim, because it sends six different kinds of failure, including the ones `axum` produces itself, and asserts the same shape for all of them. Test the invariant, not just the cases.
