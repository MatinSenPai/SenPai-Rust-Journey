# Solution — 3.3.2 Validation

The full code is `solution/src/lib.rs`; it passes every test in `solution/tests/`, including `build_test.rs` for the Build rung.

## `validate_handle`

```rust
pub fn validate_handle(handle: &str) -> Result<(), ValidationError> {
    if handle.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        Ok(())
    } else {
        Err(ValidationError::new("handle_chars")
            .with_message("handle may only contain letters, digits and underscores".into()))
    }
}
```

`.all` is `true` for an empty string, which is what the spec asks for: emptiness is `length`'s job, and each rule says one thing. `is_ascii_alphanumeric` (not `is_alphanumeric`) is why a Persian word is rejected: the spec says *ASCII* letters, and the test checks one. The error needs both a `code` (what a program can match on) and a `message` (what a person reads).

## `flatten_errors`

```rust
fn walk(errors: &ValidationErrors, prefix: &str, out: &mut BTreeMap<String, Vec<String>>) {
    for (field, kind) in errors.errors() {
        let path = format!("{prefix}{field}");
        match kind {
            ValidationErrorsKind::Field(list) => {
                let mut msgs: Vec<String> = list.iter().map(|e| e.message.as_ref().unwrap_or(&e.code).to_string()).collect();
                msgs.sort();
                out.insert(path, msgs);
            }
            ValidationErrorsKind::Struct(inner) => walk(inner, &format!("{path}."), out),
            ValidationErrorsKind::List(items) => {
                for (i, inner) in items { walk(inner, &format!("{path}[{i}]."), out); }
            }
        }
    }
}
```

`flatten_errors` creates the empty `BTreeMap`, calls `walk` with an empty prefix, and returns the map. The recursion is the point: a `Struct` or `List` node holds another `ValidationErrors`, so the function calls itself with a longer prefix (`"reviewer."`, `"notes[1]."`). Only a `Field` node is a leaf, and only there is something inserted. `errors()` (not `field_errors()`) is what exposes all three node kinds. `e.message.as_ref().unwrap_or(&e.code)` is "the message if there is one, else the code" without cloning first. A `BTreeMap` keeps the keys sorted for free, and `msgs.sort()` sorts the messages inside each key, so the output is the same on every run even though the underlying `HashMap` iterates in a random order. An empty tree loops zero times and gives an empty map.

## `ApiError`, the handlers and `app`

```rust
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Validation(errors) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({ "errors": flatten_errors(&errors) })),
            ).into_response(),
        }
    }
}
```

The tuple `(StatusCode, Json<_>)` already implements `IntoResponse` and `Json` sets `Content-Type: application/json`, the same trick as 3.2.3's `AnimeError`. `BTreeMap<String, Vec<String>>` implements `Serialize`, so `json!` takes it as is.

```rust
pub async fn create_review(
    State(store): State<Arc<ReviewStore>>,
    Json(input): Json<NewReview>,
) -> Result<(StatusCode, Json<StoredReview>), ApiError> {
    input.validate()?;
    Ok((StatusCode::CREATED, Json(store.add(input))))
}
```

`input.validate()?` converts a `ValidationErrors` into an `ApiError` through the `From` impl and returns early. Validation comes before `store.add`, so a rejected review is never stored and never uses up an id (`a_rejected_review_is_not_stored_and_uses_up_no_id`). `list_reviews` is `Json(store.list())`, and `app` is one `.route("/reviews", get(list_reviews).post(create_review))` followed by `.with_state(store)`.

## Build: `ValidatedJson<T>`

```rust
async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
    let Json(value) = Json::<T>::from_request(req, state)
        .await
        .map_err(IntoResponse::into_response)?;
    value.validate().map_err(|e| ApiError::from(e).into_response())?;
    Ok(ValidatedJson(value))
}
```

The extractor calls `Json<T>`'s own `from_request`, so `Json`'s rejections come out unchanged: that is how `400` and `415` stay as they were. Both failure paths become a `Response`, which is why `type Rejection = Response`. `T: DeserializeOwned + Validate` is the whole bound: any type that can be read from JSON and validated works, which `build_test.rs` shows with a `Ping` type that has nothing to do with reviews. To use it in the router, `create_review` takes `ValidatedJson(input): ValidatedJson<NewReview>` and drops the `validate()?` line.

## Challenge sketch

A struct-level rule is `#[validate(schema(function = "extreme_needs_reason"))]` on `NewReview`, with `fn extreme_needs_reason(r: &NewReview) -> Result<(), ValidationError>`. In `validator` 0.18.1 its error is filed under the key `__all__`, which `flatten_errors` passes through as an ordinary key. Renaming it to something like `"review"` is one line in the `Field` arm.
