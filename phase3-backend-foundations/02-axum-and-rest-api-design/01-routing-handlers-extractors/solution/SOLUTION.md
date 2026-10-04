# Solution — 3.2.1 Routing, handlers, extractors

## `greet`

```rust
pub async fn greet(Path(name): Path<String>) -> String {
    format!("Hello, {name}!")
}
```

The extraction already happened in the signature. `Path(name)` destructures the `Path<String>` wrapper, so the body only formats.

## `echo`

```rust
pub async fn echo(Json(payload): Json<EchoRequest>) -> Json<EchoResponse> {
    let length = payload.message.len();
    Json(EchoResponse { message: payload.message, length })
}
```

`length` is read before `payload.message` is moved into the response: measure first, give away second. `str::len` counts bytes, which is why the spec says bytes (a Persian letter is two).

## `get_counter` and `increment_counter`

```rust
pub async fn get_counter(State(state): State<AppState>) -> Json<CounterResponse> {
    let count = *state.counter.lock().unwrap();
    Json(CounterResponse { count })
}

pub async fn increment_counter(State(state): State<AppState>) -> Json<CounterResponse> {
    let mut guard = state.counter.lock().unwrap();
    *guard += 1;
    Json(CounterResponse { count: *guard })
}
```

`.lock().unwrap()` fails only if another thread panicked while holding the lock (a poisoned mutex); for an in-memory counter that is an acceptable crash. `CounterResponse` stores a plain `i64` copied out of the guard, so the guard is dropped at the end of the function. No `.await` happens while the lock is held, which is what makes a `std::sync::Mutex` fine here.

## `app`

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

Each `.route` consumes the router and returns it, so the whole table is one expression. Forget `.with_state(state)` and you get the `E0308` from "Errors you will meet".

## Build: `search`

```rust
#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub limit: Option<u32>,
}

pub async fn search(Query(params): Query<SearchParams>) -> Json<SearchResponse> {
    Json(SearchResponse { q: params.q, limit: params.limit.unwrap_or(10) })
}
```

`q: String` makes `q` required and `limit: Option<u32>` makes it optional, so the extractor produces the `400`s for a missing `q` or `limit=many` on its own. The default is applied in the handler, with `unwrap_or(10)`. Register it with `.route("/search", get(search))`; `SearchResponse` is a two-field `Serialize` struct (`q`, `limit`).

## Challenge

```rust
pub async fn echo(payload: Result<Json<EchoRequest>, JsonRejection>) -> Response {
    match payload {
        Ok(Json(p)) => {
            let length = p.message.len();
            Json(EchoResponse { message: p.message, length }).into_response()
        }
        Err(rejection) => (rejection.status(), "bad echo request").into_response(),
    }
}
```

Wrapping an extractor in `Result` makes it hand you its rejection instead of answering. Both arms are turned into a `Response` so they have one type.
