# 3.4.3 — Graceful shutdown, health and readiness

## At a glance

After this lesson you can:

- Stop an `axum` server with `with_graceful_shutdown` so requests already running finish, new connections are refused, and the call returns `Ok(())` when the last one is done.
- Build the shutdown future yourself, so a signal, a channel, or a test can trigger it, and explain why it is not tied to the keyboard.
- Write `/health` (liveness) and `/ready` (readiness), and say why a draining instance fails the second one and still passes the first.
- Read the `E0277` and `E0373` you get from the two classic shutdown mistakes, and spot the one that gives no error at all.

**Time:** ~100 minutes · **Prerequisites:**
[3.4.2 — Application state and dependency wiring](../02-app-state-and-dependency-wiring/README.md),
[3.2.1 — Routing, handlers, extractors](../../02-axum-and-rest-api-design/01-routing-handlers-extractors/README.md),
[3.1.3 — HTTP semantics you must know](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.md)

---

## Why this matters

Every deploy ends with someone stopping your process. Kubernetes, systemd, Docker and `kill` all do it the same way: send `SIGTERM`, wait a few seconds, then send `SIGKILL`. If your program ignores `SIGTERM`, the second signal cuts it off mid-request, and a client that was halfway through a payment or an upload gets a reset connection. In Django you rarely think about this because gunicorn does it for you: on `SIGTERM` the master tells each worker to finish its current request and exit (`--graceful-timeout` is the "then `SIGKILL`" knob). In `axum` nothing is done unless you ask. This lesson is how you ask.

The other half of the story is the load balancer in front of you. It has to know when to *stop sending* you traffic, and it knows only what you tell it over HTTP. That is what `/health` and `/ready` are for. They look alike, and mixing them up is a classic outage: a service that reports "I'm dead" while it is merely busy, or "I'm fine" while its database is gone.

This closes module 3.4. [3.4.1](../01-config-and-secrets/README.md) made the process read its settings, [3.4.2](../02-app-state-and-dependency-wiring/README.md) gave it one place for shared state, and this lesson teaches it to stop cleanly. [3.5.1 — Connecting and pooling](../../05-postgres-and-sqlx/01-connecting-and-pooling/README.md) then gives `/ready` something real to check.

---

## The concept

### What a server does when told to stop

`axum::serve(listener, app)` runs until the process dies. `.with_graceful_shutdown(signal)` adds one thing: a future that, when it completes, means "stop accepting, finish what you have". Here is a program that shows it from both sides. It starts a server, begins a slow request, triggers shutdown while that request is still inside its handler, and prints what happens (`examples/02-graceful-shutdown-timeline.rs`):

```sh
cargo run -p p3-04-03-graceful-shutdown-health-readiness --example 02-graceful-shutdown-timeline
```

```text
1. the slow request is inside its handler
2. shutdown triggered
3. a new connection is refused
4. server finished yet? false
5. slow request got: HTTP/1.1 200 OK / body "slow done"
6. server returned: Ok(())
```

Read the order. After step 2 the listener is gone, so step 3 is a refused connection, yet the slow request, which was accepted *before* the trigger, still gets its `200`. The server task is not finished at step 4, because a request is running. It returns `Ok(())` only at step 6, after the last response. That is the whole contract.

```senpai-visual
{"kind":"async","labels":["Shutdown future completes","Listener closed: new connections refused","In-flight requests keep running","Idle keep-alive connections closed","Last response sent","serve returns Ok"]}
```

### The shutdown future is just a future

`with_graceful_shutdown` takes any `Future<Output = ()> + Send + 'static`. The example above used a `tokio::sync::oneshot` receiver, which the program itself fires. A real service uses the operating system's signal instead:

```rust
async fn os_signal() {
    let ctrl_c = async { tokio::signal::ctrl_c().await.expect("install Ctrl-C handler") };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler").recv().await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {} _ = terminate => {} }
}
```

This function is in `src/lib.rs`, already written. `tokio::signal::ctrl_c()` works on Linux, macOS and Windows. `SIGTERM`, the signal deployments actually send, exists only on Unix, so that branch is compiled only there; on Windows the other branch is a future that never completes (`pending`). This machine is Windows, which is why none of the tests use it. A test cannot press Ctrl-C, and cannot send `SIGTERM` on Windows. So the design rule of this lesson is: **the code that serves takes the trigger as a parameter**. The real `main` passes `os_signal()`, and a test passes a `oneshot` receiver it controls. `select!` is from [2.9.2](../../../phase2-intermediate/09-async-in-practice/02-select-and-cancellation-safety/README.md): it completes as soon as one branch does.

### Liveness and readiness are different questions

`examples/01-health-and-ready-server.rs` serves both on port `3160`, with a graceful shutdown on Ctrl-C:

```sh
cargo run -p p3-04-03-graceful-shutdown-health-readiness --example 01-health-and-ready-server
```

```text
listening on http://127.0.0.1:3160
```

In a second terminal:

```sh
curl -i http://127.0.0.1:3160/health
curl -i http://127.0.0.1:3160/ready
```

```text
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 2
date: Sun, 04 Oct 2026 11:14:33 GMT

ok
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 5
date: Sun, 04 Oct 2026 11:14:33 GMT

ready
```

(The `date` header is the time you run it.) Both say `200` now. They diverge when something is wrong, and they answer different questions:

| Probe | Question | `200` means | `503` means | The platform reacts by |
|---|---|---|---|---|
| `/health` (liveness) | Is this process alive and able to answer at all? | yes | (you never send it: if the process can't answer, there is no response) | restarting the process |
| `/ready` (readiness) | Should this instance be sent traffic right now? | yes | not now: draining, or a dependency is down | taking it out of the pool, **not** restarting |

The two reactions explain everything. If `/health` failed whenever the database was down, the platform would restart every instance, and since a restart does not fix the database, you would get a restart loop on top of the outage. If `/ready` kept answering `200` while draining, the load balancer would keep routing new requests to an instance that has already closed its listener. The same status code, `503 Service Unavailable` (3.1.3: a `5xx` that says "try elsewhere or later"), is right for readiness because the failure is temporary and not the client's fault.

```senpai-visual
{"kind":"network","labels":["Load balancer","probes /ready","Instance A: 200 ready","Instance B: 503 draining","Traffic goes only to A"]}
```

### Draining: readiness flips first, the listener closes later

There is a subtlety. The moment the shutdown future completes, the listener is closed, so a load balancer that probes `/ready` right after gets "connection refused" and not a tidy `503`. A graceful service therefore does three steps, with a gap before the last:

1. on the signal, flip readiness to `503` ("draining");
2. wait a short *grace period* so the balancer's next probe sees it and stops sending traffic;
3. only then let the shutdown future complete, which closes the listener.

The exercise `shutdown_future` is exactly that. The state it flips is a small struct shared between the handlers and the future, so it sits behind an `Arc`, with atomic flags ([2.8.2](../../../phase2-intermediate/08-concurrency/02-rwlock-semaphore-oncelock-atomics/README.md)) inside:

```rust
pub struct Lifecycle {
    draining: AtomicBool,
    dependency_up: AtomicBool,
}
```

`dependency_up` is how `/ready` will report "my database is gone" once there is a database (module 3.5). Nothing in this lesson needs one.

### What shutdown does to idle connections

3.1.3 said an HTTP/1.1 connection stays open between requests (keep-alive). A client holding such a connection, with no request running on it, would block a naive shutdown forever, so the server closes idle connections at shutdown. `examples/03-keep-alive-closed-on-shutdown.rs` shows it:

```sh
cargo run -p p3-04-03-graceful-shutdown-health-readiness --example 03-keep-alive-closed-on-shutdown
```

```text
request 1 on the connection: HTTP/1.1 200 OK
request 2, same connection:  HTTP/1.1 200 OK
after shutdown, read() returned 0 bytes (end of stream)
server returned: Ok(())
```

Two requests shared one connection, then shutdown ended it: `read()` returning `0` bytes is the TCP end-of-stream from 3.1.1. A connection that is in the middle of a request is a different case: it is *not* closed, which is the example above it.

### Testing a shutdown without a keyboard

The tests in this lesson start a real server on `127.0.0.1:0` (port `0` means "the OS picks a free port", so tests never collide), talk to it over a plain `TcpStream`, and never sleep for a guessed duration. Two tools make that deterministic:

- A `tokio::sync::Notify` lets the slow handler say "I have started" and wait for "you may finish". The test starts the request, waits for "started", triggers shutdown, and only then releases the handler. There is no race between the three.
- "A new connection is refused" is checked by polling `connect` until it fails, not by waiting a guessed time.

On Windows a refused `connect` takes about two seconds to fail, so the test that waits for it takes about two seconds. That is the operating system, not a sleep in the test.

---

## Hands on

The three working examples are standalone programs; `src/lib.rs` is the library version of the same ideas, which you build in the exercises. Run them all, in this order, and compare:

```sh
cargo run -p p3-04-03-graceful-shutdown-health-readiness --example 01-health-and-ready-server
cargo run -p p3-04-03-graceful-shutdown-health-readiness --example 02-graceful-shutdown-timeline
cargo run -p p3-04-03-graceful-shutdown-health-readiness --example 03-keep-alive-closed-on-shutdown
```

For `01`, press Ctrl-C in its terminal. The two `println!` lines after `serve` in `main` tell you the shutdown went through: first that a signal arrived, then that every request finished.

Then read `src/lib.rs`: the doc comment on each function is its full specification.

---

## Errors you will meet

### `E0277` — the function, not the future

You wrote a nice `async fn shutdown_signal()` and passed its *name*. `examples/04-fn-item-not-a-future-broken.rs`:

```sh
cargo build -p p3-04-03-graceful-shutdown-health-readiness --example 04-fn-item-not-a-future-broken --features broken
```

```text
error[E0277]: `fn() -> impl Future<Output = ()> {shutdown_signal}` is not a future
   --> phase3-backend-foundations\04-configuration-and-app-structure\03-graceful-shutdown-health-readiness\examples\04-fn-item-not-a-future-broken.rs:15:33
    |
 15 |         .with_graceful_shutdown(shutdown_signal)
    |          ---------------------- ^^^^^^^^^^^^^^^ `fn() -> impl Future<Output = ()> {shutdown_signal}` is not a future
    |          |
    |          required by a bound introduced by this call
    |
    = help: the trait `Future` is not implemented for fn item `fn() -> impl Future<Output = ()> {shutdown_signal}`
note: required by a bound in `Serve::<L, M, S>::with_graceful_shutdown`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\serve\mod.rs:153:12
    |
151 |     pub fn with_graceful_shutdown<F>(self, signal: F) -> WithGracefulShutdown<L, M, S, F>
    |            ---------------------- required by a bound in this associated function
152 |     where
153 |         F: Future<Output = ()> + Send + 'static,
    |            ^^^^^^^^^^^^^^^^^^^ required by this bound in `Serve::<L, M, S>::with_graceful_shutdown`
help: use parentheses to call this function
    |
 15 |         .with_graceful_shutdown(shutdown_signal())
    |                                                ++

error[E0277]: `WithGracefulShutdown<TcpListener, Router, Router, ...>` is not a future
   --> phase3-backend-foundations\04-configuration-and-app-structure\03-graceful-shutdown-health-readiness\examples\04-fn-item-not-a-future-broken.rs:16:10
    |
 16 |         .await
    |          ^^^^^ `WithGracefulShutdown<TcpListener, Router, Router, ...>` is not a future
    |
    = help: the trait `IntoFuture` is not implemented for `WithGracefulShutdown<TcpListener, Router, Router, ...>`
    = note: WithGracefulShutdown<TcpListener, Router, Router, ...> must be a future or must implement `IntoFuture` to be awaited
help: the trait `IntoFuture` is implemented for `WithGracefulShutdown<L, M, S, F>`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\serve\mod.rs:332:1
    |
332 | / impl<L, M, S, F> IntoFuture for WithGracefulShutdown<L, M, S, F>
333 | | where
334 | |     L: Listener,
335 | |     L::Addr: Debug,
...   |
339 | |     S::Future: Send,
340 | |     F: Future<Output = ()> + Send + 'static,
    | |____________________________________________^
    = note: the full name for the type has been written to 'M:\SenPai-Rust-Journey\target\debug\examples\04_fn_item_not_a_future_broken.long-type-17115680415784593178.txt'
    = note: consider using `--verbose` to print the full type name to the console
help: remove the `.await`
    |
 16 -         .await
    |

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-04-03-graceful-shutdown-health-readiness` (example "04-fn-item-not-a-future-broken") due to 2 previous errors
```

(The number in the `long-type-...txt` file name changes every run.)

**What the compiler is objecting to.** An `async fn` is a function that *returns* a future. Its name is the function (a "fn item"), not a future. `with_graceful_shutdown` wants the future. The second error is a knock-on: because the first argument was invalid, the whole builder is not awaitable either.

**The fix.** Call it: `.with_graceful_shutdown(shutdown_signal())`. The compiler's own `help` says so.

**Why this is the fix.** Calling an `async fn` runs none of its body; it builds the future that will run it. `axum` stores that future and polls it for you. Passing the name hands over the recipe instead of the dish.

### `E0373` — the async block borrows what `main` owns

You want the shutdown future to flip the draining flag too, so you write an `async` block that uses `draining`. `examples/05-async-block-borrows-state-broken.rs`:

```sh
cargo build -p p3-04-03-graceful-shutdown-health-readiness --example 05-async-block-borrows-state-broken --features broken
```

```text
error[E0373]: async block may outlive the current function, but it borrows `draining`, which is owned by the current function
  --> phase3-backend-foundations\04-configuration-and-app-structure\03-graceful-shutdown-health-readiness\examples\05-async-block-borrows-state-broken.rs:15:33
   |
15 |         .with_graceful_shutdown(async {
   |                                 ^^^^^ may outlive borrowed value `draining`
16 |             tokio::signal::ctrl_c().await.unwrap();
17 |             draining.store(true, Ordering::SeqCst);
   |             -------- `draining` is borrowed here
   |
   = note: async blocks are not executed immediately and must either take a reference or ownership of outside variables they use
help: to force the async block to take ownership of `draining` (and any other referenced variables), use the `move` keyword
   |
15 |         .with_graceful_shutdown(async move {
   |                                       ++++

For more information about this error, try `rustc --explain E0373`.
error: could not compile `p3-04-03-graceful-shutdown-health-readiness` (example "05-async-block-borrows-state-broken") due to 1 previous error
```

**What the compiler is objecting to.** The bound is `'static`: the future must not borrow anything from `main`'s stack. The block, without `move`, borrows `draining`. `axum` may keep the future alive past the end of `main`'s locals, as far as the types can tell.

**The fix.** `async move { ... }`, exactly as the `help` shows. If the same `Arc` is needed elsewhere (the handlers), `clone()` it first and move the clone in, as `examples/01-health-and-ready-server.rs` does.

**Why this is the fix.** `move` makes the block own its captured variables, so it has no borrow left and satisfies `'static`. An `Arc` is cheap to clone, which is why it is the usual way to give one flag to several owners.

### No error at all: the dropped sender shuts the server down at once

This one compiles and runs. `examples/06-dropped-sender-instant-shutdown-trap.rs` wires the shutdown to a `oneshot` receiver with `stopped.await.ok();`, but the sender was dropped instead of kept:

```sh
cargo run -p p3-04-03-graceful-shutdown-health-readiness --example 06-dropped-sender-instant-shutdown-trap
```

```text
server finished on its own: Ok(Ok(()))
a client trying to connect gets: ConnectionRefused
```

**What happened.** `rx.await` returns `Err` when its sender is dropped, and `.ok()` threw that error away. The future completed, which `axum` reads as "shut down now". The server stopped before serving a single request.

**The fix.** Distinguish "the sender fired" from "the sender vanished". `src/lib.rs` has `fired(rx)`, which waits forever on a dropped sender instead of completing. And keep the sender alive: store it in the place whose job is to trigger the shutdown.

**Why this is the fix.** A shutdown trigger should fire only on purpose. Turning "I can no longer hear anything" into "stop" is a quiet way to turn an unrelated bug (a sender dropped too early) into an outage.

---

## Exercises

All code is in `src/lib.rs` (one `todo!()` per function, each with its full specification in the doc comment) and in `tests/`. The last test run before you start looks like this:

```sh
cargo test -p p3-04-03-graceful-shutdown-health-readiness --lib only_ready
```

```text
running 1 test
test tests::only_ready_is_200 ... FAILED

failures:

---- tests::only_ready_is_200 stdout ----

thread 'tests::only_ready_is_200' (23020) panicked at phase3-backend-foundations\04-configuration-and-app-structure\03-graceful-shutdown-health-readiness\src\lib.rs:87:9:
not yet implemented: map each verdict to its status code
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::only_ready_is_200

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
```

(The number in parentheses after `thread` is a thread id and changes every run.)

### Warm up

No typing. Answers below each question.

<details>
<summary>1. A client sends a request that takes 5 seconds. One second in, the shutdown future completes. What does the client receive, and when does <code>serve</code> return?</summary>

**Answer.** The client receives its normal response after the remaining 4 seconds; `serve` returns `Ok(())` right after that response. Graceful shutdown stops *accepting*; it does not cancel requests that are already running.

</details>

<details>
<summary>2. Same situation, but a second client connects 2 seconds after the shutdown future completed. What happens to it?</summary>

**Answer.** Its connection is refused, because the listener was dropped when the future completed. (With `grace` greater than zero in `shutdown_future`, the listener stays open for that long, because completion is delayed.)

</details>

<details>
<summary>3. The database is unreachable, but the process is running fine. Which of <code>/health</code> and <code>/ready</code> should return <code>503</code>, and why not the other?</summary>

**Answer.** `/ready`. The instance should receive no traffic, but restarting it does not help, and `/health` failing would make the platform restart it in a loop.

</details>

### Repair

Fix both broken examples: build each with `--features broken`, read the error against "Errors you will meet" above, and edit the file until it builds with `cargo build -p p3-04-03-graceful-shutdown-health-readiness --example <name> --features broken`.

- `examples/04-fn-item-not-a-future-broken.rs`
- `examples/05-async-block-borrows-state-broken.rs`

### Implement

In `src/lib.rs`, implement `Readiness::from_flags`, `Readiness::status`, `Readiness::body`, and the `async fn shutdown_future`. The doc comment on each is the complete spec. Tests: the four tests in `src/lib.rs`.

```sh
cargo test -p p3-04-03-graceful-shutdown-health-readiness --lib
```

### Build

Implement `app` (the router with `/health` and `/ready`) and `run` (serve on a listener, stop gracefully when `shutdown_future` completes). Tests: `tests/server_test.rs`, which starts real servers on `127.0.0.1:0` and checks that an in-flight request finishes, a new connection is refused afterwards, `/ready` turns `503` while `/health` stays `200`, and an idle keep-alive connection is closed.

```sh
cargo test -p p3-04-03-graceful-shutdown-health-readiness --test server_test
```

### Challenge (optional)

`run_with_deadline`: a request that never finishes would keep `run` waiting forever, and the platform's `SIGKILL` would arrive anyway. Give up with a `TimedOut` error after a deadline instead. Tests: `tests/challenge_test.rs`. It uses `tokio::select!` and reaches forward to the idea, picked up again in module 3.8, that shutdown is also something you observe and log.

```sh
cargo test -p p3-04-03-graceful-shutdown-health-readiness --test challenge_test
```

A model solution, with the reasoning, is in [`solution/SOLUTION.md`](solution/SOLUTION.md).

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| graceful shutdown | stop accepting, let running requests finish, then exit | every deployed service |
| shutdown future | the future `with_graceful_shutdown` waits on | tests inject it; `main` passes the OS signal |
| liveness (`/health`) | is the process alive? failing it restarts the process | platform probes |
| readiness (`/ready`) | should it receive traffic? failing it only removes it from the pool | load balancers; module 3.5 adds the database check |
| draining | the phase between "shutdown started" and "listener closed" | the grace period in `shutdown_future` |
| `SIGTERM` | the Unix signal deployments send first; Windows has no equivalent here | `tokio::signal::unix` |

### What you now know

- `with_graceful_shutdown` refuses new connections at once, lets in-flight requests finish, closes idle keep-alive connections, and then returns `Ok(())`.
- The trigger is an ordinary future, so a real server passes the OS signal and a test passes a `oneshot` receiver.
- `/health` and `/ready` answer different questions, and a draining instance must fail only the second.
- A dropped `oneshot` sender can silently trigger a shutdown if you write `.await.ok()`.

### What comes back later

- Readiness gets a real dependency in [3.5.1 — Connecting and pooling](../../05-postgres-and-sqlx/01-connecting-and-pooling/README.md): `dependency_up` becomes "can I get a connection from the pool".
- Logging the shutdown (when it began, how many requests it waited for) belongs to the tracing lessons of [module 3.8](../../08-error-handling-and-testing-at-scale/README.md).
- The shared state you put behind `Arc` here was designed in [3.4.2](../02-app-state-and-dependency-wiring/README.md); the settings (grace period, port) come from [3.4.1](../01-config-and-secrets/README.md).

### Can you explain?

- Say out loud what happens, step by step, to a request that is half-done when `SIGTERM` arrives, and to a client that connects just after.
- Why does a down database fail `/ready` and not `/health`?
- Why does `shutdown_future` take the trigger as a parameter instead of calling `tokio::signal::ctrl_c()` itself?
- What does a draining instance wait for before the listener closes, and why is that wait there?

---

## Going further

- [`axum::serve::WithGracefulShutdown`](https://docs.rs/axum/0.8.9/axum/serve/struct.WithGracefulShutdown.html), the type that `with_graceful_shutdown` returns.
- [`tokio::signal`](https://docs.rs/tokio/latest/tokio/signal/index.html): `ctrl_c`, plus the Unix and Windows signal sets.
- [Kubernetes: configure liveness, readiness and startup probes](https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/), the usual consumer of these two endpoints.
- [Gunicorn: signal handling](https://docs.gunicorn.org/en/stable/signals.html), the Python-side version of the same contract.
