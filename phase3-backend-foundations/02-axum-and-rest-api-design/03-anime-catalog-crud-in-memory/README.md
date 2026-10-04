# 3.2.3 — Anime catalog CRUD (in-memory)

## At a glance

After this lesson you can:

- Build a full create/read/update/delete resource in `axum`: a plain store with no HTTP in it, tested on its own, under a thin layer of handlers.
- Turn your own domain error into an HTTP response by implementing `IntoResponse`, so a handler can return `Result<Json<Anime>, AnimeError>` and `?` just works.
- Pick the status code for each outcome (`201`, `204`, `404`, `422`, `400`) and defend it with what 3.1.3 says about the methods.
- Read the `E0277` you get when a handler's future isn't `Send` because a `std::sync::MutexGuard` lives across an `.await`, and fix it.

**Time:** ~100 minutes · **Prerequisites:**
[3.2.1 — Routing, handlers, extractors](../01-routing-handlers-extractors/README.md),
[3.2.2 — Writing your own extractor](../02-writing-your-own-extractor/README.md),
[3.1.3 — HTTP semantics you must know](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.md)

---

## Why this matters

3.2.1 and 3.2.2 gave you the pieces: a `Router`, handlers, extractors. Their routes were separate demos. This lesson is the shape almost every REST API takes in practice: one resource, five operations, one place that holds the data, and one decision per outcome about what the client is told.

In DRF you get most of that from a `ModelViewSet`: it picks `201` for you, turns `Http404` into a response, and your serializer's `ValidationError` becomes a `400`. In `axum` nothing is chosen for you. A handler returns *something*, and you decide what. That decision, "which status, which body, for which failure", is the part of an API a client actually depends on, and it is what you practise here. 3.1.3 listed the codes and said "from the next module on, you still choose them yourself". This is where you do.

The store is deliberately in memory, gone when the process exits. [3.5.3 — Anime catalog, Postgres-backed](../../05-postgres-and-sqlx/03-anime-catalog-postgres-backed/README.md) rebuilds this exact resource on a database, so you can see which parts of a CRUD API are HTTP design (this lesson) and which are persistence (that one).

---

## The concept

### One resource, five operations, one route table

| DRF (`ModelViewSet` on `Anime`) | This lesson | Success |
|---|---|---|
| `GET /anime/` → `list()` | `GET /anime` → `list_anime` | `200` + JSON array |
| `POST /anime/` → `create()` | `POST /anime` → `create_anime` | `201` + `Location` + JSON |
| `GET /anime/{id}/` → `retrieve()` | `GET /anime/{id}` → `get_anime` | `200` + JSON |
| `PATCH /anime/{id}/` → `partial_update()` | `PATCH /anime/{id}` → `update_anime` | `200` + JSON |
| `DELETE /anime/{id}/` → `destroy()` | `DELETE /anime/{id}` → `delete_anime` | `204`, empty body |
| `Anime.objects` | `AnimeStore` | |

Where the analogy stops: a `ModelViewSet` generates this table from a class. Here you write the table, the five handlers, and the store by hand, so every status code in it is one you chose.

### Pure store, thin HTTP edge

Two layers, kept apart on purpose:

```senpai-visual
{"kind":"result","labels":["request","handler (thin)","store: Arc + Mutex","Result of Anime or AnimeError","IntoResponse","status code + JSON"]}
```

`AnimeStore` is plain Rust. It does not import `axum`, never sees a `StatusCode`, and its methods return `Result<Anime, AnimeError>`. A handler does three small jobs: take the request apart with extractors, call one store method, and hand the `Result` back to `axum`. Because the rules ("a rating is `1..=10`", "ids start at 1", "list is sorted by id") live in the store, `tests/store_test.rs` checks them with ordinary function calls: no runtime, no requests. `tests/api_test.rs` then checks only the edge, through `oneshot`, the way 3.2.1 did.

### Sharing one store: `Arc` and a `Mutex`

Every request runs a handler, possibly on a different thread, and they all must see the same catalog. So `main` builds one `Arc<AnimeStore>` and gives each handler a clone through `State`:

```rust
#[derive(Default)]
struct StoreInner {
    next_id: u64,
    items: HashMap<u64, Anime>,
}

#[derive(Default)]
pub struct AnimeStore {
    inner: Mutex<StoreInner>,
}
```

The store's methods take `&self`, not `&mut self`: many handlers hold a shared reference at once. The mutation happens behind it, which is **interior mutability** (2.6.5 showed `RefCell`; `Mutex` is its thread-safe cousin from 2.8.1). The counter and the map share *one* `Mutex` on purpose. With two locks, two concurrent creates could both read the same `next_id` and the second insert would overwrite the first.

Each store method locks, works, and returns; the guard (2.8.1's `MutexGuard`) is dropped when the method returns. The store is synchronous, so no guard can end up held across an `.await`. "Errors you will meet" shows what happens when one does.

### One path, several methods

`.route(path, ...)` takes one `MethodRouter`, and `get(handler)` returns one that you can keep chaining. One path, several methods, one `.route` call:

```rust
Router::new()
    .route("/ping", get(ping).post(pong))
    .route("/ping/{id}", get(ping_one).delete(drop_one))
    .with_state(state)
```

A method nobody registered on that path answers `405 Method Not Allowed` by itself, with an `Allow` header. Path parameters are `/{id}` in `axum` 0.8; the old `/:id` panics when the router is built.

### Your own error type as a response: `IntoResponse`

3.2.2 let a rejection be a `(StatusCode, &'static str)`, because `axum` already knows how to turn that tuple into a response. For your own error type, you teach `axum` yourself. The trait is `IntoResponse`, and it has one method:

```rust
impl IntoResponse for CardError {
    fn into_response(self) -> Response {
        match self {
            CardError::Declined => (StatusCode::PAYMENT_REQUIRED, "card declined"),
            CardError::Expired => (StatusCode::UNPROCESSABLE_ENTITY, "card expired"),
        }
        .into_response()
    }
}
```

Read it as "how does a `CardError` look on the wire?". Both arms build the tuple `(StatusCode, &'static str)`, and the tuple already knows how to become a response, so you finish by calling the method on it. A body can be anything that is itself `IntoResponse`: `Json(...)`, a `String`, another tuple.

Once your error type has the impl, `axum` has a rule for a result type: `Result<T, E>` is `IntoResponse` whenever both `T` and `E` are. A handler can now return `Result<Json<Anime>, AnimeError>`, and `?` on a store call turns the store's `Err(AnimeError)` into the handler's `Err`. `axum` calls `.into_response()` on whichever side comes back. The handler has no `match`. `examples/01-what-a-handler-return-becomes.rs` calls it by hand on three shapes your handlers return:

```text
--- Json(anime)
HTTP/1.1 200 OK
content-type: application/json

{"id":1,"title":"Frieren"}
--- (CREATED, [(LOCATION, ..)], Json(anime))
HTTP/1.1 201 Created
content-type: application/json
location: /anime/1

{"id":1,"title":"Frieren"}
--- StatusCode::NO_CONTENT
HTTP/1.1 204 No Content


```

A bare `Json` is `200`. A tuple that starts with a `StatusCode` overrides it, and a header array in the middle adds headers. A lone `StatusCode` is a response with an empty body.

Why define a domain error at all, instead of returning `(StatusCode, String)` from the store? Because the store should know nothing about HTTP. `AnimeError::NotFound` is true for the Postgres version, a CLI, and a test; "404" is only true at the edge. The one `impl` is the single place the two vocabularies meet. (This is also where Django's `exception_handler` sits in DRF. Doing it by hand once is how you see where the JSON error shape comes from. 3.8.1 makes that shape the same for every error in an API.)

### Which failure gets which status

The lesson has two domain failures and `axum` adds a few of its own before your handler even runs:

```senpai-visual
{"kind":"concept","labels":["404: well-formed request, that id does not exist","422: valid JSON, but the rating breaks the rule","400: not JSON at all, or an id that is not a number","415: no Content-Type: application/json"]}
```

- **`NotFound` → `404`.** The request was fine. The resource it names isn't there.
- **`InvalidRating(r)` → `422`.** 3.1.3's rule: `400` means the request itself is malformed; `422` means it parsed fine and the *content* is invalid. A rating of `15` is a perfectly good number, and the JSON is perfectly good JSON. It just breaks a business rule, so `422`.
- **`400`, `422` and `415` from `axum`'s extractors**, before your code runs: broken JSON is `400`; JSON that is valid but doesn't fit the struct (a missing `title`) is `422`; a missing `Content-Type: application/json` is `415`; an `{id}` that isn't a `u64` is `400`. You will see all four in "Hands on".

### `201`, `Location`, `PATCH` versus `PUT`, and `DELETE` twice

- **`POST` is not idempotent, so the answer is `201 Created`**, not `200`. 3.1.3's table has `POST` as neither safe nor idempotent: send it twice and you get two anime. A `Location` header (`Location: /anime/{id}`) tells the client where the new resource lives, and `full_crud_lifecycle` asserts it.
- **`PATCH` is for a partial update**: send only the fields you want to change, and the rest stay as they are (`UpdateAnime`, every field optional). **`PUT` replaces the whole representation**: send every field, and anything you leave out is reset. 3.1.3's table puts `PUT` among the idempotent methods (the same `PUT` twice leaves the same state) and `PATCH` with `POST`, the two with no such promise. The reason is that a patch *can* say "add 1 to the rating", and then repeating it changes things again. This API's patch only sets fields, so in practice it happens to be idempotent, but the contract does not promise it, so a client's retry logic must not assume it. "Build" adds a `PUT` so you can feel the difference.
- **`DELETE` is idempotent even though the second one answers `404`.** After the first `DELETE`, anime 1 is gone. After the second, anime 1 is still gone. The server's state is the same, and that is what idempotency measures. The *response* can differ. `204` then `404` is correct, and the `deleting_twice_answers_differently_but_leaves_the_same_state` test checks both halves.

---

## Hands on

Run the three examples that compile. (`02`, `03` and `04` are broken on purpose and sit behind the `broken` feature; "Errors you will meet" shows them. `05` runs normally and is simply wrong.)

```sh
cargo run -p p3-02-03-anime-catalog-crud-in-memory --example 01-what-a-handler-return-becomes
cargo run -p p3-02-03-anime-catalog-crud-in-memory --example 06-guard-dropped-before-await-fix
cargo run -p p3-02-03-anime-catalog-crud-in-memory --example 05-created-answers-200-trap
```

```text
GET /count -> 1
GET /count -> 2
POST /anime -> 200 OK  (should be 201 Created)
```

(The first command's output is the block in "The concept"; these are the other two.)

The tests start out red. `src/lib.rs` has the whole skeleton and every function is a `todo!()` with a doc comment that says exactly what it must do. Start with the store:

```sh
cargo test -p p3-02-03-anime-catalog-crud-in-memory --test store_test 2>&1 | grep 'test result'
```

```text
test result: FAILED. 0 passed; 15 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

When everything is green, run the real server (it listens on `127.0.0.1:3001`) and talk to it with `curl`. These transcripts were captured against the solution:

```sh
cargo run -p p3-02-03-anime-catalog-crud-in-memory &
curl -si -X POST -H 'content-type: application/json' -d '{"title":"Frieren","status":"watching","rating":9}' http://127.0.0.1:3001/anime | head -n 3
curl -s -X POST -H 'content-type: application/json' -d '{"title":"Dandadan","status":"plan_to_watch"}' http://127.0.0.1:3001/anime
curl -s http://127.0.0.1:3001/anime
```

```text
HTTP/1.1 201 Created
content-type: application/json
location: /anime/1
{"id":2,"title":"Dandadan","status":"plan_to_watch","rating":null}[{"id":1,"title":"Frieren","status":"watching","rating":9},{"id":2,"title":"Dandadan","status":"plan_to_watch","rating":null}]
```

(`curl -s` prints a body without a trailing newline, so the next output starts right after it.) Now the failures, each printed with its status code on the last line:

```sh
curl -s -w '\n%{http_code}\n' -X POST -H 'content-type: application/json' -d '{"title":"Bad","status":"watching","rating":15}' http://127.0.0.1:3001/anime
curl -s -w '\n%{http_code}\n' -X POST -H 'content-type: application/json' -d '{' http://127.0.0.1:3001/anime
curl -s -w '\n%{http_code}\n' -X POST -H 'content-type: application/json' -d '{"status":"watching"}' http://127.0.0.1:3001/anime
curl -s -w '\n%{http_code}\n' -X POST -d '{"title":"x","status":"watching"}' http://127.0.0.1:3001/anime
curl -s -w '\n%{http_code}\n' http://127.0.0.1:3001/anime/abc
```

```text
{"error":"rating must be between 1 and 10, got 15"}
422
Failed to parse the request body as JSON: EOF while parsing an object at line 1 column 1
400
Failed to deserialize the JSON body into the target type: missing field `title` at line 1 column 21
422
Expected request with `Content-Type: application/json`
415
Invalid URL: Cannot parse `abc` to a `u64`
400
```

The first line is *your* `AnimeError` through *your* `IntoResponse`. The other four are `axum`'s own rejections: plain text, not your JSON shape. 3.8.1 is where that inconsistency gets fixed. Finally `DELETE` twice, with the list after each:

```sh
curl -s -o /dev/null -w '%{http_code}\n' -X DELETE http://127.0.0.1:3001/anime/1
curl -s -w '\n%{http_code}\n' -X DELETE http://127.0.0.1:3001/anime/1
curl -s http://127.0.0.1:3001/anime
```

```text
204
{"error":"anime not found"}
404
[{"id":2,"title":"Dandadan","status":"plan_to_watch","rating":null}]
```

Stop the server when you are done (`kill %1` in the same shell). Then try these:

1. Create two anime, delete the first, create a third. What id does it get, and why can't it be `1` again?
2. Send `PATCH /anime/2` with `{"rating": null}`. Does it clear the rating? (Read `UpdateAnime`'s doc comment.)
3. In `examples/05-created-answers-200-trap.rs`, change the handler so it answers `201`. Which `use` do you need?

---

## Errors you will meet

### `E0277` — a `MutexGuard` held across an `.await`

This is the one you will meet in real services. A handler locks, does some async work, and still holds the guard:

```rust
async fn count(State(counter): State<Arc<Mutex<u32>>>) -> String {
    let mut guard = counter.lock().unwrap();
    *guard += 1;
    audit_log().await;
    format!("{}", *guard)
}
```

`examples/02-guard-across-await-broken.rs` registers it with `get(count)`:

```text
error[E0277]: the trait bound `fn(State<Arc<std::sync::Mutex<u32>>>) -> impl Future<Output = String> {count}: Handler<_, _>` is not satisfied
   --> phase3-backend-foundations\02-axum-and-rest-api-design\03-anime-catalog-crud-in-memory\examples\02-guard-across-await-broken.rs:26:30
    |
 26 |         .route("/count", get(count))
    |                          --- ^^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(State<Arc<std::sync::Mutex<u32>>>) -> impl Future<Output = String> {count}`
    |                          |
    |                          required by a bound introduced by this call
    |
    = note: Consider using `#[axum::debug_handler]` to improve the error message
note: required by a bound in `axum::routing::get`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\method_routing.rs:167:16
    |
167 |             H: Handler<T, S>,
    |                ^^^^^^^^^^^^^ required by this bound in `get`
...
441 | top_level_handler_fn!(get, GET);
    | -------------------------------
    | |                     |
    | |                     required by a bound in this function
    | in this macro invocation
    = note: this error originates in the macro `top_level_handler_fn` (in Nightly builds, run with -Z macro-backtrace for more info)

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-02-03-anime-catalog-crud-in-memory` (example "02-guard-across-await-broken") due to 1 previous error
```

**What the compiler is objecting to:** nothing in this message says `Send` or `MutexGuard`. It only says `count` is not a `Handler`, and the one hint it has is to try `#[axum::debug_handler]`. That is how this error always looks, and it is why it is so confusing. The reason is in the `Handler` trait: the future your handler returns must be `Send`. A multi-threaded runtime may pick a task up on a different worker thread after every `.await`, so everything alive across an `.await` has to be movable between threads, which is exactly 2.8.4's `Send`. `std::sync::MutexGuard` is not `Send`.

Put the attribute on the handler (it needs `axum`'s `macros` feature, which this lesson's `Cargo.toml` turns on) and the compiler tells you what it really means. This is `examples/04-guard-across-await-debug-handler-broken.rs`:

```text
error: future cannot be sent between threads safely
  --> phase3-backend-foundations\02-axum-and-rest-api-design\03-anime-catalog-crud-in-memory\examples\04-guard-across-await-debug-handler-broken.rs:17:1
   |
17 | #[axum::debug_handler]
   | ^^^^^^^^^^^^^^^^^^^^^^ future returned by `count` is not `Send`
   |
   = help: within `impl Future<Output = String>`, the trait `Send` is not implemented for `std::sync::MutexGuard<'_, u32>`
note: future is not `Send` as this value is used across an await
  --> phase3-backend-foundations\02-axum-and-rest-api-design\03-anime-catalog-crud-in-memory\examples\04-guard-across-await-debug-handler-broken.rs:21:17
   |
19 |     let mut guard = counter.lock().unwrap();
   |         --------- has type `std::sync::MutexGuard<'_, u32>` which is not `Send`
20 |     *guard += 1;
21 |     audit_log().await;
   |                 ^^^^^ await occurs here, with `mut guard` maybe used later
note: required by a bound in `__axum_macros_check_count_future::check`
  --> phase3-backend-foundations\02-axum-and-rest-api-design\03-anime-catalog-crud-in-memory\examples\04-guard-across-await-debug-handler-broken.rs:17:1
   |
17 | #[axum::debug_handler]
   | ^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `check`
   = note: this error originates in the attribute macro `axum::debug_handler` (in Nightly builds, run with -Z macro-backtrace for more info)
```

(The command's output continues with the same `E0277` as before, printed a second time at the `.route(...)` call, and ends with `could not compile ... due to 2 previous errors`.)

Now it is plain: `guard` has a type that is not `Send`, and an `.await` happens while it is alive.

**The fix:** make sure the guard is gone before the `.await`. The smallest way is a block that ends the lock's life:

```rust
let now = {
    let mut guard = counter.lock().unwrap();
    *guard += 1;
    *guard
}; // the guard is dropped here
audit_log().await;
format!("{now}")
```

`examples/06-guard-dropped-before-await-fix.rs` is the same handler with that change, and it prints `GET /count -> 1` then `GET /count -> 2`.

**Why this is the fix:** the guard's drop (2.8.1) is what releases the lock, and a block's end is when it drops. With no guard alive at the `.await`, the future holds only `Send` things, and the `Handler` bound is satisfied. Two other fixes exist. Keep the shared state behind a synchronous store, which is the shape this lesson uses, so the lock never meets an `.await` at all; or, for the rare case where you must hold a lock across an `.await`, use `tokio`'s own async `Mutex` (`tokio::sync::Mutex`), whose guard is `Send`. Holding a `std` guard across an `.await` is bad even when it compiles (with a single-threaded runtime it can deadlock: a second task waits for the lock on the very thread that holds it).

### `E0277` — an error type that is not `IntoResponse`

```text
error[E0277]: the trait bound `fn() -> impl Future<Output = Result<&'static str, ShowError>> {show}: Handler<_, _>` is not satisfied
   --> phase3-backend-foundations\02-axum-and-rest-api-design\03-anime-catalog-crud-in-memory\examples\03-error-without-into-response-broken.rs:20:57
    |
 20 |     let _app: Router = Router::new().route("/show", get(show));
    |                                                     --- ^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn() -> impl Future<Output = Result<&'static str, ShowError>> {show}`
    |                                                     |
    |                                                     required by a bound introduced by this call
    |
    = note: Consider using `#[axum::debug_handler]` to improve the error message
note: required by a bound in `axum::routing::get`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\method_routing.rs:167:16
    |
167 |             H: Handler<T, S>,
    |                ^^^^^^^^^^^^^ required by this bound in `get`
...
441 | top_level_handler_fn!(get, GET);
    | -------------------------------
    | |                     |
    | |                     required by a bound in this function
    | in this macro invocation
    = note: this error originates in the macro `top_level_handler_fn` (in Nightly builds, run with -Z macro-backtrace for more info)

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-02-03-anime-catalog-crud-in-memory` (example "03-error-without-into-response-broken") due to 1 previous error
```

**What the compiler is objecting to:** the same wording as before, for a different reason. A handler's return type must be `IntoResponse`, and `Result<&str, ShowError>` is only that if `ShowError` is too. `ShowError` is a plain enum that `axum` knows nothing about. The error does not name the type at fault, and `#[axum::debug_handler]` would point at it.

**The fix:** implement the trait for your error type:

```rust
impl IntoResponse for ShowError {
    fn into_response(self) -> Response {
        (StatusCode::NOT_FOUND, "no such show").into_response()
    }
}
```

**Why this is the fix:** `Result<T, E>` has an `IntoResponse` impl that needs both halves to have one. You supply the missing half, and that one impl is also where you choose the status code and the body for each variant.

### No error at all: `200` where `201` belongs

```text
POST /anime -> 200 OK  (should be 201 Created)
```

**What's actually broken:** `examples/05-created-answers-200-trap.rs` compiles, runs, and answers with the right body. Its handler returns a bare `Json(...)`, and a bare `Json` is a `200`. Clients that decide from the status code (a retry policy, a cache, an API gateway, a test in someone else's repo) get the wrong signal for a request that just created something.

**The fix:** return a tuple that starts with the status:

```rust
Ok((StatusCode::CREATED, Json(anime)))
```

**Why this is the fix:** the tuple's first element overrides the default `200`. The compiler cannot catch a wrong-but-valid status code, so only a test that asserts the status does. That is why every test in `tests/api_test.rs` checks `response.status()` before the body.

---

## Exercises

### Warm up

<details>
<summary>A client sends a valid <code>POST /anime</code>. Which status, and which header should come with it?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`201 Created`, plus `Location: /anime/{id}` pointing at the new resource. `POST` is not idempotent (3.1.3): each call makes another anime, so `200` ("here is something that existed") is the wrong message.

</details>

<details>
<summary>The second <code>DELETE /anime/1</code> answers <code>404</code>. Is <code>DELETE</code> still idempotent?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

Yes. Idempotency is about the server's *state*, not the reply. After one `DELETE` or after five, anime 1 is gone and the catalog is the same. The reply may differ (`204`, then `404`) without breaking the promise, which is the point 3.1.3 made.

</details>

<details>
<summary>A <code>POST</code> arrives with <code>"rating": 15</code> in well-formed JSON. <code>400</code> or <code>422</code>?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`422`. The request is not malformed: the JSON parses and every field has the right type. The *content* breaks a rule. `400` is for a request that cannot be understood at all (broken JSON, an `{id}` that isn't a number).

</details>

<details>
<summary>Why can <code>get_anime</code> return <code>Result&lt;Json&lt;Anime&gt;, AnimeError&gt;</code>? What two things have to be true?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`Json<Anime>` implements `IntoResponse`, and so does `AnimeError`, because you implemented it. `axum` implements `IntoResponse` for `Result<T, E>` exactly when both `T` and `E` do, and calls `.into_response()` on whichever one comes back.

</details>

<details>
<summary>Does the spec promise that <code>PATCH</code> is idempotent? And <code>PUT</code>?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`PUT` yes, `PATCH` no. A `PUT` states the whole new representation, so repeating it changes nothing more. A patch can describe a *change* ("add 1"), so the spec does not promise that repeating it is harmless, even if a particular API's patch (like this one) only sets fields.

</details>

### Repair

Fix all four broken examples:

1. `examples/02-guard-across-await-broken.rs` compiles: make the future `Send`. Build it with `--features broken` to check.
2. `examples/04-guard-across-await-debug-handler-broken.rs` compiles. This is the same bug with the clearer error. Do not just delete the attribute.
3. `examples/03-error-without-into-response-broken.rs` compiles, with a `NOT_FOUND` response for `ShowError::NotFound`.
4. `examples/05-created-answers-200-trap.rs` prints `201 Created`.

### Implement

Everything in `src/lib.rs` that is a `todo!()`: `AnimeError`'s `into_response`, the five store methods, the five handlers, and `app`. Each doc comment is the complete specification (status codes, JSON shapes, ordering, the validation rule), so you should never need to open the tests to know what to build. Work from the bottom of the stack up: the store first, then the error, then the handlers and the route table.

```sh
cargo test -p p3-02-03-anime-catalog-crud-in-memory
```

`tests/store_test.rs` (15 tests) checks the store with plain calls. `tests/api_test.rs` (7 tests) checks the whole stack through `oneshot`.

### Build

Add `PUT /anime/{id}`, a full replacement. Its body has the same shape as `POST`'s (`CreateAnime`), it answers `200` with the new anime, keeps the id, and answers `404` for an id that doesn't exist (it never creates; ids are the server's to give). A rating out of range is `422`, as everywhere else. Add it as an `AnimeStore::replace` plus a handler, then chain `.put(...)` into the route for `/anime/{id}`. Write your own tests in a new `tests/put_test.rs`. Include one that sends the identical `PUT` twice and asserts the two responses and the final state are the same: that is idempotency, as a test.

### Challenge (optional)

Optimistic concurrency, kept small. Two clients `PATCH` the same anime from stale copies, and the second silently overwrites the first. Give every `Anime` a `version: u64` that starts at `1` and goes up by one on every successful change. Make `PATCH` require an `If-Match: <version>` header. A mismatch answers `412 Precondition Failed` (a new `AnimeError` variant and a new arm in your `IntoResponse`). This is the lesson's one reach forward: HTTP's conditional requests and `ETag`s are the real mechanism, and in a database the same idea becomes a `WHERE version = $2` clause. There are no provided tests: write your own for the matching, the stale, and the missing header.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Domain error | an error in your problem's own vocabulary (`NotFound`), with no HTTP in it | every store, service, and repository |
| `IntoResponse` | the trait that says how a value becomes an HTTP response | every handler return type, every error type |
| Interior mutability behind `State` | one `Arc<Store>` shared by all handlers, mutated through a `Mutex` inside | shared app state, caches, counters |
| Method chaining on a route | `get(a).post(b)` registers several methods on one path | every resource with more than one verb |
| `#[axum::debug_handler]` | an attribute that makes a handler's `E0277` say what is actually wrong | any time a handler "is not a Handler" |

### What you now know

- A store is plain Rust that returns `Result<_, YourError>`. The handlers are a thin layer that turns that into HTTP, and each half is tested on its own.
- `impl IntoResponse for YourError` is the single place a domain failure becomes a status code and a body, and it is what makes `Result<_, YourError>` a legal handler return.
- Which status for which outcome: `201` + `Location` for `POST`, `204` for `DELETE`, `404` for a missing id, `422` for valid-but-invalid content, `400` for a malformed request.
- `PATCH` is a partial update with no idempotency promise, `PUT` is a full replacement that is idempotent, and `DELETE` is idempotent even though its second reply is `404`.
- A handler's future must be `Send`. A `std` `MutexGuard` held across an `.await` breaks that, and the fix is to drop the guard first.

### What comes back later

- **Replacing the `HashMap` with a real database**: [3.5.3 — Anime catalog, Postgres-backed](../../05-postgres-and-sqlx/03-anime-catalog-postgres-backed/README.md)
- **One error format for every failure, including the plain-text rejections you saw today**: [3.8.1 — Consistent error envelopes](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)
- **A real validation layer instead of one hand-written rule**: [3.3.2 — Validation](../../03-serialization-and-validation/02-validation/README.md)
- **Where a shared store like this lives in a larger application**: [3.4.2 — Application state and dependency wiring](../../04-configuration-and-app-structure/02-app-state-and-dependency-wiring/README.md)
- **What `.layer(...)` does around handlers like these**: [3.2.4 — `tower::Service` / `Layer`: middleware by hand](../04-tower-service-and-layer-middleware/README.md)
- **Paging through a long list instead of returning all of it**: [3.6.2 — Pagination: offset vs. keyset](../../06-database-design-and-query-performance/02-pagination/README.md)

### Can you explain?

- Why does the store return `AnimeError` and not an HTTP status?
- What does `impl IntoResponse for AnimeError` buy a handler, and why does `?` work on a store call inside it?
- Why is a rating of `15` a `422` and not a `400`, and what is the difference to a body that isn't JSON?
- Why is `DELETE` idempotent when the second call answers `404`?
- Why does a `PATCH` get no idempotency promise while a `PUT` does?
- Why does the compiler say a handler "is not a `Handler`" when the real problem is a `MutexGuard` across an `.await`?

---

## Going further

- [`IntoResponse` in the `axum` docs](https://docs.rs/axum/0.8.9/axum/response/trait.IntoResponse.html): every type that already implements it, including the tuple forms you used today.
- [`axum::error_handling`](https://docs.rs/axum/0.8.9/axum/error_handling/index.html): how `axum` thinks about errors, and why handlers are infallible by design.
- [RFC 9110 — HTTP Semantics](https://www.rfc-editor.org/rfc/rfc9110.html): §15.3.2 (`201 Created`), §10.2.2 (`Location`), §9.3.4 (`PUT`), §13.1.1 (`If-Match`), §15.5.21 (`422`).
- [RFC 5789 — PATCH Method for HTTP](https://www.rfc-editor.org/rfc/rfc5789.html): why `PATCH` exists next to `PUT`, and why it is not idempotent by definition.
- [2.8.4 — `Send` and `Sync`](../../../phase2-intermediate/08-concurrency/04-send-and-sync/README.md): the rule behind today's `E0277`.
