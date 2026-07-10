# Solution

```rust
pub async fn greet(Path(name): Path<String>) -> String {
    format!("Hello, {name}!")
}
```

`axum` already did the work of pulling `{name}` out of the URL and handing
it to you as an owned `String` by the time this line runs — the "extractor"
part of "routing, handlers, extractors" is entirely in the function
*signature*, not the body.

```rust
pub async fn echo(Json(payload): Json<EchoRequest>) -> Json<EchoResponse> {
    let length = payload.message.len();
    Json(EchoResponse { message: payload.message, length })
}
```

`length` is computed before `payload.message` moves into the
`EchoResponse` — `payload.message.len()` borrows, `EchoResponse { message:
payload.message, .. }` moves. Doing the borrow first and the move second
(rather than trying to read `.len()` off a value that's already moved) is
the same ordering discipline as `total_length` in the move-semantics
lesson: you can measure something before giving it away, never after.

```rust
pub async fn get_counter(State(state): State<AppState>) -> Json<CounterResponse> {
    let count = *state.counter.lock().unwrap();
    Json(CounterResponse { count })
}
```

`.lock().unwrap()` — a `Mutex::lock()` returns
`LockResult<MutexGuard<T>>`, and the only way it's `Err` is if another
thread panicked while holding the lock (a "poisoned" mutex). Unwrapping is
the standard, accepted move for a learning-scale in-memory counter; a
production service would decide deliberately whether poisoning should
crash the request or be recovered from.

```rust
pub async fn increment_counter(State(state): State<AppState>) -> Json<CounterResponse> {
    let mut guard = state.counter.lock().unwrap();
    *guard += 1;
    let count = *guard;
    drop(guard);
    Json(CounterResponse { count })
}
```

This is gotcha #8 made concrete: writing `*guard` as the bare last
expression of a block that also owns `guard`'s source can hit a real
`E0597` "borrowed value does not live long enough" in some shapes of this
pattern, because the compiler has to work out the drop order of `guard`
relative to the value being read out of it. Binding `*guard` to a named
local (`count`) *before* the guard is dropped sidesteps the question
entirely — `count` is a plain `i64`, fully independent of `guard`, the
moment that line executes. The explicit `drop(guard)` isn't strictly
required here (it would drop automatically at the end of the block anyway)
but makes the "we're done with the lock" moment visible, which matters more
as handlers grow — holding a lock any longer than necessary is exactly what
turns a `Mutex` into a bottleneck under concurrent load.

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

Each `.route` call is chained (`Router::new()` returns `Self`, `.route`
consumes and returns `Self`), building the whole router as one expression
— the same builder pattern you've already seen in `reclaim_and_extend`
from move-semantics, just at the scale of a whole API surface now.

## On the checkpoint questions

**Q1 (what you'd have to write by hand in DRF):** A DRF view has to call
`serializer.is_valid()` explicitly, check the boolean, and manually return
`Response(serializer.errors, status=400)` in the failure branch — if you
forget that check, invalid data silently flows into
`serializer.validated_data` anyway (or raises later, deeper in your code,
somewhere less obvious). `axum`'s `Json<T>` extractor makes "the handler
body only ever runs with a valid, fully-parsed `T`" a *type-level*
guarantee enforced before your code executes at all, not a convention you
have to remember to follow at every call site.

**Q2 (why cloning `AppState` doesn't give each request its own counter):**
`Arc<Mutex<i64>>` is two layers: `Arc` (atomic reference count) makes
cloning cheap and gives every clone a pointer to the *same* heap
allocation, and `Mutex` guards access to what's inside that allocation.
Cloning `AppState` clones the `Arc` (bumping a reference count, not copying
the `i64`) — every handler invocation's `State<AppState>` extraction ends
up pointing at the exact same `Mutex<i64>` in memory. This is the same
"share memory *safely* across concurrent access" role `Arc<Mutex<T>>`
played in the concurrency module, just now the "concurrent access" is
concurrent HTTP requests instead of concurrent threads you spawned by hand.

**Q3 (what handlers will `.await` starting module 4):** Database queries —
`sqlx::query(...).fetch_one(&pool).await` (module 4). A synchronous
function calling something like that would have to *block* the OS thread
it's running on until the database responds, and because `tokio` runs many
concurrent tasks on a small pool of threads (module 1's thread-per-connection
discussion), one blocking call would stall every *other* task scheduled on
that same thread too. `.await` instead yields the thread back to the
runtime while waiting, exactly like `asyncio`'s `await` yields back to the
event loop.

**Q4 (what `axum` hands `greet` before the pattern destructures it):**
`Path<String>` — a tuple struct wrapping the extracted value. `Path(name)`
in the parameter position is a *destructuring pattern* against that
wrapper, pulling the inner `String` out and binding it to `name` in one
step — equivalent to writing `path: Path<String>` and then `let name =
path.0;` as the first line of the body, just inlined into the signature.

**Q5 (automatic 404, and what does need custom code):** Django's URL
resolver is exactly the same here — an unmatched path 404s with zero code
from you in either framework; you only write a custom 404 view if you want
non-default *content* for it. Where it gets interesting: hitting a route
that exists but with the wrong HTTP method. `axum`'s `Router` also handles
this automatically — `.route("/echo", post(echo))` only registers `POST`,
so a `GET /echo` gets a `405 Method Not Allowed` with, again, zero extra
code from you, because each path's registered methods are tracked
separately from whether the path itself matched at all. Django's
class-based `View` does the equivalent (dispatching per-HTTP-method,
405-ing on an unhandled one) — a plain function-based view does *not*, and
you'd have to check `request.method` and branch yourself.
