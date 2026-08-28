# 07.4 — Tokio basics

Last lesson hand-built a `Future`, a `Poll` loop, and the tiniest possible
executor, specifically so `tokio` — the runtime nearly every async Rust
backend uses, including everything you'll build in Phase 3 (`axum` runs on
it) — stops looking like magic. `tokio` is the same idea, engineered
properly: a real scheduler, real timers, real async I/O, running your
`Future`s on a small pool of OS threads instead of one.

## `#[tokio::main]` and `#[tokio::test]`

```rust
#[tokio::main]
async fn main() {
    // this function body runs inside a tokio runtime
}
```

`async fn main()` alone doesn't work as a program's actual entry point —
`main` can't be async on its own (something has to create and run the
executor first). `#[tokio::main]` is a macro that generates the ordinary,
synchronous `fn main()` the OS expects, which builds a `tokio` runtime and
uses it to drive your `async fn main()` body to completion. `#[tokio::test]`
does the same thing for test functions — every test in this lesson uses it
instead of plain `#[test]`.

## `tokio::spawn`: async's answer to `thread::spawn`

```rust
let handle = tokio::spawn(async {
    // runs concurrently with whatever spawned it
    42
});
let result = handle.await.unwrap();
```

`tokio::spawn` hands a `Future` to the runtime as an independent **task** —
conceptually similar to `thread::spawn` from lesson 1, but tasks are far
cheaper than OS threads (you can spawn thousands of them) and are
scheduled *cooperatively* onto a small pool of real threads, switching
between tasks specifically at `.await` points rather than being preempted
by the OS at arbitrary instructions. `handle.await` (not `.join()` — tasks
use `.await`, threads use `.join()`) gives back a `Result<T, JoinError>`,
same shape and same reason as `JoinHandle::join`'s `Result` in lesson 1.

## The actual point: concurrent waiting, not parallel computing

```rust
// sequential: total time ≈ sum of every delay
for id in ids {
    fetch_simulated(id).await;
}

// concurrent: total time ≈ the SLOWEST single delay
let handles: Vec<_> = ids.into_iter().map(|id| tokio::spawn(fetch_simulated(id))).collect();
for handle in handles {
    handle.await.unwrap();
}
```

This is the entire value proposition of async for backend work: if you're
waiting on 10 independent network calls that each take 100ms, doing them
one after another (`await`ing each in turn) takes ~1000ms; spawning all 10
as separate tasks and then awaiting them takes ~100ms — because while task
1 is waiting on its network response, the runtime is free to make progress
on tasks 2 through 10 on the very same thread(s), instead of that thread
sitting idle. Threads *can* achieve the same overlap, but spawning 10,000
OS threads to wait on 10,000 slow requests is far more expensive (each OS
thread has real memory/scheduling overhead) than 10,000 cheap async tasks.

## Your task

Implement both functions in `src/lib.rs`.

## Next

`solution/SOLUTION.md` — but only after a real attempt. That's Phase 2 complete —
[Side-quest 2: Telegram Quiz Bot](../../../side-quests/sq-02-telegram-quiz-bot/README.md)
next, then [Phase 3](../../../phase3-backend-foundations/README.md), where
`tokio` stops being something you use directly and becomes the engine
running underneath `axum` and `sqlx`.
