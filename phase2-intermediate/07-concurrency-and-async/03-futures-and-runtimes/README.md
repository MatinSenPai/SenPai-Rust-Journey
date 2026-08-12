# 07.3 — Futures and runtimes

Threads (this module's first two lessons) are Rust's tool for genuine
parallelism — real OS-level concurrency, good for CPU-bound work. `async`
is a *different* tool, for a different problem: **I/O-bound** work — a
network call, a database query, a file read — where most of the "waiting"
is spent doing nothing, and you'd like to do something else with that
thread in the meantime instead of blocking it. This lesson demystifies
what `async` actually compiles to, by hand-building the smallest possible
piece of it, before next lesson hands you `tokio` (a real, production-grade
version of the same idea) to actually use.

## A `Future` is lazy — a `poll()`-able state machine, not a running task

```rust
use std::future::Future;
use std::task::{Context, Poll};
use std::pin::Pin;

trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
```

(This is the *real* `std::future::Future` trait, simplified only by
omitting `Pin` details this lesson doesn't need yet.) `Poll<T>` is an enum:
`Poll::Ready(T)` (done, here's the value) or `Poll::Pending` (not done yet,
try again later). Critically: **creating** a `Future` does nothing at all
— `async fn foo() { ... }` called as `foo()` doesn't run a single line of
the body. Only *polling* it drives it forward, one step at a time. This is
the single biggest surprise for developers coming from JavaScript's
`Promise` (whose executor starts when the promise is constructed). Python
coroutine objects are lazy too: calling an `async def` function does not run
its body until the coroutine is awaited or scheduled. Rust makes the polling
contract explicit through `Future`.

## A "runtime" is just: the thing that calls `poll()`

Rust's standard library deliberately ships the `Future` trait but **no
runtime** — no code anywhere that actually calls `.poll()` for you. That's
a deliberate, debated design choice: different use cases want genuinely
different schedulers (a web server, an embedded device, a game engine all
have different needs), so the language gives you the trait and lets the
ecosystem provide runtimes. `tokio` (next lesson) is by far the most common
choice for backend work — but underneath its sophisticated scheduler,
timers, and I/O event loop, its actual job is exactly what this lesson's
`block_on` does at the smallest possible scale: call `poll()`, and if it's
`Pending`, arrange to call `poll()` again later.

## Your task

`src/lib.rs` defines `Countdown`, a toy `Future` that becomes `Ready` after
being polled a fixed number of times (nothing here waits on real I/O — it's
a pure demonstration), and `block_on`, the simplest possible executor:
poll in a loop until `Ready`. Implement `Countdown::poll` and `block_on`.

**Note on `block_on`'s busy-loop**: real executors do *not* spin in a tight
loop re-polling constantly — that would burn 100% CPU for nothing. They use
the `Waker` (the `_cx: &Context` parameter you're ignoring here) to be
notified exactly when it's worth polling again, then sleep in between.
This lesson's `block_on` skips that sophistication on purpose, to keep the
core "poll until ready" idea visible without the machinery real wakers need
— `tokio`'s executor (next lesson) is the production version.

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`.
