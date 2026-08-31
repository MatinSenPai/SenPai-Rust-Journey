# 2.9.3 — Streams

## At a glance

After this lesson you can:

- Explain exactly how `Stream` relates to `Iterator` ([2.2.4](../../02-iterators-and-closures/04-implementing-iterator/README.md)) and `Future` ([2.8.5](../../08-concurrency/05-futures-and-runtimes/README.md)), write the `poll_next` signature from memory, and say which crate it actually comes from.
- Drive a stream with `while let Some(item) = stream.next().await`, and chain `StreamExt` adapters — `.filter()`, `.map()`, `.take()` — on it, the same adapter habit [2.2](../../02-iterators-and-closures/README.md) gave you.
- Build a real stream whose items arrive one at a time over real time, consume it, and race its `.next().await` with `select!` ([2.9.2](../02-select-and-cancellation-safety/README.md)) against a timeout.

**Time:** ~55 minutes · **Prerequisites:**
[2.9.2 — `select!` and cancellation safety](../02-select-and-cancellation-safety/README.md),
[2.2.4 — Implementing `Iterator` and `IntoIterator`](../../02-iterators-and-closures/04-implementing-iterator/README.md),
[2.8.5 — Futures and runtimes](../../08-concurrency/05-futures-and-runtimes/README.md)

## Why this matters

[2.9.1](../01-spawn-joinset-structured-concurrency/README.md) and [2.9.2](../02-select-and-cancellation-safety/README.md) both circled the same thing [2.8.5](../../08-concurrency/05-futures-and-runtimes/README.md) defined: a `Future`, a *single* value that becomes ready once, eventually. `JoinSet` drives many of those single values at once; `select!` races several of them against each other. But a lot of real async data isn't a single value at all — it's a **sequence**: rows coming back from a database query one at a time, messages arriving on a websocket, lines coming off a subprocess's output — each one, over time.

[2.2](../../02-iterators-and-closures/README.md) already solved exactly this shape for the synchronous world: `Iterator`. But `Iterator::next()` is synchronous — if the next value isn't ready yet, calling it blocks the thread right there. For an async sequence — one whose next element might still be in flight over the network — that's exactly the problem `Future` solved for a single value: instead of blocking, return `Pending` and hand control back to the runtime. `Stream` is precisely that combination: the shape of `Iterator`, polled like `Future`.

## The concept

### Where `Stream` actually comes from

`Stream` isn't `tokio`'s own trait. It comes from `futures_core` — a small, foundational crate that holds a handful of core async traits (`Stream` among them) without pulling in any executor or extra adapters. `tokio_stream` — the crate this lesson uses — re-exports that same trait and adds adapters on top of it; this course doesn't need the full `futures` crate.

```rust
pub trait Stream {
    type Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;
}
```

Put that next to the two traits you already know:

- `Iterator::next(&mut self) -> Option<Self::Item>` — synchronous, immediate: you either have a value or `None`.
- `Future::poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>` — the same polling mechanics [2.8.5](../../08-concurrency/05-futures-and-runtimes/README.md) gave you, for *one* value.
- `Stream::poll_next` has both at once: `Future`'s polling signature, but its output is `Poll<Option<T>>`, not `Poll<T>` — because a stream, unlike a `Future`, may go `Ready` many times, one item at a time, until it finally returns `Ready(None)` and is done.

```senpai-visual
{"kind":"async","labels":["first poll_next: Ready(Some(item 1))","second poll_next: Ready(Some(item 2))","third poll_next: Ready(Some(item 3))","fourth poll_next: Ready(None) — done"]}
```

### `StreamExt` and the core idiom

`Stream` itself has no convenience methods — exactly like `Iterator`, everything rides on that one required method. `StreamExt` (from `tokio_stream`) plays the same role the `Iterator` adapters did in [2.2](../../02-iterators-and-closures/README.md) — with one important difference: `StreamExt` is not in the prelude, so you have to `use` it yourself.

The simplest way to build a stream is turning a plain iterator into one:

```rust
use tokio_stream::StreamExt;

let mut numbers = tokio_stream::iter(vec![10, 20, 30]);

while let Some(n) = numbers.next().await {
    println!("got {n}");
}
```

```text
got 10
got 20
got 30
```

`while let Some(item) = stream.next().await { ... }` is exactly the idiom this lesson uses from here on. If you've worked with Python, the mental model is `async for item in stream:` — but Rust has no such syntax; `StreamExt::next()` fills that exact gap with an ordinary method instead of a new keyword.

### Adapters chain — exactly like `Iterator`

The chaining habit from [2.2.2](../../02-iterators-and-closures/02-iterator-adapters/README.md) carries over almost unchanged:

```rust
let mut doubled_evens = tokio_stream::iter(1..=10)
    .filter(|n| n % 2 == 0)
    .map(|n| n * 2);

while let Some(n) = doubled_evens.next().await {
    println!("{n}");
}
```

```text
4
8
12
16
20
```

Like `Iterator` adapters, these are **lazy**: `.filter()`/`.map()` alone do nothing, they only describe a new stream; only `.next().await` actually drives a step forward.

### A real stream: values over time

So far `tokio_stream::iter` handed back every value it had immediately. A real stream more often comes from a channel receiver or data arriving over the network; to see the real behavior without a network, we combine `.then()` — which runs one `Future` per item — with a real `sleep`:

```rust
let start = Instant::now();
let ticks = tokio_stream::iter([40u64, 40, 40]).then(|delay_ms| async move {
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    delay_ms
});
tokio::pin!(ticks);
```

Building `ticks` hasn't run a single `sleep` yet — a stream, exactly like a `Future`, is lazy. `tokio::pin!` is also required: the `Future` `.then()` builds from our closure is, just like the state machine [2.8.5](../../08-concurrency/05-futures-and-runtimes/README.md) showed you every real `async fn` compiles to, not `Unpin` — and `StreamExt::next()` needs an `Unpin` stream.

Now consume it:

```rust
while let Some(delay_ms) = ticks.next().await {
    println!("tick (waited {delay_ms}ms) — elapsed so far: {:?}", start.elapsed());
}
```

```text
tick (waited 40ms) — elapsed so far: 43.4923ms
tick (waited 40ms) — elapsed so far: 89.5897ms
tick (waited 40ms) — elapsed so far: 137.5148ms
```

(The exact "elapsed" numbers will vary a little on your machine — but they always land close to a multiple of 40ms, never near zero.) Every `sleep` genuinely happened, one after another — `.then()` never pulls the next item out of the underlying stream until the current item's `Future` finishes.

### `select!` on a stream — exactly like any other `Future`

[2.9.2](../02-select-and-cancellation-safety/README.md) gave you `select!`: it lines up several `Future`s at once, and whichever becomes ready first wins. `stream.next()` returns a `Future` too — so no new mechanism is needed, just one more branch inside the same `select!`:

```rust
loop {
    tokio::select! {
        item = ticks.next() => match item {
            Some(delay_ms) => println!("tick (waited {delay_ms}ms)"),
            None => {
                println!("stream ended");
                break;
            }
        },
        _ = tokio::time::sleep(Duration::from_millis(200)) => {
            println!("timed out waiting for the next tick");
            break;
        }
    }
}
```

```text
tick (waited 30ms)
tick (waited 30ms)
tick (waited 30ms)
stream ended
```

(Here three 30ms ticks race a 200ms timeout — with that much headroom, the stream always wins; if the timeout were shorter than the gap between ticks, the `sleep` branch would win every time instead. `select!`'s choice between two branches that become ready at genuinely the same instant is not guaranteed; the gap here is deliberately wide enough that the outcome stays the same every run.)

```senpai-visual
{"kind":"concurrency","labels":["ticks.next() is awaited","sleep(200ms) is awaited in parallel","a stream tick arrives before the deadline","print it, loop back into select! again","stream ends: break out of the loop"]}
```

## Hands on

```sh
cargo run -p p2-09-03-streams --example 01-iter-and-next
cargo run -p p2-09-03-streams --example 02-adapters-chained
cargo run -p p2-09-03-streams --example 03-time-spaced-stream
cargo run -p p2-09-03-streams --example 04-select-vs-timeout
```

Then the two broken ones:

```sh
cargo run -p p2-09-03-streams --example 05-forgot-streamext --features broken
cargo run -p p2-09-03-streams --example 06-for-loop-on-stream --features broken
```

Then try these:

1. In `02-adapters-chained`, swap the order of `.filter()` and `.map()` — `.map(|n| n * 2)` first, then `.filter(|n| n % 2 == 0)`. Is the output the same? Why or why not?
2. In `03-time-spaced-stream`, change the three `[40, 40, 40]` delays to `[10, 100, 10]`. Guess which number each "elapsed so far" will land near — then run it and check.
3. In `04-select-vs-timeout`, change `200` to `20`. Which branch wins every time now, and why?

## Errors you will meet

### `E0599` — no method named `next` found

```text
error[E0599]: no method named `next` found for struct `tokio_stream::Iter<I>` in the current scope
   --> phase2-intermediate\09-async-in-practice\03-streams\examples\05-forgot-streamext.rs:11:33
    |
 11 |     while let Some(n) = numbers.next().await {
    |                                 ^^^^
    |
   ::: C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\tokio-stream-0.1.18\src\stream_ext.rs:144:8
    |
144 |     fn next(&mut self) -> Next<'_, Self>
    |        ---- the method is available for `tokio_stream::Iter<std::vec::IntoIter<{integer}>>` here
    |
    = help: items from traits can only be used if the trait is in scope
help: trait `StreamExt` which provides `next` is implemented but not in scope; perhaps you want to import it
    |
  8 + use tokio_stream::StreamExt;
    |
help: there is a method `try_next` with a similar name
    |
 11 |     while let Some(n) = numbers.try_next().await {
    |                                 ++++

For more information about this error, try `rustc --explain E0599`.
```

**What the compiler is objecting to:** unlike `Iterator`, whose `.next()` is free and in the prelude, `StreamExt` is an ordinary trait you must explicitly `use`. The compiler even sees that `tokio_stream::Iter<...>` really does have a `next` method — it just comes from a trait that isn't in scope.

**The fix:** the compiler's own suggested line:

```rust
use tokio_stream::StreamExt;
```

**Why this is the fix:** with that `use`, every `StreamExt` method — `.next()`, `.map()`, `.filter()`, everything you've seen — becomes available on any `Stream`; exactly what the compiler's help note already said.

### `E0277` — a `Stream` is not an `Iterator`

```text
error[E0277]: `tokio_stream::Iter<std::vec::IntoIter<{integer}>>` is not an iterator
  --> phase2-intermediate\09-async-in-practice\03-streams\examples\06-for-loop-on-stream.rs:11:14
   |
11 |     for n in numbers {
   |              ^^^^^^^ `tokio_stream::Iter<std::vec::IntoIter<{integer}>>` is not an iterator
   |
   = help: the trait `Iterator` is not implemented for `tokio_stream::Iter<std::vec::IntoIter<{integer}>>`
   = note: required for `tokio_stream::Iter<std::vec::IntoIter<{integer}>>` to implement `IntoIterator`

For more information about this error, try `rustc --explain E0277`.
```

**What the compiler is objecting to:** `for x in value` needs `IntoIterator`, and `Stream` deliberately does not implement it — a synchronous `for` loop has nowhere to return `Pending` and hand control back. This is exactly the trap `tokio_stream`'s own documentation explicitly warns newcomers about.

**The fix:** `while let` plus `.next().await`:

```rust
while let Some(n) = numbers.next().await {
    println!("{n}");
}
```

**Why this is the fix:** `while let` only ever `.await`s one step at a time — exactly the mechanism that lets a stream genuinely suspend between two items, something `for` cannot do.

## Exercises

### Warm up

<details>
<summary>What does this print? (assume <code>StreamExt</code> is already <code>use</code>d)</summary>

```rust
let mut s = tokio_stream::iter(vec!["a", "b"]);
println!("{:?}", s.next().await);
println!("{:?}", s.next().await);
println!("{:?}", s.next().await);
```

</details>

<details>
<summary>Answer</summary>

```text
Some("a")
Some("b")
None
```

The `Vec` had two elements; the third `.next().await` finds the stream exhausted and returns `None` — exactly like the last `.next()` of an `Iterator`.

</details>

<details>
<summary>Does this compile?</summary>

```rust
use tokio_stream::StreamExt;

let numbers = tokio_stream::iter(vec![1, 2, 3]);
for n in numbers {
    println!("{n}");
}
```

</details>

<details>
<summary>Answer</summary>

No — `E0277`, "is not an iterator". Even with `StreamExt` in scope, `for` needs `IntoIterator`, and `Stream` doesn't implement it. You need `while let Some(n) = numbers.next().await` instead.

</details>

<details>
<summary>True or false: building <code>tokio_stream::iter(v).then(|x| async move { ... })</code>, on its own, immediately starts whatever <code>sleep</code> is inside it.</summary>

</details>

<details>
<summary>Answer</summary>

False. A stream, exactly like a `Future`, is lazy — until something `.next().await`s it, no `poll_next` is ever called and no `sleep` ever starts.

</details>

### Repair

Fix both broken examples:

1. Fix `examples/05-forgot-streamext.rs` by adding `use tokio_stream::StreamExt;`.
2. Fix `examples/06-for-loop-on-stream.rs` by turning `for n in numbers { ... }` into a `while let Some(n) = numbers.next().await { ... }` loop.

### Implement

Two functions in `src/lib.rs`:

- `sum_stream(values: Vec<i32>) -> i32` — turns `values` into a stream with `tokio_stream::iter` and sums every item by driving it with a `while let Some(...) = ... .next().await` loop (not `values.iter().sum()`). An empty `values` sums to `0`.
- `first_n_even(values: Vec<i32>, limit: usize) -> Vec<i32>` — builds a stream from `values`, keeps only the even numbers, keeps only the first `limit` of those (in their original order), and returns them as a `Vec<i32>`. If fewer than `limit` even numbers exist, returns all of them.

```sh
cargo test -p p2-09-03-streams
```

### Build

Write `pub async fn ticks_after(delays_ms: Vec<u64>) -> Vec<u64>`: exactly like the `03-time-spaced-stream` pattern — it waits on a real `tokio::time::sleep` for each element of `delays_ms`, one after another — then returns all the values, in the same order, in a `Vec`. Then write yourself a small test that uses `Instant` to prove the whole thing genuinely takes close to the sum of `delays_ms` — not something near zero.

### Challenge (optional)

Without any adapter and without `tokio_stream::iter`, build a `CountdownStream` type that holds a `remaining: u32` and implements `Stream` for it by hand — the same spirit as `Countdown::poll` in [2.8.5](../../08-concurrency/05-futures-and-runtimes/README.md), this time for `Stream`: each `poll_next`, if `remaining` is zero, returns `Poll::Ready(None)`; otherwise it subtracts one from `remaining` and returns the new value wrapped in `Poll::Ready(Some(...))`. (Because `CountdownStream` holds no internal references, it's automatically `Unpin` — no need for `Box::pin` or `tokio::pin!`.)

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `Stream` | The async counterpart of `Iterator`; `poll_next` is polled like `Future::poll`, but returns `Poll<Option<T>>` | Any sequence of values that arrive over time |
| `StreamExt` | `Stream`'s adapters — `.next()`, `.map()`, `.filter()`, `.take()`, `.then()` — from `tokio_stream`, not the prelude | `use tokio_stream::StreamExt;` at the top of any file that touches a stream |
| `tokio_stream::iter` | The simplest way to build a `Stream` from an ordinary iterator | Examples, tests, and anywhere your data is already in memory |
| `while let Some(item) = stream.next().await` | The core idiom for driving a stream — Rust has no `async for` | Anywhere you need to consume a stream |
| `.then()` | Runs one `Future` per item; the resulting stream is often not `Unpin` | Building a stream whose every value genuinely takes time |

### What you now know

- Exactly how `Stream::poll_next` relates to `Iterator` and `Future`, and you can write the signature from memory.
- Why you must `use tokio_stream::StreamExt;` yourself, and why `for x in stream` never compiles.
- The core idiom for consuming a stream — `while let Some(item) = stream.next().await` — and that `.map()`/`.filter()`/`.take()` are lazy and chainable exactly like `Iterator`.
- How to build a stream with `.then()` and a real `sleep` whose values genuinely arrive over time, and why you need `tokio::pin!` before `.next()` on it.
- How `select!` races a stream's `.next().await` against another `Future`, with no new mechanism at all.

### What comes back later

- **Async traits, and moving heavy/blocking work off the runtime with `spawn_blocking`** — [2.9.4 — Async traits and `spawn_blocking`](../04-async-traits-and-blocking/README.md), this module's closing lesson.
- **Real streams: rows from an `sqlx` query, messages from an `axum` websocket** — [Phase 3](../../../phase3-backend-foundations/README.md), where you'll see this exact `while let Some(...) = ... .next().await` over real data, not a `Vec` sitting in memory.

### Can you explain?

- Why does `Stream::poll_next`'s signature look like both `Future::poll` and `Iterator::next`? In your own words, say which part of the signature comes from which.
- Why do you have to `use` `StreamExt` yourself, when you never had to `use` `Iterator`?
- Why doesn't `for x in stream` compile?
- Why is `tokio::pin!` needed before `.next()` on a stream built with `.then()`?
- How can `select!` race a stream's `.next().await` against another `Future`, with no new mechanism at all?

## Going further

- [`futures_core::Stream` docs](https://docs.rs/futures-core/latest/futures_core/stream/trait.Stream.html) — the exact trait you saw today, with no adapters on top.
- [`tokio_stream::StreamExt` docs](https://docs.rs/tokio-stream/latest/tokio_stream/trait.StreamExt.html) — the full adapter list: `.merge()`, `.chain()`, `.timeout()`, `.throttle()`, and more.
- [The "Streams" chapter of the official tokio tutorial](https://tokio.rs/tokio/tutorial/streams) — the same idiom, with a real channel example.
- [The `async-stream` crate](https://docs.rs/async-stream/latest/async_stream/) — a macro that gives you `yield`-like syntax for building streams; `tokio_stream`'s own docs point to it for whenever today's `.then()` pattern isn't enough.
