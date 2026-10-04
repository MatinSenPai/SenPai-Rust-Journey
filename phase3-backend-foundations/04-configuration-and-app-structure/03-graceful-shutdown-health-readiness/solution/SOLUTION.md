# Solution — 3.4.3 Graceful shutdown, health and readiness

The full code is `solution/src/lib.rs`; it passes every test in `solution/tests/` and the unit tests in `lib.rs`.

## `Readiness`

```rust
pub fn from_flags(draining: bool, dependency_up: bool) -> Readiness {
    if draining {
        Readiness::Draining
    } else if !dependency_up {
        Readiness::DependencyDown
    } else {
        Readiness::Ready
    }
}
```

The order of the branches *is* the rule: draining is checked first, so a draining instance with a down dependency reports `Draining` (`draining_beats_a_down_dependency`). `status` is a `match` with `Ready` as `200 OK` and the other two as `503 Service Unavailable`; `body` is a `match` returning the three exact strings. The verdict is a three-variant enum and not a `bool` because the *reason* for a `503` is worth putting in the body: an operator reading `draining` and one reading `dependency down` do different things next.

## `shutdown_future`

```rust
pub async fn shutdown_future(
    trigger: impl Future<Output = ()>,
    lifecycle: Arc<Lifecycle>,
    grace: Duration,
) {
    trigger.await;
    lifecycle.start_draining();
    tokio::time::sleep(grace).await;
}
```

Three steps in the order the spec gives. Nothing happens before `trigger` completes. `start_draining` runs the instant it does, so `/ready` turns `503` at once. The sleep afterwards is the grace period: `axum` does not close the listener until this future completes, so the listener stays open during it and a load balancer can still see the `503`. The unit test uses `#[tokio::test(start_paused = true)]`, a clock that only advances when every task is idle, so a "10 second" grace takes no real time.

## `app`

```rust
pub fn app(lifecycle: Arc<Lifecycle>) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/ready", get(ready))
        .with_state(lifecycle)
}

async fn ready(State(lifecycle): State<Arc<Lifecycle>>) -> (StatusCode, &'static str) {
    let readiness = lifecycle.readiness();
    (readiness.status(), readiness.body())
}
```

`/health` never looks at the lifecycle: liveness is "I can answer", and answering is the proof. `/ready` reads the state per request, so a change to the flags shows up on the next probe. A `(StatusCode, &str)` tuple already implements `IntoResponse`.

## `run`

```rust
pub async fn run(listener: TcpListener, router: Router, lifecycle: Arc<Lifecycle>,
                 trigger: impl Future<Output = ()> + Send + 'static, grace: Duration) -> io::Result<()> {
    let shutdown = shutdown_future(trigger, lifecycle, grace);
    axum::serve(listener, router).with_graceful_shutdown(shutdown).await
}
```

`shutdown_future(...)` is passed *called*, not by name (the `E0277` of the lesson). The `Send + 'static` bounds on `trigger` are there because `with_graceful_shutdown` requires them of the whole future, and this future contains `trigger`. The `.await` completes when the shutdown future has completed and every in-flight request has finished.

## The challenge, `run_with_deadline`

```rust
let (drain_started, drain_started_rx) = oneshot::channel::<()>();
let shutdown = async move {
    shutdown_future(trigger, lifecycle, grace).await;
    let _ = drain_started.send(());
};
let serve = axum::serve(listener, router).with_graceful_shutdown(shutdown);
tokio::select! {
    result = serve => result,
    _ = async { let _ = drain_started_rx.await; tokio::time::sleep(deadline).await; }
        => Err(io::ErrorKind::TimedOut.into()),
}
```

The deadline must start when the shutdown future *completes*, not when the server starts, so a `oneshot` carries that moment from inside the shutdown future to the second `select!` branch. That branch waits for the signal, then sleeps `deadline`. If `serve` finishes first, its result wins and the sleep is dropped; if the sleep finishes first, `select!` drops `serve`, which drops the stuck connections, and the caller sees `TimedOut`. `let _ = drain_started.send(())` ignores the error a dropped receiver would give: that only happens if `serve` already won.

## Why the tests do not flake

- Every wait is "until something observable happens", never a guessed duration. The `Notify` pair makes the slow handler say when it has started and wait to be told it may finish, so "request running, then shutdown, then release" is the only possible order.
- Ports are `127.0.0.1:0`, so parallel tests cannot collide.
- `ready_turns_503_while_draining_but_health_stays_200` uses a 60 second grace period on purpose, so the listener stays open for the probes; the test never waits it out, because the runtime drops the server task when the test ends.
- A refused `connect` takes about two seconds on Windows, so `in_flight_request_finishes_and_new_connections_are_refused` takes about that long. The suite was run three times in a row, green each time.
