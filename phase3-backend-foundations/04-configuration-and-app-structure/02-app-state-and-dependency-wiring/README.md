# 3.4.2 — Application state and dependency wiring

## At a glance

After this lesson you can:

- Build an `AppState` that is cheap to clone because every shared piece sits behind an `Arc`, and explain why a clone shares the data instead of copying it.
- Hand each handler only the piece of state it needs with `#[derive(FromRef)]` and `State<Piece>`, and read the errors you get when the wiring is wrong.
- Put a dependency such as a clock behind a trait, choose between a concrete type, a generic and an `Arc<dyn Trait>`, and test the whole app with a fake clock through `oneshot`.
- Decide, with reasons, whether a given dependency deserves a trait at all.

**Time:** ~100 minutes · **Prerequisites:**
[3.2.1 — Routing, handlers, extractors](../../02-axum-and-rest-api-design/01-routing-handlers-extractors/README.md),
[3.2.3 — Anime catalog CRUD (in-memory)](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.md),
[2.3.7 — Static vs dynamic dispatch](../../../phase2-intermediate/03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md),
[2.8.4 — `Send` and `Sync`](../../../phase2-intermediate/08-concurrency/04-send-and-sync/README.md)

---

## Why this matters

Every service has things that many handlers need and nobody should rebuild per request: the store, the settings, the clock, later a database pool and a mailer. 3.2.1 passed one `Arc<Mutex<..>>` through `State`, and 3.2.3 a store. That scales to one dependency. By the time you have five, you need a rule for where they live, how a handler asks for one, and how a test swaps the real one for a fake.

Django never makes you decide. A view just writes `from django.conf import settings` or calls `timezone.now()`: the dependency is a module-level import, found by name. That is convenient, and it works because Python lets a test reach in and replace the name afterwards (`mock.patch("app.views.timezone")`, or `freezegun`). Rust has no such back door. A function that calls `SystemTime::now()` will call it in every test, forever, and no test can say "pretend it is 2030". If you want to control time, the clock has to arrive *through the function's inputs*. That is all "dependency injection" means here: not a framework, just a parameter.

This lesson builds that parameter once, on a small watch log (`POST /watch`, `GET /watch/recent`) whose answers depend on the time. The same shape carries the database pool in module 5. And it asks the question the Java world rarely asks: when is this worth doing, and when is it ceremony?

---

## The concept

### One struct, cloned for every request

`axum` hands every request its own copy of the state. So the state type must be `Clone`, and cloning must be cheap. The way to get both is the rule from 3.2.3: put each shared thing behind an `Arc`, so a clone copies a pointer and bumps a counter. `examples/01-clone-shares-the-store.rs` shows it with `Arc::strong_count`:

```rust
#[derive(Clone, Default)]
struct AppState {
    titles: Arc<Mutex<Vec<String>>>,
}
// main: let per_request = state.clone();
//       per_request.titles.lock().unwrap().push("Frieren".to_string());
```

```text
handles after creating the state: 1
handles after one clone:          2
seen through the original:        ["Frieren"]
same allocation:                  true
handles after the clone is gone:  1
```

```senpai-visual
{"kind":"ownership","labels":["AppState (original)","AppState (clone for request 1)","AppState (clone for request 2)","one Arc allocation: the store","strong count = 3"]}
```

A clone of `AppState` is a new struct whose `Arc` fields point at the same allocation. Nothing behind the pointer is copied, and a push through one handle is visible through every other. When the request finishes and its clone is dropped, the count goes back down. The store is shared because the *pointer* is, and the `Mutex` inside (3.2.3, 2.8.1) is what makes mutating it through a shared handle safe.

A plain field that is not behind an `Arc` is different: it is copied on every clone. For a small `Config` of two integers that is nothing. For a large struct it is a real cost, and the fix is the same: wrap it in an `Arc`.

### Asking for one piece: `FromRef` and `State<Piece>`

A handler can take the whole state, `State<AppState>`, and reach into it. That works, but then every handler's signature says "I might use anything", and a test of one handler has to build all of it. `axum` has a better tool: `FromRef`. A type implements `FromRef<AppState>` when it can be taken out of an `&AppState`, and then `State<ThatType>` works in a handler even though the router's state is `AppState`. The derive writes those impls for you, one per field. `examples/02-fromref-substates.rs`:

```rust
#[derive(Clone, FromRef)]
struct AppState {
    greeting: Greeting,
    hits: Arc<AtomicU32>,
}

async fn count(State(hits): State<Arc<AtomicU32>>) -> String { /* ... */ }
async fn hello(State(greeting): State<Greeting>) -> String { /* ... */ }
// .route("/count", get(count)).route("/hello", get(hello)).with_state(state)
```

```text
GET /hello -> konnichiwa
GET /count -> hit number 1
GET /count -> hit number 2
GET /both -> konnichiwa (hits so far: 2)
```

`/both` takes two `State` arguments at once. Each one is pulled from the same `AppState` by its own `FromRef` impl. `State` is an extractor like the ones in 3.2.1 (it just never fails), so a handler can have as many as it likes. In the real `axum` 0.8.9 source the rule is one impl: `State<Inner>` is `FromRequestParts<Outer>` whenever `Inner: FromRef<Outer>`. And `axum-core` has one blanket impl, `impl<T: Clone> FromRef<T> for T`. That blanket impl is why plain `State<AppState>` has always worked: a state can always be taken from itself.

Two consequences you will meet in "Errors you will meet". The derive keys on the field's *type*, so two fields of the same type collide. And `FromRef` *clones* the field out: an `Arc` field gives you a cheap handle, a `Config` field gives you a fresh copy of the config.

3.2.2's challenge used the other side of the same trait: an extractor generic over any state `S` with a `KeyStore: FromRef<S>` bound, which lets a library extractor work in an app it has never seen. The derive here is how an app says "yes, I can produce a `KeyStore`".

### A dependency behind a trait: the `Clock`

The watch log needs "now". Here is the dependency, as a trait:

```rust
pub trait Clock: Send + Sync {
    fn now(&self) -> u64; // seconds since the Unix epoch
}
```

`Send + Sync` are supertraits (2.3.6) on purpose. The state is shared across the worker threads that run handlers, so everything inside it must be `Send + Sync`. If the trait does not promise it, `Arc<dyn Clock>` does not have it either ("Errors you will meet" shows the message). Two implementations exist: `SystemClock` reads the operating system, and `FakeClock` is a number that only moves when a test says so. The state holds `Arc<dyn Clock>`, the store holds only data, and the handlers get both through `State`:

```rust
#[derive(Clone, FromRef)]
pub struct AppState {
    pub clock: Arc<dyn Clock>,
    pub store: Arc<WatchStore>,
    pub config: Config,
}
```

The store does not know about the clock: `store.add(title, at)` is told when. That keeps the store a plain data structure with trivial tests (`tests/store_test.rs` needs no clock and no HTTP), and it keeps the one decision "what time is it" in one place, the handler.

```senpai-visual
{"kind":"concept","labels":["main: SystemClock","test: FakeClock","Arc of dyn Clock in AppState","handler reads State of Arc dyn Clock","same handler, either clock"]}
```

### Concrete, generic or trait object?

A clock can be wired three ways. `examples/03-three-ways-to-hold-a-clock.rs` builds all three:

```text
concrete: 100
its type: 03_three_ways_to_hold_a_clock::ConcreteState
generic:  200
its type: 03_three_ways_to_hold_a_clock::GenericState<03_three_ways_to_hold_a_clock::Fixed>
dyn:      300
its type: 03_three_ways_to_hold_a_clock::DynState
```

The behaviour is identical. What differs is the type, which is the thing that spreads through your code:

| | Concrete (`Arc<SystemClock>`) | Generic (`AppState<C: Clock>`) | Trait object (`Arc<dyn Clock>`) |
|---|---|---|---|
| Choice made | at compile time, fixed | at compile time, per instantiation | at run time |
| The state's type | `AppState` | `AppState<Fixed>`, `AppState<SystemClock>` | `AppState` |
| Swap in a test? | no | yes | yes |
| Cost | none | none per call; one copy of the code per type | one pointer hop per call (a vtable call) |
| What it infects | nothing | every function, handler and `app()` that names the state gets a `<C>` and a bound | nothing: the type is still `AppState` |

For a dependency that does I/O or reads the clock, the pointer hop is unmeasurable next to the work around it, so the trait object wins on simplicity. A generic parameter on the state is the right tool when the call is in a hot loop, or when you want the compiler to see through the call. Two things worth knowing about the generic state. `#[derive(FromRef)]` refuses it outright (the macro reports "`#[derive(FromRef)]` doesn't support generics"), so you write each `FromRef` impl by hand. And `#[derive(Clone)]` on `AppState<C>` adds a `C: Clone` bound that `Arc<C>` never needed. The trait object has neither paper cut. This lesson uses `Arc<dyn Clock>`, and the Challenge lets you feel the generic version yourself.

### Testing with a fake clock

With the clock injected, a test builds the app around a `FakeClock` and keeps its own handle to it. Because `FakeClock` uses interior mutability (an atomic, all methods `&self`), the test can move time *after* the app has been built:

```rust
fn fixture(start: u64, config: Config) -> (Router, Arc<FakeClock>) {
    let clock = Arc::new(FakeClock::at(start));
    let state = AppState::new(clock.clone(), config);
    (app(state), clock)
}
// in a test:
//   post "first", clock.advance(60), post "second", clock.advance(30)
//   GET /watch/recent  ->  ["first", "second"]
//   clock.advance(20); GET /watch/recent  ->  ["second"]
```

The test never sleeps: "an hour later" is one `advance(3600)`. And it can reach every boundary exactly (an entry exactly one window old still counts; one second more and it is gone), which a real clock would only hit by luck. A gap that Django covers with `freezegun`, Rust covers by construction, at the price of passing the clock in. Where the analogy stops: `freezegun` changes time for *all* code in the process, including libraries. Here only the code that was handed this clock sees it, and that is a feature (nothing is faked behind your back) and a limit (a library that calls `SystemTime::now()` itself stays real).

### When injection earns its keep, and when it is ceremony

Opinionated, so you can disagree with reasons. Put a dependency behind a trait when at least one of these is true:

1. **It is not deterministic**: the clock, a random number generator, an id generator. A test cannot assert on something that changes.
2. **It talks to the outside world**: a mailer, a payment gateway, a webhook caller, another service over HTTP. Tests must not send real emails, and a failure case ("the gateway is down") must be producible on demand.
3. **It has two real implementations**: an in-memory store and a Postgres one. This is where a repository trait pays off, and it is the topic of [3.5.5 — The repository pattern](../../05-postgres-and-sqlx/05-repository-pattern/README.md).

And do not, when:

- **A trait would have exactly one implementation that no test ever fakes.** The trait is pure ceremony: one more file, one more name, one more indirection to read through for a seam nobody uses. Add the trait the first time a test needs to fake the thing, not before. Adding it later is a mechanical change; removing a speculative one costs the same, and you pay for the speculation every day until then.
- **The dependency is pure logic.** A function that validates a title, a function that sums prices. Call it. Faking it tests your fake.
- **You are reaching for a container.** There are "dependency injection" crates. This lesson's whole mechanism is one struct, one `Arc` per field, and one derive. If your wiring does not fit on a screen, the application has a structure problem that a container would only hide.

That is why the watch log's *store* is a concrete struct, not a trait: in-memory is the real thing for now, and no test needs a second one. The clock is a trait because rule 1 applies. When 3.5.5 brings in Postgres, the store becomes the second kind (rule 3), and only then.

---

## Hands on

Run the three examples that compile. `04` to `07` are broken on purpose and sit behind the `broken` feature; "Errors you will meet" shows each of them. (The outputs of `01`, `02` and `03` are the blocks in "The concept".)

```sh
cargo run -p p3-04-02-app-state-and-dependency-wiring --example 01-clone-shares-the-store
cargo run -p p3-04-02-app-state-and-dependency-wiring --example 02-fromref-substates
cargo run -p p3-04-02-app-state-and-dependency-wiring --example 03-three-ways-to-hold-a-clock
```

The tests start out red. `src/lib.rs` has the whole skeleton, and every `todo!()` has a doc comment saying exactly what it must do. Check where you stand:

```sh
for t in clock_test store_test api_test; do cargo test -p p3-04-02-app-state-and-dependency-wiring --test $t 2>&1 | grep 'test result'; done
```

```text
test result: FAILED. 0 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: FAILED. 0 passed; 8 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: FAILED. 0 passed; 12 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

When all three are green, run the real server. It listens on `127.0.0.1:3150` and wires `SystemClock` (`src/main.rs` is the only place the real dependencies are chosen). These transcripts were captured against the solution:

```sh
cargo run -p p3-04-02-app-state-and-dependency-wiring &
curl -s -w '\n%{http_code}\n' -X POST -H 'content-type: application/json' -d '{"title":"  Frieren  "}' http://127.0.0.1:3150/watch
curl -s -w '\n%{http_code}\n' -X POST -H 'content-type: application/json' -d '{"title":"   "}' http://127.0.0.1:3150/watch
curl -s http://127.0.0.1:3150/watch/recent
```

```text
{"id":1,"title":"Frieren","watched_at":1791112811}
201
invalid title
422
[{"id":1,"title":"Frieren","watched_at":1791112811}]
```

(`watched_at` is the real time of the request, so your number differs.) The title was trimmed before it was stored, the blank one was rejected with `422`, and the second request saw the entry the first one made: both handlers hold clones of the same state. Stop the server with `kill %1`. Then try these:

1. In `src/main.rs`, swap `SystemClock` for a `FakeClock::at(0)`. What does `watched_at` say now, and which other file did you not have to touch?
2. Give `Config` a `max_title_len` of `5` and post a six-character title. Which handler decides, and where did it get the number?
3. Change `recent_watches` to take `State<AppState>` instead of three separate `State`s. Does it still compile? What did the signature stop telling you?

---

## Errors you will meet

Each broken example below also prints a handful of `unused variable` warnings from the unfinished skeleton in `src/lib.rs` before the error. They disappear once you implement it, and are left out of the transcripts.

### `E0277` — the state is not `Clone`

```rust
struct AppState {
    visits: Arc<Mutex<u64>>,
}
```

`examples/04-state-not-clone-broken.rs` forgets `#[derive(Clone)]`:

```text
error[E0277]: the trait bound `fn(State<AppState>) -> impl Future<Output = String> {visits}: Handler<_, _>` is not satisfied
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\04-state-not-clone-broken.rs:27:31
    |
 27 |         .route("/visits", get(visits))
    |                           --- ^^^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(State<AppState>) -> impl Future<Output = String> {visits}`
    |                           |
    |                           required by a bound introduced by this call
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

error[E0277]: the trait bound `AppState: Clone` is not satisfied
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\04-state-not-clone-broken.rs:26:24
    |
 26 |     let _app: Router = Router::new()
    |                        ^^^^^^^^^^^^^ the trait `Clone` is not implemented for `AppState`
    |
note: required by a bound in `Router::<S>::new`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:140:8
    |
140 |     S: Clone + Send + Sync + 'static,
    |        ^^^^^ required by this bound in `Router::<S>::new`
...
146 |     pub fn new() -> Self {
    |            --- required by a bound in this associated function
help: consider annotating `AppState` with `#[derive(Clone)]`
    |
 14 + #[derive(Clone)]
 15 | struct AppState {
    |

error[E0277]: the trait bound `AppState: Clone` is not satisfied
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\04-state-not-clone-broken.rs:27:10
    |
 27 |         .route("/visits", get(visits))
    |          ^^^^^ the trait `Clone` is not implemented for `AppState`
    |
note: required by a bound in `Router::<S>::route`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:140:8
    |
140 |     S: Clone + Send + Sync + 'static,
    |        ^^^^^ required by this bound in `Router::<S>::route`
...
178 |     pub fn route(self, path: &str, method_router: MethodRouter<S>) -> Self {
    |            ----- required by a bound in this associated function
help: consider annotating `AppState` with `#[derive(Clone)]`
    |
 14 + #[derive(Clone)]
 15 | struct AppState {
    |

error[E0277]: the trait bound `AppState: Clone` is not satisfied
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\04-state-not-clone-broken.rs:27:27
    |
 27 |         .route("/visits", get(visits))
    |                           ^^^^^^^^^^^ the trait `Clone` is not implemented for `AppState`
    |
note: required by a bound in `axum::routing::get`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\method_routing.rs:169:16
    |
169 |             S: Clone + Send + Sync + 'static,
    |                ^^^^^ required by this bound in `get`
...
441 | top_level_handler_fn!(get, GET);
    | -------------------------------
    | |                     |
    | |                     required by a bound in this function
    | in this macro invocation
    = note: this error originates in the macro `top_level_handler_fn` (in Nightly builds, run with -Z macro-backtrace for more info)
help: consider annotating `AppState` with `#[derive(Clone)]`
    |
 14 + #[derive(Clone)]
 15 | struct AppState {
    |

error[E0277]: the trait bound `AppState: Clone` is not satisfied
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\04-state-not-clone-broken.rs:28:10
    |
 28 |         .with_state(state);
    |          ^^^^^^^^^^ the trait `Clone` is not implemented for `AppState`
    |
note: required by a bound in `Router::<S>::with_state`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:140:8
    |
140 |     S: Clone + Send + Sync + 'static,
    |        ^^^^^ required by this bound in `Router::<S>::with_state`
...
408 |     pub fn with_state<S2>(self, state: S) -> Router<S2> {
    |            ---------- required by a bound in this associated function
help: consider annotating `AppState` with `#[derive(Clone)]`
    |
 14 + #[derive(Clone)]
 15 | struct AppState {
    |

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-04-02-app-state-and-dependency-wiring` (example "04-state-not-clone-broken") due to 5 previous errors

```

**What the compiler is objecting to:** `axum` copies the state for every request, so `Router` needs `S: Clone + Send + Sync + 'static`. The state has no `Clone`. The compiler prints five errors. The first is the vague "not a `Handler`" again, because the handler mentions `State<AppState>`. The other four are the same complaint, `AppState: Clone` is not satisfied, once at each method that carries the bound (`Router::new`, `route`, `get`, `with_state`), and the `help:` line under each is the whole answer.

**The fix:** `#[derive(Clone)]` on `AppState`.

**Why this is the fix:** the derive clones each field in turn, and each field is an `Arc`, whose clone is the cheap pointer copy from the start of the lesson. Note what this does not say: it does not say the *data* is `Clone`. `Mutex<u64>` is not, and does not need to be.

### `E0277` — `dyn Trait` without `Send + Sync`

```rust
trait Clock {
    fn now(&self) -> u64;
}
```

`examples/05-dyn-without-send-sync-broken.rs` has an `Arc<dyn Clock>` in its state, but the trait makes no promise about threads:

```text
error[E0277]: the trait bound `fn(State<AppState>) -> impl Future<Output = String> {now}: Handler<_, _>` is not satisfied
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\05-dyn-without-send-sync-broken.rs:39:56
    |
 39 |     let _app: Router = Router::new().route("/now", get(now)).with_state(state);
    |                                                    --- ^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(State<AppState>) -> impl Future<Output = String> {now}`
    |                                                    |
    |                                                    required by a bound introduced by this call
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

error[E0277]: `(dyn Clock + 'static)` cannot be shared between threads safely
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\05-dyn-without-send-sync-broken.rs:39:24
    |
 39 |     let _app: Router = Router::new().route("/now", get(now)).with_state(state);
    |                        ^^^^^^^^^^^^^ `(dyn Clock + 'static)` cannot be shared between threads safely
    |
    = help: the trait `Sync` is not implemented for `(dyn Clock + 'static)`
    = note: required for `Arc<(dyn Clock + 'static)>` to implement `Send`
note: required because it appears within the type `AppState`
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\05-dyn-without-send-sync-broken.rs:19:8
    |
 19 | struct AppState {
    |        ^^^^^^^^
note: required by a bound in `Router::<S>::new`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:140:16
    |
140 |     S: Clone + Send + Sync + 'static,
    |                ^^^^ required by this bound in `Router::<S>::new`
...
146 |     pub fn new() -> Self {
    |            --- required by a bound in this associated function

error[E0277]: `(dyn Clock + 'static)` cannot be sent between threads safely
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\05-dyn-without-send-sync-broken.rs:39:24
    |
 39 |     let _app: Router = Router::new().route("/now", get(now)).with_state(state);
    |                        ^^^^^^^^^^^^^ `(dyn Clock + 'static)` cannot be sent between threads safely
    |
    = help: the trait `Send` is not implemented for `(dyn Clock + 'static)`
    = note: required for `Arc<(dyn Clock + 'static)>` to implement `Send`
note: required because it appears within the type `AppState`
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\05-dyn-without-send-sync-broken.rs:19:8
    |
 19 | struct AppState {
    |        ^^^^^^^^
note: required by a bound in `Router::<S>::new`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:140:16
    |
140 |     S: Clone + Send + Sync + 'static,
    |                ^^^^ required by this bound in `Router::<S>::new`
...
146 |     pub fn new() -> Self {
    |            --- required by a bound in this associated function

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-04-02-app-state-and-dependency-wiring` (example "05-dyn-without-send-sync-broken") due to 3 previous errors

```

**What the compiler is objecting to:** the state must be `Send + Sync`, and `Arc<T>` is only `Send` and `Sync` when `T` is both (2.8.4). `T` here is `dyn Clock`, and a bare `dyn Clock` is an *unknown* implementor: it could be a type holding an `Rc`. The compiler must assume the worst, and the first message ("not a `Handler`") is again the generic one, with the real cause in the two `cannot be ...` errors below it.

**The fix:** make the guarantee part of the trait: `trait Clock: Send + Sync`. (Or write `Arc<dyn Clock + Send + Sync>` at the use site, which works but must be repeated everywhere.)

**Why this is the fix:** supertraits (2.3.6) turn "every `Clock` is `Send + Sync`" into a rule the compiler checks at each `impl Clock for ...`. A fake that stores an `Rc` would be rejected there, at the place you wrote it, and not as an error three layers away.

### `E0308` — a sub-state with no `FromRef`

`examples/06-no-fromref-for-substate-broken.rs` has a handler taking `State<Config>`, but the state is an `AppState` that only derives `Clone`:

```text
error[E0308]: mismatched types
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\06-no-fromref-for-substate-broken.rs:33:77
    |
 33 |     let _app: Router = Router::new().route("/hello", get(hello)).with_state(state);
    |                                                                  ---------- ^^^^^ expected `Config`, found `AppState`
    |                                                                  |
    |                                                                  arguments to this method are incorrect
    |
note: method defined here
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:408:12
    |
408 |     pub fn with_state<S2>(self, state: S) -> Router<S2> {
    |            ^^^^^^^^^^

For more information about this error, try `rustc --explain E0308`.
error: could not compile `p3-04-02-app-state-and-dependency-wiring` (example "06-no-fromref-for-substate-broken") due to 1 previous error

```

**What the compiler is objecting to:** this one is surprising: it is not `E0277`. Without a `FromRef<AppState>` impl for `Config`, the only way `State<Config>` can work is the blanket impl (`FromRef<T> for T`), which says "`Config` can be taken from a `Config`". So the compiler concludes the router's state *is* `Config`, and rejects the `AppState` you pass to `.with_state` with "expected `Config`, found `AppState`". The message describes the symptom (wrong state type), and the cause is the missing derive.

**The fix:** `#[derive(Clone, FromRef)]` on `AppState` (the `macros` feature of `axum` is on in this lesson's `Cargo.toml`), exactly as in `examples/02-fromref-substates.rs`.

**Why this is the fix:** the derive adds `impl FromRef<AppState> for Config`, and the router's state is `AppState` again. If you ever see "expected `X`, found `AppState`" in a `.with_state` call, check which handler asks for `State<X>`. The same confusion shows up if a handler asks for `State<AppState>` and you forgot `.with_state(...)` altogether. 3.2.1 covers that case; here the cause is different and the message is the same family.

### `E0119` — two fields of the same type

`examples/07-two-fields-same-type-broken.rs` keeps two counters, `hits` and `misses`, both `Arc<AtomicU32>`:

```text
error[E0119]: conflicting implementations of trait `FromRef<AppState>` for type `Arc<Atomic<u32>>`
  --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\07-two-fields-same-type-broken.rs:19:13
   |
18 |     hits: Arc<AtomicU32>,
   |           --- first implementation here
19 |     misses: Arc<AtomicU32>,
   |             ^^^ conflicting implementation for `Arc<Atomic<u32>>`

For more information about this error, try `rustc --explain E0119`.
error: could not compile `p3-04-02-app-state-and-dependency-wiring` (example "07-two-fields-same-type-broken") due to 1 previous error

```

**What the compiler is objecting to:** the derive writes one `FromRef<AppState>` impl per field, and the impl's target is the field's *type*. Two `Arc<AtomicU32>` fields would mean two impls of the same trait for the same type, and `State<Arc<AtomicU32>>` could not say which one it wants. (The type is printed as `Atomic<u32>` because `AtomicU32` is an alias.)

**The fix:** give each one its own type, with a newtype:

```rust
#[derive(Clone)]
struct Hits(Arc<AtomicU32>);
#[derive(Clone)]
struct Misses(Arc<AtomicU32>);

#[derive(Clone, FromRef)]
struct AppState { hits: Hits, misses: Misses }
```

**Why this is the fix:** `State<Hits>` and `State<Misses>` are now different types, so each has exactly one impl, and a handler cannot take `misses` where it meant `hits`. The newtype is also what makes the wiring self-documenting. (The other fix is to stop asking for one counter at a time: take `State<AppState>` in the handlers that need both.)

---

## Exercises

### Warm up

<details>
<summary>Every request gets its own clone of <code>AppState</code>. Does that copy the store? Why is it still safe for two requests to write at once?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No. The state holds an `Arc<WatchStore>`, so a clone copies a pointer and increments a counter, and every clone points at the one store. Writing at once is safe because the store keeps its entries behind a `Mutex` (3.2.3, 2.8.1), and `Arc` only shares access, it does not lock.

</details>

<details>
<summary><code>AppState</code> has a field <code>config: Config</code> and derives <code>FromRef</code>. A handler takes <code>State&lt;Config&gt;</code>. What does it receive: the original config, or a copy?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

A copy. `FromRef` clones the field out of the state (`from_ref(&AppState) -> Config`), so the handler owns a fresh `Config`. For two integers that is free. For an `Arc` field the "copy" is a cheap handle to the shared value, which is why shared things go behind `Arc` and plain data can stay plain.

</details>

<details>
<summary>The test builds the app with <code>FakeClock::at(1000)</code>, posts "a", calls <code>advance(250)</code>, posts "b". What is <code>watched_at</code> of "b"?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`1250`. The fake clock reads what it was set to plus everything advanced since, and the handler reads it through the same shared handle at the moment of the request.

</details>

<details>
<summary>With a window of 100 seconds, an entry recorded at time 500 and the clock at 600: does <code>GET /watch/recent</code> list it? And at 601?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

At 600 yes, at 601 no. The cutoff is `now - window`, and an entry exactly at the cutoff is included (the specification says "still counts"). At 601 the cutoff is 501, and 500 is before it.

</details>

<details>
<summary>A teammate adds <code>trait Hasher</code> with one implementation, wrapping a pure function, and never fakes it. Is the trait earning its keep?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No. It is not nondeterministic, it does not touch the outside world, and it has one implementation. The trait adds a name, a file and an indirection and buys no seam that anyone uses. Call the function directly, and add the trait the day a test genuinely needs to fake it.

</details>

<details>
<summary>In Python you can <code>mock.patch</code> a function that a view imported. Why can you not do the same to <code>SystemTime::now()</code> in Rust?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

Python resolves the name at call time, through a module dictionary a test can overwrite. Rust resolves the call at compile time to one specific function. There is nothing to overwrite, so the only way to substitute it is to make the *caller* take the dependency as an input, which is what the `Clock` in the state is.

</details>

### Repair

Fix all four broken examples. Build each with `--features broken` to check:

1. `examples/04-state-not-clone-broken.rs` compiles.
2. `examples/05-dyn-without-send-sync-broken.rs` compiles. Fix it at the trait, not at the use site.
3. `examples/06-no-fromref-for-substate-broken.rs` compiles, without changing the handler.
4. `examples/07-two-fields-same-type-broken.rs` compiles, and the `/hits` handler still reads the `hits` counter.

### Implement

Everything in `src/lib.rs` that is a `todo!()`: `SystemClock::now`, `FakeClock::at`, `advance` and `now`, the three `WatchStore` methods, `AppState::new`, the two handlers and `app`. Each doc comment is the complete specification (status codes, JSON shapes, the boundary of the window, what happens to a title with spaces around it), so you should not need to open the tests. Work from the bottom up: the clocks, then the store, then the state and the router.

```sh
cargo test -p p3-04-02-app-state-and-dependency-wiring
```

`tests/clock_test.rs` (6 tests) and `tests/store_test.rs` (8) need no HTTP at all. `tests/api_test.rs` (12) drives the whole router with `oneshot` and a fake clock.

### Build

Add a second injected dependency without touching the first. A `Notifier` trait (it must be `Send + Sync`, with one method, `notify(&self, message: &str)`), a do-nothing `NullNotifier`, and an `Arc<dyn Notifier>` field named `notifier` in `AppState`. `AppState::new` must keep its signature and default to `NullNotifier`, so the provided tests stay green, and a new `AppState::with_notifier(self, notifier: Arc<dyn Notifier>) -> Self` replaces it. `POST /watch` calls `notify` exactly once per *recorded* watch, with the text `watched: <title>` (the trimmed title), and never for a rejected one. Write your own `RecordingNotifier` (a `Mutex<Vec<String>>` is enough) and your own tests in a new `tests/build_test.rs`: one that two posts notify twice in order, one that a blank title notifies nobody.

### Challenge (optional)

Make the clock a generic parameter instead of a trait object: `AppState<C: Clock>`, handlers generic over `C`, `app<C>`. Get the provided tests to pass against it (you will have to change the fixtures, which is your first finding). Then write a few lines in your own words: what in the code now names `C` that did not before, did `#[derive(FromRef)]` still work as-is, and in what situation would you pay that price? There are no provided tests: this is a design experiment, and the answer is yours.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Application state | one `Clone` struct that holds every shared dependency, attached with `.with_state` | every real service |
| `Arc` field | a shared handle, so cloning the state copies a pointer, not the data | the store, the pool, anything big or mutable |
| `FromRef` | "this type can be taken out of that state"; derive it to get one impl per field | `State<Piece>` in handlers, library extractors |
| Sub-state | one field of the state, requested by its own type | handlers that name only what they use |
| `Arc<dyn Trait>` | a dependency chosen at run time, behind a pointer | clocks, mailers, gateways |
| Fake | a stand-in with real, simple behaviour, swapped in through the trait | `FakeClock`, an in-memory store |
| Injection | handing a dependency in from outside (here: building the state in `main` or a test) | everywhere a test needs control |

### What you now know

- An `AppState` is `Clone` and cheap to clone because each shared thing is behind an `Arc`. A clone shares the data.
- `#[derive(FromRef)]` lets a handler ask for `State<Piece>` instead of the whole struct. It clones the field out and keys on its type, so two fields of one type need newtypes.
- A trait with `Send + Sync` supertraits is what makes `Arc<dyn Trait>` legal inside the state.
- A concrete type, a generic and a trait object are three ways to hold one dependency. The trait object keeps the state's type simple, and the generic keeps the call cost at zero and spreads a type parameter through your code.
- A fake clock makes time-dependent behaviour testable through `oneshot`, with no sleeping and every boundary reachable.
- A trait earns its place for something nondeterministic, something that touches the outside world, or something with two real implementations. Otherwise it is ceremony.

### What comes back later

- **A missing `.with_state(...)` and the first look at `State`**: [3.2.1 — Routing, handlers, extractors](../../02-axum-and-rest-api-design/01-routing-handlers-extractors/README.md)
- **A library extractor generic over the state with a `FromRef` bound**: [3.2.2 — Writing your own extractor](../../02-axum-and-rest-api-design/02-writing-your-own-extractor/README.md)
- **Where `Config` comes from in a real service**: [3.4.1 — 12-factor config and secrets](../01-config-and-secrets/README.md)
- **The next piece of plumbing in this module**: [3.4.3 — Graceful shutdown, health and readiness](../03-graceful-shutdown-health-readiness/README.md)
- **The in-memory store replaced by Postgres, behind a trait**: [3.5.5 — The repository pattern](../../05-postgres-and-sqlx/05-repository-pattern/README.md)
- **A fake dependency in a bigger test suite**: [3.8.4 — Test data factories and fixtures](../../08-error-handling-and-testing-at-scale/04-test-data-factories-and-fixtures/README.md)

### Can you explain?

- Why must the state be `Clone`, and why does cloning it not copy the store?
- What does `#[derive(FromRef)]` generate, and why do two fields of the same type collide?
- Why does `Arc<dyn Clock>` need `Send + Sync`, and where is the cleanest place to say so?
- When would you pick a generic over `Arc<dyn Clock>`, and what would it cost you?
- How does a test move time forward, and why does the handler see it?
- Name one dependency in a service of yours that deserves a trait and one that does not, and say why.

---

## Going further

- [`State` in the `axum` docs](https://docs.rs/axum/0.8.9/axum/extract/struct.State.html): the "substates" section with `FromRef`, and the advice on `Arc` versus cloning, which this lesson follows.
- [`FromRef` in `axum-core`](https://docs.rs/axum-core/0.5.6/axum_core/extract/trait.FromRef.html): the trait and its one blanket impl.
- [2.3.7 — Static vs dynamic dispatch](../../../phase2-intermediate/03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md): the trade-off in the table above, from first principles.
- [`tower::ServiceExt::oneshot`](https://docs.rs/tower/0.5.3/tower/trait.ServiceExt.html#method.oneshot): how every test in this lesson sends a request without a socket.
