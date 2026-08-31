# 2.9.2 — `select!` and cancellation safety

## At a glance

After this lesson you can:

- Explain exactly what `select!` does — races several `Future`s on one task, only the first to finish wins, and every other branch's future is dropped that same instant — and contrast it with [2.9.1](../01-spawn-joinset-structured-concurrency/README.md)'s `join!`/`JoinSet`, which wait for every one of them instead.
- Race a real operation against a deadline with `select!` — either by hand (a `sleep` branch) or with the ready-made shortcut `tokio::time::timeout` — and say why the two do exactly the same job.
- Spot a not-cancellation-safe future from a real, reproducible bug, and fix it by moving the risky side effect out of the raced branch; and use `CancellationToken` to ask a running task to stop cooperatively from another part of a program.

**Time:** ~75 minutes · **Prerequisites:**
[2.9.1 — `spawn`, `JoinSet`, structured concurrency](../01-spawn-joinset-structured-concurrency/README.md),
[2.8.1 — Threads, `Mutex`, `Arc`](../../08-concurrency/01-threads-mutex-arc/README.md),
[2.8.6 — `tokio` basics](../../08-concurrency/06-tokio-basics/README.md)

## Why this matters

The `JoinSet` [2.9.1](../01-spawn-joinset-structured-concurrency/README.md) gave you carries one fixed assumption: every task you hand it, you eventually need — so it holds onto every one until they're all finished.

Plenty of real programs want the exact opposite. An HTTP request you'll wait at most 2 seconds for, never longer. A background worker that has to stop right now — not "whenever it happens to finish" — the moment the program is shutting down. A client that fires the same request at two mirrored servers and only cares about whichever answer arrives first, not both. In all three, you don't *want* to wait for everything; you want whichever finishes first, and you want to throw the rest away. `select!` is exactly that tool — and that "throwing away" is what this entire lesson is about.

## The concept

### `select!`: whichever finishes first wins, the rest get dropped

Run this:

```rust
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    tokio::select! {
        _ = sleep(Duration::from_millis(200)) => {
            println!("the 200ms sleep won");
        }
        _ = sleep(Duration::from_millis(50)) => {
            println!("the 50ms sleep won");
        }
    }
    println!("select! returned - the 200ms sleep's future is already gone, not just paused");
}
```

```text
the 50ms sleep won
select! returned - the 200ms sleep's future is already gone, not just paused
```

Each `select!` branch has a pattern, an `=`, an async expression (here `sleep(...)`), a `=>`, then a body that runs only when *that* branch wins. `select!` builds both `sleep` futures, then polls both of them, over and over, until one of them returns `Ready`. Here the 50ms future becomes `Ready` first; that branch's body runs, and **the whole `select!` returns right then** — not once the 200ms future finishes too. That 200ms future is **dropped** right there: not paused, not suspended, gone — exactly like any other value in Rust is dropped once it goes out of scope.

This is precisely the opposite of what [2.9.1](../01-spawn-joinset-structured-concurrency/README.md) gave you. `join!` and `JoinSet` hold onto **every** future you hand them until all of them finish — none of them ever get thrown away. `select!` is the exact reverse: it keeps only the winner, and the instant it wins, every other branch is dropped, permanently, with no way back.

```senpai-visual
{"kind":"concurrency","labels":["two futures are built: 200ms and 50ms","both get polled together","the 50ms future becomes Ready first","that branch's body runs","the 200ms future is dropped right here - not paused, gone"]}
```

### The canonical use: racing a real operation against a deadline

The most common reason to reach for `select!` is exactly this: race a real operation against a timer to give it a deadline. Say you have this function - a simulated fetch that sometimes takes too long:

```rust
async fn fetch(delay_ms: u64) -> String {
    sleep(Duration::from_millis(delay_ms)).await;
    format!("data after {delay_ms}ms")
}
```

With `select!`, one branch is the fetch, the other is a `sleep` playing the role of the deadline:

```rust
async fn fetch_with_budget(delay_ms: u64, budget_ms: u64) -> Option<String> {
    tokio::select! {
        data = fetch(delay_ms) => Some(data),
        _ = sleep(Duration::from_millis(budget_ms)) => None,
    }
}
```

```rust
match fetch_with_budget(20, 100).await {
    Some(data) => println!("fast fetch: got {data:?}"),
    None => println!("fast fetch: timed out"),
}
match fetch_with_budget(300, 100).await {
    Some(data) => println!("slow fetch: got {data:?}"),
    None => println!("slow fetch: timed out"),
}
```

```text
fast fetch: got "data after 20ms"
slow fetch: timed out
```

With a fixed 100ms budget: the fetch that only takes 20ms arrives first and wins; the fetch that takes 300ms never gets the chance — its future is dropped, mid-flight, the instant the 100ms `sleep` wins, without ever reaching the `sleep(delay_ms)` inside it.

This pattern is common enough that `tokio` has a built-in shortcut for it — `tokio::time::timeout`, which has exactly this same `select!` shape underneath, just handing back a `Result` instead of an `Option`:

```rust
match tokio::time::timeout(Duration::from_millis(100), fetch(20)).await {
    Ok(data) => println!("fast fetch: got {data:?}"),
    Err(_) => println!("fast fetch: timed out"),
}
match tokio::time::timeout(Duration::from_millis(100), fetch(300)).await {
    Ok(data) => println!("slow fetch: got {data:?}"),
    Err(_) => println!("slow fetch: timed out"),
}
```

```text
fast fetch: got "data after 20ms"
slow fetch: timed out
```

Same two outcomes, same two inputs - just `Ok`/`Err` instead of `Some`/`None`. Anywhere you have exactly this shape - "one operation against a fixed deadline" - reach for `tokio::time::timeout`; reach for `select!` itself when you need the deadline to be **something other than** a plain `sleep` - exactly what you'll see a few sections down with `CancellationToken`.

### Cancellation safety, defined concretely

Now go back to that sentence above: "the losing future gets dropped." That sentence was harmless when the losing future was only ever `sleep`ing — an unfinished `sleep`, thrown away, breaks nothing outside itself. But plenty of futures, partway through, do something outside themselves: bump a shared counter, half-write a buffer, take a lock. If `select!` drops that future at exactly that moment, mid-work, the side effect that already happened stays done, but whatever was supposed to undo it later never runs. No panic, no error, nothing to catch — just an outside state that stays wrong forever.

A future is **cancellation-safe** if being dropped mid-poll never leaves any outside state — a shared counter, a partially-written buffer, a lock — in a wrong state. Let's see this as a real bug, not a hypothetical one. This function tracks an "in-flight request" counter — a completely realistic pattern, the kind of thing a metric or a monitoring gauge is built from:

```rust
async fn tracked_fetch(in_flight: Arc<Mutex<u32>>, delay_ms: u64) -> String {
    *in_flight.lock().unwrap() += 1; // "a fetch just started"
    sleep(Duration::from_millis(delay_ms)).await; // the actual work
    *in_flight.lock().unwrap() -= 1; // "it finished" - never runs if dropped first
    "done".to_string()
}
```

Now call this five times, against a budget that always wins, in a loop:

```rust
let in_flight = Arc::new(Mutex::new(0u32));
let attempts = 5;
for _ in 0..attempts {
    tokio::select! {
        _ = tracked_fetch(in_flight.clone(), 200) => {}
        _ = sleep(Duration::from_millis(20)) => {}
    }
}
println!("attempts made: {attempts}");
println!("in_flight counter: {}", *in_flight.lock().unwrap());
```

```text
attempts made: 5
in_flight counter: 5
```

The budget (20ms) always beats the fetch (200ms), so every single time `tracked_fetch` gets dropped mid-way through its own `sleep(delay_ms)` — after the `+= 1` line, but always before it ever reaches the `-= 1` line. That happened five times; the counter went up five times; it never came back down. No panic fired, no error surfaced - just a permanently wrong number nobody would notice without reading this exact code closely.

```senpai-visual
{"kind":"concept","labels":["+= 1 runs: one request is now in flight","sleep starts","the deadline arrives first","select! returns; tracked_fetch is dropped","-= 1 never runs; the counter stays wrong forever"]}
```

### The fix: keep the risky side effect out of the raced branch

The fix needs no new tool at all - just move that `+= 1`/`-= 1` pair out of the future that might lose, into something that never itself gets raced:

```rust
async fn run_one(in_flight: Arc<Mutex<u32>>, delay_ms: u64, budget_ms: u64) -> Option<String> {
    *in_flight.lock().unwrap() += 1; // outside the raced future: always runs
    let result = tokio::select! {
        data = fetch(delay_ms) => Some(data),
        _ = sleep(Duration::from_millis(budget_ms)) => None,
    };
    *in_flight.lock().unwrap() -= 1; // select! has already returned by this line
    result
}
```

The same five-attempt loop as above, this time with `run_one` instead of `tracked_fetch`:

```rust
let in_flight = Arc::new(Mutex::new(0u32));
let attempts = 5;
for _ in 0..attempts {
    run_one(in_flight.clone(), 200, 20).await;
}
println!("attempts made: {attempts}");
println!("in_flight counter: {}", *in_flight.lock().unwrap());
```

```text
attempts made: 5
in_flight counter: 0
```

The structural point: `run_one` itself is never raced against anything - the caller just `.await`s it, plainly, to completion. So `+= 1` and `-= 1` always run in pairs, every single time. The `select!` inside it still does exactly what it did before - it still drops the losing future - but now no dangerous bookkeeping lives inside that droppable future anymore; only the `fetch` itself does, and losing a half-finished `fetch` is completely harmless. The general rule: **keep whatever must not be left half-done by a drop outside the `select!` branch, or restructure so its effect only happens once the winner is already known.**

### `CancellationToken`: asking a task to stop cooperatively

The example above used a *time* deadline. Sometimes the deadline you want isn't time at all - it's an outside signal: the user clicked "cancel," the program is shutting down, some other request no longer needs this one. `CancellationToken`, from the **separate** `tokio-util` crate (not `tokio` itself - it needs its own import), gives you exactly that: a cloneable, cheaply-shareable handle. Every clone of one `CancellationToken` points at the same underlying cancellation; call `.cancel()` on any of them, and all of them find out. `.cancelled()` is itself a future - one that stays `Pending` until somebody calls `.cancel()`, and becomes `Ready` the instant that happens - so it drops straight into a `select!` branch exactly like any other future.

```rust
async fn worker(token: CancellationToken) -> u32 {
    let mut ticks = 0;
    loop {
        tokio::select! {
            _ = token.cancelled() => break,
            _ = sleep(Duration::from_millis(40)) => {
                ticks += 1;
                println!("worker: tick {ticks}");
            }
        }
    }
    ticks
}
```

A controller that wants to stop the worker after a while:

```rust
let token = CancellationToken::new();
let handle = tokio::spawn(worker(token.clone()));
sleep(Duration::from_millis(140)).await;
println!("controller: asking the worker to stop");
token.cancel();
println!("worker completed {} ticks before stopping", handle.await.unwrap());
```

```text
worker: tick 1
worker: tick 2
worker: tick 3
controller: asking the worker to stop
worker: cancelled, stopping
worker completed 3 ticks before stopping
```

(The exact order of the "controller" line relative to "tick 3" varies between runs - the worker and the controller are two fully independent tasks - but the final tick count, with this timing, always stays 3.) The worker ticks every 40ms; the controller calls `cancel()` after 140ms - squarely between the third tick (at 120ms) and the fourth (at 160ms). The worker only breaks out of the loop once it polls `.cancelled()` and finds it `Ready` - not the instant `.cancel()` is called from outside, but exactly when that branch of `select!` next gets its turn to be polled. This is precisely what [2.8.5](../../08-concurrency/05-futures-and-runtimes/README.md) meant by "cooperative" task scheduling: nobody interrupts the worker mid-work; the worker itself, at a point it chose, asks "should I stop?"

```senpai-visual
{"kind":"concurrency","labels":["controller calls token.cancel()","token.cancelled() becomes Ready inside the worker task","select! inside worker picks the cancelled branch","worker breaks out of the loop"]}
```

### `biased;`: turning off the randomness

When more than one `select!` branch is ready at the same time - both already `Ready` right now, rather than one arriving before the other - `tokio` by default picks **randomly** among the ready ones, not necessarily always the first one you wrote. The `biased;` keyword, as the first line inside a `select!` block, turns that randomness off: branches are polled in exactly the order you wrote them, and the first one that's ready - not a random one among the ready ones - wins.

```rust
tokio::select! {
    biased;
    _ = std::future::ready(()) => println!("biased: branch A won"),
    _ = std::future::ready(()) => println!("biased: branch B won"),
}
```

```text
biased: branch A won
```

With `biased;`, this output stays the same every single run - branch one is always polled first. Without `biased;`, run the same two lines several times and sometimes A wins, sometimes B (the code in `examples/07-biased-select.rs` shows both side by side). Most of the time you don't need this - the default randomness exists specifically so a branch that always happens to be written first doesn't starve the rest; reach for `biased;` only when one branch genuinely must always be checked first (a cancellation branch, say, which should always be checked before anything else).

## Hands on

```sh
cargo run -p p2-09-02-select-and-cancellation-safety --example 01-race-two-sleeps
cargo run -p p2-09-02-select-and-cancellation-safety --example 02-fetch-with-timeout
cargo run -p p2-09-02-select-and-cancellation-safety --example 03-timeout-shorthand
cargo run -p p2-09-02-select-and-cancellation-safety --example 04-not-cancellation-safe
cargo run -p p2-09-02-select-and-cancellation-safety --example 05-cancellation-safe
cargo run -p p2-09-02-select-and-cancellation-safety --example 06-cancellation-token
cargo run -p p2-09-02-select-and-cancellation-safety --example 07-biased-select
```

Then the three broken ones:

```sh
cargo run -p p2-09-02-select-and-cancellation-safety --example 08-moved-value-broken --features broken
cargo run -p p2-09-02-select-and-cancellation-safety --example 09-double-borrow-broken --features broken
cargo run -p p2-09-02-select-and-cancellation-safety --example 10-mismatched-types-broken --features broken
```

Then try these:

1. In `04-not-cancellation-safe`, change `attempts` from 5 to 50. What does the final counter become, and why exactly that relationship?
2. In `06-cancellation-token`, shrink the `sleep` before `token.cancel()` from 140 to 10 milliseconds (less than one tick). How many ticks get printed?
3. Run `07-biased-select` ten times in a row (or with a shell loop). How many times did the "unbiased" branch print A vs. B? What about "biased"?

## Errors you will meet

### `E0382` — use of a value that was already moved

```text
error[E0382]: use of moved value: `payload`
  --> phase2-intermediate\09-async-in-practice\02-select-and-cancellation-safety\examples\08-moved-value-broken.rs:18:25
   |
15 |     let payload = String::from("hello");
   |         ------- move occurs because `payload` has type `String`, which does not implement the `Copy` trait
16 |     tokio::select! {
17 |         _ = send(payload) => {}
   |                  ------- value moved here
18 |         _ = log_dropped(payload) => {}
   |                         ^^^^^^^ value used here after move
   |
note: consider changing this parameter type in function `send` to borrow instead if owning the value isn't necessary
  --> phase2-intermediate\09-async-in-practice\02-select-and-cancellation-safety\examples\08-moved-value-broken.rs:5:24
   |
 5 | async fn send(payload: String) {
   |          ----          ^^^^^^ this parameter takes ownership of the value
   |          |
   |          in this function
help: consider cloning the value if the performance cost is acceptable
   |
17 |         _ = send(payload.clone()) => {}
   |                         ++++++++

For more information about this error, try `rustc --explain E0382`.
```

**What the compiler is actually objecting to:** `select!` only runs a branch's body once it wins, but *every* branch's expression - `send(payload)` and `log_dropped(payload)` both - gets built the instant execution reaches `select!`, before any polling happens at all. That means `payload` has to move into both functions at once, and `String` isn't `Copy` - moving it into `send` leaves nothing left to move into `log_dropped`, even though in practice only one of these two futures will actually run.

**The fix:** either have each branch take its own clone:

```rust
tokio::select! {
    _ = send(payload.clone()) => {}
    _ = log_dropped(payload) => {}
}
```

or - if the functions don't genuinely need ownership - change their signatures to take `&str`.

**Why this is the fix:** the compiler has no way to know which branch will win - that's decided only at run time - so it has to accept code that works out correctly no matter *which* one does. Cloning guarantees exactly that: each branch has its own copy, no matter which one actually runs.

### `E0499` — cannot borrow a variable as mutable more than once at the same time

```text
error[E0499]: cannot borrow `total` as mutable more than once at a time
  --> phase2-intermediate\09-async-in-practice\02-select-and-cancellation-safety\examples\09-double-borrow-broken.rs:23:21
   |
21 | /     tokio::select! {
22 | |         _ = add_one(&mut total) => {}
   | |                     ---------- first mutable borrow occurs here
23 | |         _ = add_two(&mut total) => {}
   | |                     ^^^^^^^^^^ second mutable borrow occurs here
24 | |     }
   | |_____- first borrow later used here

For more information about this error, try `rustc --explain E0499`.
```

**What the compiler is actually objecting to:** the same root cause as the previous error, this time for borrows instead of ownership. `select!` has to build both the `add_one(&mut total)` and `add_two(&mut total)` futures up front, together, so it can poll both - so both `&mut total` references have to be alive at the same time, exactly what the aliasing rule ([1.3.1](../../../phase1-fundamentals/03-borrowing-and-references/01-shared-and-mutable-refs/README.md)) forbids.

**The fix:** put `total` behind a `Mutex` (exactly what you saw for `in_flight` in "The concept"), or - if the meaning allows it - give each branch its own counter and add them up after `select!` returns.

**Why this is the fix:** the race itself - two futures alive at once until one wins - is exactly what forces two simultaneous borrows; this is structural, not accidental. A `Mutex` moves the aliasing rule from compile time to run time ([2.8.1](../../08-concurrency/01-threads-mutex-arc/README.md)), precisely where this structure needs it.

### `E0308` — `match` arms have incompatible types

```text
error[E0308]: `match` arms have incompatible types
  --> phase2-intermediate\09-async-in-practice\02-select-and-cancellation-safety\examples\10-mismatched-types-broken.rs:17:49
   |
15 |       let result = tokio::select! {
   |  __________________-
16 | |         data = fetch() => data,
   | |                           ---- this is found to be of type `&str`
17 | |         _ = sleep(Duration::from_millis(50)) => 404,
   | |                                                 ^^^ expected `&str`, found integer
18 | |     };
   | |_____- `match` arms have incompatible types

For more information about this error, try `rustc --explain E0308`.
```

**What the compiler is actually objecting to:** the error message gives it away - `select!` expands into a `match` underneath, and exactly like any other `match` ([1.5.4](../../../phase1-fundamentals/05-your-own-types/04-match-in-depth/README.md)), once you bind its result into a variable, every arm has to return exactly one type. Here one branch produces `&str`, the other an integer - the compiler has no idea what type `result` is supposed to be.

**The fix:** bring both branches to one shared type - both `String`, say, or both wrapped in an `Option`:

```rust
let result = tokio::select! {
    data = fetch() => Some(data.to_string()),
    _ = sleep(Duration::from_millis(50)) => None,
};
```

**Why this is the fix:** exactly the same pattern as `fetch_with_budget` above - wrapping each outcome in an `Option` (or any other shared type) is precisely what lets two branches with different meanings ("found it" / "the deadline won") share one common type.

## Exercises

### Warm up

<details>
<summary>What does this print?</summary>

```rust
#[tokio::main]
async fn main() {
    tokio::select! {
        _ = tokio::time::sleep(Duration::from_millis(10)) => {
            println!("A");
        }
        _ = tokio::time::sleep(Duration::from_millis(300)) => {
            println!("B");
        }
    }
    println!("done");
}
```

</details>

<details>
<summary>Answer</summary>

```text
A
done
```

The 10ms branch becomes `Ready` first, its body prints, `select!` returns right then; the 300ms branch never runs its `println!("B")` because its future is dropped before it ever reaches that value.

</details>

<details>
<summary>Does this compile?</summary>

```rust
async fn a(s: String) {}
async fn b(s: String) {}

async fn run() {
    let s = String::from("x");
    tokio::select! {
        _ = a(s) => {}
        _ = b(s) => {}
    }
}
```

</details>

<details>
<summary>Answer</summary>

No — `E0382`. Both branches' expressions, `a(s)` and `b(s)`, get built before any polling happens, so `s` has to move into both at once; `String` isn't `Copy`. Exactly what you saw in "Errors you will meet".

</details>

<details>
<summary>True or false: <code>select!</code>, like <code>join!</code>, waits for every one of its branches to finish.</summary>

</details>

<details>
<summary>Answer</summary>

False. `join!` waits for everything; `select!` is the exact opposite - it only ever waits for the *first* one, and drops the rest that same instant. That's exactly the distinction this whole lesson is built around.

</details>

### Repair

Fix all three broken examples:

1. Fix `examples/08-moved-value-broken.rs` by cloning the value for one of the two branches.
2. Fix `examples/09-double-borrow-broken.rs` by putting `total` behind an `Arc<Mutex<u32>>` - each branch takes its own `.clone()` of the `Arc`.
3. Fix `examples/10-mismatched-types-broken.rs` by wrapping both branches in one shared `Option<String>`.

### Implement

Two functions in `src/lib.rs`:

- `fetch_with_deadline(delay_ms: u64, budget_ms: u64) -> Option<String>` — a simulated fetch that waits `delay_ms` milliseconds and then gives back exactly the string `format!("data after {delay_ms}ms")`, raced against a deadline of `budget_ms` milliseconds. If the fetch finishes first, return `Some` of that string; if the deadline wins first, return `None`.
- `run_until_cancelled(token: CancellationToken, tick_ms: u64) -> u32` — a loop that "ticks" every `tick_ms` milliseconds, until `token` is cancelled. Each iteration races one tick (a `tick_ms` sleep) against `token.cancelled()`. A tick only counts if its sleep finishes before cancellation is observed; a tick still sleeping when the token is cancelled does not get counted. Once the loop stops, return the total number of completed ticks.

```sh
cargo test -p p2-09-02-select-and-cancellation-safety
```

### Build

Write `pub async fn fetch_with_retries(token: CancellationToken, delay_ms: u64, budget_ms: u64, max_attempts: u32) -> Option<String>`: retries `fetch_with_deadline` with the same `delay_ms`/`budget_ms` up to `max_attempts` times; the moment one attempt succeeds (returns `Some`), return that; if `token` gets cancelled at any point - between attempts or mid-attempt - return `None` immediately; if all `max_attempts` attempts time out, return `None`. Then write yourself a few small tests: one for the first attempt succeeding, one for every attempt losing, one for an already-cancelled token.

### Challenge (optional)

Using [2.9.1](../01-spawn-joinset-structured-concurrency/README.md)'s `JoinSet`, write `pub async fn fetch_first_of(delays: Vec<u64>, budget_ms: u64) -> Option<String>`: spawn each value in `delays` as its own `tokio::spawn` task (one `fetch` per value), then use `select!` to race "the first task to finish" against one shared `sleep(budget_ms)`. If any task finishes before the deadline, return its result; if the deadline beats all of them, return `None`. (Hint: `JoinSet::join_next()` is itself an async method - you can call it directly as a branch expression inside `select!`.)

## Wrapping up

`select!` and [2.9.1](../01-spawn-joinset-structured-concurrency/README.md)'s `JoinSet`/`spawn` are two completely opposite tools, not two versions of one idea: one holds onto everything you hand it until it's all finished; the other, the instant one of them wins, deliberately throws the rest away. That "throwing away" isn't free - its price is exactly what this lesson called cancellation safety: any future that leaves unfinished work outside itself, if it has any chance of losing a race, has to be written so that work never stays half-done. This is one of the sharpest edges in async Rust - not because it's complicated, but because it's silent: no panic fires, no error surfaces, just a wrong number you find one day, in a log.

| Term | What it means | Where you'll use it |
|---|---|---|
| `select!` | Races several futures on one task; the winner's body runs, the rest are dropped that instant | Racing with a deadline, cancellation, "whichever finishes first" |
| `tokio::time::timeout` | The ready-made shortcut for exactly one shape of `select!`: an operation against a `sleep` | Anywhere the deadline is only time, not some other signal |
| Cancellation safety | A losing future being dropped mid-work never leaves its outside state in a wrong state | Any future that touches an outside counter/buffer/lock before it finishes |
| `CancellationToken` | `tokio-util`'s cloneable handle for cooperative shutdown; `.cancelled()` is itself a future | One part of a program asks a task to stop |
| `biased;` | Polls branches in written order instead of randomly among the ready ones | When one branch (like cancellation) genuinely must always be checked first |

### What you now know

- `select!` races several futures on one task and keeps only the winner - the rest are dropped that same instant; this is the exact opposite of `join!`/`JoinSet`.
- How to race a real operation against a deadline, either by hand with a `sleep` branch or with the `tokio::time::timeout` shortcut.
- What cancellation safety means, and why dropping a losing future can permanently break an outside state - with no panic and no error at all.
- The fix: keep the risky side effect outside the raced branch, so it's never dropped half-done.
- What `CancellationToken` is, which crate it comes from, and how to use `.cancelled()` as a `select!` branch.
- What `biased;` does and when you actually need it.

### What comes back later

- **Async sequences — values that arrive one at a time, over time** — [2.9.3 — Streams](../03-streams/README.md)
- **Async traits, and moving heavy/blocking work off the runtime with `spawn_blocking`** — [2.9.4](../04-async-traits-and-blocking/README.md)

### Can you explain?

- What exactly is the difference between `select!` and `join!`/`JoinSet`? Say it in one sentence.
- Define "cancellation safety" in your own words - without using this lesson's own wording.
- In the buggy example, why exactly does the counter always go up but never come back down?
- Why does the fix actually work? Which line, exactly, never gets dropped anymore?
- How do you compare `CancellationToken.cancelled()` to a `sleep`? Both are futures - where's the difference?

## Going further

- [`tokio::select!` docs](https://docs.rs/tokio/latest/tokio/macro.select.html) — every detail, including `biased;`, `if` preconditions, and the exact behavior of refutable patterns.
- [`tokio::time::timeout` docs](https://docs.rs/tokio/latest/tokio/time/fn.timeout.html) — the exact signature and more examples.
- [`tokio_util::sync::CancellationToken` docs](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html) — including `child_token()` for nested cancellation trees, which this lesson didn't cover.
- [The "Select" chapter of the official tokio tutorial](https://tokio.rs/tokio/tutorial/select) — the same cancellation safety, with more examples.
