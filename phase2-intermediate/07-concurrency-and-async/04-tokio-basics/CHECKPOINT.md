# Checkpoint

1. Rewrite `fetch_all_concurrently` to `.await` each `fetch_simulated` call
   directly in a loop, without `tokio::spawn` at all (still `async`, just
   sequential). Run `concurrent_fetches_are_actually_concurrent` against
   your sequential version — does it still pass? What does that prove
   about the difference between "using `async`/`.await`" and "actually
   running things concurrently"?
2. `#[tokio::test]` was used everywhere instead of `#[test]`. Try changing
   one test function to plain `#[test]` while keeping its `async fn`
   signature. What compile or runtime error do you get, and what does that
   tell you about what `#[tokio::test]` is actually doing for you?
3. `handle.await.unwrap()` appears when collecting spawned task results.
   What's inside the `Result` that `.unwrap()` is unwrapping — what could
   make a spawned task's `JoinHandle` resolve to `Err`?
4. You now know two ways to run work "at the same time": OS threads
   (`thread::spawn`, lesson 1) and async tasks (`tokio::spawn`, this
   lesson). Given what each actually costs (real OS thread vs. lightweight
   scheduled task) and what kind of work each is suited for (CPU-bound vs.
   I/O-bound waiting), which would you reach for to: (a) resize 10,000
   images on disk, (b) make 10,000 concurrent HTTP requests to a slow
   third-party API?
