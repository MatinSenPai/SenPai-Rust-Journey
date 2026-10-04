# 3.8.1 — Consistent error envelopes

## At a glance

After this lesson you can:

- Define one JSON shape for every failure an API can produce, and say which field a client may branch on and which one it must never parse.
- Build it in one place: one `ApiError` enum whose `IntoResponse` impl is the only code that turns a failure into a status and a body, including the failures `axum` produces before your handler runs (broken JSON, a bad `{id}`, an unknown route, a wrong method).
- Keep a `5xx` from leaking internals while a `4xx` tells the client exactly what to fix, with per-field validation errors in the same envelope.
- Compare your envelope with RFC 9457 "problem details" (`application/problem+json`) and say what each one buys.

**Time:** ~100 minutes · **Prerequisites:**
[3.2.3 — Anime catalog CRUD (in-memory)](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.md),
[2.5.4 — Designing an error taxonomy for a service](../../../phase2-intermediate/05-error-handling/04-error-taxonomy-for-a-service/README.md),
[3.2.2 — Writing your own extractor (`FromRequestParts`)](../../02-axum-and-rest-api-design/02-writing-your-own-extractor/README.md),
[3.1.3 — HTTP semantics you must know](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.md)

---

## Why this matters

3.2.3 ended on a promise. Its handler answered a missing anime with `{"error":"anime not found"}`, and in the same transcript `axum`'s own rejections answered `Invalid URL: ...` and `Expected request with Content-Type: ...` as plain text. One API, already two shapes after one lesson. Now picture thirty endpoints and four authors. A frontend cannot write one error handler, it has to special-case every endpoint. An SDK generator cannot infer a schema. A support engineer cannot run `jq '.error.code'` over the logs and count.

In DRF you get most of this from a single place: `REST_FRAMEWORK["EXCEPTION_HANDLER"]` wraps every exception the views raise, so every error leaves through one function. `axum` has no such setting. What it has is a type system, and the fix is the same idea built from types: decide the shape once, build it in one `impl`, and make it structurally hard for any code path to bypass it.

This is also a security lesson. What a `500` says to a stranger is a decision, and the easiest way to get it wrong is to let the server's own error text travel to the client. You make that decision once, here, so no handler ever makes it again.

---

## The concept

### Seven failures, three shapes

Here is a tiny API with no envelope at all: one handler of ours that fails with `{"error": "..."}`, and `axum` doing everything else by default. `examples/01-four-shapes.rs` sends it seven bad requests and prints what comes back:

```text
GET /shows/9 -> 404 [application/json]
    {"error":"show 9 not found"}
GET /shows/abc -> 400 [text/plain; charset=utf-8]
    Invalid URL: Cannot parse `abc` to a `u64`
POST /shows -> 400 [text/plain; charset=utf-8]
    Failed to parse the request body as JSON: EOF while parsing an object at line 1 column 1
POST /shows -> 422 [text/plain; charset=utf-8]
    Failed to deserialize the JSON body into the target type: missing field `title` at line 1 column 2
POST /shows -> 415 [text/plain; charset=utf-8]
    Expected request with `Content-Type: application/json`
GET /nope -> 404 [-]
    (empty body)
DELETE /shows/1 -> 405 [-]
    (empty body)
```

Count the shapes: our own JSON object, plain English text, and an empty response with no `Content-Type` at all (twice). A client that wants to show "what went wrong" has to guess per response. None of them is *wrong* in isolation: `axum`'s defaults are sensible. The trouble is only that they are three different things.

### The shape: an envelope with a stable `code`

Every error body in this lesson has the same outer layer, and the same inner fields:

```json
{"error": {"code": "not_found", "message": "show 999 not found"}}
```

An **error envelope** is that fixed outer shape. Two fields do different jobs:

- **`code`** is machine-readable and stable. A client may `switch` on it. It is part of your API's contract, like a URL.
- **`message`** is for humans: a log line, a developer's terminal, a toast. It can be reworded, fixed, or translated without breaking anyone, *because nobody is allowed to parse it*. **`code` is the contract, `message` is a courtesy.**

A third field, `fields`, appears only on validation failures (below), and is simply absent otherwise. The envelope is also where the status class from 3.1.3 pays off: the HTTP status says which side is at fault (`4xx` you, `5xx` us), and `code` says exactly what. Clients already switch on the status first; `code` refines it.

### One enum, one `IntoResponse`

3.2.3 taught you `impl IntoResponse for AnimeError` for two variants. Here the same move scales to every failure an API has, so the interesting part is how it is organised. One enum lists *all* the ways a request can fail (with `#[derive(Debug, thiserror::Error)]` on top, from 2.5.3):

```rust
pub enum ApiError {
    NotFound(String),
    MethodNotAllowed,
    BadRequest(String),
    UnsupportedMediaType(String),
    InvalidBody(String),
    Validation(Vec<FieldError>),
    Internal(String),
}
```

Then three small `match`es, one per question a response needs answered, and one `IntoResponse` that asks all three:

```senpai-visual
{"kind":"result","labels":["any failure","ApiError","status() + code() + message()","one IntoResponse","same JSON envelope"]}
```

| Variant | `status()` | `code()` |
|---|---|---|
| `NotFound` | `404` | `not_found` |
| `MethodNotAllowed` | `405` | `method_not_allowed` |
| `BadRequest` | `400` | `bad_request` |
| `UnsupportedMediaType` | `415` | `unsupported_media_type` |
| `InvalidBody` | `422` | `invalid_body` |
| `Validation` | `422` | `validation_failed` |
| `Internal` | `500` | `internal_error` |

Why three methods and not one `match` that returns a tuple, like 3.2.3 did? Because each answer is now useful alone: a test can check `status()` and `code()` for every variant without building a response, a logger can print `code()` and nothing else, and a new variant forces you to answer all three questions. The compiler enforces that last part, and "Errors you will meet" shows the exact message.

### What the client may know: `4xx` versus `5xx`

3.1.3's table said `4xx` is "fix the request" and `5xx` is "not your fault". That is also the rule for what goes in `message`:

- A `4xx` message **tells the client how to fix it**: `title must be 1 to 100 characters`, `show 9 not found`. The client caused it and can only repair it with information.
- A `5xx` message **tells the client nothing**: always the same fixed sentence, `something went wrong on our side`. The *cause* (a refused database connection, a panic message, a file path) is for the operator. It goes to the server's log, never into the body. The reader of a body is a stranger on the internet, and a connection string is a gift to them.

So `ApiError::Internal(String)` carries text, and `message()` for it ignores that text. The one `IntoResponse` impl prints the text to standard error (a stand-in for the logging you will do properly in 3.8.2) and sends the fixed sentence. A client who wants to report the problem needs something to quote; [3.8.2 — Request tracing and correlation IDs](../02-request-tracing-and-correlation-ids/README.md) is where one ID per request connects the client's `500` to the server's log line.

### Pulling `axum`'s own failures into the envelope

Your handlers can only use the envelope on failures that reach your handler. The ones from the table at the start never do: the extractors fail first and answer for themselves, and an unknown route never has a handler. Three tools close the gaps.

**A `From` impl per rejection type.** 3.2.2 showed that a rejection is the response an extractor gives when it refuses to let the handler run. Here you convert one into your enum, using its status to pick the variant and `body_text()` for the message:

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

**A wrapper extractor with a derive.** `axum` can generate an extractor that behaves like `Json<T>` but fails with *your* error type, so a handler writes `ApiJson<NewShow>` and nothing else changes:

```rust
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct ApiJson<T>(pub T);
```

`via(axum::Json)` says "do the work with `Json`", and `rejection(ApiError)` says "and if it fails, convert with `From`". (`#[derive(FromRequest)]` needs `axum`'s `macros` feature, which this lesson's `Cargo.toml` turns on.) `ApiPath` is the same for `Path<T>` with `FromRequestParts`. A handler that still takes a plain `Path` or `Json` is the one hole left, and `examples/05-one-envelope.rs` leaves it open on purpose for `Path`: that rejection is plain text again.

**Two fallbacks for the router.** An unknown path and a known path with an unregistered method are different cases in `axum`, and each has its own hook:

```rust
Router::new()
    .route("/shows", post(create_show))
    .route("/shows/{id}", get(get_show))
    .fallback(route_not_found)
    .method_not_allowed_fallback(method_not_allowed)
    .with_state(store)
```

`.fallback(...)` answers requests that match no route (the `404`). `.method_not_allowed_fallback(...)` answers requests that match a path but not its methods (the `405`). Forgetting the second is a classic trap: the `fallback` does not cover it, and the client gets an empty `405`, the last line of the first transcript. Each handler is one line: it returns an `ApiError`, and the `IntoResponse` impl does the rest.

### Validation: one envelope, one entry per broken field

A validation failure is the one `4xx` where "tell the client how to fix it" needs more than one sentence, because a form with three bad fields has three things to fix. So `Validation` carries a list, and the envelope grows an optional `fields` array, one entry per broken rule:

```json
{"error": {"code": "validation_failed",
  "message": "the request body has invalid fields",
  "fields": [{"field": "title", "code": "length", "message": "title must be 1 to 100 characters"}]}}
```

Each entry has the same two-job split one level down: `field` and `code` are for the client's code ("highlight the `title` input"), `message` is for the human. The outer `code` stays `validation_failed` for every one of them, so a client that only cares that *something* is wrong branches on one string. Reporting *every* broken field in one response, rather than the first, is what saves a user three round trips. The status is `422`, from 3.2.3's rule: the JSON is well formed, the content breaks a rule. [3.3.2 — Validation](../../03-serialization-and-validation/02-validation/README.md) produces these per-field errors with a library; in this lesson `validate_new_show` writes the two rules by hand, so the envelope is the only new thing.

### A standard for the same job: RFC 9457

You did not invent the idea. RFC 9457, "Problem Details for HTTP APIs" (it obsoletes RFC 7807), standardises an error body with the media type `application/problem+json`. Its members are `type` (a URI that identifies the kind of problem, and defaults to `about:blank` when absent), `title` (a short summary of the type that should not change between occurrences), `status` (the HTTP status, advisory: the real one is the response's), `detail` (an explanation of *this* occurrence, which the RFC says should help the client correct the problem and which clients should not parse), and `instance` (a URI reference identifying this occurrence). Any other members are allowed as extensions, and clients must ignore the ones they do not know. `examples/06-problem-json.rs` renders the same `404` both ways:

```text
envelope: 404 application/json
  {"error":{"code":"not_found","message":"show 9 not found"}}
problem : 404 application/problem+json
  {"detail":"show 9 not found","instance":"/shows/9","status":404,"title":"Not Found","type":"https://api.example.com/problems/not_found"}
```

(The key order is alphabetical because `serde_json`'s `json!` stores objects in a sorted map. JSON objects are unordered.) The mapping between the two is close: `code` plays the part of `type`, `message` of `detail`, and the RFC's `fields`-like extension is whatever you name it (its own validation example uses `errors`). What the RFC buys you is recognition: standard clients, gateways and tooling already understand `application/problem+json`, and `type` is a URI you can point documentation at. What it costs is ceremony: a URI per problem type to design and keep stable. For an API whose consumers are your own frontends, the small envelope is plenty. For a public API that outsiders integrate with, the standard is worth the ceremony. Switching later is easy *because* there is one `IntoResponse`: you change one function, not thirty handlers. "Challenge" has you do exactly that.

---

## Hands on

Run the examples that compile. (`02` and `03` are broken on purpose and sit behind the `broken` feature; "Errors you will meet" shows them. `04` runs normally and is simply wrong, and "Errors you will meet" shows its output too.) The outputs of `01-four-shapes` and `06-problem-json` are in "The concept"; run them yourself and compare. Now the fix in miniature:

```sh
cargo run -p p3-08-01-consistent-error-envelopes --example 05-one-envelope
```

```text
GET /shows/9 -> 404 {"error":{"code":"not_found","message":"show 9 not found"}}
GET /shows/abc -> 400 Invalid URL: Cannot parse `abc` to a `u64`
POST /shows -> 400 {"error":{"code":"rejected","message":"Failed to parse the request body as JSON: EOF while parsing an object at line 1 column 1"}}
POST /shows -> 422 {"error":{"code":"rejected","message":"Failed to deserialize the JSON body into the target type: missing field `title` at line 1 column 2"}}
POST /shows -> 415 {"error":{"code":"rejected","message":"Expected request with `Content-Type: application/json`"}}
GET /nope -> 404 {"error":{"code":"not_found","message":"no route for /nope"}}
DELETE /shows/1 -> 405 
GET /boom -> 500 {"error":{"code":"internal_error","message":"something went wrong on our side"}}
```

```text
internal error: connect to postgres://anime:hunter2@db.internal refused
```

(The last block is standard error, which the terminal shows between the lines above; the client-facing body of `/boom` has no trace of it.) Two lines are still not envelopes on purpose: the bad `{id}` (a `Path` rejection nobody converted) and the `DELETE` (no `method_not_allowed_fallback`). The solution closes both.

The tests start out red. `src/lib.rs` has the whole skeleton, and every function you implement is a `todo!()` with a doc comment that is its complete specification:

```sh
cargo test -p p3-08-01-consistent-error-envelopes --no-fail-fast 2>&1 | grep 'test result'
```

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: FAILED. 0 passed; 12 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: FAILED. 0 passed; 14 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

When everything is green, run the real server (it listens on `127.0.0.1:3220`). These transcripts were captured against the solution. First the happy path and two kinds of domain failure:

```sh
cargo run -p p3-08-01-consistent-error-envelopes &
curl -s -w '\n%{http_code}\n' -X POST -H 'content-type: application/json' -d '{"title":"Frieren","episodes":28}' http://127.0.0.1:3220/shows
curl -s -w '\n%{http_code}\n' http://127.0.0.1:3220/shows/999
curl -s -w '\n%{http_code}\n' -X POST -H 'content-type: application/json' -d '{"title":"","episodes":0}' http://127.0.0.1:3220/shows
```

```text
{"id":1,"title":"Frieren","episodes":28}
201
{"error":{"code":"not_found","message":"show 999 not found"}}
404
{"error":{"code":"validation_failed","message":"the request body has invalid fields","fields":[{"field":"title","code":"length","message":"title must be 1 to 100 characters"},{"field":"episodes","code":"range","message":"episodes must be 1 to 2000"}]}}
422
```

Now the failures `axum` produced as plain text or empty bodies in the very first transcript:

```sh
curl -s -w '\n%{http_code}\n' -X POST -H 'content-type: application/json' -d '{' http://127.0.0.1:3220/shows
curl -s -w '\n%{http_code}\n' -X POST -H 'content-type: application/json' -d '{"episodes":3}' http://127.0.0.1:3220/shows
curl -s -w '\n%{http_code}\n' -X POST -d '{"title":"x","episodes":1}' http://127.0.0.1:3220/shows
curl -s -w '\n%{http_code}\n' http://127.0.0.1:3220/shows/abc
curl -s -w '\n%{http_code}\n' http://127.0.0.1:3220/nope
curl -s -w '\n%{http_code}\n' -X DELETE http://127.0.0.1:3220/shows/1
```

```text
{"error":{"code":"bad_request","message":"Failed to parse the request body as JSON: EOF while parsing an object at line 1 column 1"}}
400
{"error":{"code":"invalid_body","message":"Failed to deserialize the JSON body into the target type: missing field `title` at line 1 column 14"}}
422
{"error":{"code":"unsupported_media_type","message":"Expected request with `Content-Type: application/json`"}}
415
{"error":{"code":"bad_request","message":"Invalid URL: Cannot parse `abc` to a `u64`"}}
400
{"error":{"code":"not_found","message":"no route for /nope"}}
404
{"error":{"code":"method_not_allowed","message":"method not allowed for this route"}}
405
```

Seven different failure paths, one shape. Finally the `500`, and the server's own terminal:

```sh
curl -s -w '\n%{http_code}\n' http://127.0.0.1:3220/simulate-failure
```

```text
{"error":{"code":"internal_error","message":"something went wrong on our side"}}
500
```

```text
internal error: connect to postgres://anime:hunter2@db.internal:5432 refused
```

The second block is what the *server's* standard error printed for that request. The client saw only the first. Stop the server when you are done (`kill %1` in the same shell). Then try these:

1. Add `-i` to the `curl` of `/shows/999`. Which `content-type` does the response carry, and who set it?
2. Send `{"title":"x","episodes":"many"}`. Which `code` do you get, and why is it not `validation_failed`?
3. In `examples/05-one-envelope.rs`, add a `method_not_allowed_fallback` so the `DELETE` line becomes an envelope.

---

## Errors you will meet

### `E0004` — a new variant that `status()` does not cover

You add `Conflict` to the enum for a duplicate title and forget one of the `match`es:

```rust
fn status(&self) -> u16 {
    match self {
        ApiError::NotFound(_) => 404,
        ApiError::Internal(_) => 500,
    }
}
```

`examples/02-missing-arm-broken.rs`, built with `--features broken`:

```sh
cargo run -p p3-08-01-consistent-error-envelopes --example 02-missing-arm-broken --features broken
```

```text
error[E0004]: non-exhaustive patterns: `&ApiError::Conflict(_)` not covered
  --> phase3-backend-foundations\08-error-handling-and-testing-at-scale\01-consistent-error-envelopes\examples\02-missing-arm-broken.rs:14:15
   |
14 |         match self {
   |               ^^^^ pattern `&ApiError::Conflict(_)` not covered
   |
note: `ApiError` defined here
  --> phase3-backend-foundations\08-error-handling-and-testing-at-scale\01-consistent-error-envelopes\examples\02-missing-arm-broken.rs:6:6
   |
 6 | enum ApiError {
   |      ^^^^^^^^
 7 |     NotFound(String),
 8 |     Conflict(String),
   |     -------- not covered
   = note: the matched value is of type `&ApiError`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
16 ~             ApiError::Internal(_) => 500,
17 ~             &ApiError::Conflict(_) => todo!(),
   |

For more information about this error, try `rustc --explain E0004`.
error: could not compile `p3-08-01-consistent-error-envelopes` (example "02-missing-arm-broken") due to 1 previous error
```

(Before the error, the command also prints three `unused variable` warnings from this lesson's own skeleton, which has `todo!()`s in it. They are not part of the error and go away once you implement the functions.)

**What the compiler is objecting to:** a `match` must cover every variant, and `Conflict` has no arm. The note points at the variant that is not covered.

**The fix:** add the arm, `ApiError::Conflict(_) => 409`. Not a `_ =>` wildcard, and not the `todo!()` the help line suggests.

**Why this is the fix:** this error is the *reason* the enum has three small `match`es. Every time you add a failure mode, the compiler walks you to every place that has to say what it means on the wire (status, code, message), and an arm that was never written cannot ship. A `_ => 500` wildcard would silence it and answer your new `409` with a `500` instead, without a single warning.

### `E0277` — `?` on a rejection with no `From` impl

A handler takes the extractor's `Result` itself and uses `?`, so the rejection can be returned as an `ApiError`:

```rust
async fn create(body: Result<Json<NewShow>, JsonRejection>) -> Result<String, ApiError> {
    let Json(show) = body?;
    Ok(show.title)
}
```

`examples/03-question-mark-without-from-broken.rs`:

```text
error[E0277]: `?` couldn't convert the error to `ApiError`
  --> phase3-backend-foundations\08-error-handling-and-testing-at-scale\01-consistent-error-envelopes\examples\03-question-mark-without-from-broken.rs:26:26
   |
26 |     let Json(show) = body?;
   |                      ----^ the trait `From<JsonRejection>` is not implemented for `ApiError`
   |                      |
   |                      this can't be annotated with `?` because it has type `Result<_, JsonRejection>`
   |
note: `ApiError` needs to implement `From<JsonRejection>`
  --> phase3-backend-foundations\08-error-handling-and-testing-at-scale\01-consistent-error-envelopes\examples\03-question-mark-without-from-broken.rs:17:1
   |
17 | struct ApiError;
   | ^^^^^^^^^^^^^^^
   = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the `From` trait

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-08-01-consistent-error-envelopes` (example "03-question-mark-without-from-broken") due to 1 previous error
```

(The same three skeleton warnings come first, omitted.)

**What the compiler is objecting to:** `?` does not just return the error, it calls `From::from` on it to reach the function's error type (1.6.5's rule). The compiler tells you which impl is missing, and even marks the type that needs it.

**The fix:** implement `From<JsonRejection> for ApiError`, as in "The concept". With the `ApiJson` derive, the same impl is what `rejection(ApiError)` calls, and the handler no longer mentions `JsonRejection` at all.

**Why this is the fix:** `From` is how errors from a lower layer enter your one enum. Each new source of failure costs exactly one `impl From`, written once, and `?` does the rest everywhere.

### No error at all: the `500` that leaks

```text
500 Internal Server Error
{"error":{"code":"internal_error","message":"connect to postgres://anime:hunter2@db.internal:5432 refused"}}
```

**What's actually broken:** `examples/04-leaky-500-trap.rs` compiles, runs, and has a perfectly consistent envelope. Its `IntoResponse` copies the `Internal` variant's text into `message`. The shape is right and the content is a disaster: the client now knows a database hostname, a user, and a password. A consistent envelope makes this *easier* to do by accident, because `message` is right there for every variant.

**The fix:** give `Internal` its own `message()` arm that ignores the text and returns a fixed sentence, and print the text to the server's log instead.

**Why this is the fix:** the type already separates the two audiences. `Internal(String)` holds the server's text, and the only function that decides what a client sees is `message()`. Test it by asserting that a response never contains a secret you planted in the text. `into_response_hides_the_internal_detail` in `tests/error_test.rs` does exactly that, and no compiler can catch this class of mistake.

---

## Exercises

### Warm up

<details>
<summary>An app shows the user whatever <code>message</code> a failed request returned, and branches on <code>code</code>. The team rewords a message from "show 9 not found" to "We couldn't find show 9". What breaks?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

Nothing, which is the point. The app branches on `code` (`not_found`), which did not change, and merely displays `message`. Had it matched on the sentence, every rewording would be a silent breaking change.

</details>

<details>
<summary>Why can a <code>5xx</code> envelope use the same fixed <code>message</code> every time, but a <code>4xx</code> one cannot?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

A `4xx` is the client's to repair, so the body must carry what to fix (`title must be 1 to 100 characters`). A `5xx` is ours to repair; the client can only retry or report, and anything more specific is a leak of internals. 3.1.3's classes drive the rule.

</details>

<details>
<summary>A route is registered for <code>GET /shows/{id}</code> and the router has <code>.fallback(f)</code>. A client sends <code>DELETE /shows/1</code>. Does <code>f</code> run?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No. The path matched, only the method did not, and that case has its own hook, `method_not_allowed_fallback`. Without it the client gets an empty `405`, as the `DELETE` line of `examples/05-one-envelope.rs` shows.

</details>

<details>
<summary>The envelope has <code>code</code> and each validation entry has its own <code>code</code>. Why is that not duplication?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

They answer different questions. The outer `code` (`validation_failed`) says what kind of failure the *request* was. The inner `code` (`length`, `range`) says which *rule* one field broke. A client that only needs to know "was it a validation failure" reads one string, and a form that needs to highlight inputs reads the list.

</details>

<details>
<summary>RFC 9457 says <code>status</code> is advisory, and that the response must use the same code anyway. Why repeat it in the body at all?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

So the information survives when the body is separated from its HTTP response: stored in a log, forwarded through a queue, or when an intermediary changed the status. The RFC names exactly those cases. Software that reads the HTTP message uses the real status line.

</details>

### Repair

Fix the three examples that carry the mistakes of "Errors you will meet":

1. `examples/02-missing-arm-broken.rs` compiles with `--features broken`: add the missing arm so `Conflict` answers `409`. Do not use a wildcard.
2. `examples/03-question-mark-without-from-broken.rs` compiles with `--features broken`: add the missing impl. `ApiError` must answer `400` for every rejection.
3. `examples/04-leaky-500-trap.rs` runs but must not print `hunter2`. The body must keep the same shape, with the fixed message `something went wrong on our side`.

### Implement

Everything in `src/lib.rs` that is a `todo!()`: `ApiError`'s `status`, `code` and `message`, its `IntoResponse`, the two `From` impls for `axum`'s rejections, and `validate_new_show`. Each doc comment is the complete specification (the statuses, the codes, the exact messages, the order of validation entries), so you should never need to open the tests to know what to build. The store, the handlers, the wrapper extractors and the router are given: this lesson's job is the error layer.

```sh
cargo test -p p3-08-01-consistent-error-envelopes
```

`tests/error_test.rs` (14 tests) checks the error layer with plain function calls, no router and no requests. `tests/api_test.rs` (12 tests) sends real requests through `oneshot` and checks that every failure, including the ones `axum` produces, has the envelope.

### Build

Add a failure of your own: creating a show whose title already exists (compared exactly, as sent) is a `409 Conflict` with the code `conflict`. Add `ApiError::Conflict(String)`; the compiler will walk you through every `match` that needs an arm. Make `POST /shows` check the store before it inserts, and answer with the message `a show titled "<title>" already exists`. Write your own tests in a new `tests/conflict_test.rs`, one for the status and envelope and one that sends the same `POST` twice and asserts the second answers `409` while the first stays `201`.

### Challenge (optional)

Render the same `ApiError` as an RFC 9457 problem. Add `ApiError::problem_response(&self, instance: &str) -> Response`, which answers with the same status and with `Content-Type: application/problem+json`, and a body with `type` (`https://api.example.com/problems/<code>`), `title` (the status's canonical reason phrase), `status`, `detail` (the same text as `message()`), `instance`, and, for `Validation`, an `errors` extension of the field entries. The RFC in "Going further" is the whole specification; nothing else in the course depends on this. Write your own tests.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Error envelope | one fixed JSON shape (`{"error": {...}}`) for every failure an API returns | every API with more than one endpoint |
| `code` vs `message` | `code` is a stable, machine-readable contract; `message` is a human courtesy that may change | clients, tests, log analysis |
| Client-safe message | the text a client may see: how to fix a `4xx`, a fixed sentence for a `5xx` | every error variant |
| Rejection conversion | `From<SomeRejection> for ApiError`, plus a derived wrapper extractor, so `axum`'s own failures use the envelope | `Json`, `Path`, `Query`, anything with a rejection |
| `fallback` / `method_not_allowed_fallback` | the two router hooks for "no route" and "known route, wrong method" | every public API |
| Problem details (RFC 9457) | the standard error body, `application/problem+json`, with `type`, `title`, `status`, `detail`, `instance` | public APIs, API gateways |

### What you now know

- A consistent API has one error shape, and the shape is cheap to get: one enum, one `IntoResponse`, one place that knows how a failure looks on the wire.
- `code` is a contract and `message` is a courtesy, so clients branch on the first and merely show the second.
- A `5xx` says a fixed sentence and logs the real cause; a `4xx` says how to fix it. The type separates the two so no handler can mix them up.
- The failures `axum` produces before your handler (bad JSON, a bad `{id}`, no route, wrong method) need three separate hooks: a `From` per rejection with a derived wrapper extractor, `fallback`, and `method_not_allowed_fallback`.
- Validation errors fit the same envelope as a list with one entry per broken field, reported all at once.
- RFC 9457 standardises the same job. The envelope in this lesson maps onto it closely, and one `IntoResponse` makes switching cheap.

### What comes back later

- **Following a client's `500` to the server's log line**: [3.8.2 — Request tracing and correlation IDs](../02-request-tracing-and-correlation-ids/README.md)
- **Per-field validation errors with a real validation library**: [3.3.2 — Validation](../../03-serialization-and-validation/02-validation/README.md)
- **A typed description of these error bodies in an API contract**: [3.3.3 — API contracts and OpenAPI (`utoipa`)](../../03-serialization-and-validation/03-api-contracts-and-openapi/README.md)
- **Where the in-memory store is replaced by a database**: [3.5.3 — Anime catalog, Postgres-backed](../../05-postgres-and-sqlx/03-anime-catalog-postgres-backed/README.md)
- **Auth as `tower` middleware**: [3.7.3 — JWTs and `tower` middleware](../../07-auth-and-security/03-jwt-and-tower-middleware/README.md)
- **Integration tests against a real, disposable database**: [3.8.3 — Integration tests with `testcontainers`](../03-integration-tests-with-testcontainers/README.md)

### Can you explain?

- Why does the envelope have both `code` and `message`, and which one may a client match on?
- Why does a `5xx` answer the same fixed sentence every time, and where does the real cause go?
- Why is a plain-text `400` from `axum` still a problem for your API, even though the status is right?
- What do you add to the router so a `DELETE` on a `GET`-only path gets an envelope, and why does `.fallback` not cover it?
- Why is it good that `status()`, `code()` and `message()` are three `match`es, and what does `E0004` do for you when you add a variant?
- What would you gain and what would you pay for by moving to `application/problem+json`?

---

## Going further

- [RFC 9457 — Problem Details for HTTP APIs](https://www.rfc-editor.org/rfc/rfc9457.html): short and readable; §3.1 defines the five members, §3.2 extensions, §4.2.1 `about:blank`. The validation example is in §3.
- [`axum::extract::rejection`](https://docs.rs/axum/0.8.9/axum/extract/rejection/index.html): every rejection type, with the status each answers.
- [`axum::Router::method_not_allowed_fallback`](https://docs.rs/axum/0.8.9/axum/struct.Router.html#method.method_not_allowed_fallback): the hook for a matched path with a missing method.
- [`#[derive(FromRequest)]`](https://docs.rs/axum/0.8.9/axum/extract/derive.FromRequest.html): the `via` and `rejection` attributes this lesson used.
- [Django REST framework — custom exception handling](https://www.django-rest-framework.org/api-guide/exceptions/#custom-exception-handling): the one-function equivalent you already know.
