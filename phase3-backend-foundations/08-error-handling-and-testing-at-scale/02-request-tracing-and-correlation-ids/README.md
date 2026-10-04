# 3.8.2 — Request tracing and correlation IDs

## At a glance

After this lesson you can:

- Give every request one ID, taken from the caller's `x-request-id` header when it is safe and generated otherwise, and put it on the request extensions, on a `tracing` span, on the response header and in the error body.
- Read a log where many requests are interleaved, by filtering on one `request_id`, and explain why a `tokio::spawn` inside a handler drops the ID.
- Test log output: install a subscriber that writes into memory, and assert that the ID appears in the log line.
- Decide what to do with an incoming ID that is empty, malformed or absurdly long, and say why "trust the caller" is the wrong answer.
- Replace the hand-written middleware with `SetRequestIdLayer`, `TraceLayer` and `PropagateRequestIdLayer`, and name what the ready-made version leaves to you.

**Time:** ~100 minutes · **Prerequisites:**
[3.2.3 — Anime catalog CRUD (in-memory)](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.md),
[3.2.4 — `tower::Service` and `Layer`: middleware by hand](../../02-axum-and-rest-api-design/04-tower-service-and-layer-middleware/README.md),
[3.8.1 — Consistent error envelopes](../01-consistent-error-envelopes/README.md)

---

## Why this matters

At 03:00 a user writes: "the page broke at about three, I got an error." Your server handled four hundred requests in that minute, and every one of them logged a line like `loading from the database`. You have the time and nothing else to search for. Which of the four hundred was theirs?

A **correlation ID** (in this lesson, the request ID) answers that. The server gives each request one short string, writes it on every log line that request causes, and sends it back in a header and in the error body. The user pastes `request_id` from the error, you filter the log on it, and you see that one request's whole story: what came in, what the handler did, how it ended. A frontend developer gets the same handle for free: the ID is in the response they are already looking at.

In Django you reach for `django-guid` or write a logging filter, and both do the same two jobs: put an ID somewhere the log can see it, and echo it on the response. This lesson builds both jobs by hand on top of the middleware you wrote in [3.2.4](../../02-axum-and-rest-api-design/04-tower-service-and-layer-middleware/README.md), because the ID is not the hard part. The hard part is the rest: where the ID lives while a request runs through async code, what to do when the caller sends garbage, and how to test a log line.

---

## The concept

### Many requests, one log

`examples/01-logs-without-an-id.rs` sends two requests at once to a handler that logs two lines. This is the log:

```text
 INFO loading from the database
 INFO loading from the database
 WARN the query was slow
 WARN the query was slow
200 OK and 200 OK
```

Four lines, two requests, and nothing says which `WARN` belongs to which `INFO`. The handler's lines are fine on their own. What is missing is something that **every** line has in common with the others from its own request and in no other. (The subscriber here is `tracing_subscriber::fmt()` with ANSI colours, timestamps and the module path switched off, so the transcripts in this lesson stay readable and stable. The crate's `LogBuffer` below does the same.)

### One ID, five places

The plan has five steps, and the middleware does all of them, so no handler ever has to know about IDs:

```senpai-visual
{"kind":"network","labels":["request arrives, maybe with x-request-id","middleware: accept it or generate one","ID goes into the request extensions","handler runs inside a span that carries the ID","response gets x-request-id, error body gets request_id"]}
```

1. **Choose** the ID: the caller's if it is acceptable, otherwise a fresh UUID.
2. **Store** it in the request's *extensions*, the typed side-pocket of an `http::Request` (the `Parts` that a `FromRequestParts` extractor from [3.2.2](../../02-axum-and-rest-api-design/02-writing-your-own-extractor/README.md) receives carry it), so a handler can ask for it.
3. **Tag** the work with it: open a span with the ID as a field, and run the rest of the request inside that span.
4. **Echo** it: set `x-request-id` on the response.
5. **Embed** it in the error body, next to the code and message.

Steps 1, 2 and 4 are plain Rust. Step 3 is new, so it comes first.

### Events and spans

`tracing` has two things you record. An **event** is something that happened at one moment: `tracing::info!("loading from the database")`. It is what you used to call a log line. A **span** is a stretch of time with a name and fields: it has a start and an end, and any event recorded while the span is *entered* is stamped with the span's fields. A **subscriber** receives both and decides what to do with them; `tracing_subscriber::fmt()` prints them.

The middleware in `examples/02-id-in-the-span.rs` opens one span per request and runs the rest of the app inside it:

```rust
async fn with_request_id(request: Request, next: Next) -> Response {
    let id = request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("none")
        .to_string();
    let span = tracing::info_span!("request", request_id = %id);
    let mut response = next.run(request).instrument(span).await;
    response
        .headers_mut()
        .insert("x-request-id", HeaderValue::from_str(&id).unwrap());
    response
}
```

The handler is the same as in example 01. It has no ID in it. Two requests with the IDs `req-a` and `req-b`:

```text
 INFO request{request_id=req-a}: loading from the database
 INFO request{request_id=req-b}: loading from the database
 WARN request{request_id=req-b}: the query was slow
 WARN request{request_id=req-a}: the query was slow
echoed: "req-a" "req-b"
```

(The order of the two `WARN` lines can differ between runs, which is exactly the situation the ID is for.) Every line now starts with `request{request_id=...}:`, the span's name and fields, and the handler did nothing to earn it.

Two details are worth reading slowly. `request_id = %id` records the field with `Display`, so it prints bare. A plain `request_id = id` on a `&str` is recorded with `Debug` and prints in quotes, which you will see in `examples/05-ready-made-layers.rs`. And `.instrument(span)` is the whole trick: it wraps a **future** so that the span is entered every time the future is polled and exited when the poll returns. It is attached to the future, not to the thread, and that is why it survives `.await` points even when `tokio` moves the task to another thread.

```senpai-visual
{"kind":"async","labels":["future is polled: span entered","handler hits .await: span exited","task may move to another thread","future polled again: span entered again","every event in between carries request_id"]}
```

Where the Django picture stops being exact: `django-guid` and a logging filter keep the ID in a `contextvars.ContextVar`, which Python copies into every `asyncio` task you create. `tracing` does not copy anything for you into a task created with `tokio::spawn`. "Errors you will meet" shows the line that loses its ID.

### The ID is user input

Example 02 trusted the header. `unwrap_or("none")` is not even a plan: every request without a header shares the ID `none`, which defeats the point. And the last request of the same example shows what trusting the header costs:

```text
--- a caller who sends a crafted id
 INFO request{request_id=x level=ERROR forged=true}: loading from the database
 WARN request{request_id=x level=ERROR forged=true}: the query was slow
```

The caller chose a header value with spaces and `=` signs in it, and our log now has fields we did not write. Whoever reads this log, a person or a tool that splits on spaces, sees `level=ERROR forged=true`. This is **log injection**, and the header is the door. The fix has three parts, and they are the rules of this lesson:

- **Absent:** generate an ID. A UUID v4 (`67e55044-10b1-426f-9247-bb680e5fe0c8`) is 36 characters and needs no coordination between servers.
- **Acceptable:** keep it. A gateway or a frontend may have set it, and keeping it is the entire point of a *correlation* ID: one ID across several systems.
- **Anything else: replace it, do not repair it.** Acceptable means 1 to 64 bytes of ASCII letters, digits, `-`, `_` and `.`. Longer, empty, or any other character, and the server ignores the caller's value and generates its own. It does not trim, truncate or strip characters: a repaired ID might collide with someone else's, and silently changing what the caller sent is a surprise.

The limit is not arbitrary. An ID is copied into every log line of the request, into a response header and into a body. 5,000 bytes of ID repeated in each of twenty lines is 100 kilobytes of log for one request, and the caller chooses the size. `src/lib.rs` has `is_valid_request_id`, `resolve_request_id` and `error_body` as the first exercise, because these three decisions are pure functions you can test without a server.

`examples/03-server-with-an-id.rs` is a real server on `127.0.0.1:3230` with the same middleware plus the check. Start it in one terminal:

```sh
cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 03-server-with-an-id
```

```text
listening on http://127.0.0.1:3230
```

Then, in another, send an ID, no ID, and a malformed one (`curl -i` prints the response headers):

```sh
curl -i -H 'x-request-id: abc-123' http://127.0.0.1:3230/anime/1
curl -i http://127.0.0.1:3230/anime/1
curl -i -H 'x-request-id: not valid!' http://127.0.0.1:3230/anime/1
```

```text
HTTP/1.1 200 OK
content-type: application/json
x-request-id: abc-123
content-length: 31
date: Sun, 04 Oct 2026 11:13:46 GMT

{"id":1,"title":"Cowboy Bebop"}
HTTP/1.1 200 OK
content-type: application/json
x-request-id: 7b2effea-0bf6-4cb9-aa49-14543468845a
content-length: 31
date: Sun, 04 Oct 2026 11:13:46 GMT

{"id":1,"title":"Cowboy Bebop"}
HTTP/1.1 200 OK
content-type: application/json
x-request-id: 7d1d0ab0-02c5-4523-8d8a-3a746090d303
content-length: 31
date: Sun, 04 Oct 2026 11:13:46 GMT

{"id":1,"title":"Cowboy Bebop"}
```

(The generated IDs and the `date` header change on every run.) The valid ID comes back unchanged. The other two get a fresh UUID. A request with a 5,000-character ID (`-H "x-request-id: $(printf 'a%.0s' $(seq 1 5000))"`) gets one too. The server's own log, which is what you filter when a user hands you an ID:

```text
 INFO request{request_id=abc-123 method=GET path=/anime/1}: looking up anime id=1
 INFO request{request_id=7b2effea-0bf6-4cb9-aa49-14543468845a method=GET path=/anime/1}: looking up anime id=1
 WARN ignoring a malformed x-request-id
 INFO request{request_id=7d1d0ab0-02c5-4523-8d8a-3a746090d303 method=GET path=/anime/1}: looking up anime id=1
```

Notice the `WARN`: it has no `request{...}` prefix. At that moment the middleware had not yet chosen an ID, so no span was open. An event outside the span carries no ID, so anything you want to find by ID has to be logged inside it. Stop the server with Ctrl+C when you are done.

### The ID in the request extensions

The middleware also does `request.extensions_mut().insert(RequestId(id.clone()))`, where `RequestId` is a one-field struct in `src/lib.rs`. A handler asks for it with the `Extension` extractor:

```rust
async fn whoami(Extension(id): Extension<RequestId>) -> String {
    id.0
}
```

```text
c1c82f17-6d11-4ac2-ba5e-e6f3511ea514
```

That is the body of `curl http://127.0.0.1:3230/whoami`. Most handlers never need this, since the span already tags their log lines. You need the extension when the ID has to go somewhere that is not a log line: into a call to another service (so *their* log carries the same ID), into a job you queue, or into a response body. The type is `RequestId`, not `String`, because extensions are keyed by type, and a bare `String` in there would collide with anything else that stores one.

### The ID in the error body

A user can copy `x-request-id` out of the browser's network tab, but most people never open it. They read the error on the page. So the ID also goes in the body, in the error envelope shape [3.8.1](../01-consistent-error-envelopes/README.md) is about:

```text
{"error":{"code":"not_found","message":"anime 99 not found","request_id":"abc-404"}}
```

A handler returns `Err(ApiError::not_found(...))` and has no ID to put in it. Rather than pass the ID to every handler, the lesson's `ApiError` renders the plain body and leaves its code and message on the response's own extensions (`ErrorInfo`). The middleware, which knows the ID, finds that, and rewrites the body with `error_body(...)`. The server above stops short of that: its `404` still looks like this, because the Build exercise is where you add the rewrite:

```text
{"error":{"code":"not_found","message":"anime 99 not found"}}
```

Log the failure too: one `WARN` event for a 4xx with the error `code`, one `ERROR` for a 5xx, both inside the span. Then "the user pasted `req-9f2`" turns into one filter on the log, and the 4xx-versus-5xx level makes the 5xx stand out when you scan.

### Testing a log line

Everything above prints to a terminal, and a test cannot read a terminal. `tracing` lets the program pick where events go by choosing a subscriber, and `tracing_subscriber::fmt` lets that subscriber write to anything that implements `MakeWriter`: a trait with one method, `make_writer`, that hands out a fresh `io::Write` every time an event is recorded. `LogBuffer` in `src/lib.rs` is given, and it is small: a `Clone`-able handle on an `Arc<Mutex<Vec<u8>>>`, which implements both `io::Write` (push the bytes) and `MakeWriter` (return a clone of itself). `examples/04-capture-the-log.rs` uses it:

```rust
let buffer = LogBuffer::new();
let guard = tracing::subscriber::set_default(buffer.subscriber());

let id = "abc-123";
let span = tracing::info_span!("request", request_id = %id);
span.in_scope(|| tracing::info!(status = 200, "finished"));
drop(guard);

println!("{:?}", buffer.contents());
println!("contains the id: {}", buffer.contents().contains("request_id=abc-123"));
```

```text
" INFO request{request_id=abc-123}: finished status=200\n"
contains the id: true
```

`set_default` installs the subscriber **for the current thread only** and returns a guard that removes it again when dropped. That is what makes it safe in tests, which `cargo test` runs in parallel on separate threads: each test has its own buffer, and a `#[tokio::test]` runs on a single thread, so one guard covers the whole request. The global `.init()` you used in the examples can be called once per process, which is right for `main` and wrong for a test suite.

### The ready-made version

You have built what `tower-http` ships as three layers, `SetRequestIdLayer`, `TraceLayer` and `PropagateRequestIdLayer` (resolved here as `tower-http` 0.6.11, with the `request-id` and `trace` features on in this lesson's `Cargo.toml`). `examples/05-ready-made-layers.rs` stacks them with `ServiceBuilder`, which in [3.2.4](../../02-axum-and-rest-api-design/04-tower-service-and-layer-middleware/README.md) listed the outermost layer first:

```rust
ServiceBuilder::new()
    .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
    .layer(trace)
    .layer(PropagateRequestIdLayer::x_request_id())
```

`SetRequestIdLayer` fills in the header from a generator (`MakeRequestUuid`) when the request has none. `TraceLayer` opens a span per request and logs "started" and "finished" events; the `trace` above is a `TraceLayer` that the example configures to copy the header into a `request_id` field of its span. `PropagateRequestIdLayer` copies the header from the request to the response. Three requests, the same ones as before:

```text
 INFO request{request_id="abc-123" path=/anime}: started processing request
 INFO request{request_id="abc-123" path=/anime}: finished processing request latency=0 ms status=200
sent Some("abc-123") -> echoed "abc-123"
 INFO request{request_id="96e7832f-e45e-4386-bee1-9514143f7605" path=/anime}: started processing request
 INFO request{request_id="96e7832f-e45e-4386-bee1-9514143f7605" path=/anime}: finished processing request latency=0 ms status=200
sent None -> echoed "96e7832f-e45e-4386-bee1-9514143f7605"
 INFO request{request_id="not valid!" path=/anime}: started processing request
 INFO request{request_id="not valid!" path=/anime}: finished processing request latency=0 ms status=200
sent Some("not valid!") -> echoed "not valid!"
```

The generated ID, the span, the latency and the echo came for free, and the last request shows what did not: `not valid!` went straight through, into the log and onto the response. `SetRequestIdLayer` leaves an existing header alone, whatever it contains. So the ready-made layers cover steps 1 (partly), 3 and 4, and your code still has to do the rest: validate or replace an incoming ID (the log-injection hole is open), put a typed `RequestId` in the extensions for handlers, and write the ID into the error body. The two versions are not rivals. A real service often uses the ready-made layers and adds one small middleware of its own for the checks, which is the **Challenge** at the end.

(`TraceLayer`'s defaults log at `DEBUG`, so with a default subscriber you see nothing. The example sets the levels to `INFO` to get the lines above, and this is a common reason for "I added `TraceLayer` and my logs are empty".)

### `x-request-id` is a convention

`x-request-id` is not in any standard. Many proxies and clouds use it, which is what makes it useful as the thing a gateway sets and your service keeps. For tracing across many services with timing, there is a standard, W3C Trace Context (the `traceparent` header), and OpenTelemetry builds on it. A request ID is the small, 1-header version of that idea, and everything you learn here (a span that carries the ID, a middleware that creates it, a log you can filter) carries over.

---

## Hands on

```sh
cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 01-logs-without-an-id
cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 02-id-in-the-span
cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 04-capture-the-log
cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 05-ready-made-layers
```

Then the server on port 3230, with `curl` from a second terminal, as in "The ID is user input":

```sh
cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 03-server-with-an-id
```

And the three that go wrong. `06` and `08` compile and run, and are wrong at run time. `07` does not compile:

```sh
cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 06-spawn-loses-the-id
cargo build -p p3-08-02-request-tracing-and-correlation-ids --example 07-enter-across-await-broken --features broken
cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 08-extension-without-middleware
```

Then try these:

1. In `02-id-in-the-span`, add a third concurrent request with the ID `req-c`. How many lines mention `req-c`?
2. With the server from `03` running, send `-H 'x-request-id: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'` (64 `a`s) and then 65. Which is kept?
3. In `05-ready-made-layers`, remove `.on_response(...)` and run again. Which lines are gone, and why?

---

## Errors you will meet

### `E0277` — the span guard that is not `Send`

`examples/07-enter-across-await-broken.rs` keeps a span entered with `let _guard = span.entered();` and then awaits. The output, with this lesson's `todo!()` warnings left out:

```text
error[E0277]: the trait bound `FromFn<..., (), ..., _>: Service<...>` is not satisfied
   --> phase3-backend-foundations\08-error-handling-and-testing-at-scale\02-request-tracing-and-correlation-ids\examples\07-enter-across-await-broken.rs:23:16
    |
 23 |         .layer(from_fn(with_request_id));
    |          ----- ^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
    |          |
    |          required by a bound introduced by this call
    |
    = help: the trait `tower_service::Service<axum::http::Request<Body>>` is not implemented for `FromFn<fn(Request<Body>, Next) -> ... {with_request_id}, (), ..., _>`
    = help: the following other types implement trait `tower_service::Service<Request>`:
              axum::middleware::FromFn<F, S, I, (T1, T2)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8, T9)>
            and 8 others
note: required by a bound in `Router::<S>::layer`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:306:21
    |
303 |     pub fn layer<L>(self, layer: L) -> Router<S>
    |            ----- required by a bound in this associated function
...
306 |         L::Service: Service<Request> + Clone + Send + Sync + 'static,
    |                     ^^^^^^^^^^^^^^^^ required by this bound in `Router::<S>::layer`
    = note: the full name for the type has been written to 'M:\SenPai-Rust-Journey\target\debug\examples\07_enter_across_await_broken.long-type-9151973810204884499.txt'
    = note: consider using `--verbose` to print the full type name to the console

error[E0277]: the trait bound `FromFn<..., (), ..., _>: Service<...>` is not satisfied
  --> phase3-backend-foundations\08-error-handling-and-testing-at-scale\02-request-tracing-and-correlation-ids\examples\07-enter-across-await-broken.rs:21:15
   |
21 |       let app = Router::new()
   |  _______________^
22 | |         .route("/", get(|| async { "ok" }))
23 | |         .layer(from_fn(with_request_id));
   | |________________________________________^ unsatisfied trait bound
   |
   = help: the trait `tower_service::Service<axum::http::Request<Body>>` is not implemented for `FromFn<fn(Request<Body>, Next) -> ... {with_request_id}, (), ..., _>`
   = help: the following other types implement trait `tower_service::Service<Request>`:
             axum::middleware::FromFn<F, S, I, (T1, T2)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8, T9)>
           and 8 others
   = note: the full name for the type has been written to 'M:\SenPai-Rust-Journey\target\debug\examples\07_enter_across_await_broken.long-type-9151973810204884499.txt'
   = note: consider using `--verbose` to print the full type name to the console

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-08-02-request-tracing-and-correlation-ids` (example "07-enter-across-await-broken") due to 2 previous errors
```

(The number in the `long-type-` file name differs on your machine, and on every build.)

**What the compiler is objecting to:** the message never says `Send`, and that is what makes it hard. It says that `FromFn<...>` does not implement `Service`, with a list of look-alikes, and that the call to `.layer` required it. The reason is one step removed: `from_fn` only builds a `Service` if the middleware's future is `Send`, and a future that holds a value across an `.await` is only `Send` if the value is. `span.entered()` returns an `EnteredSpan`, which `tracing` makes `!Send` on purpose (in `tracing` 0.1.44's source it carries a `PhantomNotSend` field). Two errors are reported, one for the `.layer` call and one for the `Router::new()` expression that contains it, and both are the same mistake.

**The fix:** do not hold a span guard across an await. Wrap the future instead:

```rust
let span = tracing::info_span!("request", request_id = "abc-123");
next.run(request).instrument(span).await
```

**Why this is the fix:** a guard enters the span on *this thread* and leaves it when dropped. Across an `.await` the task can be moved to a thread that never entered the span, or other tasks can run on this thread while the guard is still entered, so events would be stamped with the wrong request's ID. `tracing` makes the guard `!Send` so that the compiler stops you. `.instrument(span)` enters and exits around each poll of the future, as in "Events and spans". (A plain `span.enter()` does compile, because its guard borrows the span and is `Send`, and it is wrong in the same way. It is the harder one to notice, because nothing stops you.)

### A silent bug: a spawned task has no span

```text
 INFO request{request_id="abc-123"}: handler: queueing the welcome email
 INFO worker: sending the welcome email
```

**What's actually broken:** `examples/06-spawn-loses-the-id.rs` compiles and runs without complaint. The handler's line carries the ID. The line from the `tokio::spawn`ed task does not, because a spawned task starts with no span: it is a new future that nothing instrumented. If you filter this log on `abc-123`, you see half of what happened, and the half you are missing is usually the part that failed.

**The fix:** instrument the spawned future with the current span:

```rust
let worker = tokio::spawn(
    async { tracing::info!("worker: sending the welcome email") }
        .instrument(tracing::Span::current()),
);
```

**Why this is the fix:** `Span::current()` is the span the handler is running in at this moment, and `.instrument(...)` makes the new task enter it on every poll, just as the middleware does for the whole request. With this change the example prints `INFO request{request_id="abc-123"}: worker: sending the welcome email`. A span can also outlive its request this way (a background task holds it open), which is fine for a short job and worth knowing for a long one. In Python, `asyncio.create_task` copies the current `contextvars` context into the new task, so the same mistake would not lose the ID there. This is one of the places the comparison breaks.

### A run-time `500`: the extension is missing

```text
status: 500 Internal Server Error
body: Missing request extension: Extension of type `p3_08_02_request_tracing_and_correlation_ids::RequestId` was not found. Perhaps you forgot to add it? See `axum::Extension`.
```

**What's actually broken:** `examples/08-extension-without-middleware.rs` serves `/whoami`, whose handler asks for `Extension<RequestId>`, from a router that was built without the middleware that inserts one. The types all line up, so the compiler is satisfied. `axum` answers every such request with `500` and this message, which is accurate, and the failure happens per request, not at startup.

**The fix:** serve the routes through the middleware, as `app()` does: `routes().layer(from_fn(request_id_middleware))`. (That works once your Build exercise is done; until then `request_id_middleware` is a `todo!()`. The server in example 03 has a working copy.) If a handler can run both with and without the middleware (in a test, say), take `Option<Extension<RequestId>>` instead and decide what the missing case means.

**Why this is the fix:** an extension is a promise made by something earlier in the pipeline, and extractors cannot check promises at compile time. The `500` is `axum` doing its job: the handler asked for something that was never put there. A test of `app()` that sends one request to every route that uses `Extension<RequestId>` catches this before a user does.

---

## Exercises

### Warm up

<details>
<summary>A user reports an error and gives you the <code>request_id</code> from the response body. What do you do with it, and why does it work?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

Filter the server log on that ID. It works because the span carries `request_id` and every event recorded inside the span, from the middleware, the handler and the code the handler calls, is stamped with it. One filter gives you that request's lines and nobody else's.

</details>

<details>
<summary>A caller sends <code>x-request-id: ../../etc/passwd</code>. Does the server keep it, and what does it do instead?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

It does not keep it: `/` is outside the allowed characters (letters, digits, `-`, `_`, `.`). The server generates a new UUID and uses that on the span, in the extensions, on the response and in the error body. It does not strip the bad characters and keep the rest: a replaced ID is never a guess about what the caller meant.

</details>

<details>
<summary>Why does a test use <code>tracing::subscriber::set_default</code> rather than <code>.init()</code>?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`.init()` installs a subscriber for the whole process, and only the first call can succeed, but `cargo test` runs many tests at once. `set_default` installs one for the current thread and undoes it when its guard is dropped, so each test gets its own buffer and cannot see another test's lines.

</details>

<details>
<summary>The handler calls <code>tokio::spawn</code> and the spawned task logs. Does that line carry <code>request_id</code>?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No. A new task starts with no span. Wrap its future with `.instrument(tracing::Span::current())` to give it the handler's span.

</details>

### Repair

Fix two of the examples:

1. `examples/06-spawn-loses-the-id.rs` prints the worker's line with `request_id="abc-123"` in front.
2. `examples/07-enter-across-await-broken.rs` compiles (run it with `--features broken`) and still tags the request's log lines with the ID.

### Implement

Three small functions in `src/lib.rs`, in `tests/resolve_test.rs`: `is_valid_request_id`, `resolve_request_id` and `error_body`.

```sh
cargo test -p p3-08-02-request-tracing-and-correlation-ids --test resolve_test
```

The doc comment above each one is the whole specification: the exact character set and length limit, which input wins, when the generator is called and when it is not, and the exact JSON shape. You never need to read the tests. They are pure functions, so they need no server and no runtime.

### Build

`request_id_middleware` in `src/lib.rs`: the full middleware, in the shape of `Log` and `from_fn` from [3.2.4](../../02-axum-and-rest-api-design/04-tower-service-and-layer-middleware/README.md). It chooses the ID, stores it in the extensions, runs the app in a `request` span, logs `finished`, sets the response header, and rewrites an error response's body with the ID.

```sh
cargo test -p p3-08-02-request-tracing-and-correlation-ids --test middleware_test
```

Again the doc comment is the specification, step by step. The tests send requests with `oneshot`, as since 3.2.1, and read the log back through `LogBuffer`: one asserts the ID is in the handler's own line, one that a hostile ID never reaches the log, one that a 404 is logged at `WARN` and a 500 at `ERROR`. When it is green, run `examples/03-server-with-an-id.rs` with your own middleware in place of its copy and send the `404` request again.

### Challenge (optional)

Rebuild the same behaviour from the `tower-http` layers of `examples/05-ready-made-layers.rs`, plus **one** small `from_fn` middleware of your own that sits between `SetRequestIdLayer` and the rest and replaces an invalid incoming ID. Write down which of the five steps the ready-made layers took over and which one you could not give away. Nothing tests this one. Check it with `oneshot` requests of your own: the ID in the echoed header must be the ID in the log. If they differ, which layer order explains it?

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| correlation ID | one string that follows a request through every log line, header and error body | finding one request in a busy log |
| `x-request-id` | the conventional header that carries it, in and out | gateways, frontends, other services |
| event / span | an event is one moment; a span is a named stretch of time with fields | `tracing::info!`, `info_span!` |
| `.instrument(span)` | enters the span on every poll of a future | middleware, spawned tasks |
| subscriber | receives events and spans and decides where they go | `tracing_subscriber::fmt()` |
| `MakeWriter` | the trait that tells `fmt` where to write | an in-memory buffer for tests |
| log injection | a caller-chosen value that forges fields in a log line | validating incoming IDs |
| request extensions | the typed side-pocket on a request | `Extension<RequestId>` |
| `SetRequestIdLayer` etc. | the ready-made `tower-http` layers | production, with your own validation |

### What you now know

- A request ID is accepted when it is safe, generated when it is absent, and replaced (never repaired) when it is empty, malformed or longer than 64 bytes.
- The middleware stores it in the extensions, runs the request inside a span that carries it, echoes it, and writes it into the error body.
- A span follows a future, not a thread: `.instrument` is how, and `tokio::spawn` is where it is lost.
- `set_default` plus a `MakeWriter` over `Arc<Mutex<Vec<u8>>>` lets a test read its own log.
- `tower-http` ships the generating, tracing and echoing parts, and does not validate an incoming ID.

### What comes back later

- **Tests that drive the app over real storage** — [3.8.3 — Integration tests with `testcontainers`](../03-integration-tests-with-testcontainers/README.md)
- **Test data you do not write by hand** — [3.8.4 — Test data factories and fixtures](../04-test-data-factories-and-fixtures/README.md)

### Can you explain?

- Why does a user's `request_id` find their request in the log, and what makes a line carry it?
- What are the three things a server can do with an incoming `x-request-id`, and why is "keep it as it is" not one of them?
- Why is a span attached to a future and not to a thread, and what does `tokio::spawn` do to it?
- How does a test read a log line, and why does it use `set_default` instead of `.init()`?
- Which parts of this lesson do `SetRequestIdLayer`, `TraceLayer` and `PropagateRequestIdLayer` give you, and which parts are still yours?
- Where does `contextvars` in Django (`django-guid`) behave like a span, and where does it not?

---

## Going further

- [`tracing` span docs](https://docs.rs/tracing/0.1.44/tracing/span/index.html): entering, exiting, `Instrument` and why guards are `!Send` across awaits.
- [`tracing_subscriber::fmt::MakeWriter`](https://docs.rs/tracing-subscriber/0.3.23/tracing_subscriber/fmt/trait.MakeWriter.html): the trait this lesson implements for the in-memory buffer.
- [`tower_http::request_id`](https://docs.rs/tower-http/0.6.11/tower_http/request_id/index.html): the three request-ID layers and their ordering rules.
- [`tower_http::trace`](https://docs.rs/tower-http/0.6.11/tower_http/trace/index.html): `TraceLayer`, its default levels and the callbacks you can replace.
- [W3C Trace Context](https://www.w3.org/TR/trace-context/): the standard header (`traceparent`) for tracing across services, the next step after a request ID.
- [`django-guid`](https://github.com/snok/django-guid): the Django package that the comparison in this lesson is about; read its README for the settings it exposes.
