# 09 — Async in practice

`tokio-basics` showed you that `async fn` works. This module is where you
build with it: spawning and structuring real concurrent work, cancelling it
safely, and knowing when a task needs `spawn_blocking` instead of an
`.await`.

1. [`spawn`, `JoinSet`, structured concurrency](01-spawn-joinset-structured-concurrency/README.md)
2. [`select!` and cancellation safety](02-select-and-cancellation-safety/README.md)
3. [Streams: `futures::Stream` and `tokio-stream`](03-streams/README.md)
4. [Async traits and `spawn_blocking`](04-async-traits-and-blocking/README.md)
