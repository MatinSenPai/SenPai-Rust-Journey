# 02.1 — Routing, handlers, extractors

## Everything module 1 built by hand, now a library's job

Module 1 hand-rolled a `TcpListener` loop, a line-by-line request reader, and
a byte-level HTTP parser. `axum` (built on `tokio` and `hyper`) does all of
that for you — accepting connections, reading and parsing requests, matching
a path to the right function, and serializing your return value back into
an HTTP response. What's left for you to write is exactly the part Django
also leaves to you: **routes** and **views** (here, "handlers").

| Django / DRF | `axum` |
|---|---|
| `urls.py` — `path("greet/<name>/", views.greet)` | `Router::new().route("/greet/{name}", get(greet))` |
| a view function `def greet(request, name): ...` | an async handler `async fn greet(Path(name): Path<String>) -> String` |
| DRF serializer parsing `request.data` | an extractor argument like `Json<EchoRequest>` |
| `request.GET["q"]`, URL kwargs | extractor arguments like `Path<String>`, `Query<...>` |
| middleware (`MIDDLEWARE` list) | `tower::Layer`s wrapping the `Router` (module 6) |

## Handlers are async functions with typed parameters

```rust
async fn greet(Path(name): Path<String>) -> String {
    format!("Hello, {name}!")
}
```

Two things are new if you're coming from Django, and both matter:

1. **`async fn`.** Every `axum` handler runs on `tokio`'s async runtime
   (module 1's "why threads don't scale" discussion, resolved). Nothing in
   this lesson actually `.await`s anything *inside* a handler body yet —
   that shows up for real once a handler talks to a database (module 4) —
   but the function still has to be declared `async fn` because that's the
   trait `axum` requires a handler to implement.
2. **The parameter list is the request, destructured by type.** `axum`
   looks at each parameter's *type* and calls that type's `FromRequest`
   (or `FromRequestParts`) implementation to pull it out of the incoming
   request. `Path<String>` pulls a URL path segment. `Json<T>` parses (and
   validates, via `serde`) a JSON body into `T`. `State<S>` retrieves
   whatever shared state you attached to the `Router` (below). This is
   *type-directed* parsing — there's no `request.GET[...]`-style manual
   dictionary lookup; you declare what you want in the signature, and it's
   just there in the body, already parsed, or the request never reaches
   your function at all (see the next section).

## Extractors: `Path`, `Json`, `State`

- **`Path<T>`** — one or more `{name}` segments from the route pattern,
  parsed into `T` (a single `String`/`i64`/etc., or a tuple/struct for
  multiple segments). `Router::new().route("/greet/{name}", get(greet))`
  registers `{name}` as a path parameter; `Path(name): Path<String>` in the
  handler's signature pulls it out.
- **`Json<T>`** — deserializes the request body as JSON into `T` (`T` must
  implement `serde::Deserialize`), the same job `serializer.is_valid()` +
  `serializer.validated_data` does in DRF. Returning `Json<T>` from a
  handler does the mirror-image job: serializes `T` back to a JSON response
  body with the right `Content-Type` header set automatically.
- **`State<S>`** — retrieves a clone of whatever value you passed to
  `Router::with_state(s)` when building the router. This is how a handler
  reaches shared, long-lived state (a connection pool in module 4, an
  in-memory store in the next lesson) without global mutable variables —
  the same role Django's `apps.py` `AppConfig` + a database connection
  (managed by the ORM, mostly invisible to you) plays, made explicit here.

## Extractor order matters, and failure is automatic

`axum` runs extractors **in the order they appear in the function
signature**, and if *any* extractor fails (a `Path<i64>` segment that isn't
a valid number, a `Json<T>` body that doesn't deserialize into `T`), `axum`
short-circuits and returns an error response (400/422-ish) **before your
handler body ever runs** — you never see a "half-parsed" request. This is
stricter than DRF, where a serializer's `.is_valid()` is a step *you*
remember to call and branch on; here, invalid input can't reach your
function body by construction.

## Composing a `Router`

```rust
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/", get(hello))
        .route("/greet/{name}", get(greet))
        .route("/echo", post(echo))
        .route("/counter", get(get_counter))
        .route("/counter/increment", post(increment_counter))
        .with_state(state)
}
```

Each `.route(path, method_router)` call registers one path against one or
more HTTP methods (`get(handler)`, `post(handler)`, etc. — these wrap your
async function into something `axum::Router` can dispatch to). `.with_state`
attaches the shared `AppState` every `State<AppState>` extractor in your
handlers will receive a clone of.

## Testing without a real socket, again

Just like module 1's `run_echo` never opened a real `TcpListener` in its
tests, `axum::Router` implements `tower::Service`, which means you can drive
a full request through routing + extraction + your handler + serialization
with `tower::ServiceExt::oneshot(request)` — **no bound port, no real
network I/O at all**. `tests/routes_test.rs` does exactly this. This is the
same "push I/O to the edges, keep the core testable" idea from every
previous Phase 3 lesson, just now at the HTTP-framework layer instead of
the raw-socket layer.

## Your task

Implement the `todo!()`s in `src/lib.rs`:

- `greet` — use the extracted `Path<String>` to build a greeting.
- `echo` — use the extracted `Json<EchoRequest>` to build an `EchoResponse`.
- `get_counter` / `increment_counter` — use the extracted
  `State<AppState>` to read/mutate the shared counter.
- `app` — wire all five routes onto a `Router`, attaching `state`.

`src/main.rs` is provided — it builds an `AppState`, calls `app(state)`,
and serves it on `127.0.0.1:3000` with `tokio`.

## Try it for real

```sh
cargo run -p p3-02-01-routing-handlers-extractors &
curl http://127.0.0.1:3000/
curl http://127.0.0.1:3000/greet/senpai
curl -X POST -H 'content-type: application/json' -d '{"message":"hi"}' http://127.0.0.1:3000/echo
curl -X POST http://127.0.0.1:3000/counter/increment
curl http://127.0.0.1:3000/counter
```

## Next

`solution/SOLUTION.md` — but only after a real attempt.
