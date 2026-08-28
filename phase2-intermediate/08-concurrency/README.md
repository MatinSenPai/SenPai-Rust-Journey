# 08 — Concurrency

This is the module the aliasing rule was for. Every guarantee borrowing has
been giving you since Phase 1 — no two mutable accesses to the same data at
once — is exactly what makes concurrent Rust hard to get wrong instead of
merely hard to get right. It ends where async begins.

1. [Threads, `Mutex`, `Arc`](01-threads-mutex-arc/README.md)
2. [`RwLock`, `Semaphore`, `OnceLock`/`LazyLock`, atomics](02-rwlock-semaphore-oncelock-atomics/README.md)
3. [Channels and message passing](03-channels-message-passing/README.md)
4. [`Send` and `Sync`: what they are and why your type isn't `Send`](04-send-and-sync/README.md)
5. [Futures and runtimes: what `async fn` desugars to](05-futures-and-runtimes/README.md)
6. [`tokio` basics](06-tokio-basics/README.md)
