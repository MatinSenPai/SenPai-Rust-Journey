# 3.2.1 — Routing, handlers, extractors

## At a glance

After this lesson you can:

- Build an `axum` `Router` from async handlers, and say which request fields each handler parameter (`Path`, `Query`, `Json`, `State`) reads.
- Predict the status code `axum` answers on its own when an extractor refuses a request (`400`, `415`, `422`, `404`, `405`), and why the handler body never runs.
- Test a whole router, from routing through your handler to the serialized response, with `oneshot` and no socket, then try the same thing for real with `curl`.
- Read the `E0277` `Handler<_, _>` error you get when a body extractor is not last, and fix it.

**Time:** ~90 minutes · **Prerequisites:**
[3.1.2 — Hand-rolled HTTP parser](../../01-networking-and-http-from-scratch/02-hand-rolled-http-parser/README.md),
[3.1.3 — HTTP semantics you must know](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.md),
[2.8.6 — `tokio` basics](../../../phase2-intermediate/08-concurrency/06-tokio-basics/README.md)

---

## Why this matters

Module 1 built HTTP by hand: a `TcpListener` loop in 3.1.1, then in 3.1.2 a `parse_request` that turned bytes into an `HttpRequest` and returned a named error for every way those bytes could be wrong. `axum` is that same job done by a library, for every request, and what is left for you is what Django also leaves to you: **routes** (`urls.py`) and **handlers** (views).

The part worth seeing is the link between the two. When 3.1.2 said "`axum` hands back an automatic 400 when `Json<T>` or `Path<T>` doesn't match", it meant that an *extractor* is a `parse_request` that someone else wrote and that runs before your function does. In this lesson you read exactly what it answers, and you will see that the answer is not always `400`.

| Django / DRF | `axum` |
|---|---|
| `urls.py`: `path("greet/<name>/", views.greet)` | `Router::new().route("/greet/{name}", get(greet))` |
| a view `def greet(request, name)` | a handler `async fn greet(Path(name): Path<String>) -> String` |
| `request.GET["q"]` | `Query<T>` |
| DRF serializer reading `request.data` | `Json<T>` |
| URL kwargs | `Path<T>` |
| middleware list | `tower` layers, built by hand in [3.2.4](../04-tower-service-and-layer-middleware/README.md) |

The table has a limit. A Django view receives *one* `request` object and digs into it. An `axum` handler receives only what it declared, already parsed, or is never called.

---

## The concept

### A handler is an async function whose parameters are the request

```rust
async fn greet(Path(name): Path<String>) -> String {
    format!("Hello, {name}!")
}
```

Served over a real socket (`examples/05-serve-on-a-real-port.rs` serves this exact handler), `curl -i http://127.0.0.1:3021/greet/senpai` prints:

```text
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 14
date: Thu, 24 Sep 2026 22:49:24 GMT

Hello, senpai!
```

(The `date` value is whatever time you run it.) Three things happened that you did not write: `axum` read the request, put the `senpai` segment into `name`, and turned the returned `String` into a response with a status line, a `content-type`, and a correct `content-length`. That last step is the work 3.1.2's `HttpResponse::to_bytes` did by hand.

Two rules are worth stating now:

- **The parameter list is the request.** `axum` looks at each parameter's *type* and asks that type to pull itself out of the request. You declare what you need in the signature, and it arrives parsed.
- **The return type is the response.** Anything that implements `IntoResponse` works: `&'static str`, `String`, `Json<T>`, a `(StatusCode, body)` pair. 3.2.3 teaches how to make your own error type one.

The function is `async fn` because that is what `axum` requires of a handler. You have not met anything to `.await` yet; that arrives with the database in module 5.

### The `Router` maps (method, path) to a handler

```rust
let app = Router::new()
    .route("/", get(hello))
    .route("/greet/{name}", get(greet));
```

Each `.route(path, method_router)` registers one path, and `get(handler)`, `post(handler)` choose the HTTP method. Path parameters are written `{name}`, with braces. `examples/01-first-router.rs` builds this router and sends it four requests:

```text
GET  /              -> 200 OK  body: "Hello, world!"
GET  /greet/senpai  -> 200 OK  body: "Hello, senpai!"
GET  /nope          -> 404 Not Found  body: ""
POST /greet/senpai  -> 405 Method Not Allowed  body: ""  allow: "GET,HEAD"
```

You wrote no code for the last two lines. A path that matches nothing is `404`. A path that exists but was registered only for other methods is `405`, with an `allow` header listing the methods that would have worked. That is 3.1.3's method semantics, applied by the router.

```senpai-visual
{"kind":"network","labels":["request arrives: method + path","Router picks the handler","extractors run, left to right","any extractor refuses: axum answers 400, 415 or 422, handler skipped","handler runs and returns a value","IntoResponse turns it into the HTTP response"]}
```

### Extractors: `Path`, `Query`, `Json`, `State`

Four extractors cover most handlers:

- **`Path<T>`** reads `{name}` segments from the route pattern.
- **`Query<T>`** reads the query string. In 3.1.2 you split `?status=watching` on `&` and `=` by hand and kept the pairs in a `Vec`. `Query<T>` does that split, percent-decoding included, and fills a struct:

```rust
#[derive(Debug, Deserialize)]
struct AnimeFilter {
    status: String,
    page: Option<u32>,
}

async fn list_anime(Query(filter): Query<AnimeFilter>) -> String {
    format!("{filter:?}")
}
```

```text
GET /anime?status=watching
    -> 200 OK: AnimeFilter { status: "watching", page: None }
GET /anime?status=watching&page=2
    -> 200 OK: AnimeFilter { status: "watching", page: Some(2) }
GET /anime?status=plan%20to%20watch
    -> 200 OK: AnimeFilter { status: "plan to watch", page: None }
GET /anime?page=2
    -> 400 Bad Request: Failed to deserialize query string: missing field `status`
GET /anime?status=watching&page=two
    -> 400 Bad Request: Failed to deserialize query string: page: invalid digit found in string
```

That is `examples/03-query-extractor.rs`. A field typed `Option<u32>` is optional; a field typed `String` is required. This is what separates `Path` from `Query`: a path segment identifies *which* resource (`/anime/7`), a query parameter adjusts *how* you want it (`?page=2`) and is often optional.

- **`Json<T>`** parses the request body as JSON into `T`, which must implement `Deserialize`. Returning `Json<T>` serializes `T` and sets `content-type: application/json`. It is DRF's serializer, with the validity check built in.
- **`State<S>`** hands the handler a clone of the value given to `.with_state(...)`:

```rust
#[derive(Clone, Default)]
struct AppState {
    visits: Arc<Mutex<u64>>,
}

async fn visit(State(state): State<AppState>) -> String {
    let mut visits = state.visits.lock().unwrap();
    *visits += 1;
    format!("visit #{}", *visits)
}
```

```text
clone of the same router -> visit #1
clone of the same router -> visit #2
clone of the same router -> visit #3
a brand-new AppState     -> visit #1
```

(`examples/04-shared-state.rs`.) Cloning an `AppState` clones the `Arc`, not the number, so every request sees the same counter; a fresh `AppState` starts over. This is [2.6.3](../../../phase2-intermediate/06-smart-pointers/03-rc-and-arc/README.md)'s `Arc<Mutex<T>>` with HTTP requests as the sharing parties, and the `Mutex` is why it is safe when two requests arrive at once.

### When an extractor refuses: what `axum` answers for you

3.1.2's `parse_request` returned `Err(HttpParseError::...)` for each kind of bad input. An `axum` extractor does the same and, on `Err`, sends a response instead of calling your handler. `examples/02-what-axum-answers-for-you.rs` sends a handler taking `Json<EchoRequest>` and another taking `Path<u32>` seven requests:

```text
POST /echo {"message":"hi"}
    -> 200 OK: echo: hi
POST /echo not json
    -> 400 Bad Request: Failed to parse the request body as JSON: expected ident at line 1 column 2
POST /echo {"msg":"hi"}
    -> 422 Unprocessable Entity: Failed to deserialize the JSON body into the target type: missing field `message` at line 1 column 12
POST /echo {"message":42}
    -> 422 Unprocessable Entity: Failed to deserialize the JSON body into the target type: message: invalid type: integer `42`, expected a string at line 1 column 13
POST /echo {"message":"hi"}   (no Content-Type header)
    -> 415 Unsupported Media Type: Expected request with `Content-Type: application/json`
GET /anime/7
    -> 200 OK: anime #7
GET /anime/seven
    -> 400 Bad Request: Invalid URL: Cannot parse `seven` to a `u32`
```

So "an automatic 400" was a simplification. The real answers, for `axum` 0.8.9:

| What arrived | Status | Why |
|---|---|---|
| body that is not JSON at all | `400` | malformed: the syntax is broken |
| valid JSON of the wrong shape (missing field, wrong type) | `422` | it parsed, but the content is invalid |
| JSON body without `Content-Type: application/json` | `415` | the media type is wrong, whatever the body says |
| `Path` or `Query` value that will not parse into the type | `400` | the request line itself is wrong |

The first two rows are exactly 3.1.3's `400`-versus-`422` rule, applied by the extractor: `400` means the request is malformed, `422` means it parsed fine but its content is invalid. `415` is a status 3.1.3 did not list. It is the server saying that it refuses the *format*, which is content negotiation's failure case. The last row shows the rule is not applied uniformly: `Path` and `Query` use `400` even though `?page=two` is closer to "parsed fine, invalid content". Those are `axum`'s choices, so check them instead of assuming.

Two consequences follow. Your handler body can assume its parameters are valid, because invalid requests never get in. And the error text comes from `serde` and `axum`, plain text, not JSON; a uniform JSON error body is [3.8.1](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)'s job.

### Extractor order: the body comes last

Extractors run in the order of the parameters, left to right. A request body is a stream that can be read once, so `Json<T>` (and `String`, `Bytes`) may only be the **last** parameter. `axum` enforces this in the types: every parameter except the last must implement `FromRequestParts`, and the last must implement `FromRequest`. `Path`, `Query` and `State` are the first kind; `Json` is the second. Put `Json` first and the function is not a handler at all, which is the `E0277` in "Errors you will meet".

### Under the `Router`: `tokio` tasks, not threads

3.1.1 spawned one OS thread per connection and said `tokio` would replace that. `axum::serve(listener, app)` runs on `tokio`: it accepts connections in a loop and, for each one, calls `tokio::spawn` for a task that drives that connection (checked in `axum` 0.8.9's `serve` module). A task is far cheaper than an OS thread, as [2.8.6](../../../phase2-intermediate/08-concurrency/06-tokio-basics/README.md) showed, and that is why `async fn` is the shape of a handler: while one request waits, the thread serves another. A task is per connection, not per request: requests arriving one after another on a kept-alive connection (3.1.3) are served by the same task.

### Testing without a socket: `oneshot`

A `Router` is a `tower::Service`: something that takes a request and returns a response. So a test can hand it a `Request` directly with `tower::ServiceExt::oneshot`, and routing, extraction, your handler and serialization all run with no port bound and no network:

```rust
let response = app(AppState::default())
    .oneshot(
        Request::builder()
            .uri("/echo") // a GET, but only POST is registered
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();

assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
```

This is the same "I/O at the edges" idea as 3.1.1's `run_echo`, one layer up. In the finished solution, `cargo test --test routes_test` prints (the order of the lines varies):

```text
running 7 tests
test wrong_method_on_a_known_route_returns_405 ... ok
test unknown_route_returns_404 ... ok
test root_returns_hello_world ... ok
test greet_uses_the_path_segment ... ok
test echo_rejects_a_malformed_json_body_before_the_handler_runs ... ok
test counter_starts_at_zero_and_increments ... ok
test echo_round_trips_json_and_reports_length ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

---

## Hands on

Run the five examples. The first four use `oneshot`; the fifth binds a real port.

```sh
cargo run -p p3-02-01-routing-handlers-extractors --example 01-first-router
cargo run -p p3-02-01-routing-handlers-extractors --example 02-what-axum-answers-for-you
cargo run -p p3-02-01-routing-handlers-extractors --example 03-query-extractor
cargo run -p p3-02-01-routing-handlers-extractors --example 04-shared-state
cargo run -p p3-02-01-routing-handlers-extractors --example 05-serve-on-a-real-port
```

Leave the fifth running, and in a second terminal try it for real. Port `3021` is used so it does not collide with other servers:

```sh
curl -si -X POST -H 'content-type: application/json' -d '{"message":"hi"}' http://127.0.0.1:3021/shout
curl -si -X POST -H 'content-type: application/json' -d '{"message":' http://127.0.0.1:3021/shout
curl -si http://127.0.0.1:3021/shout
```

```text
HTTP/1.1 200 OK
content-type: application/json
content-length: 16
date: Thu, 24 Sep 2026 22:49:24 GMT

{"shouted":"HI"}
HTTP/1.1 400 Bad Request
content-type: text/plain; charset=utf-8
content-length: 96
date: Thu, 24 Sep 2026 22:49:24 GMT

Failed to parse the request body as JSON: message: EOF while parsing a value at line 1 column 11
HTTP/1.1 405 Method Not Allowed
allow: POST
content-length: 0
date: Thu, 24 Sep 2026 22:49:24 GMT

```

Stop the server with Ctrl+C when you are done. Then try these:

1. In `03-query-extractor`, make `page` a plain `u32` instead of `Option<u32>`. Which of the five requests changes, and to what?
2. In `02-what-axum-answers-for-you`, change `Path<u32>` to `Path<String>`. What does `/anime/seven` return now?
3. In `04-shared-state`, remove `.clone()` from the first call and use `shared` twice. What does the compiler say, and why?

---

## Errors you will meet

### `E0277` — `Handler<_, _>` is not satisfied

```text
error[E0277]: the trait bound `fn(Json<NewNote>, State<AppState>) -> impl Future<Output = String> {add_note}: Handler<_, _>` is not satisfied
   --> phase3-backend-foundations\02-axum-and-rest-api-design\01-routing-handlers-extractors\examples\06-body-extractor-not-last-broken.rs:33:31
    |
 33 |         .route("/notes", post(add_note))
    |                          ---- ^^^^^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(Json<NewNote>, State<AppState>) -> impl Future<Output = String> {add_note}`
    |                          |
    |                          required by a bound introduced by this call
    |
    = note: Consider using `#[axum::debug_handler]` to improve the error message
note: required by a bound in `post`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\method_routing.rs:167:16
    |
167 |             H: Handler<T, S>,
    |                ^^^^^^^^^^^^^ required by this bound in `post`
...
445 | top_level_handler_fn!(post, POST);
    | ---------------------------------
    | |                     |
    | |                     required by a bound in this function
    | in this macro invocation
    = note: this error originates in the macro `top_level_handler_fn` (in Nightly builds, run with -Z macro-backtrace for more info)

For more information about this error, try `rustc --explain E0277`.
```

**What the compiler is objecting to:** the error is on `post(add_note)`, not inside `add_note`. `post` requires its argument to implement `Handler`, and `axum` implements `Handler` for an `async fn` only if every parameter but the last is a `FromRequestParts` and the last is a `FromRequest`. Here `Json<NewNote>` comes first, and `Json` needs the body, so it can only be `FromRequest`. The message names which function failed and not which parameter is wrong, which is why it feels unhelpful. Read the first line for the shape of your function: `fn(Json<NewNote>, State<AppState>)`, and notice the body extractor is not at the end.

**The fix:** move `Json<NewNote>` to the last position:

```rust
async fn add_note(State(state): State<AppState>, Json(note): Json<NewNote>) -> String {
```

**Why this is the fix:** the body can be read once, so it has to come after everything that only looks at the request's head (method, path, headers, state). The note in the output mentions `#[axum::debug_handler]`, an attribute that turns this error into one pointing at the offending parameter. It is behind `axum`'s optional `macros` feature, which this workspace does not enable, so there is no example for it here; when you have the feature on, put the attribute on the handler.

### A run-time panic — the old `/:id` path syntax

```text
building the router...
thread 'main' (13560) panicked at phase3-backend-foundations\02-axum-and-rest-api-design\01-routing-handlers-extractors\examples\07-colon-path-syntax-broken.rs:18:38:
Path segments must not start with `:`. For capture groups, use `{capture}`. If you meant to literally match a segment starting with a colon, call `without_v07_checks` on the router.
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(The number in parentheses is the thread's id and changes every run.)

**What's actually broken:** `axum` up to 0.7 wrote path parameters as `/:id`. Code and tutorials written for it are everywhere. In 0.8 the syntax is `/{id}`, and `.route` panics the moment it sees a segment starting with `:`. It is a run-time panic, not a compile error, because a route is just a string. The good news is that it happens when the router is *built*, at startup, never on the first request.

**The fix:** `.route("/anime/{id}", get(anime))`.

**Why this is the fix:** `{...}` is the 0.8 spelling, and it also lets a literal `:` appear in a path. Look at the line number in the panic: it points at your `.route` call, which is the line to fix.

### `E0308` — forgetting `.with_state(...)`

```text
error[E0308]: mismatched types
   --> phase3-backend-foundations\02-axum-and-rest-api-design\01-routing-handlers-extractors\examples\08-missing-with-state-broken.rs:24:36
    |
 24 |     Router::new().route("/visits", get(visits))
    |                   -----            ^^^^^^^^^^^ expected `MethodRouter`, found `MethodRouter<AppState>`
    |                   |
    |                   arguments to this method are incorrect
    |
    = note: expected struct `MethodRouter<()>`
               found struct `MethodRouter<AppState>`
note: method defined here
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:178:12
    |
178 |     pub fn route(self, path: &str, method_router: MethodRouter<S>) -> Self {
    |            ^^^^^

For more information about this error, try `rustc --explain E0308`.
error: could not compile `p3-02-01-routing-handlers-extractors` (example "08-missing-with-state-broken") due to 1 previous error
```

**What the compiler is objecting to:** `Router<S>` and `MethodRouter<S>` carry the type of state they are still *missing*. `Router::new()` here is a `Router<()>` that needs no state, but `get(visits)` is a `MethodRouter<AppState>` because `visits` asks for `State<AppState>`. The two disagree. `.with_state(state)` is what turns a `Router<AppState>` into a `Router<()>`, and `fn app() -> Router` means `Router<()>`.

**The fix:** give the router its state:

```rust
Router::new().route("/visits", get(visits)).with_state(AppState::default())
```

**Why this is the fix:** the state's type now flows from the handlers into the router, and `.with_state` supplies the value, so nothing is missing any more. Notice that the error is reported on the *handler argument*, not on a missing method call; this is a type that "still needs something", and the fix is almost always `.with_state`.

---

## Exercises

### Warm up

<details>
<summary>A handler takes <code>Json&lt;T&gt;</code> and <code>State&lt;AppState&gt;</code>. In which order must they be written, and why?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`State` first, `Json` last. The body can only be read once, so the extractor that consumes it must be the last parameter; `axum` enforces that by requiring all earlier parameters to be `FromRequestParts`.

</details>

<details>
<summary>A client posts <code>{"msg":"hi"}</code> with <code>Content-Type: application/json</code> to a handler that takes <code>Json&lt;EchoRequest&gt;</code>, where <code>EchoRequest</code> has a <code>message</code> field. Which status does it get, and does the handler run?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`422 Unprocessable Entity`, and the handler does not run. The body is valid JSON, so it is not a `400`; it has the wrong shape, which is the "parsed fine but invalid" case from 3.1.3.

</details>

<details>
<summary>You want the anime id in <code>/anime/7</code> and the page number in <code>/anime?page=2</code>. Which extractor for each?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`Path<u32>` for the id (it identifies the resource, and the route is `/anime/{id}`), `Query<...>` for the page (it adjusts the listing and is usually optional).

</details>

<details>
<summary>A <code>GET</code> arrives at a path that only has a <code>post(...)</code> registered. What does the router answer?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`405 Method Not Allowed` with an `allow: POST` header, and no code from you. `404` is for a path that matches no route at all.

</details>

### Repair

Fix all three broken examples (the commands are in "Errors you will meet"):

1. `examples/06-body-extractor-not-last-broken.rs` compiles with `--features broken`.
2. `examples/07-colon-path-syntax-broken.rs` prints `router built`.
3. `examples/08-missing-with-state-broken.rs` compiles.

### Implement

Five functions in `src/lib.rs`: `greet`, `echo`, `get_counter`, `increment_counter` and `app`.

```sh
cargo test -p p3-02-01-routing-handlers-extractors --test routes_test
```

Each doc comment is the full specification (paths, methods, status codes, JSON shapes), so you do not need to open the tests. `hello` is given as a model.

### Build

Add `GET /search?q=...&limit=...` to the router. In `src/lib.rs`, write a `search` handler that takes `Query<T>` and answers `200` with JSON `{"q":"naruto","limit":3}` for `/search?q=naruto&limit=3`.

Specification: `q` is required; `limit` is optional and defaults to `10` (`/search?q=naruto` answers `{"q":"naruto","limit":10}`). `q` is returned decoded (`q=one%20piece` gives `"one piece"`). A request without `q`, or with a `limit` that is not a number, is rejected by the extractor with `400` before your handler runs, so you write no code for it. Register the route in `app` and run:

```sh
cargo test -p p3-02-01-routing-handlers-extractors --test search_test
```

### Challenge (optional)

Make `echo` report its own error. Change its parameter to a `Result` of the extractor and its rejection (`axum::extract::rejection::JsonRejection`). On `Err`, answer with the rejection's own status (`rejection.status()`) and the body text `bad echo request`; on `Ok`, answer as before. The two arms must return the same type, so convert both with `.into_response()` (from `axum::response::IntoResponse`) and make the handler return `Response`. The existing tests must still pass. This reaches forward: the full story of turning *your own* errors into responses is 3.2.3.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Handler | an async function whose parameters are extractors and whose return value becomes the response | every route |
| `Router` | the table mapping (method, path) to a handler | the whole API |
| Extractor | a type that pulls one piece out of the request, or refuses it | handler signatures |
| `Path<T>` / `Query<T>` | the path segment / the query string, parsed into `T` | identifying vs. adjusting a resource |
| `Json<T>` | the body parsed into `T`; as a return value, `T` serialized | request and response bodies |
| `State<S>` | the value given to `.with_state`, cloned per request | shared data such as a counter or a pool |
| Rejection | the response `axum` sends when an extractor fails: `400`, `415` or `422` | every handler with extractors |
| `oneshot` | pushing one request through a `Router` with no socket | all route tests |

### What you now know

- A handler declares what it needs in its parameter types, and the router runs the extractors left to right before the body executes.
- An extractor that refuses a request answers for you. For `Json`: `400` for broken syntax, `422` for the wrong shape, `415` for the wrong `Content-Type`. For `Path` and `Query`: `400`.
- The body extractor must be last, and `E0277` `Handler<_, _>` is how a mistake there shows up.
- In `axum` 0.8 path parameters are `/{id}`, and the old `/:id` panics at startup.
- `axum::serve` runs on `tokio` and spawns one task per connection, not one OS thread.
- You can test a whole router with `oneshot` and try it for real with `curl`.

### What comes back later

- **Writing your own extractor** — [3.2.2 — Writing your own extractor](../02-writing-your-own-extractor/README.md)
- **Choosing status codes and turning your own errors into responses, over a full CRUD resource** — [3.2.3 — Anime catalog CRUD (in-memory)](../03-anime-catalog-crud-in-memory/README.md)
- **What `.layer(...)` is underneath, and `tower::Service`** — [3.2.4 — `tower::Service` and `Layer`](../04-tower-service-and-layer-middleware/README.md)
- **`State` holding a database pool, and the lock-across-`.await` question** — [3.5.1 — Connecting and pooling](../../05-postgres-and-sqlx/01-connecting-and-pooling/README.md)
- **One JSON error shape for every rejection** — [3.8.1 — Consistent error envelopes](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)

### Can you explain?

- What does `axum` answer for a body that is not JSON, for valid JSON of the wrong shape, and for a missing `Content-Type`, and how does that connect to 3.1.3's `400` versus `422`?
- Why must `Json<T>` be the last parameter, and what does the error look like when it is not?
- When do you reach for `Path<T>` and when for `Query<T>`?
- Why does cloning `AppState` not give each request its own counter?
- What does `axum::serve` create per connection, and how does that differ from 3.1.1?
- How can a test send a request through the whole router without opening a port?

---

## Going further

- [`axum` docs: extractors](https://docs.rs/axum/0.8.9/axum/extract/index.html) — the full list, the order rules, and how to handle rejections yourself.
- [`axum` docs: debugging handler type errors](https://docs.rs/axum/0.8.9/axum/handler/index.html) — the `Handler` bound and `#[debug_handler]`.
- [`axum` 0.8 announcement](https://tokio.rs/blog/2025-01-01-announcing-axum-0-8-0) — why paths changed from `/:id` to `/{id}`.
- [`tower::ServiceExt::oneshot`](https://docs.rs/tower/0.5.3/tower/trait.ServiceExt.html#method.oneshot) — the method the tests use.
