# Solution — 3.8.2 Request tracing and correlation IDs

## `is_valid_request_id`

```rust
pub fn is_valid_request_id(candidate: &str) -> bool {
    (1..=MAX_REQUEST_ID_LEN).contains(&candidate.len())
        && candidate
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}
```

Two conditions joined with `&&`. The length check uses `str::len`, which counts bytes, and the spec says bytes, so `"café"` is 5 long and also fails the character check, because `é` is not ASCII. `is_ascii_alphanumeric` is the ASCII-only test: `char::is_alphanumeric` would also accept `é` and Persian letters, and then a caller could put Unicode into your log. The empty string fails the length check, so no special case is needed. The tests `length_limit_is_64_bytes_inclusive` and `rejects_characters_outside_the_allowed_set` pin both edges.

## `resolve_request_id`

```rust
pub fn resolve_request_id(incoming: Option<&str>, generate: impl FnOnce() -> String) -> String {
    match incoming {
        Some(id) if is_valid_request_id(id) => id.to_string(),
        _ => generate(),
    }
}
```

The match guard does the whole job. The `_` arm covers both a missing header and an invalid one, which are the same case as far as the caller of this function is concerned: make a new one. `generate` is `FnOnce` and appears only in the second arm, so a valid ID never calls it (the test passes a closure that panics to prove it). Nothing here trims or edits the input: an invalid ID is replaced whole.

## `error_body`

```rust
pub fn error_body(request_id: &str, code: &str, message: &str) -> Value {
    json!({"error": {"code": code, "message": message, "request_id": request_id}})
}
```

`serde_json::json!` builds a `Value`, and it escapes the strings for you, so a message containing a quote or a newline stays valid JSON. Building the text by hand with `format!` would not.

## `request_id_middleware`

```rust
let incoming = request
    .headers()
    .get(REQUEST_ID_HEADER)
    .and_then(|value| value.to_str().ok());
let id = resolve_request_id(incoming, new_request_id);
request.extensions_mut().insert(RequestId(id.clone()));
```

`HeaderValue::to_str` fails for bytes outside visible ASCII, and `.ok()` turns that failure into `None`, so "not text" and "absent" both end up generating an ID, as the spec says. `new_request_id` is passed as a function, not called, so it only runs when `resolve_request_id` needs it.

```rust
let span = tracing::info_span!(
    "request",
    request_id = %id,
    method = %request.method(),
    path = %request.uri().path(),
);
async move { /* ... */ }.instrument(span).await
```

`%` records each field with `Display`, so the ID prints bare (`request_id=abc-123`), which is what the tests look for. `uri().path()` is the path without the query string: the test `the_path_in_the_span_has_no_query_string` sends `?secret=hunter2` and checks it never reaches the log. The span is built from `request` before `next.run(request)` moves it. Everything after is inside one `async move` block, wrapped with `.instrument(span)`. That is the whole span story: the span is attached to the future, so it holds across every `.await`, and it is not an `enter()` guard, which would not compile here (the `E0277` in the lesson's "Errors you will meet").

```rust
if let Some(info) = response.extensions_mut().remove::<ErrorInfo>() {
    if response.status().is_server_error() {
        tracing::error!(code = %info.code, "request failed");
    } else {
        tracing::warn!(code = %info.code, "request failed");
    }
    let (mut parts, _old_body) = response.into_parts();
    parts.headers.remove(CONTENT_LENGTH);
    parts.headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let body = error_body(&id, &info.code, &info.message).to_string();
    response = Response::from_parts(parts, Body::from(body));
}
```

`ApiError::into_response` leaves an `ErrorInfo` in the response extensions, and `remove` takes it out so it does not travel any further. Only a response that has one is rewritten: a `200` and the router's own 404 for an unknown path are returned as the app made them, plus the header (`a_route_that_does_not_exist_still_gets_an_id`). The status is kept because `into_parts` keeps it. `content-length` has to go: the old body was 61 bytes, the new one is longer, and a stale `content-length` would no longer match the body. With it removed, the length comes from the new `Body`. The log level follows the status class: 4xx is the caller's mistake and a `WARN`, 5xx is ours and an `ERROR` (`failures_are_logged_with_their_code_at_the_right_level`).

```rust
tracing::info!(status = response.status().as_u16(), "finished");
response.headers_mut().insert(REQUEST_ID_HEADER, HeaderValue::from_str(&id).unwrap());
```

`status` is recorded as a number, so the line reads `status=200`. The header is set last, with `insert`, which replaces any `x-request-id` the handler may have set. The `unwrap` cannot fail: an ID is either a valid ASCII string that was already a header value, or a UUID.

## Test output

```text
running 12 tests
test an_oversized_id_is_replaced ... ok
test a_hostile_id_never_reaches_the_log ... ok
test a_malformed_id_is_replaced ... ok
test a_missing_id_is_generated_and_echoed ... ok
test a_valid_incoming_id_is_kept_and_echoed ... ok
test a_generated_id_is_the_one_in_the_log ... ok
test an_error_body_carries_the_id_and_keeps_the_status ... ok
test handler_log_lines_carry_the_id_through_the_span ... ok
test the_path_in_the_span_has_no_query_string ... ok
test failures_are_logged_with_their_code_at_the_right_level ... ok
test a_route_that_does_not_exist_still_gets_an_id ... ok
test a_successful_response_body_is_untouched ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

```text
running 8 tests
test accepts_ordinary_ids ... ok
test error_body_has_the_documented_shape ... ok
test generated_ids_are_uuids_and_unique ... ok
test valid_incoming_id_is_kept_and_generate_is_not_called ... ok
test length_limit_is_64_bytes_inclusive ... ok
test missing_incoming_id_is_generated ... ok
test rejects_characters_outside_the_allowed_set ... ok
test invalid_incoming_id_is_replaced_not_repaired ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

(The order of the lines and the timings can differ between runs: `cargo test` runs tests in parallel.)

## Challenge (optional)

Nothing tests it, and one arrangement that works:

```rust
ServiceBuilder::new()
    .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
    .layer(from_fn(replace_invalid_id))
    .layer(trace)
    .layer(PropagateRequestIdLayer::x_request_id())
```

`replace_invalid_id` reads `x-request-id` from the request, and if `is_valid_request_id` says no, overwrites the header with `new_request_id()`. It sits after `SetRequestIdLayer` (which filled the header if it was missing) and before `TraceLayer` (so the span reads the corrected value). The ready-made layers took over generating, the span with latency, and echoing. What you could not give away: judging whether an incoming ID is acceptable, the typed `RequestId` in the extensions, and the ID in the error body. The order matters for the echo. With `PropagateRequestIdLayer` listed *before* `replace_invalid_id` (so outside it), the response echoed `"not valid!"` while the log showed the new UUID: the layer reads the request's header when the request reaches it, before the replacement. Listed after it, as above, the echoed header and the log carry the same ID.
