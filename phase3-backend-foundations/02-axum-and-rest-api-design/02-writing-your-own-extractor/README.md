# 3.2.2 — Writing your own extractor (`FromRequestParts`)

## At a glance

After this lesson you can:

- Explain why a handler may have many extractors but only one that reads the body, and say which trait each kind implements.
- Write an extractor by hand with `FromRequestParts`: an API key from a header, pagination from the query string, a client version from a header.
- Pick a rejection type that `axum` already knows how to send, and read the four compile errors you hit when you pick wrong.
- Make an extractor optional in `axum` 0.8, where `Option<T>` no longer works for every extractor.

**Time:** ~75 minutes · **Prerequisites:**
[3.2.1 — Routing, handlers, extractors](../01-routing-handlers-extractors/README.md),
[2.3.1 — Defining and implementing traits](../../../phase2-intermediate/03-traits-and-generics/01-defining-and-implementing-traits/README.md),
[2.3.5 — Associated types versus generic parameters](../../../phase2-intermediate/03-traits-and-generics/05-associated-types/README.md),
[2.9.4 — Async traits and `spawn_blocking`](../../../phase2-intermediate/09-async-in-practice/04-async-traits-and-blocking/README.md)

---

## Why this matters

[3.2.1](../01-routing-handlers-extractors/README.md) gave you `Path`, `Query`, `Json` and `State`. They cover what `axum` can know about every API. They don't cover what *your* API needs: an `x-api-key` header on every route, a `?page=` and `?per_page=` pair with a default and a cap, the version of the client app that sent the request.

Without your own extractor, that logic is copied to the top of every handler. In Django you have seen both fixes. The copy-paste one is a helper like `get_pagination(request)` that every view calls first. The tidy one is a DRF authentication class: you list it once, and by the time the view runs `request.user` is already filled in, or the request has already been turned away with a `401`. An `axum` extractor is the second kind, with one difference. The handler doesn't call it. It names it in its signature, and `axum` calls it before the handler runs.

This lesson also pays off a promise from 3.2.1. That lesson told you `Json` has to be the last argument. Here you find out why, and the reason is a single trait choice.

---

## The concept

### A request is two pieces

```rust
let (parts, body) = request.into_parts();
println!("method:        {}", parts.method);
println!("api key:       {:?}", parts.headers.get("x-api-key"));
println!("api key again: {:?}", parts.headers.get("x-api-key"));
let bytes = to_bytes(body, 1024).await.unwrap();
```

`examples/01-parts-and-body.rs` splits a `POST` into its two pieces and reads each one:

```text
method:        POST
path:          /anime
query:         Some("page=2")
api key:       Some("secret-123")
api key again: Some("secret-123")
body:          {"title":"Frieren"}
```

`Parts` is the method, the URI, the headers and the extensions: plain data. You can look at it as often as you like, from as many places as you like. The body is different. It is a stream of bytes that may still be arriving, and `to_bytes(body, ...)` takes it *by value*. After that call the stream is gone. You cannot read it twice, because the bytes were not kept anywhere.

```senpai-visual
{"kind":"concept","labels":["a request arrives","Parts: method, uri, headers, extensions","Body: a stream you can read once","extractors on Parts: any number, any order","only the last extractor may take the Body"]}
```

### Two traits, because of that split

`axum` has one extractor trait for each piece (from `axum-core` 0.5.6, the version this course resolves, with the doc comments removed):

```rust
pub trait FromRequestParts<S>: Sized {
    type Rejection: IntoResponse;
    fn from_request_parts(parts: &mut Parts, state: &S)
        -> impl Future<Output = Result<Self, Self::Rejection>> + Send;
}
pub trait FromRequest<S, M = private::ViaRequest>: Sized {
    type Rejection: IntoResponse;
    fn from_request(req: Request, state: &S)
        -> impl Future<Output = Result<Self, Self::Rejection>> + Send;
}
```

That is the definition from `axum-core`'s source, so there is nothing to run. Compare the two arguments. `FromRequestParts` gets `&mut Parts`: it can look at the headers and the URI, but it cannot reach the body, because the body is not in `Parts`. `FromRequest` gets the whole `Request`, body included, *by value*. Whoever takes it owns the only copy of the stream.

So `Path`, `Query`, `State`, `HeaderMap` and `Method` implement `FromRequestParts`. `Json`, `String` and `Bytes` implement `FromRequest`. When `axum` calls your handler, it runs every argument but the last through `from_request_parts`, left to right, on the same `Parts`. It then puts `Parts` and the body back together into a `Request` and hands that to the last argument's `from_request`. There is no second request to give out. That is why only the last argument may read the body, and it is the real reason behind 3.2.1's "`Json` must be last" rule. The compiler enforces the rule by requiring `FromRequestParts` of every argument but the last, and `FromRequest` of the last one.

Two details make this work out. First, anything that implements `FromRequestParts` is also accepted in the last position, so an extractor of yours can go anywhere. Second, since 0.8 both traits use plain `async fn` in the trait, the same feature [2.9.4](../../../phase2-intermediate/09-async-in-practice/04-async-traits-and-blocking/README.md) showed you. The `#[async_trait]` attribute that older tutorials put on every `impl` is gone.

### Your first extractor

```rust
struct UserAgent(String);

impl<S: Send + Sync> FromRequestParts<S> for UserAgent {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let rejection = (StatusCode::BAD_REQUEST, "missing or unreadable user-agent header");
        let value = parts.headers.get("user-agent").ok_or(rejection)?;
        let text = value.to_str().map_err(|_| rejection)?;
        Ok(UserAgent(text.to_string()))
    }
}
```

The function looks inside `parts` and either builds a `UserAgent` or returns a rejection. `Rejection` is an associated type, as in [2.3.5](../../../phase2-intermediate/03-traits-and-generics/05-associated-types/README.md): every `impl` fixes it once. `rejection` is used twice because `(StatusCode, &'static str)` is `Copy`. The `<S: Send + Sync>` says the extractor works with any application state; this one never looks at the state, so it doesn't care which.

Now a handler asks for it in its signature, and the function body never touches a header:

```rust
async fn hello(UserAgent(agent): UserAgent) -> String {
    println!("  (the handler ran)");
    format!("hello, {agent}")
}
```

`examples/02-user-agent-extractor.rs` sends two requests through a `Router` with `oneshot`, the way 3.2.1 did:

```text
GET / with a user-agent header
  (the handler ran)
  -> 200 OK: hello, curl/8.9.1
GET / without one
  -> 400 Bad Request: missing or unreadable user-agent header
```

On the second request `(the handler ran)` never prints. The rejection became the response, and `hello` was never called. This is the DRF picture again: `AuthenticationFailed` raised inside an authentication class also means the view never starts. The difference to remember is *where* it is declared. A DRF view gets its classes from a class attribute or a global setting, and you can't tell from the view's signature which ones will run. An `axum` handler's extractors are its arguments, so the signature is the whole list, and the compiler checks it.

```senpai-visual
{"kind":"network","labels":["request","ApiKey: reads headers","Pagination: reads the query","handler runs","a rejection ends the request here"]}
```

### The rejection is a response

Every rejection above is a `(StatusCode, &'static str)`. The trait demands `type Rejection: IntoResponse`, and `axum` already implements `IntoResponse` for that pair: the status code becomes the status line and the string becomes the body. You can also use `StatusCode` on its own or a plain `&'static str`. Writing `IntoResponse` for your *own* error type, so a rich error becomes a JSON body, is the topic of [3.2.3](../03-anime-catalog-crud-in-memory/README.md). Until then a tuple is enough, and "Errors you will meet" shows what the compiler says when you skip it.

### Calling another extractor from yours

Your extractor doesn't have to parse everything itself. It can call an existing one on the same `parts` and then add rules of its own:

```rust
async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    let Query(params) = Query::<SearchParams>::from_request_parts(parts, state)
        .await
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid query string"))?;
    let term = params.q.unwrap_or_default().trim().to_string();
    if term.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "missing search term: add ?q=..."));
    }
    Ok(SearchTerm(term))
}
```

This is the body of `SearchTerm`'s `from_request_parts` in `examples/03-composing-query.rs`. `Query` does the parsing, `SearchTerm` adds "present and not blank". Four requests:

```text
/search?q=frieren    -> 200 OK: searching for "frieren"
/search?q=%20%20     -> 400 Bad Request: missing search term: add ?q=...
/search              -> 400 Bad Request: missing search term: add ?q=...
/search?q=a&q=b      -> 400 Bad Request: invalid query string
```

The last line shows the two layers. A repeated `q` is `Query`'s own failure, which `SearchTerm` turned into its own message with `map_err`. The blank `q` passes `Query` happily and fails `SearchTerm`'s rule. That is exactly what `Pagination` does in the exercises below, with defaults and a cap in place of a blank check.

### Making an extractor optional

Sometimes the header is allowed to be missing. The tool that works for *every* extractor is `Result<T, T::Rejection>`: the extractor still runs, but its failure arrives in the handler as an `Err` instead of ending the request.

```rust
async fn greet(agent: Result<UserAgent, (StatusCode, &'static str)>) -> String {
    match agent {
        Ok(UserAgent(agent)) => format!("hello, {agent}"),
        Err((_, reason)) => format!("hello, whoever you are ({reason})"),
    }
}
```

`examples/04-optional-with-result.rs` runs it with and without the header:

```text
200 OK: hello, curl/8.9.1
200 OK: hello, whoever you are (missing or unreadable user-agent header)
```

`Option<UserAgent>` would read better, and in `axum` 0.7 it worked for every extractor. In 0.8 it doesn't. `axum-core`'s changelog for 0.5.0 says it: `Option<T>` as an extractor now requires `T` to implement a second trait, `OptionalFromRequestParts` (or `OptionalFromRequest` for a body extractor). The reason is that "extraction failed" can mean two different things, "the header is not there" and "the header is there but garbage", and a blanket `Option` had to treat both as `None`. Now each extractor says what `None` means for itself. "Errors you will meet" shows what happens when you write `Option<UserAgent>` anyway. Some built-in extractors, such as `MatchedPath`, implement the new trait. Yours doesn't unless you write that `impl`, and `Result` is usually the quicker answer.

### Why the `S: Send + Sync`

The `S` in `FromRequestParts<S>` is the application state from `.with_state(...)`. An extractor that doesn't use state should accept any `S`, which is why the `impl` is generic. `FromRequestParts` demands that the future you return is `Send`, because `axum` may move it to another thread. An `async fn` keeps all its arguments alive inside that future, including `&S`, and a `&S` is only `Send` when `S: Sync`. That is the whole reason for the bound. Leave it out and the compiler tells you, in the last error below. An extractor that *does* use the state (to check a key against a list, say) puts a stronger bound on `S`. That's the Challenge.

---

## Hands on

```sh
cargo run -p p3-02-02-writing-your-own-extractor --example 01-parts-and-body
cargo run -p p3-02-02-writing-your-own-extractor --example 02-user-agent-extractor
cargo run -p p3-02-02-writing-your-own-extractor --example 03-composing-query
cargo run -p p3-02-02-writing-your-own-extractor --example 04-optional-with-result
```

Then the four broken ones, each behind the `broken` feature:

```sh
cargo build -p p3-02-02-writing-your-own-extractor --example 05-rejection-not-into-response-broken --features broken
cargo build -p p3-02-02-writing-your-own-extractor --example 06-option-extractor-broken --features broken
cargo build -p p3-02-02-writing-your-own-extractor --example 07-missing-send-sync-bound-broken --features broken
cargo build -p p3-02-02-writing-your-own-extractor --example 08-body-extractor-not-last-broken --features broken
```

Then try these:

1. In `02-user-agent-extractor`, add a third request with `user-agent` set to the empty string. Is that a rejection? Change the extractor so it is.
2. In `03-composing-query`, make the extractor lowercase the term. Which request lines change?
3. In `04-optional-with-result`, print the status code that the rejection carried. What is it?

---

## Errors you will meet

Until you finish the Implement exercise, `cargo` also prints `unused` warnings (from the `todo!()` bodies in `src/lib.rs`) above every error below. They are left out here, so the output starts at the error itself.

### `E0277` — the rejection type isn't a response

```text
error[E0277]: the trait bound `UserAgentError: IntoResponse` is not satisfied
  --> phase3-backend-foundations\02-axum-and-rest-api-design\02-writing-your-own-extractor\examples\05-rejection-not-into-response-broken.rs:20:22
   |
20 |     type Rejection = UserAgentError;
   |                      ^^^^^^^^^^^^^^ unsatisfied trait bound
   |
help: the trait `IntoResponse` is not implemented for `UserAgentError`
  --> phase3-backend-foundations\02-axum-and-rest-api-design\02-writing-your-own-extractor\examples\05-rejection-not-into-response-broken.rs:12:1
   |
12 | enum UserAgentError {
   | ^^^^^^^^^^^^^^^^^^^
   = help: the following other types implement trait `IntoResponse`:
             &'static [u8; N]
             &'static [u8]
             &'static str
             ()
             (R,)
             (Response<()>, R)
             (Response<()>, T1, R)
             (Response<()>, T1, T2, R)
           and 120 others
note: required by a bound in `axum::extract::FromRequestParts::Rejection`
  --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-core-0.5.6\src\extract\mod.rs:56:21
   |
56 |     type Rejection: IntoResponse;
   |                     ^^^^^^^^^^^^ required by this bound in `FromRequestParts::Rejection`
   = note: the full name for the type has been written to 'M:\SenPai-Rust-Journey\target\debug\examples\05_rejection_not_into_response_broken.long-type-10887280357418551112.txt'
   = note: consider using `--verbose` to print the full type name to the console

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-02-02-writing-your-own-extractor` (example "05-rejection-not-into-response-broken") due to 1 previous error
```

(The number in the `long-type-` file name differs on your machine, and on every build.)

**What the compiler is objecting to:** the example's `Rejection` is a plain `enum` it made up. The last lines point at the rule: the trait declares `type Rejection: IntoResponse`, so whatever you pick has to be something `axum` can turn into a response. The list of types that qualify is long, and an `enum` of yours isn't on it.

**The fix:** use a type that already qualifies:

```rust
type Rejection = (StatusCode, &'static str);
```

**Why this is the fix:** `axum` has to send *something* back when an extractor fails, and it can only do that for a type it knows how to convert. A tuple of a status code and text is already on that list. Giving your own error enum an `IntoResponse` implementation is the better long-term answer, and 3.2.3 teaches it.

### `E0277` — `Option<UserAgent>` in `axum` 0.8

```text
error[E0277]: the trait bound `fn(Option<UserAgent>) -> impl Future<Output = String> {greet}: Handler<_, _>` is not satisfied
   --> phase3-backend-foundations\02-axum-and-rest-api-design\02-writing-your-own-extractor\examples\06-option-extractor-broken.rs:38:53
    |
 38 |     let _app: Router = Router::new().route("/", get(greet));
    |                                                 --- ^^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(Option<UserAgent>) -> impl Future<Output = String> {greet}`
    |                                                 |
    |                                                 required by a bound introduced by this call
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
error: could not compile `p3-02-02-writing-your-own-extractor` (example "06-option-extractor-broken") due to 1 previous error
```

**What the compiler is objecting to:** this error does not name the missing trait. It says the function `greet` is not a `Handler`, and points at the line where you passed it to `get`. A function is a handler only if every argument is an extractor, and `Option<UserAgent>` is not one: `axum`'s `Option<T>` implementation requires `T: OptionalFromRequestParts`, and `UserAgent` has only `FromRequestParts`. You don't read that off the screen. You get it from knowing that the one new thing in the handler is the `Option`.

**The fix:** ask for `Result`, which works for every extractor, as in `examples/04-optional-with-result.rs`:

```rust
async fn greet(agent: Result<UserAgent, (StatusCode, &'static str)>) -> String {
```

**Why this is the fix:** `Result<T, T::Rejection>` has a blanket implementation, so it needs nothing from `UserAgent`. If you want the `Option` spelling, implement `OptionalFromRequestParts` for `UserAgent` and decide what "no header" means yourself.

### `E0277` — a body extractor that isn't last

```text
error[E0277]: the trait bound `fn(String, Method) -> impl Future<Output = String> {upload}: Handler<_, _>` is not satisfied
   --> phase3-backend-foundations\02-axum-and-rest-api-design\02-writing-your-own-extractor\examples\08-body-extractor-not-last-broken.rs:17:54
    |
 17 |     let _app: Router = Router::new().route("/", post(upload));
    |                                                 ---- ^^^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(String, Method) -> impl Future<Output = String> {upload}`
    |                                                 |
    |                                                 required by a bound introduced by this call
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
error: could not compile `p3-02-02-writing-your-own-extractor` (example "08-body-extractor-not-last-broken") due to 1 previous error
```

**What the compiler is objecting to:** the same message as the last error, with a different cause. `String` reads the body, so it implements `FromRequest` only. In the `Handler` implementation every argument except the last must be `FromRequestParts`, and `String` is first.

**The fix:** move the body extractor to the end:

```rust
async fn upload(method: Method, body: String) -> String {
```

**Why this is the fix:** the earlier arguments take their pieces from `Parts`, then the last one takes the whole request, body included. This is the 3.2.1 rule, and now you know what enforces it.

### No error code — the future can't be sent between threads

```text
error: future cannot be sent between threads safely
  --> phase3-backend-foundations\02-axum-and-rest-api-design\02-writing-your-own-extractor\examples\07-missing-send-sync-bound-broken.rs:18:67
   |
18 |     async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
   |                                                                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ future returned by `from_request_parts` is not `Send`
   |
note: captured value is not `Send` because `&` references cannot be sent unless their referent is `Sync`
  --> phase3-backend-foundations\02-axum-and-rest-api-design\02-writing-your-own-extractor\examples\07-missing-send-sync-bound-broken.rs:18:52
   |
18 |     async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
   |                                                    ^^^^^^ has type `&S` which is not `Send`, because `S` is not `Sync`
note: required by a bound in `FromRequestParts::from_request_parts::{anon_assoc#0}`
  --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-core-0.5.6\src\extract\mod.rs:62:64
   |
62 |     ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send;
   |                                                                ^^^^ required by this bound in `FromRequestParts::from_request_parts::{anon_assoc#0}`
help: consider restricting type parameter `S` with trait `Sync`
   |
15 | impl<S: std::marker::Sync> FromRequestParts<S> for UserAgent {
   |       +++++++++++++++++++

error: could not compile `p3-02-02-writing-your-own-extractor` (example "07-missing-send-sync-bound-broken") due to 1 previous error
```

There is no error code here, so there is nothing to put in the example's `expected:` line but the sentence. The example's header says so.

**What the compiler is objecting to:** the trait promises a `+ Send` future. An `async fn` holds all its arguments, so the future holds `&S`, and `&S` is `Send` only if `S: Sync`. This `impl<S>` says nothing about `S`.

**The fix:** `rustc` suggests `S: Sync`. Use `Send + Sync`, the bound `axum`'s own documentation puts on its custom extractors:

```rust
impl<S: Send + Sync> FromRequestParts<S> for UserAgent {
```

**Why this is the fix:** it states the one thing the future needs: that sharing a reference to the state across threads is safe. It costs the caller nothing, because every real application state already satisfies it.

---

## Exercises

### Warm up

<details>
<summary>Can a handler have two extractors that both read the request body?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No. The body is a stream that can be read once. Every extractor that reads it implements `FromRequest` and takes the whole request by value, so at most one can exist, and `axum` insists it is the last argument. You get a compile error otherwise.

</details>

<details>
<summary>An <code>ApiKey</code> extractor finds no <code>x-api-key</code> header and rejects with <code>401</code>. Does the handler run?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No. `axum` turns the rejection into the response and returns it. The handler is never called, and the extractors after `ApiKey` in the argument list never run.

</details>

<details>
<summary>An extractor reads only the <code>authorization</code> header. <code>FromRequestParts</code> or <code>FromRequest</code>?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`FromRequestParts`. Headers live in `Parts`, and nothing here needs the body. Choosing `FromRequest` would force it to be the last argument for no reason.

</details>

<details>
<summary>In <code>axum</code> 0.8, does <code>async fn f(key: Option&lt;ApiKey&gt;)</code> compile when <code>ApiKey</code> only implements <code>FromRequestParts</code>?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No. `Option<T>` as an extractor needs `T: OptionalFromRequestParts`. Write `Result<ApiKey, (StatusCode, &'static str)>` instead, or implement the optional trait.

</details>

### Repair

Fix the four broken examples. Keep building them with `--features broken`; each should compile once fixed, and each fix is a line or two:

1. `examples/05-rejection-not-into-response-broken.rs`: use a rejection type `axum` can send.
2. `examples/06-option-extractor-broken.rs`: make the handler compile, still treating a missing header as "no user agent".
3. `examples/07-missing-send-sync-bound-broken.rs`: fix the `impl` header.
4. `examples/08-body-extractor-not-last-broken.rs`: fix the argument order.

### Implement

Two extractors in `src/lib.rs`:

```sh
cargo test -p p3-02-02-writing-your-own-extractor
```

Every `impl` is fully specified in its doc comment, so you don't need to read the tests:

- `ApiKey`: the `x-api-key` header, trimmed, or a `401`.
- `Pagination`: `page` and `per_page` from the query string, with defaults, a cap, and two kinds of `400`. It is meant to call `Query` the way `SearchTerm` did.

The router `app()` and its two handlers are already written. Before you start, all fourteen tests fail with `not yet implemented`. `ApiKey` and `Pagination` make eleven of them pass; the last three belong to the next rung. One test, `extractors_run_left_to_right_so_the_key_is_checked_first`, checks the order in which extractors run, which you saw above.

### Build

`ClientVersion` in the same file: a `MAJOR.MINOR` version from the `x-client-version` header, such as `1.4`. It has its own doc comment with two different `400` messages, and its tests are the three in `client_version`. Think about what counts as "exactly two whole numbers" before you start: `1.2.3`, `1.` and `-1.2` are all rejected.

### Challenge (optional)

`ApiKey` accepts any key. Write an `Authorized` extractor that checks the key against a set of known keys kept in the application state, and rejects an unknown key with `403`, while a missing key stays `401`. It must work with any state that can *produce* a `KeyStore`, which means a `KeyStore: FromRef<S>` bound on the `impl`, not a concrete state type. `FromRef` is in `axum::extract`. The simplest route is to call `ApiKey`'s own extractor from inside yours. `State` and where it lives get a proper treatment in [3.4.2](../../04-configuration-and-app-structure/02-app-state-and-dependency-wiring/README.md), so the lesson only asks you to try it.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `FromRequestParts` | the trait for extractors that read only method, URI, headers and extensions | every extractor that isn't a body reader |
| `FromRequest` | the trait for extractors that take the whole request, body included | `Json`, `String`, `Bytes`; at most one, and last |
| `Parts` | the request without its body: plain, shareable data | the first argument of `from_request_parts` |
| rejection | the error type an extractor returns; it has to be a response | `(StatusCode, &'static str)` until 3.2.3 |
| composing extractors | calling one extractor from inside another on the same `parts` | `Pagination` over `Query` |
| `Result<T, T::Rejection>` | an extractor whose failure arrives in the handler instead of ending the request | optional headers, custom error pages |
| `OptionalFromRequestParts` | the 0.8 trait that makes `Option<T>` mean something for `T` | optional extractors that read `Parts` |

### What you now know

- A request is `Parts` (readable any number of times) and a body (readable once). `FromRequestParts` is for the first, `FromRequest` for the second.
- "`Json` must be last" is not a style rule. The body is consumed by whoever reads it, so only one extractor can, and the last one is the one that gets the request.
- Writing an extractor means choosing a `Rejection`, writing an `async fn from_request_parts`, and keeping the `S: Send + Sync` bound.
- A rejection is turned into the response and the handler never runs. Extractors run left to right.
- In 0.8, `Option<T>` needs `OptionalFromRequestParts`. `Result<T, T::Rejection>` works for everything.

### What comes back later

- **Turning your own error type into an HTTP response with `IntoResponse`** — [3.2.3 — Anime catalog CRUD (in-memory)](../03-anime-catalog-crud-in-memory/README.md)
- **Code that runs before *every* route, not only the ones that ask for an extractor** — [3.2.4 — `tower::Service` / `Layer`: middleware by hand](../04-tower-service-and-layer-middleware/README.md)
- **Application state and the `FromRef` wiring behind a state-aware extractor** — [3.4.2 — Application state and dependency wiring](../../04-configuration-and-app-structure/02-app-state-and-dependency-wiring/README.md)
- **Offset versus keyset pagination, the real design behind `?page=`** — [3.6.2 — Pagination: offset vs. keyset](../../06-database-design-and-query-performance/02-pagination/README.md)
- **Authenticating a request with a JWT instead of a fixed key** — [3.7.3 — JWTs and `tower` middleware](../../07-auth-and-security/03-jwt-and-tower-middleware/README.md)
- **One shape for every error body your API sends** — [3.8.1 — Consistent error envelopes](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)

### Can you explain?

- Why can a handler have six `FromRequestParts` extractors but only one `FromRequest` extractor?
- What does `axum` do with an extractor's `Rejection`, and does the handler run?
- Why does the `impl` need `S: Send + Sync` even when the extractor never touches the state?
- What changed about `Option<T>` as an extractor between `axum` 0.7 and 0.8, and what is the quick workaround?
- How is an `axum` extractor like a DRF authentication class, and where does the comparison stop?

---

## Going further

- [`axum::extract`](https://docs.rs/axum/0.8/axum/extract/index.html): the official page on extractors, including "Defining custom extractors" and "Optional extractors".
- [`FromRequestParts`](https://docs.rs/axum/0.8/axum/extract/trait.FromRequestParts.html) and [`OptionalFromRequestParts`](https://docs.rs/axum/0.8/axum/extract/trait.OptionalFromRequestParts.html): the two traits from this lesson, as documented.
- [`axum-core`'s changelog](https://docs.rs/crate/axum-core/0.5.6/source/CHANGELOG.md): the 0.5.0 entry where `#[async_trait]` was removed and `Option<T>` changed.
- [DRF — Authentication](https://www.django-rest-framework.org/api-guide/authentication/): the Django-side picture this lesson compared against.
