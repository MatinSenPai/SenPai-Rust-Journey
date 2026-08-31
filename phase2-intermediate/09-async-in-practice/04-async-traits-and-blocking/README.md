# 2.9.4 — Async traits and `spawn_blocking`

## At a glance

After this lesson you can:

- Explain why you can write `async fn` directly inside a trait, no macro required, and call it through a generic bound.
- Read the "not dyn compatible" error an async trait method produces, fix it with `#[async_trait]`, and say exactly what that fix costs you.
- Move a genuinely CPU-heavy computation or a blocking call off the runtime with `spawn_blocking` — and say why you reach for it only for that, never for a fast synchronous function.

**Time:** ~70 minutes · **Prerequisites:**
[2.9.3 — Streams](../03-streams/README.md),
[2.3.7 — Static versus dynamic dispatch, and object safety](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md)

---

## Why this matters

The three lessons before this one, in this module, all circled the same axis: shaping work that is already async. [2.9.1](../01-spawn-joinset-structured-concurrency/README.md) showed you how to spawn several tasks at once and wait on them. [2.9.2](../02-select-and-cancellation-safety/README.md) showed you how to race several `Future`s and cancel one in time. [2.9.3](../03-streams/README.md) showed you how to process a sequence of values that arrive one at a time, over time. This lesson, the module's last, goes somewhere else entirely: not the shape of the work, but two places where async collides with the rest of Rust's reality and shows its sharp edge.

The first edge is contracts. A real backend service — exactly Phase 3, where this module lands — is full of traits like "anything that can look up a user by id" or "anything that can deliver a message somewhere." Now that those traits' methods are naturally `async` (the most natural shape there is, since every one of them is talking to the network or a database), one simple question sits right in front of you: can you keep several different implementations of that trait, behind one single type, inside a `Vec` — the same `dyn Trait` [2.3.7](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md) gave you? For a bare `async fn`, the answer is no. The first half of this lesson shows you that no with real code, and gives you the fix.

The second edge is physical reality. Not everything is async. Hashing a password, compressing an image, calling into a C library that has never heard of `tokio` — all of it either genuinely occupies the CPU or is genuinely blocking, and none of it revolves around `.await`. The second half of this lesson shows you exactly what happens to every other task on the same worker thread when that kind of work lands directly inside an `async fn` — and how `spawn_blocking` does the same work without that damage.

---

## The concept

### Writing `async fn` directly inside a trait

For a few years now (Rust 1.75 onward — this course's toolchain is well past that) writing a trait with async methods no longer needs any macro at all. Build a small trait exactly like any other, just with an `async fn` inside it:

```rust
trait Fetcher {
    async fn fetch(&self) -> String;
}

struct Server;

impl Fetcher for Server {
    async fn fetch(&self) -> String {
        "data from Server".to_string()
    }
}

async fn print_fetch<F: Fetcher>(f: &F) {
    println!("{}", f.fetch().await);
}
```

That's it. No macro, nothing extra — exactly the same `impl Trait for Type` [2.3.1](../../03-traits-and-generics/01-defining-and-implementing-traits/README.md) gave you, just with an `async` method this time. `print_fetch` is an ordinary generic bounded function too: static dispatch, one compiled copy per concrete type that actually implements `Fetcher` — the exact mechanism [2.3.7](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md) introduced as static dispatch. Call it:

```rust
#[tokio::main]
async fn main() {
    print_fetch(&Server).await;
}
```

```text
data from Server
```

### Why `Box<dyn Fetcher>` doesn't compile: object safety, this time for async methods

[2.3.7](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md) showed you two common ways to break object safety — a method returning `Self` by value, and a method with its own generic parameter — and said plainly that those are "the most common real reasons, not the complete list," and promised it would come back here to show you a third wall, the one that `async fn` inside a trait wrestled with for years. Go back to `Fetcher` above and do exactly what you did with `Spinoff` and `Rated` — build a `dyn Fetcher`:

```rust
#[tokio::main]
async fn main() {
    let f: Box<dyn Fetcher> = Box::new(Server);
    println!("{}", f.fetch().await);
}
```

It doesn't compile — the exact error is in "Errors you will meet," the same `E0038` you already know from [2.3.7](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md), just this time saying "because method `fetch` is `async`." The reason traces back to the same vtable: behind `dyn Fetcher`, the real type is erased — the compiler only knows "this implements `Fetcher`," not how much room it needs. An `async fn`, when called, returns its own unnameable type — a state machine the compiler builds specifically for that one `async fn`, sized however its body needs, and no two `async fn`s share that type. A vtable cannot reserve a slot for a size it doesn't know — the exact same problem `Self`-by-value had, from a different angle.

Before Rust 1.75, this wall was even wider: the `async fn` syntax inside a trait did not exist at all. Writing *any* trait with an async method — whether it was ever going to be `dyn` or not — required a macro. That same macro, `#[async_trait]`, still exists; its job is just narrower now, limited to exactly this one problem.

```senpai-visual
{"kind":"async","labels":["async fn in a trait: static dispatch, no allocation","dyn Trait needs a vtable with a fixed slot","async fn has no fixed size for that slot","async_trait: rewrites it into a boxed future","now a trait object works, one heap allocation per call"]}
```

### `#[async_trait]`: how it makes a trait dyn-compatible, and what it costs

The fix is the same trait, with one extra line — on the trait definition itself, **and** on every `impl` of it:

```rust
use async_trait::async_trait;

#[async_trait]
trait Fetcher {
    async fn fetch(&self) -> String;
}

struct Server;

#[async_trait]
impl Fetcher for Server {
    async fn fetch(&self) -> String {
        "data from Server".to_string()
    }
}
```

```rust
#[tokio::main]
async fn main() {
    let f: Box<dyn Fetcher> = Box::new(Server);
    println!("{}", f.fetch().await);
}
```

```text
data from Server
```

This time it compiles. Under the hood, `#[async_trait]` effectively rewrites `fetch`'s signature into something like `fn fetch<'a>(&'a self) -> Pin<Box<dyn Future<Output = String> + Send + 'a>>` — the same signature behind the `E0195` error you'll meet below, which is exactly what happens when this rewrite lands on the trait but not on the `impl`. An ordinary function returning a boxed future always has the same size — one pointer — which is exactly what a vtable can reserve a slot for. The problem is solved, but not for free: every call to `fetch` now performs a real `Box::pin` — a heap allocation — that the raw version from the previous section never paid.

### Decision rule: native or `#[async_trait]`

One line: **reach for plain `async fn` inside the trait by default; reach for `#[async_trait]` only when you genuinely need `dyn Trait` — meaning `Box<dyn Trait>`, or anywhere several different implementations must sit behind one single type.** If you're always working with one fixed concrete type (or a generic bound, like `print_fetch` above), the native version is cheaper — no extra allocation involved. If you need a `Vec<Box<dyn Repository>>`, or a function that wants "anything with this trait" without making the whole function generic, then the cost of one heap allocation per call is a genuinely small price for something the native version cannot give you at all.

### Why heavy or blocking work stalls the runtime

[2.8.5](../../08-concurrency/05-futures-and-runtimes/README.md) called this "cooperative scheduling": a task only hands control back to the runtime exactly at the points it explicitly marks — `.await` points — never at some moment the OS decides on its own. That means the runtime can only advance another task on the same worker thread when the current task actually reaches an `.await` and suspends there. A genuinely CPU-heavy computation — or a blocking call into a synchronous library — is neither: no `.await` lives inside it, so it never hands control back, not until it finishes on its own.

Watch this — `slow_sum` is real, synchronous work with no async anywhere in its body, and `ticker` is supposed to print a tick every 50 milliseconds:

```rust
fn slow_sum(n: u64) -> u64 {
    let mut total: u64 = 0;
    for i in 0..n {
        total = total.wrapping_add(i);
    }
    total
}

async fn ticker() {
    for tick in 1..=3 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        println!("tick {tick}");
    }
}
```

With a `current_thread` runtime (a single worker thread — the flavor [2.8.6](../../08-concurrency/06-tokio-basics/README.md) introduced), spawn `ticker`, then call `slow_sum` **directly**, with no `spawn_blocking` at all:

```rust
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let start = Instant::now();
    let ticks = tokio::spawn(ticker());

    println!("heavy result: {}", slow_sum(20_000_000));

    ticks.await.unwrap();
    println!("elapsed: {:?}", start.elapsed());
}
```

```text
heavy result: 199999990000000
tick 1
tick 2
tick 3
elapsed: 304.935ms
```

`ticker` was spawned before `slow_sum` even ran — it was supposed to be ticking every 50 milliseconds from the very start. But not a single "tick" prints before "heavy result" — and that isn't luck, it's a guarantee of the cooperative scheduling model itself: with one worker thread, absolutely nothing else gets a chance to run — not one step — until `slow_sum` returns. The exact `elapsed` number will differ on your machine, but this ordering — zero ticks before "heavy result" — is always the same.

### `spawn_blocking`: moving the work to a separate thread pool

`tokio::task::spawn_blocking` solves exactly this: it takes a plain closure (not `async`) and hands it to tokio's own, separate blocking thread pool — entirely apart from the async worker threads — returning a `JoinHandle`, the exact same type [2.9.1](../01-spawn-joinset-structured-concurrency/README.md) handed you from `tokio::spawn`, `.await`ed the same way to get the result back. Same `slow_sum`, same `ticker`, one change in `main`:

```rust
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let start = Instant::now();
    let ticks = tokio::spawn(ticker());

    let heavy = tokio::task::spawn_blocking(|| slow_sum(20_000_000))
        .await
        .unwrap();
    println!("heavy result: {heavy}");

    ticks.await.unwrap();
    println!("elapsed: {:?}", start.elapsed());
}
```

```text
tick 1
heavy result: 199999990000000
tick 2
tick 3
elapsed: 169.5612ms
```

This time "tick 1" printed before "heavy result" — the single worker thread was free to advance `ticker` while `slow_sum` ran on the separate pool. **Exactly how many ticks land before "heavy result" varies run to run** — this is real wall-clock timing between two separate threads, not a guarantee; running this same example several times in a row, I saw one tick land before it once, and two ticks land before it another time. The one thing that is guaranteed: **at least one tick** always lands before "heavy result" — something that never happens in the version above, without `spawn_blocking`. The `elapsed` number also dropped, on this machine, on this run, from around 304ms to around 170ms: the heavy work and the ticking now overlap instead of queuing one after the other.

```senpai-visual
{"kind":"concurrency","labels":["heavy work directly inside an async fn","the worker thread is stuck until it returns","every other task gets zero chance to run","spawn_blocking: the same work on a separate thread pool","the worker thread stays free, other tasks keep going"]}
```

### Decision rule: when `spawn_blocking`, when not

One line: **reach for `spawn_blocking` for genuinely CPU-heavy work or an unavoidable blocking call — never for a synchronous function just because it happens to be fast.** `spawn_blocking` itself isn't free: it schedules new work on a different thread, and for something that takes microseconds — parsing a short string, summing a handful of numbers — that overhead outweighs the work itself. The practical line: if you're calling into a synchronous library with no `.await`-able version, or your computation genuinely takes a few milliseconds or more, reach for `spawn_blocking`; if the work is fast, call it right there, directly — wrapping it in `.await` buys you nothing but an extra, useless layer.

---

## Hands on

(Every run also prints an `unused variable: n` warning first — from the still-`todo!()` `run_cpu_work_off_the_runtime` in `src/lib.rs`, which you complete yourself later in "Exercises." Ignore it; the output below it is what matters.)

```sh
cargo run -p p2-09-04-async-traits-and-blocking --example 01-native-async-trait
cargo run -p p2-09-04-async-traits-and-blocking --example 02-async-trait-macro
cargo run -p p2-09-04-async-traits-and-blocking --example 03-blocking-call-stalls-the-runtime
cargo run -p p2-09-04-async-traits-and-blocking --example 04-spawn-blocking-fixes-it
```

Then the three broken ones:

```sh
cargo run -p p2-09-04-async-traits-and-blocking --example 05-dyn-native-async-trait-broken --features broken
cargo run -p p2-09-04-async-traits-and-blocking --example 06-forgot-async-trait-on-impl-broken --features broken
cargo run -p p2-09-04-async-traits-and-blocking --example 07-spawn-blocking-forgot-move-broken --features broken
```

Then try these:

1. In `03-blocking-call-stalls-the-runtime`, multiply `20_000_000` by ten. How does the "elapsed" number change? Does the ordering (every "tick" after "heavy result") change too?
2. In `04-spawn-blocking-fixes-it`, run it several times in a row. Exactly how many "tick"s land before "heavy result" each time? Is it always the same?
3. In `02-async-trait-macro`, remove the `#[async_trait]` line from above `impl Fetcher for Server` (not from above the `trait` itself). What error do you get? (Hint: it's the one right below, in "Errors you will meet.")

---

## Errors you will meet

### `E0038` — an async trait method is not dyn compatible

```text
error[E0038]: the trait `Fetcher` is not dyn compatible
  --> phase2-intermediate\09-async-in-practice\04-async-traits-and-blocking\examples\05-dyn-native-async-trait-broken.rs:23:20
   |
23 |     let f: Box<dyn Fetcher> = Box::new(Server);
   |                    ^^^^^^^ `Fetcher` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>
  --> phase2-intermediate\09-async-in-practice\04-async-traits-and-blocking\examples\05-dyn-native-async-trait-broken.rs:10:14
   |
 9 | trait Fetcher {
   |       ------- this trait is not dyn compatible...
10 |     async fn fetch(&self) -> String;
   |              ^^^^^ ...because method `fetch` is `async`
   = help: consider moving `fetch` to another trait
   = help: only type `Server` implements `Fetcher`; consider using it directly instead.

For more information about this error, try `rustc --explain E0038`.
```

**What the compiler is actually objecting to:** its own message is precise — `fetch` breaks object safety because it is `async`. An `async fn`, behind the scenes, returns its own unnameable type — a state machine sized however its own body needs. A vtable has to know, before any call, exactly how much room every method's return value needs; a type whose size isn't known ahead of time has no place in that table.

**The fix:** put `#[async_trait]` on the trait itself, **and** on every `impl` of it — exactly as you saw in "The concept":

```rust
#[async_trait]
trait Fetcher {
    async fn fetch(&self) -> String;
}
```

**Why this is the fix:** `#[async_trait]` rewrites `fetch`'s signature into something that returns a boxed future — `Pin<Box<dyn Future<Output = String> + Send>>` — and a `Box`'s size is always fixed, no matter what's behind it. Now the vtable can reserve a slot for it, at the cost of one heap allocation per call.

### `E0195` — the `impl`'s signature no longer matches the trait

```text
error[E0195]: lifetime parameters or bounds on method `fetch` do not match the trait declaration
  --> phase2-intermediate\09-async-in-practice\04-async-traits-and-blocking\examples\06-forgot-async-trait-on-impl-broken.rs:20:19
   |
12 | #[async_trait]
   | -------------- this bound might be missing in the impl
13 | trait Fetcher {
14 |     async fn fetch(&self) -> String;
   |              ------------
   |              |     |
   |              |     this bound might be missing in the impl
   |              lifetimes in impl do not match this method in trait
...
20 |     async fn fetch(&self) -> String {
   |                   ^ lifetimes do not match method in trait

For more information about this error, try `rustc --explain E0195`.
```

**What the compiler is actually objecting to:** `#[async_trait]` sits on `trait Fetcher` itself, so `fetch`'s signature inside the trait definition has already been rewritten — with an extra lifetime for the boxed future. `impl Fetcher for Server`, with no macro of its own on it, still writes the original, plain signature — the two signatures are no longer the same one, and Rust only accepts an `impl` that repeats the trait's exact signature.

**The fix:** put that same `#[async_trait]` on the `impl` too:

```rust
#[async_trait]
impl Fetcher for Server {
    async fn fetch(&self) -> String {
        "data from Server".to_string()
    }
}
```

**Why this is the fix:** now both sides — the trait and the implementation — have gone through the same macro and carry the exact same rewritten signature. `#[async_trait]` is an all-or-nothing decision: it sits on the trait and every one of its `impl`s, or on none of them at all.

### `E0373` — a `spawn_blocking` closure without `move`

```text
error[E0373]: closure may outlive the current function, but it borrows `n`, which is owned by the current function
  --> phase2-intermediate\09-async-in-practice\04-async-traits-and-blocking\examples\07-spawn-blocking-forgot-move-broken.rs:21:46
   |
21 |     let handle = tokio::task::spawn_blocking(|| slow_sum(n));
   |                                              ^^          - `n` is borrowed here
   |                                              |
   |                                              may outlive borrowed value `n`
   |
note: function requires argument type to outlive `'static`
  --> phase2-intermediate\09-async-in-practice\04-async-traits-and-blocking\examples\07-spawn-blocking-forgot-move-broken.rs:21:18
   |
21 |     let handle = tokio::task::spawn_blocking(|| slow_sum(n));
   |                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
help: to force the closure to take ownership of `n` (and any other referenced variables), use the `move` keyword
   |
21 |     let handle = tokio::task::spawn_blocking(move || slow_sum(n));
   |                                              ++++

For more information about this error, try `rustc --explain E0373`.
```

**What the compiler is actually objecting to:** `spawn_blocking`'s closure runs on an entirely different thread, possibly long after the calling function — `main`, right here — has already returned. That's why `spawn_blocking` requires its closure to be `'static`: it must own everything it needs, not merely borrow it. The closure `|| slow_sum(n)` only borrows `n`, and `n` belongs to `main` — a promise the compiler cannot keep.

**The fix:** put `move` in front of the closure:

```rust
let handle = tokio::task::spawn_blocking(move || slow_sum(n));
```

**Why this is the fix:** `n` is a `u64` — `Copy` — so `move` costs nothing, it just hands the closure ownership of a small copy. The closure no longer depends on anything outside itself; it can now run whenever, wherever, on the blocking thread pool, with no dependency on how long `n` lives inside `main`.

---

## Exercises

### Warm up

<details>
<summary>What does this print?</summary>

```rust
trait Namer {
    async fn name(&self) -> String;
}

struct Cat;

impl Namer for Cat {
    async fn name(&self) -> String {
        "Neko".to_string()
    }
}

async fn print_name<N: Namer>(n: &N) {
    println!("{}", n.name().await);
}
```

Assume this is called inside `main`: `print_name(&Cat).await;`

</details>

<details>
<summary>Answer</summary>

```text
Neko
```

Nothing surprising — an `async fn` inside a trait, called through a generic bound, behaves exactly like any other trait method.

</details>

<details>
<summary>Does this compile?</summary>

```rust
trait Namer {
    async fn name(&self) -> String;
}

struct Cat;

impl Namer for Cat {
    async fn name(&self) -> String {
        "Neko".to_string()
    }
}

fn boxed() -> Box<dyn Namer> {
    Box::new(Cat)
}
```

</details>

<details>
<summary>Answer</summary>

No — `E0038`, "the trait `Namer` is not dyn compatible," for exactly the same reason `Fetcher` failed above: `name` is an `async fn`, and an `async fn` has no fixed slot in a vtable. The fix is the same too — `#[async_trait]` on the trait and on the `impl`.

</details>

<details>
<summary>Does this compile?</summary>

```rust
use async_trait::async_trait;

#[async_trait]
trait Namer {
    async fn name(&self) -> String;
}

struct Cat;

impl Namer for Cat {
    async fn name(&self) -> String {
        "Neko".to_string()
    }
}
```

</details>

<details>
<summary>Answer</summary>

No — `E0195`. `#[async_trait]` sits on `trait Namer` itself and rewrites `name`'s signature, but `impl Namer for Cat`, with no macro on it, still has the original, plain signature. `#[async_trait]` is all-or-nothing — either on the trait and every one of its `impl`s, or on none of them.

</details>

<details>
<summary>True or false: with a <code>current_thread</code> runtime, if you've already spawned another task and then directly (no <code>spawn_blocking</code>) call a synchronous function that takes several hundred milliseconds, that other task might still take a step right in the middle of it.</summary>

</details>

<details>
<summary>Answer</summary>

False. With a single worker thread, nothing else gets a chance to run at all — not a step, not half a step — until the synchronous function returns. Control is only handed back at an `.await`, and a synchronous function has none.

</details>

### Repair

Fix all three broken examples:

1. Fix `examples/05-dyn-native-async-trait-broken.rs` by adding `#[async_trait]` on `trait Fetcher` **and** on `impl Fetcher for Server`.
2. Fix `examples/06-forgot-async-trait-on-impl-broken.rs` by adding that same `#[async_trait]` — already on the trait — to the `impl` as well.
3. Fix `examples/07-spawn-blocking-forgot-move-broken.rs` by putting `move` in front of the `spawn_blocking` closure.

### Implement

Four things in `src/lib.rs` — two already complete, two carry a `todo!()`:

- `sum_range(n: u64) -> u64` — already complete. Computes the sum of `0..n` with wrapping addition; the CPU-heavy work the exercise below offloads.
- `greet_with<G: Greeter>(greeter: &G, name: &str) -> String` — already complete. Awaits `greeter.greet(name)` and returns the result — an ordinary generic bounded function, no `dyn` involved.
- **TODO** — `Formal::greet(&self, name: &str) -> String`: returns exactly the string `format!("Good day, {name}.")` — e.g. for `name = "Sara"`, the string `"Good day, Sara."`.
- **TODO** — `run_cpu_work_off_the_runtime(n: u64) -> u64`: must return the result of `sum_range(n)`, **without ever calling `sum_range` directly inside this `async fn`** — use `spawn_blocking` to run it on tokio's blocking thread pool and return its result.

```sh
cargo test -p p2-09-04-async-traits-and-blocking
```

### Build

Add a second `Greeter` shape — `Casual` — whose `greet` returns exactly the string `format!("Hey {name}!")`. Now that you have two different types that are both `Greeter`, and want to keep both behind one single type inside a `Vec`, rebuild `Greeter` itself: put `#[async_trait]` on the trait definition, and on both the `Formal` and `Casual` `impl`s. Then write:

```rust
pub async fn greet_all(greeters: &[Box<dyn Greeter>], name: &str) -> Vec<String>
```

which `.await`s every member of `greeters` in order and returns the results, in that same order, as a `Vec`.

### Challenge (optional)

Write `pub async fn sum_many_off_the_runtime(ns: Vec<u64>) -> Vec<u64>`: for every `n` in `ns`, fire off a separate `spawn_blocking(move || sum_range(n))` — all of them, in a single pass, before any `.await` — then `.await` them one by one and return the results in the same order as `ns`. This is exactly the spawn-everything-first-then-await pattern [2.9.1](../01-spawn-joinset-structured-concurrency/README.md) taught you for `tokio::spawn` — `spawn_blocking` returns the exact same kind of `JoinHandle`, so the same pattern works here without any change.

---

## Wrapping up

The full path through this module, start to finish: [2.9.1](../01-spawn-joinset-structured-concurrency/README.md) showed you how to launch several tasks at once with `spawn` and `JoinSet` and wait on them. [2.9.2](../02-select-and-cancellation-safety/README.md) showed you how to race several `Future`s with `select!` and cancel one in time. [2.9.3](../03-streams/README.md) showed you how to process an async sequence, item by item, with `Stream`. Today, the last lesson, saw two practical edges of async trait methods: object safety — why an `async fn` inside a trait cannot be `dyn`, and how `#[async_trait]` changes that, at the cost of one heap allocation — and the physical reality of work — why heavy or blocking work stalls the runtime, and how `spawn_blocking` does that same work without the stall. With that, module 2.9 — and Phase 2 — has one more piece: [2.10 — The Rust toolbox](../../10-rust-toolbox/README.md), four things that don't belong to any one module on their own — pattern matching in depth, your first macro, cargo features, and `unsafe` written for real.

| Term | What it means | Where you'll use it |
|---|---|---|
| `async fn` in a trait (AFIT) | Writing `async fn` directly inside a trait, no macro | Async traits, by default — until `dyn` is genuinely needed |
| Object safety (this time for async) | The third reason — after `Self`-by-value and a generic method from [2.3.7](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md) — a trait fails to be `dyn` | The `E0038` error on any raw `async fn` behind `dyn` |
| `#[async_trait]` | A macro that rewrites `async fn` into a method returning `Pin<Box<dyn Future<...> + Send>>` | Only when you genuinely need `Box<dyn Trait>` |
| `spawn_blocking` | Runs a synchronous closure on tokio's own, separate blocking thread pool | CPU-heavy work or an unavoidable blocking call |

### What you now know

- Why you can write `async fn` directly inside a trait, no macro required, and call it through a generic bound.
- Why that same raw trait fails with `E0038` behind `Box<dyn Trait>` — and that this is exactly the same object-safety wall from [2.3.7](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md), seen from a new angle.
- How `#[async_trait]` removes that wall — by rewriting the method into a boxed future — and exactly what it costs you (one heap allocation per call) to do it.
- Why a CPU-heavy computation or a blocking call, dropped directly inside an `async fn`, completely starves every other task on the same worker thread.
- How `spawn_blocking` moves that same work onto a separate thread pool, and when you actually reach for it — never for a fast synchronous function.

### What comes back later

- **Async traits and `spawn_blocking`, inside a real backend** — this lesson left neither of them half-finished; from here on they simply stop being "a lesson's exercise" and become ordinary code you write for a real service — [Phase 3 — Backend Foundations](../../../phase3-backend-foundations/README.md).

### Can you explain?

- Why does an `async fn` inside a trait work fine through a generic bound but fail to compile behind `Box<dyn Trait>`?
- What does `#[async_trait]` actually do? Why does it need to sit on both the trait and every `impl`? What do you pay that the native version doesn't?
- Why does a CPU-heavy computation, if called directly inside an `async fn`, starve every other task on the same worker thread? Explain it in terms of cooperative scheduling itself.
- How does `spawn_blocking` fix that? Why should you not reach for it on every synchronous function — even a fast one?

---

## Going further

- [The dyn compatibility reference](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility) — the exact page the compiler itself links to inside the `E0038` error.
- [The `async-trait` crate docs](https://docs.rs/async-trait/latest/async_trait/) — including `#[async_trait(?Send)]`, for when your `Future` shouldn't have to be `Send`.
- [The `spawn_blocking` docs](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html) — the exact signature and the official explanation of the blocking thread pool.
- [The "Spawning" chapter of the official tokio book](https://tokio.rs/tokio/tutorial/spawning) — the same page [2.8.6](../../08-concurrency/06-tokio-basics/README.md) introduced, the task/thread distinction.
