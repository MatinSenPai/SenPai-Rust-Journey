# Solution — 3.2.4 `tower::Service` and `Layer`: middleware by hand

## `ResponseTimeLayer` and `ResponseTime`

```rust
impl<S> Layer<S> for ResponseTimeLayer {
    type Service = ResponseTime<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ResponseTime { inner }
    }
}
```

```rust
fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), S::Error>> {
    self.inner.poll_ready(cx)
}

fn call(&mut self, request: Request) -> Self::Future {
    let start = Instant::now();
    let future = self.inner.call(request);
    Box::pin(async move {
        let mut response = future.await?;
        let elapsed_ms = start.elapsed().as_millis() as u64;
        response
            .headers_mut()
            .insert("x-response-time-ms", HeaderValue::from(elapsed_ms));
        Ok(response)
    })
}
```

The layer only moves `inner` into the struct. `poll_ready` forwards, because a wrapper is ready exactly when what it wraps is ready. In `call`, the clock starts when `call` is invoked and is read after `future.await?`, so it covers the whole handler, not just the call that built the future. The test `times_the_whole_handler_not_just_the_call` has a handler sleep 20 ms to prove it.

`?` returns the inner error untouched, so no header is added on failure (`passes_an_inner_error_through_unchanged`). `HeaderValue::from(u64)` writes plain decimal digits, and `headers_mut().insert` replaces an existing header instead of adding a second one, which is what `append` would do. Nothing here is cloned, so no `mem::replace` is needed: `call` uses `inner` right away and moves only its future into the box.

## `MaintenanceLayer` and `Maintenance`

```rust
fn layer(&self, inner: S) -> Self::Service {
    Maintenance {
        inner,
        flag: Arc::clone(&self.flag),
    }
}
```

```rust
fn call(&mut self, request: Request) -> Self::Future {
    if self.flag.load(Ordering::SeqCst) {
        let response = Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(Body::from("down for maintenance"))
            .unwrap();
        return Box::pin(async move { Ok(response) });
    }
    Box::pin(self.inner.call(request))
}
```

`Arc::clone` gives every wrapped service a pointer to the same `AtomicBool`, so flipping the flag outside affects all of them on their next request. Copying the bool instead would have given each service its own flag that nobody can reach.

The short-circuit is the `return` before `self.inner.call(request)`: when the flag is on the inner service's `call` is never invoked, so the handler never runs (`flag_on_never_runs_the_handler` counts runs). The flag is read once per request, at `call` time.

## On the challenge (optional)

The `from_fn` version of the timer is shorter:

```rust
async fn timed(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        "x-response-time-ms",
        HeaderValue::from(start.elapsed().as_millis() as u64),
    );
    response
}
```

It has no `Layer`, no `Service`, no boxed future and no `Clone` derive to remember. What it cannot do is carry a configuration type with its own builder methods, work outside `axum`, or control `poll_ready`. For `ResponseTimeLayer::new("x-took-ms")` you add a `header: &'static str` field to the layer, copy it into the service in `layer`, and use it in `call`: the same shape as `LogLayer`'s `name`.

## What this lesson was really about

None of this middleware is long. What makes it feel hard is that the compiler's complaints show up at `.layer(...)`, far from the line that is wrong. Now you can read them: `Clone` is the derive you forgot, `Send` is the bound on your boxed future, and `Infallible` means your middleware must answer instead of failing.
