# 2.8.6 — `tokio` basics

## At a glance

After this lesson you can:

- Explain what the `#[tokio::main]` macro actually expands into — and write the same program by hand, with no macro, using `Runtime::new().block_on(...)`.
- Contrast `tokio::spawn` with `thread::spawn` from [2.8.1](../01-threads-mutex-arc/README.md) and say concretely why a task is dramatically cheaper than an OS thread.
- Build a small program that spawns several tasks, has each one wait on a real `sleep`, and collects their results in order — and show with real numbers exactly why that is faster than the sequential version.

**Time:** ~65 minutes · **Prerequisites:**
[2.8.1 — Threads, `Mutex`, `Arc`](../01-threads-mutex-arc/README.md),
[2.8.4 — `Send` and `Sync`: what they are and why your type isn't `Send`](../04-send-and-sync/README.md),
[2.8.5 — Futures and runtimes](../05-futures-and-runtimes/README.md)

## Why this matters

Last lesson you hand-built a `Future`, a `poll` loop, and the smallest possible executor (`block_on`) — specifically so that when `tokio` shows up today, none of it looks like magic. `tokio` is the same idea, just engineered and industrial-strength: a real scheduler, real timers, a real pool of OS threads — exactly what last lesson's `block_on` did at the smallest possible scale, minus the naive busy-loop that burned 100% of a CPU core for nothing.

This lesson also closes the concurrency module. Up to here, [2.8.1](../01-threads-mutex-arc/README.md) through [2.8.5](../05-futures-and-runtimes/README.md) gave you two genuinely different models of concurrency, and the formal rule underneath both of them. Today, for the first time, you see all of them together in one real, runnable program — and today's "Wrapping up" is a wrap-up of the whole module, not just this one lesson.

## The concept

### `#[tokio::main]`: sugar, not magic

Run this:

```rust
use std::time::Duration;

#[tokio::main]
async fn main() {
    tokio::time::sleep(Duration::from_millis(50)).await;
    println!("done after a real 50ms sleep");
}
```

```text
done after a real 50ms sleep
```

Rust itself does not let `fn main` be `async` — something has to build an executor and run it first, and the compiler does not do that for you (see the exact error in "Errors you will meet"). `#[tokio::main]` does precisely that one job: it generates an ordinary, synchronous `fn main` — the kind the operating system expects — whose body is: build a `Runtime`, then `block_on` your `async fn main` body on it. That's the whole trick, nothing else hidden. By hand, with no macro, the exact same program looks like this:

```rust
use std::time::Duration;

fn main() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        println!("done after a real 50ms sleep");
    });
}
```

```text
done after a real 50ms sleep
```

Two programs, one output. `Runtime::new()` is literally what `#[tokio::main]` calls behind the scenes (with the same defaults), and `.block_on(fut)` is exactly the same repeated-polling you hand-wrote in [2.8.5](../05-futures-and-runtimes/README.md) — except here, instead of a busy loop, a real `Waker` tells the runtime *precisely* when polling again is worth doing; between those moments the runtime's threads never sit idle waiting, they're free to push other tasks forward.

If you've used Python, this pattern is familiar: `asyncio.run(main())` plays the exact same role — build an event loop, drive `main()` on it to completion.

```python
async def main():
    await asyncio.sleep(0.05)
    print("done after a real 50ms sleep")

asyncio.run(main())  # builds, runs, and tears down — exactly what #[tokio::main] does
```

You'll see exactly where this comparison breaks a couple of sections down, once you get to `tokio`'s two runtime flavors.

### `tokio::spawn` vs. `thread::spawn`: a task, not a thread

[2.8.1](../01-threads-mutex-arc/README.md) showed you that `thread::spawn(closure)` hands your closure to a genuinely separate OS thread, and `.join()` blocks until it finishes. `tokio::spawn(future)` plays a similar role — it hands a `Future` to the runtime as a fully independent **task**, and gives back a handle you `.await` (not `.join()`) to get the result — but underneath that surface-level similarity sit three real differences:

- **Cost.** An OS thread reserves a multi-megabyte stack, and creating one is real work for the operating system itself. A task takes up only as much room as whatever its `Future` actually holds — usually a small fraction of that. That's why spawning several thousand tasks is completely ordinary, where several thousand OS threads would quickly exhaust memory and OS handles.
- **Scheduling.** The OS can preempt a thread mid-instruction and run something else instead. A task is scheduled **cooperatively**: it only ever hands control back at an `.await` point. A task that never awaits and just sits in a heavy computation starves every other task sharing its worker thread — the real cost of being cooperative.
- **Where it runs.** `tokio`'s default runtime spreads tasks across a small pool of real OS threads, not just one — which is exactly why `tokio::spawn` requires your `Future` to be `Send + 'static` ([2.8.4](../04-send-and-sync/README.md) gave you that exact rule): the runtime may move your task from one worker thread to another between polls.

This is exactly where the Python comparison breaks. `asyncio.Task` is also cheap and cooperative — that part still matches — but Python's default event loop runs on **exactly one** OS thread (and [2.8.1](../01-threads-mutex-arc/README.md)'s GIL is lurking there too, in case you tried to work around it). `tokio`'s default runtime, because Rust has no GIL, genuinely runs tasks across several cores at once — not merely interleaved on one.

### Building it up: spawn everything first, then await

Here's the helper — simulated I/O that genuinely `.await`s, not something synchronous wearing a mask:

```rust
async fn fetch(id: u32, delay_ms: u64) -> String {
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    format!("item-{id} (waited {delay_ms}ms)")
}
```

Now spawn two of these and `.await` both handles in turn:

```rust
let start = Instant::now();
let one = tokio::spawn(fetch(1, 200));
let two = tokio::spawn(fetch(2, 200));

println!("{}", one.await.unwrap());
println!("{}", two.await.unwrap());
println!("total: {:?}", start.elapsed());
```

```text
item-1 (waited 200ms)
item-2 (waited 200ms)
total: 201.7782ms
```

(The exact final number will vary a little on your machine — it always stays just above 200ms, never anywhere near 400.) Two 200ms sleeps, but the whole program took only about 200ms, not 400. The structural point is right here: **both `tokio::spawn` calls run before either `.await`.** By the moment the second line runs, both tasks are already independently counting down their own timers; `one.await` just waits for task one to finish — task two is still, at the same time, making progress. If you had instead written `tokio::spawn(fetch(...)).await` over and over inside one loop, each spawn would be awaited the instant it happened and fully finish before the next one starts — sequential again, just with pointless spawning overhead on top.

```senpai-visual
{"kind":"concurrency","labels":["spawn task 1","spawn task 2","both sleeping now","await task 1","await task 2"]}
```

### Two runtime flavors: `multi_thread` vs. `current_thread`

`#[tokio::main]` builds a **`multi_thread`** runtime by default — exactly what you saw above, spreading tasks across several real OS threads. There's another flavor too, for when you genuinely don't need more than one thread (a small CLI tool, a simple test, or code you deliberately want pinned to one thread):

```rust
#[tokio::main(flavor = "current_thread")]
async fn main() {
    // exactly the same body as above — nothing else changes
}
```

Running the full file (`examples/04-current-thread-flavor.rs` — the same body as the last example, just with this one attribute swapped) gives:

```text
item-1 (waited 200ms)
item-2 (waited 200ms)
total: 210.7788ms
```

Same rough number, same concurrency — even with a single OS thread. This is exactly what "2.8.6 at a glance" promised: `tokio`'s concurrency comes from **cooperative task scheduling**, not from genuine multi-core parallelism; running on several threads is an extra decision layered on top, for when you also want real parallelism. This is also exactly where the `asyncio.run` comparison above breaks: Python never even offers you this choice — its event loop always stays on that one `current_thread`-like thread; `tokio` gives you both, and defaults to multi-threaded.

## Hands on

```sh
cargo run -p p2-08-06-tokio-basics --example 01-tokio-main-sugar
cargo run -p p2-08-06-tokio-basics --example 02-desugared-block-on
cargo run -p p2-08-06-tokio-basics --example 03-spawn-two-tasks
cargo run -p p2-08-06-tokio-basics --example 04-current-thread-flavor
```

Then the three broken ones:

```sh
cargo run -p p2-08-06-tokio-basics --example 05-forgot-tokio-main --features broken
cargo run -p p2-08-06-tokio-basics --example 06-await-outside-async --features broken
cargo run -p p2-08-06-tokio-basics --example 07-spawn-without-runtime --features broken
```

Then try these:

1. In `03-spawn-two-tasks`, replace the two `tokio::spawn` lines with directly `fetch(1, 200).await` and `fetch(2, 200).await` (no spawn). Does the program still compile? What does `total` become, and why exactly that number?
2. On the same file, instead of 200ms for both, make one 50 and the other 500. Predict which number `total` ends up close to — then run it and see.
3. In `01-tokio-main-sugar`, remove `#[tokio::main]` and try `#[tokio::test]` on a small test function instead (add a `#[cfg(test)] mod tests` right in the same file). How is it different from a plain `#[test]`?

## Errors you will meet

### `E0752` — `main` function is not allowed to be `async`

```text
error[E0752]: `main` function is not allowed to be `async`
  --> phase2-intermediate\08-concurrency\06-tokio-basics\examples\05-forgot-tokio-main.rs:10:1
   |
10 | async fn main() {
   | ^^^^^^^^^^^^^^^ `main` function is not allowed to be `async`

For more information about this error, try `rustc --explain E0752`.
```

**What the compiler is actually objecting to:** the language itself, with no macro involved, has no idea how to run an `async fn main` — something has to build a runtime first and `poll` the body on it, and `rustc` does not do that on its own.

**The fix:** add `#[tokio::main]` right above the function:

```rust
#[tokio::main]
async fn main() {
    tokio::time::sleep(Duration::from_millis(10)).await;
    println!("done");
}
```

**Why this is the fix:** that one line is exactly what you saw in "The concept" — it builds a real, synchronous `fn main` that builds a `Runtime` and `block_on`s your `async` body on it.

### `E0728` — `await` is only allowed inside `async` functions and blocks

```text
error[E0728]: `await` is only allowed inside `async` functions and blocks
  --> phase2-intermediate\08-concurrency\06-tokio-basics\examples\06-await-outside-async.rs:12:51
   |
11 | fn main() {
   | --------- this is not `async`
12 |     tokio::time::sleep(Duration::from_millis(10)).await;
   |                                                   ^^^^^ only allowed inside `async` functions and blocks

For more information about this error, try `rustc --explain E0728`.
```

**What the compiler is actually objecting to:** `.await` is precisely the point where a task hands control back to a runtime. A plain `fn` has no runtime underneath it to hand control back to — so the expression has no meaning at all.

**The fix:** either make the function `async` and put `#[tokio::main]` on it, or — if it genuinely has to stay a plain `fn` — wrap the body in `Runtime::new().unwrap().block_on(async { ... })`, exactly as you saw in "The concept".

**Why this is the fix:** both routes do the exact same job: build a real runtime so `.await` has somewhere to hand control back to. Without one of the two, `.await` never has meaning.

### Panic — "there is no reactor running"

```text
thread 'main' (30988) panicked at phase2-intermediate\08-concurrency\06-tokio-basics\examples\07-spawn-without-runtime.rs:12:5:
there is no reactor running, must be called from the context of a Tokio 1.x runtime
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**What the compiler is actually objecting to:** this isn't even a compiler error — `tokio::spawn` is an ordinary function, not an `async fn`, so calling it needs neither `.await` nor an `async` function; it compiles cleanly. But it still needs a runtime that is *actually running* to hand the task to, and this `fn main` never built one — so the program runs, and panics the moment it reaches `tokio::spawn`. (The `(30988)` is this thread's OS id — like the timing numbers elsewhere in this lesson, it'll be a different number on your own machine and even between runs.)

**The fix:** the same one line as always: make the function `async` and put `#[tokio::main]` on it.

**Why this is the fix:** with that change, by the time `tokio::spawn` is called, the runtime has already been built and is already running — exactly the thing the panic message said was missing.

## Exercises

### Warm up

<details>
<summary>What does this print, and roughly how long does the whole program take?</summary>

```rust
#[tokio::main]
async fn main() {
    tokio::time::sleep(Duration::from_millis(50)).await;
    println!("first");
    tokio::time::sleep(Duration::from_millis(50)).await;
    println!("second");
}
```

</details>

<details>
<summary>Answer</summary>

```text
first
second
```

And the whole program takes something close to the sum of the two 50ms sleeps (typically 115-130ms on this machine — the exact number depends on the machine, but it never gets close to 50) — two `.await`s back to back, no `spawn` anywhere, so it's entirely sequential. `.await` by itself makes nothing concurrent; it just waits.

</details>

<details>
<summary>Does this compile?</summary>

```rust
fn main() {
    let handle = tokio::spawn(async { 42 });
    println!("{handle:?}");
}
```

</details>

<details>
<summary>Answer</summary>

No — it *compiles* fine (`tokio::spawn` is an ordinary function), but it panics the moment it runs: no runtime is running to hand the task to. Exactly what you saw in "Errors you will meet".

</details>

<details>
<summary>True or false: <code>tokio::spawn</code> creates a fresh OS thread.</summary>

</details>

<details>
<summary>Answer</summary>

False. `tokio::spawn` creates a **task** — an independent `Future` the runtime schedules, typically onto a pool of threads that already exist. No new thread is created per `spawn`; that's exactly the difference that makes a task so much cheaper than a thread.

</details>

### Repair

Fix all three broken examples:

1. Fix `examples/05-forgot-tokio-main.rs` so it compiles.
2. Fix `examples/06-await-outside-async.rs` **two** different ways: once by turning `main` into `async` + `#[tokio::main]`, once by keeping a plain `fn main` and wrapping the body in `Runtime::new().unwrap().block_on(async { ... })`.
3. Fix `examples/07-spawn-without-runtime.rs` so it no longer panics, and genuinely prints `"hi"`.

### Implement

Two functions in `src/lib.rs`:

- `fetch_simulated(id: u32, delay_ms: u64) -> String` — genuinely `.await`s for `delay_ms` milliseconds (with `tokio::time::sleep`), then returns exactly the string `"item-{id}"` — e.g. for `id = 7`, the string `"item-7"`.
- `fetch_all_concurrently(ids: Vec<u32>, delay_ms: u64) -> Vec<String>` — fetches every id in `ids` **concurrently** (not one at a time) with `fetch_simulated`, returning the results in a `Vec` ordered exactly like `ids` (not the order the tasks actually finished in).

```sh
cargo test -p p2-08-06-tokio-basics
```

### Build

Write `pub async fn fetch_with_custom_delays(requests: Vec<(u32, u64)>) -> Vec<String>`: fetches every `(id, delay_ms)` pair with `fetch_simulated`, concurrently — this time each with its own delay, not one shared delay — returning the results in the order of `requests`. Then write yourself a small test with `Instant` proving that three requests of 80, 80, and 240ms, run concurrently, take about 240ms, not 400.

### Challenge (optional)

Look up the `tokio::join!` macro's docs — this lesson never formally taught it, but it's a first look at something [2.9](../../09-async-in-practice/README.md) opens up properly. For exactly two ids (not a `Vec`), write a version of `fetch_all_concurrently` that uses `tokio::join!` instead of `tokio::spawn`. Does its timing match the spawned version? Why does this one never need `Send + 'static`? (Hint: nothing here is ever handed to the runtime — everything stays inside the same one calling task, just alternating between its `.await` points.)

## Wrapping up

The module's actual arc, start to finish: [2.8.1](../01-threads-mutex-arc/README.md) gave you the first concurrency model — real OS threads sharing state through `Mutex`/`Arc`. [2.8.2](../02-rwlock-semaphore-oncelock-atomics/README.md) filled that model out with tools for more specific shapes of the same problem: `RwLock` for read-heavy access, a semaphore for bounding how many threads touch a resource at once, `OnceLock`/`LazyLock` for one-time setup, atomics for a single small value. [2.8.3](../03-channels-message-passing/README.md) offered a genuinely different model instead of sharing memory at all — moving ownership between threads through a channel. [2.8.4](../04-send-and-sync/README.md) was the payoff lesson: the formal `Send`/`Sync` rule that was quietly underneath every one of those tools the whole time, finally explaining why `Arc`/`Mutex` cross threads safely and `Rc`/`RefCell` never do. [2.8.5](../05-futures-and-runtimes/README.md) then pivoted the whole module: not more OS-thread tools, but what `async fn` and `.await` actually *are* underneath — a state machine implementing `Future`, doing nothing until something polls it. Today, `tokio` was that something: a real scheduler standing in for the toy `block_on` you hand-built last lesson, so the same `Future`s you now understand mechanically can actually run.

| Term | What it means | Where you'll use it |
|---|---|---|
| `#[tokio::main]` | Generates a synchronous `fn main` that builds a `Runtime` and `block_on`s your body | The entry point of every `tokio` program |
| `Runtime::new().block_on(...)` | The manual, macro-free version of the same thing | Understanding what the macro actually does |
| Task | The independent unit of work `tokio::spawn` hands to a runtime | `tokio::spawn(future)` |
| `tokio::spawn` | Runs a `Future` as a task, concurrently with everything else | Anywhere several I/O-bound jobs need to progress at once |
| `tokio::time::sleep` | A genuine `.await`, with no network needed, to demonstrate concurrency | This lesson's examples and tests |
| Runtime flavor (`multi_thread` / `current_thread`) | Several real OS threads vs. just one, under the same concurrency model | `#[tokio::main(flavor = "...")]` |

### What you now know

- Exactly what `#[tokio::main]` expands into, and how to write the same program by hand with `Runtime::new().block_on(...)`.
- `tokio::spawn` creates a task, not a thread — cheaper, cooperatively scheduled, and (under the default flavor) spread across several real threads.
- Why you have to spawn every task first and only then `.await` them — and why spawn-then-immediately-await is fully sequential all over again.
- The difference between the two runtime flavors, `multi_thread` (the default) and `current_thread`, and why concurrency works identically under both.
- Three errors you'll hit the moment you forget `#[tokio::main]` — two at compile time, one at run time — and why all three share the exact same fix.

### What comes back later

- **Spawning a whole group of tasks in a structured way, with `JoinSet`** — [2.9.1](../../09-async-in-practice/01-spawn-joinset-structured-concurrency/README.md)
- **Waiting on several `Future`s at once and cancelling them safely with `select!`** — [2.9.2](../../09-async-in-practice/02-select-and-cancellation-safety/README.md)
- **Async sequences — values that arrive one at a time, over time** — [2.9.3](../../09-async-in-practice/03-streams/README.md)
- **Async traits, and moving heavy/blocking work off the runtime with `spawn_blocking`** — [2.9.4](../../09-async-in-practice/04-async-traits-and-blocking/README.md)
- Further out on the horizon: [Phase 3](../../../phase3-backend-foundations/README.md), where `tokio` stops being something you call directly by hand and becomes the engine running underneath `axum` and `sqlx`, keeping your work moving.

### Can you explain?

- What does `#[tokio::main]` actually do? In your own words, without saying "magic".
- Why is a `tokio` task dramatically cheaper than an OS thread? Name three concrete reasons.
- Why does `tokio::spawn(fut).await` inside a loop give you no concurrency at all, while spawning everything first and awaiting after does?
- What's the difference between `multi_thread` and `current_thread`? Why do both give you the same concurrency?
- Why can `tokio::spawn` still panic, even without ever being `.await`ed?

## Going further

- [`tokio::main` docs](https://docs.rs/tokio/latest/tokio/attr.main.html) — every macro option, including `flavor`, `worker_threads`, and `start_paused`.
- [The "Spawning" chapter of the official tokio tutorial](https://tokio.rs/tokio/tutorial/spawning) — the same task/thread distinction, with more examples.
- [`Runtime` docs](https://docs.rs/tokio/latest/tokio/runtime/struct.Runtime.html) — `Runtime::new`'s exact signature, and why it sits behind the `rt-multi-thread` feature.
- [`tokio::join!`](https://docs.rs/tokio/latest/tokio/macro.join.html) — for the "Challenge" above; waiting on several `Future`s without spawning any of them.
