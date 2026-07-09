# Checkpoint

1. Add a `println!` at the top of `Countdown::poll` and run
   `block_on(Countdown::new(3))`. How many times does it print? Does that
   match what you expected from "polled 3 times"?
2. `block_on` requires `F: Future + Unpin`. `Countdown` satisfies `Unpin`
   automatically (it only contains a `u32` field — no self-referential
   pointers). Real `async fn` bodies are frequently *not* `Unpin`. What do
   you think that implies about why `block_on` needed to bound on `Unpin`
   at all, rather than working for any `Future`?
3. This lesson's `block_on` ignores the `Waker` completely and just loops.
   For `Countdown` specifically, is that actually wasteful, given that
   `Countdown` is *always* ready to make progress the instant it's polled
   again (no real I/O wait involved)? Now imagine a `Future` that's
   genuinely waiting on a network response that might take 200ms — what
   would busy-polling that one in a tight loop cost, compared to a real
   executor using the `Waker` to sleep until data arrives?
4. `async fn foo() { ... }` compiles to a value implementing `Future` —
   calling `foo()` doesn't run any of the body. Contrast that with calling
   an ordinary (non-async) function. What surprised you most about this
   difference, coming from Python's synchronous-by-default functions (or
   `asyncio`, if you've used it)?
