# 3.2.4 — `tower::Service` and `Layer`: middleware by hand

## At a glance

After this lesson you can:

- Say what `poll_ready` and `call` each promise, and explain why `.oneshot(...)` worked on a `Router` in [3.2.1](../01-routing-handlers-extractors/README.md): a `Router` is a `tower::Service`.
- Write a `Layer` and its `Service` by hand, attach them with `.layer(...)`, and read the three `E0277` errors `axum` gives when the pair breaks its rules.
- Predict which of two layers sees the request first, both with repeated `.layer(...)` calls and with `ServiceBuilder`.
- Choose between a hand-written `Service` and `axum::middleware::from_fn` for a given piece of middleware.

**Time:** ~90 minutes · **Prerequisites:**
[3.2.1 — Routing, handlers, extractors](../01-routing-handlers-extractors/README.md),
[3.2.3 — Anime catalog CRUD (in-memory)](../03-anime-catalog-crud-in-memory/README.md),
[2.8.5 — Futures and runtimes](../../../phase2-intermediate/08-concurrency/05-futures-and-runtimes/README.md)

---

## Why this matters

You have written `.oneshot(request)` in every test since 3.2.1 without asking where it came from. It is not a `Router` method. It comes from a trait, and the `Router` is just one type that implements it. The same trait is behind `.layer(...)`, which is the call that adds logging, timeouts, authentication and CORS to an `axum` app.

In Django you already know the idea: `MIDDLEWARE` is a list, and each entry wraps the one after it. You never wrote the wrapping yourself, because a middleware was one small function. In `axum` the next lesson, [3.2.5 — CORS and frontend integration](../05-cors-and-frontend-integration/README.md), hands you `CorsLayer`, a finished layer from `tower-http`. If you use it without knowing what a layer is, `.layer(CorsLayer::permissive())` is one more incantation. This lesson builds a layer from nothing first, so that when a layer misbehaves, or when [3.8.2 — Request tracing and correlation IDs](../../08-error-handling-and-testing-at-scale/02-request-tracing-and-correlation-ids/README.md) asks you to write one that tags every request, you know what is under the call.

---

## The concept

### `oneshot` is `poll_ready` plus `call`

`examples/01-router-is-a-service.rs` sends one request to a `Router` twice. The first time it drives the router by hand. The second time it uses `oneshot`:

```rust
let ready = ServiceExt::<Request<Body>>::ready(&mut app);
let response = ready.await.unwrap().call(request).await.unwrap();
println!("by hand:  {}", response.status());

let response = app.oneshot(request).await.unwrap();
println!("oneshot:  {}", response.status());
```

```text
by hand:  200 OK
oneshot:  200 OK
```

Two steps, `ready` and then `call`, and `oneshot` is both of them in one method. Both come from `tower`'s `Service` trait. This is the real definition, from `tower-service` 0.3.3:

```rust
pub trait Service<Request> {
    type Response;
    type Error;
    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    fn call(&mut self, req: Request) -> Self::Future;
}
```

Read it with what Phase 2 gave you. `Response`, `Error` and `Future` are **associated types** ([2.3.5](../../../phase2-intermediate/03-traits-and-generics/05-associated-types/README.md)): one implementation fills each in once, the way `Iterator` fills in `Item`. `call` does not return a response. It returns a `Future` ([2.8.5](../../../phase2-intermediate/08-concurrency/05-futures-and-runtimes/README.md)), and the response exists once something polls that future to `Ready`. `Request` is a generic parameter on the trait, not an associated type, because one service can accept more than one request type.

A **service** is therefore anything that turns a request into a future response. That is what a `Router` does, and it is also all a handler is, once `axum` has wrapped it.

### `poll_ready`: can you take a request now?

`call` takes `&mut self` and never says "I'm busy". The question "may I send you a request right now?" is a separate method, `poll_ready`, and it is how a service says no. It returns `Poll::Pending` while it cannot take a request, the same `Pending` you met in `Future::poll`. `examples/02-poll-ready-backpressure.rs` wraps a trivial service in `tower`'s `ConcurrencyLimit` with a limit of one, sends a request without finishing it, and asks for readiness again:

```rust
let first = svc.ready().await.unwrap().call(1);
println!("first request is in flight (its future is not awaited yet)");

let second = tokio::time::timeout(Duration::from_millis(50), svc.ready()).await;
// ... prints "still waiting" when the timeout fires ...
println!("first request finishes: {:?}", first.await.unwrap());
svc.ready().await.unwrap();
println!("now ready again");
```

```text
first request is in flight (its future is not awaited yet)
second ready() after 50 ms: still waiting
first request finishes: 1
now ready again
```

This is **backpressure** ([2.8.3](../../../phase2-intermediate/08-concurrency/03-channels-message-passing/README.md) met it as a full bounded channel): the service tells the caller to wait, instead of piling up work or failing. The contract is that a caller must see `Ready` from `poll_ready` before each `call`, and a service is allowed to panic if it does not. You will meet that panic in "Errors you will meet".

A `Router` answers `Ready` every time, because `axum` handlers are always ready. The `axum` docs say so directly: `axum` expects every service in your app not to care about backpressure. That is why this lesson's middleware can forward `poll_ready` and never think about it again.

```senpai-visual
{"kind":"async","labels":["caller: poll_ready","service: Pending, not yet","service wakes the caller","poll_ready: Ready(Ok)","caller: call(request)","service returns a Future, polled to Ready(response)"]}
```

### A `Layer` is a function from one service to another

A **middleware** wraps a service and is a service itself, so wrappers stack. Django's version is a function that takes `get_response` and returns a new callable. Here is a timing middleware in the Django shape:

```python
def timing_middleware(get_response):
    def middleware(request):
        start = time.monotonic()
        response = get_response(request)
        elapsed = int((time.monotonic() - start) * 1000)
        response["X-Response-Time-Ms"] = str(elapsed)
        return response
    return middleware
```

Illustrative Python, not part of this crate. It has two layers of function. The outer `timing_middleware` runs once, at startup, and receives the thing to wrap. The inner `middleware` runs for every request and calls the thing it wrapped. `tower` gives each of those a name and a trait. The outer function is a **`Layer`**:

```rust
pub trait Layer<S> {
    type Service;

    fn layer(&self, inner: S) -> Self::Service;
}
```

The inner function is the `Service` that `layer` returns, with `inner` stored in a field the way Python's closure holds `get_response`. A `Layer` is a factory with configuration, and each service it builds holds one wrapped `inner`. That is why `Router::layer(...)` takes a layer rather than a service: it wraps every route in its own copy.

Where the Django picture stops being exact: a Django middleware is one function that does both jobs, and its `get_response` is synchronous. In `tower` the two jobs are split into two types. Everything in the inner function is asynchronous, so `call` returns a future. And Python has no `poll_ready`, because a WSGI callable has no way to say "not now".

### The same thing, by hand: `Log`

`src/lib.rs` contains a finished layer, `LogLayer`, that prints a line when a request goes in and a line when its response comes out. The layer is the short part:

```rust
#[derive(Clone)]
pub struct LogLayer {
    name: &'static str,
}

impl<S> Layer<S> for LogLayer {
    type Service = Log<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Log { inner, name: self.name }
    }
}
```

The service is the long part. First the struct the layer builds, which holds `inner` the way Python's closure holds `get_response`:

```rust
#[derive(Clone)]
pub struct Log<S> {
    inner: S,
    name: &'static str,
}
```

Then its `Service` impl, starting with the types and `poll_ready`:

```rust
impl<S> Service<Request> for Log<S>
where
    S: Service<Request, Response = Response>,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), S::Error>> {
        self.inner.poll_ready(cx)
    }
```

`poll_ready` forwards: a wrapper is ready exactly when what it wraps is ready. `Error = S::Error` means the wrapper adds no new way to fail. Then `call`:

```rust
    fn call(&mut self, request: Request) -> Self::Future {
        let name = self.name;
        println!("{name}: request in  ({} {})", request.method(), request.uri());
        let future = self.inner.call(request);
        Box::pin(async move {
            let response = future.await?;
            println!("{name}: response out ({})", response.status());
            Ok(response)
        })
    }
}
```

Nothing here ran yet: `call` printed "request in" at once, handed the request down, and got back the inner future. What it returns is a new future that waits for the inner one and then prints "response out". `examples/03-log-layers-onion.rs` uses it:

```text
B: request in  (GET /)
A: request in  (GET /)
A: response out (200 OK)
B: response out (200 OK)
```

### Why the future is boxed

`type Future = Pin<Box<dyn Future<Output = ...> + Send>>` is the part that looks heavy. The reason is that the `async` block in `call` has a type the compiler invents and nobody can write down, exactly like a closure's ([2.3.7](../../../phase2-intermediate/03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md)). An associated type needs a name. There are two ways to get one.

You can write a named future struct, with its own `poll` and pinning ([2.8.5](../../../phase2-intermediate/08-concurrency/05-futures-and-runtimes/README.md) did this for `FlipOnce`). That costs no allocation and a lot of code. Or you can put the future behind a trait object, `Pin<Box<dyn Future<...>>>`: the type erasure from 2.3.7, with a single heap allocation per request and an easy `Box::pin(async move { ... })`. This lesson boxes. The bounds follow from where the future lives. `+ Send` is there because a multi-threaded `tokio` runtime moves a task between threads between polls ([2.8.4](../../../phase2-intermediate/08-concurrency/04-send-and-sync/README.md)). `S::Future: Send + 'static` is there because the inner future is moved into the box.

### What `.layer(...)` requires

`axum` 0.8.9's `Router::layer` has this signature (copied from the resolved source, `src/routing/mod.rs`):

```rust
pub fn layer<L>(self, layer: L) -> Router<S>
where
    L: Layer<Route> + Clone + Send + Sync + 'static,
    L::Service: Service<Request> + Clone + Send + Sync + 'static,
    <L::Service as Service<Request>>::Response: IntoResponse + 'static,
    <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
    <L::Service as Service<Request>>::Future: Send + 'static,
```

Every line is a rule you can break, and in "Errors you will meet" you break three of them. `Clone` is there because `axum` clones services: one per route, and one per connection. `Error: Into<Infallible>` means a middleware that can fail is not allowed here: it must turn every failure into a response, because `hyper` would otherwise close the connection without answering. The `Router` side of the picture is the same trait: `Route`, the type `L` wraps, is itself a `Service<Request, Error = Infallible>`.

### The clone trap, and `std::mem::replace`

`Log` called `self.inner.call(request)` straight away, inside `call`, and moved only the future into the box. That is the simplest way to write a wrapper, and it dodges a trap. The trap appears when a middleware needs to call `inner` inside the `async` block, after an `.await`. The `async` block must own `inner`, and the obvious move is `self.inner.clone()`. But the service that answered `Ready` was `self.inner`, and a clone of a service is not guaranteed to be ready, even when the original is. `ConcurrencyLimit` is the clearest case: a clone starts with no permit. `tower`'s own docs show the fix. Keep the ready service, and leave the fresh clone behind in `self`:

```rust
fn call(&mut self, request: u32) -> Self::Future {
    let clone = self.inner.clone();
    let mut inner = std::mem::replace(&mut self.inner, clone);
    Box::pin(async move { inner.call(request).await })
}
```

```text
answer = 7
```

`mem::replace` swaps `self.inner` for the clone and gives you the original, so the future owns the service that was polled ready and `self` holds a clone that will be polled ready next time. You only need this pattern when `call` has to use `inner` after an `.await`. "Errors you will meet" shows what happens without it.

### The onion: which layer goes first

Each `.layer(L)` call wraps everything that is already there, so the layer you add last is the outermost: it sees the request first and the response last. `examples/03-log-layers-onion.rs` is `.layer(LogLayer::new("A")).layer(LogLayer::new("B"))`, and the output above shows B going in first.

```senpai-visual
{"kind":"concept","labels":["request enters the outermost layer: B","B passes it to A","A passes it to the Router and handler","handler returns the response","response goes back out through A","then out through B"]}
```

`tower::ServiceBuilder` reads the other way. It composes its layers so that the first one listed is the outermost, which reads top to bottom like the `MIDDLEWARE` list. `examples/04-service-builder-order.rs` lists A, then B:

```text
A: request in  (GET /)
B: request in  (GET /)
B: response out (200 OK)
A: response out (200 OK)
```

The `axum` docs recommend `ServiceBuilder` for several layers for exactly this reason. Remember one more rule from them: `.layer` only wraps routes added before it. Add a route afterwards and the new route is not wrapped.

### The shortcut, and when not to take it

`axum::middleware::from_fn` does the Log job in a few lines (`examples/05-from-fn-version.rs`):

```rust
async fn log(request: Request, next: Next) -> Response {
    println!("fn: request in  ({} {})", request.method(), request.uri());
    let response = next.run(request).await;
    println!("fn: response out ({})", response.status());
    response
}
// ...
Router::new().route("/", get(|| async { "hello" })).layer(from_fn(log))
```

```text
fn: request in  (GET /)
fn: response out (200 OK)
```

`from_fn` is a ready-made `Layer`: it builds the boxed future and the `Clone` impl for you, and `next.run(request)` is the call to `inner`. Use it for middleware that belongs to your app. Write the `Service` by hand when:

- the middleware has to work outside `axum`, or be published as a crate, since a `from_fn` middleware only works with `axum`;
- it needs its own configuration type, with builder methods on the layer (this is the `TraceLayer` and `CorsLayer` shape);
- it has to control `poll_ready` itself, which a `from_fn` function never sees;
- the one allocation per request matters, and you are ready to write a named future.

One thing the hand-written version already shows, which the next lesson relies on: a middleware can answer without calling `inner` at all. It returns a response itself, and the rest of the onion never runs. `CorsLayer` does this for a preflight `OPTIONS` request. The "Build" exercise has you do it.

---

## Hands on

```sh
cargo run -p p3-02-04-tower-service-and-layer-middleware --example 01-router-is-a-service
cargo run -p p3-02-04-tower-service-and-layer-middleware --example 02-poll-ready-backpressure
cargo run -p p3-02-04-tower-service-and-layer-middleware --example 03-log-layers-onion
cargo run -p p3-02-04-tower-service-and-layer-middleware --example 04-service-builder-order
cargo run -p p3-02-04-tower-service-and-layer-middleware --example 05-from-fn-version
```

Then the four broken ones. `06` is a run-time panic, the other three fail to compile:

```sh
cargo run -p p3-02-04-tower-service-and-layer-middleware --example 06-clone-without-ready-broken --features broken
cargo build -p p3-02-04-tower-service-and-layer-middleware --example 07-future-not-send-broken --features broken
cargo build -p p3-02-04-tower-service-and-layer-middleware --example 08-service-not-clone-broken --features broken
cargo build -p p3-02-04-tower-service-and-layer-middleware --example 09-error-not-infallible-broken --features broken
```

Then try these:

1. In `03-log-layers-onion`, add a third layer `C` with `.layer(LogLayer::new("C"))` after `B`. Where does `C` print?
2. In `04-service-builder-order`, swap the two `.layer(...)` lines. What changed, and what did not?
3. In `02-poll-ready-backpressure`, raise the limit from `1` to `2`. What does the second `ready()` do now?

---

## Errors you will meet

Every transcript below is the real output for the example named, with this lesson's own `todo!()` warnings left out.

### `E0277` — a boxed future that is not `Send`

```text
error[E0277]: `(dyn Future<Output = Result<Response<Body>, Infallible>> + 'static)` cannot be sent between threads safely
    --> phase3-backend-foundations\02-axum-and-rest-api-design\04-tower-service-and-layer-middleware\examples\07-future-not-send-broken.rs:54:16
     |
  54 |         .layer(NoopLayer);
     |          ----- ^^^^^^^^^ `(dyn Future<Output = Result<Response<Body>, Infallible>> + 'static)` cannot be sent between threads safely
     |          |
     |          required by a bound introduced by this call
     |
     = help: the trait `Send` is not implemented for `(dyn Future<Output = Result<Response<Body>, Infallible>> + 'static)`
     = note: required for `std::ptr::Unique<(dyn Future<Output = Result<Response<Body>, Infallible>> + 'static)>` to implement `Send`
note: required because it appears within the type `Box<(dyn Future<Output = Result<Response<Body>, Infallible>> + 'static)>`
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\alloc\src\boxed.rs:234:12
     |
 234 | pub struct Box<
     |            ^^^
note: required because it appears within the type `Pin<Box<(dyn Future<Output = Result<Response<Body>, Infallible>> + 'static)>>`
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\pin.rs:1092:12
     |
1092 | pub struct Pin<Ptr> {
     |            ^^^
note: required by a bound in `Router::<S>::layer`
    --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:309:51
     |
 303 |     pub fn layer<L>(self, layer: L) -> Router<S>
     |            ----- required by a bound in this associated function
...
 309 |         <L::Service as Service<Request>>::Future: Send + 'static,
     |                                                   ^^^^ required by this bound in `Router::<S>::layer`

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-02-04-tower-service-and-layer-middleware` (example "07-future-not-send-broken") due to 1 previous error
```

**What the compiler is objecting to:** the error is reported at `.layer(...)`, not at the line where you wrote the type, and the first line names a type you never wrote. It is your `Pin<Box<dyn Future<...>>>`, whose `dyn Future` carries no `+ Send`. The last `note:` is the actual rule: `Router::layer` requires `Future: Send + 'static`.

**The fix:** add the bound to the boxed type:

```rust
type Future = Pin<Box<dyn Future<Output = Result<Response, S::Error>> + Send>>;
```

**Why this is the fix:** `dyn Future` erases the concrete type, and with it the knowledge that the type is `Send` ([2.8.4](../../../phase2-intermediate/08-concurrency/04-send-and-sync/README.md)). Erasure keeps only what you list. Writing `+ Send` is how you promise it. That promise then has to be true: the `async` block inside `call` must only hold `Send` things, which is why `Log` copies `name` out of `self` instead of capturing `&self`.

### `E0277` — the service is not `Clone`

```text
error[E0277]: the trait bound `Noop<Route>: Clone` is not satisfied
   --> phase3-backend-foundations\02-axum-and-rest-api-design\04-tower-service-and-layer-middleware\examples\08-service-not-clone-broken.rs:53:16
    |
 53 |         .layer(NoopLayer);
    |          ----- ^^^^^^^^^ the trait `Clone` is not implemented for `Noop<Route>`
    |          |
    |          required by a bound introduced by this call
    |
note: required by a bound in `Router::<S>::layer`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:306:40
    |
303 |     pub fn layer<L>(self, layer: L) -> Router<S>
    |            ----- required by a bound in this associated function
...
306 |         L::Service: Service<Request> + Clone + Send + Sync + 'static,
    |                                        ^^^^^ required by this bound in `Router::<S>::layer`
help: consider annotating `Noop<Route>` with `#[derive(Clone)]`
    |
 28 + #[derive(Clone)]
 29 | struct Noop<S> {
    |

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-02-04-tower-service-and-layer-middleware` (example "08-service-not-clone-broken") due to 1 previous error
```

**What the compiler is objecting to:** `Noop<Route>` is the service your layer built, with `inner` filled in with `axum`'s `Route`. `Router::layer` needs it to be `Clone`, and the struct has no derive.

**The fix:** the compiler wrote it for you: `#[derive(Clone)]` on the service struct.

**Why this is the fix:** `axum` clones services all the time: one copy per route, and one per connection. The derive works because `Route` is `Clone` and your other fields are too. `Log` has a `&'static str`, which is `Copy`. If a field is not `Clone`, wrap it in an `Arc` and clone that, the way `MaintenanceLayer` does in "Build".

### `E0277` — the error type is not `Infallible`

```text
error[E0277]: the trait bound `Infallible: From<String>` is not satisfied
   --> phase3-backend-foundations\02-axum-and-rest-api-design\04-tower-service-and-layer-middleware\examples\09-error-not-infallible-broken.rs:55:16
    |
 55 |         .layer(NoopLayer);
    |          ----- ^^^^^^^^^ the trait `From<String>` is not implemented for `Infallible`
    |          |
    |          required by a bound introduced by this call
    |
help: the trait `From<String>` is not implemented for `Infallible`
      but trait `From<!>` is implemented for it
   --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\convert\mod.rs:988:1
    |
988 | impl const From<!> for Infallible {
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    = help: for that trait implementation, expected `!`, found `String`
    = note: required for `String` to implement `Into<Infallible>`
note: required by a bound in `Router::<S>::layer`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:308:50
    |
303 |     pub fn layer<L>(self, layer: L) -> Router<S>
    |            ----- required by a bound in this associated function
...
308 |         <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
    |                                                  ^^^^^^^^^^^^^^^^ required by this bound in `Router::<S>::layer`

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-02-04-tower-service-and-layer-middleware` (example "09-error-not-infallible-broken") due to 1 previous error
```

**What the compiler is objecting to:** the middleware declares `type Error = String`. `Router::layer` demands `Error: Into<Infallible>`, and the `help:` says why that fails: nothing converts into `Infallible` except the never type `!`.

**The fix:** make the middleware's error the inner service's, `type Error = S::Error`, and turn any failure of your own into a response (a `500` or `503`) instead of an `Err`.

**Why this is the fix:** `Infallible` is the error type of a `Router` because every handler must produce a response. The `axum` docs explain the reason: if a middleware returned an error, `hyper` would close the connection without sending anything. If you really need a custom error, `axum` offers `HandleErrorLayer` to convert it into a response, but "never fail, always answer" is the default to prefer.

### A run-time panic: calling a clone that was never ready

```text
thread 'main' (6552) panicked at C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\tower-0.5.3\src\limit\concurrency\service.rs:86:14:
max requests in-flight; poll_ready must be called first
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(The number in parentheses is the thread's id and changes every run.)

**What's actually broken:** `examples/06-clone-without-ready-broken.rs` calls `poll_ready` on `self.inner`, then calls `call` on `self.inner.clone()`. The compiler is satisfied: both are the same type. But a `ConcurrencyLimit` reserves its permit in `poll_ready` and takes it in `call`, and a fresh clone has no permit, so its `call` hits `expect(...)`. The `Service` docs say it directly: a service may panic if `call` is invoked without `Ready` from `poll_ready`.

**The fix:** swap the clone in and keep the ready service, the pattern from "The concept":

```rust
let clone = self.inner.clone();
let mut inner = std::mem::replace(&mut self.inner, clone);
```

**Why this is the fix:** the service you call must be the very one that answered `Ready`. After the swap, `inner` is that service, and `self.inner` is the clone, which `poll_ready` will check before the next request. With the swap in place, the example prints `answer = 7`.

---

## Exercises

### Warm up

<details>
<summary>What question does <code>poll_ready</code> answer, and what does <code>Poll::Pending</code> from it mean?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

"Can you take a request right now?" `Pending` means no: the service is at capacity, and the caller should wait until it is woken and ask again. Calling `call` anyway is something a service is allowed to panic on.

</details>

<details>
<summary>A <code>Router</code> has <code>.layer(A).layer(B)</code>. Which layer sees the request first, and which sees the response last?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`B` sees the request first, because the last layer added is the outermost. `B` also sees the response last, since the response leaves through the same layers in reverse order.

</details>

<details>
<summary>In the Django <code>timing_middleware</code> above, which function is the <code>Layer</code> and which is the <code>Service</code>?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

The outer `timing_middleware(get_response)` is the `Layer`: it runs once and builds the wrapper from the thing it wraps. The inner `middleware(request)` is the `Service`: it runs for every request, and `get_response` is its `inner`.

</details>

<details>
<summary>What does <code>ServiceBuilder::new().layer(A).layer(B)</code> do differently from <code>.layer(A).layer(B)</code> on the router?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`ServiceBuilder` reads top to bottom, so `A` is outermost and sees the request first. Chained `.layer` calls on the router make the last one outermost.

</details>

### Repair

Fix all four broken examples:

1. `examples/06-clone-without-ready-broken.rs` prints `answer = 7` instead of panicking.
2. `examples/07-future-not-send-broken.rs` compiles.
3. `examples/08-service-not-clone-broken.rs` compiles.
4. `examples/09-error-not-infallible-broken.rs` compiles, and its failure case becomes a response (not an `Err`).

### Implement

`ResponseTimeLayer` and `ResponseTime` in `src/lib.rs`: a middleware that stamps an `x-response-time-ms` header on every response.

```sh
cargo test -p p3-02-04-tower-service-and-layer-middleware --test response_time_test
```

The doc comment above `ResponseTime` is the whole specification: what the header contains, what happens if it is already there, and what must not change. You never need to read the tests. `Log` above it is a finished example of the same shape. The tests check the header's presence and format, not its value, because timings are different on every run. The tests send requests with `oneshot`, as in 3.2.1.

### Build

`MaintenanceLayer` and `Maintenance`, in the same file: a middleware that, while a shared flag is on, answers `503 Service Unavailable` without calling the inner service at all.

```sh
cargo test -p p3-02-04-tower-service-and-layer-middleware --test maintenance_test
```

Again the doc comment is the specification. The point of the exercise is the short-circuit: when the flag is on, the inner handler must not run, which a test checks by counting how many times it did. This is the same move `CorsLayer` makes for a preflight request in [3.2.5](../05-cors-and-frontend-integration/README.md). The flag is an `Arc<AtomicBool>` ([2.8.2](../../../phase2-intermediate/08-concurrency/02-rwlock-semaphore-oncelock-atomics/README.md)), so you can flip it from outside while the app keeps running.

### Challenge (optional)

Write the timing middleware a second time with `axum::middleware::from_fn`, in a scratch file of your own, and compare the two: how many lines each, and what you can configure in one but not in the other. Then give `ResponseTimeLayer` a field, so that `ResponseTimeLayer::new("x-took-ms")` chooses the header name. Nothing tests this one. Check it with a `oneshot` request of your own.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `tower::Service` | a trait: `poll_ready` plus `call`, which returns a future | `Router`, handlers and middleware are all services |
| `poll_ready` | asks "can you take a request now?"; `Pending` means wait | backpressure, and forwarding to `inner` in every wrapper |
| `Layer` | a factory that turns one service into a wrapping one | `.layer(...)`, `CorsLayer`, `TraceLayer` |
| middleware | a service that holds an inner service and calls it | logging, timing, auth, CORS |
| the onion | each layer wraps all earlier ones; the last added is outermost | predicting the order of middleware |
| `ServiceBuilder` | composes layers so the first listed is outermost | applying several layers at once |
| `from_fn` | `axum`'s ready-made layer built from an `async fn` | app-specific middleware |
| short-circuit | a middleware answers itself and never calls `inner` | maintenance mode, CORS preflight |

### What you now know

- `oneshot` was `poll_ready` plus `call`, and a `Router` is a `Service` whose `Error` is `Infallible`.
- `poll_ready` is backpressure, and a caller must see `Ready` from it before every `call`.
- A `Layer` is a function from a service to a wrapping service, the same shape as a Django middleware, split into a factory and an instance.
- A hand-written middleware needs `Clone`, a `Send + 'static` future (usually a boxed one), and `Error = Infallible` to pass `Router::layer`.
- When `call` has to use `inner` after an `.await`, swap with `std::mem::replace` instead of calling a bare clone.
- The last `.layer(...)` is outermost, `ServiceBuilder` reads top to bottom, and a middleware may answer without calling `inner`.

### What comes back later

- **`CorsLayer`, a ready-made `Layer`, and the preflight short-circuit** — [3.2.5 — CORS and frontend integration](../05-cors-and-frontend-integration/README.md)
- **A middleware that tags every request with an ID** — [3.8.2 — Request tracing and correlation IDs](../../08-error-handling-and-testing-at-scale/02-request-tracing-and-correlation-ids/README.md)
- **A middleware that rejects unauthenticated requests** — [3.7.3 — JWTs and `tower` middleware](../../07-auth-and-security/03-jwt-and-tower-middleware/README.md)

### Can you explain?

- Why does `.oneshot(request)` work on a `Router`, and what are the two steps inside it?
- What does `poll_ready` protect, and why is a clone of a ready service not guaranteed to be ready?
- How is a Django middleware like a `Layer` plus a `Service`, and where does the comparison stop being exact?
- Why is the future in a hand-written middleware usually `Pin<Box<dyn Future<...> + Send>>`, and which of the three `E0277` errors do you get when you leave out a piece?
- Given `.layer(A).layer(B)`, and then `ServiceBuilder` with `A` and `B`, which layer sees the request first in each?
- When would you write the `Service` by hand instead of using `from_fn`?

---

## Going further

- [tower's guide: Building a middleware from scratch](https://github.com/tower-rs/tower/blob/master/guides/building-a-middleware-from-scratch.md): the same middleware with a named future instead of a boxed one.
- [`tower::Service` on docs.rs](https://docs.rs/tower-service/0.3.3/tower_service/trait.Service.html): the trait with its full documentation on backpressure and cloning inner services.
- [`axum::middleware`](https://docs.rs/axum/0.8.9/axum/middleware/index.html): the ways to write middleware in `axum`, ordering, and backpressure.
- [`tower::ServiceBuilder`](https://docs.rs/tower/0.5.3/tower/struct.ServiceBuilder.html): how its layers are composed, with the ordering rule.
