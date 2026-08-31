# 2.9.1 — `spawn`, `JoinSet`, structured concurrency

## At a glance

After this lesson you can:

- Explain why `tokio::join!` never creates a separate task, and say exactly how that differs from spawning each future onto its own task ([2.8.6](../../08-concurrency/06-tokio-basics/README.md)).
- Collect a batch of work whose size isn't known until run time with `JoinSet`, and read the results back in the order they actually finish — not the order you spawned them in.
- Say why dropping a `JoinSet` cancels every task still running inside it on its own, and call that behavior, in your own words, "structured concurrency."

**Time:** ~70 minutes · **Prerequisites:**
[2.8.6 — `tokio` basics](../../08-concurrency/06-tokio-basics/README.md)

## Why this matters

[2.8.6](../../08-concurrency/06-tokio-basics/README.md) spawned exactly two tasks — by hand, each into its own variable, then `.await`ed both in turn. That works fine for two jobs. Maybe five. But a real service usually doesn't know ahead of time how many jobs it's about to run concurrently — one request arrives with a list of ten items, the next with three. You can't write a separate variable for each one.

This lesson gives you two new tools, for two different shapes of that same problem. First `tokio::join!`, for when the amount of work is fixed and known ahead of time but you don't want to pay the cost of spawning. Then `JoinSet<T>`, for exactly the case described above — a count that's only known at run time. And along the way you get a named idea: **structured concurrency** — tying a task's lifetime to a scope in your code, instead of turning it loose to live its own life.

## The concept

### `tokio::join!`: several known futures, on the same one task

Look at this:

```rust
async fn fetch(id: u32, delay_ms: u64) -> String {
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
    format!("item-{id} (waited {delay_ms}ms)")
}

let start = Instant::now();
let (a, b) = tokio::join!(fetch(1, 150), fetch(2, 50));
println!("{a}");
println!("{b}");
println!("total: {:?}", start.elapsed());
```

Running the full file (`examples/01-join-two-futures.rs`) gives:

```text
item-1 (waited 150ms)
item-2 (waited 50ms)
total: 162.1631ms
```

(The exact final number will vary a little on your machine — it always stays just above 150ms, never anywhere near 200.) Two sleeps — one 150ms, one 50 — but the whole program took only about 150ms, not 200. So `fetch(1, ...)` and `fetch(2, ...)` genuinely ran concurrently. But there's no `tokio::spawn` anywhere here, and no handle was ever taken either.

That's exactly what `tokio::join!` does: it takes every future written out as its arguments, and alternates between them — driving one until its first point of waiting, then the other, and so on, until all of them are done — and only once *every* one is ready does it hand back their values in a tuple, in the exact order you wrote them. Set it side by side with `tokio::spawn`, from [2.8.6](../../08-concurrency/06-tokio-basics/README.md):

- **There is no separate task.** Both `fetch` calls stay inside the exact same one task that called `join!`. That's why they never run on two separate OS threads — `join!` never takes you outside the calling task, it only alternates between several futures *inside* it.
- **No handle is ever taken.** Since it never created a separate task, there's nothing to return a `JoinHandle` for or cancel either — the result lands directly, right at the `join!` call site.
- **Every future has to be written out by hand, up front.** `fetch(1, 150)` and `fetch(2, 50)` are two separate arguments, in code you wrote yourself. There's no way to hand `join!` a `Vec` of futures built at run time and say "wait on whatever's in here" — that case needs a different tool.

```senpai-visual
{"kind":"concurrency","labels":["poll fetch 1","poll fetch 2","interleave, one task","both ready together"]}
```

### `Vec<JoinHandle<T>>` solves the count, not the order

The "count isn't known ahead of time" limitation is one [2.8.6](../../08-concurrency/06-tokio-basics/README.md) already had — its Implement exercise asked for exactly that: take a `Vec<u32>` of any length, spawn one task per entry. Its answer was: spawn inside a `.map(...).collect()`, and gather the handles into a `Vec<JoinHandle<T>>`:

```rust
let requests = vec![(1, 150), (2, 10), (3, 80)];
let handles: Vec<_> = requests
    .into_iter()
    .map(|(id, delay)| tokio::spawn(fetch(id, delay)))
    .collect();

for handle in handles {
    println!("{}", handle.await.unwrap());
}
```

Running the full file (`examples/02-vec-joinhandle-spawn-order.rs`):

```text
item-1 (waited 150ms)
item-2 (waited 10ms)
item-3 (waited 80ms)
```

Three sleeps with three genuinely different delays — 150, 10, 80ms — but the results printed in exactly the order they were spawned: 1, 2, 3. Task 2 (only 10ms) had finished long before task 1 — but because the loop `.await`s task 1's handle first, it sits there, even though task 2's result was already sitting ready. With a `Vec<JoinHandle<T>>`, that's always what happens: results come back in *spawn* order, never *completion* order. If you want to know which one genuinely finished first — without waiting for its turn in the loop to arrive — this pattern has no way to give you that.

### `JoinSet<T>`: a growable set of tasks, in completion order

`tokio::task::JoinSet<T>` fills exactly that gap. Instead of keeping every handle yourself in a `Vec`, you hand tasks over to a `JoinSet` — and it hands each one back to you the moment it actually finishes:

```rust
let requests = vec![(1, 150), (2, 10), (3, 80)];
let mut set = JoinSet::new();
for (id, delay) in requests {
    set.spawn(fetch(id, delay));
}

println!("{} tasks outstanding", set.len());
while let Some(result) = set.join_next().await {
    println!("finished: {}", result.unwrap());
}
println!("{} tasks outstanding", set.len());
```

Running the full file (`examples/03-joinset-completion-order.rs`):

```text
3 tasks outstanding
finished: item-2 (waited 10ms)
finished: item-3 (waited 80ms)
finished: item-1 (waited 150ms)
0 tasks outstanding
```

The same three requests as before — 150, 10, 80ms — but this time the results arrived in exactly **completion** order: 2 (10ms) first, 3 (80ms) second, 1 (150ms) last. (That ordering is reliable here specifically because the three delays are well apart from each other; nothing in `JoinSet`'s own API promises which of two *equally* delayed tasks comes back first.)

Three methods do the actual work:

- **`.spawn(future)`** — exactly like `tokio::spawn`, creates a genuinely new task (the same `Send + 'static` rule from [2.8.4](../../08-concurrency/04-send-and-sync/README.md) still applies) — except this time the `set` itself both knows about and owns the task. `.spawn()` does return a value — an `AbortHandle` for cancelling that one task individually — but you don't need to hold onto it: `set` already owns the task, and the result comes back later through `.join_next()`, not through a handle you keep.
- **`.join_next().await`** — waits for *whichever* task inside `set` finishes first, and returns `Option<Result<T, JoinError>>`: `Some(result)` for as long as any task remains, `None` once `set` has emptied out. Calling it inside a `while let Some(...) = ...` loop means exactly "keep grabbing whatever finishes, until nothing's left."
- **`.len()` / `.is_empty()`** — how many tasks are still outstanding inside `set`, without waiting on any of them.

```senpai-visual
{"kind":"queue","labels":["spawn 3 tasks","fastest finishes first","join_next returns it","slowest finishes last"]}
```

### How a task fails: `Result<T, JoinError>`

Whatever `.join_next().await` pulls out of that `Option` is itself a `Result<T, JoinError>` — not `T` directly. `Ok(value)` means the task finished normally. `Err(join_error)` means something went wrong — either the task panicked, or (briefly noted here, [2.9.2](../02-select-and-cancellation-safety/README.md) covers it properly) it was cancelled/aborted from outside. `JoinError::is_panic()` tells the two apart.

```rust
let mut set = JoinSet::new();
set.spawn(async { 1 + 1 });
set.spawn(async {
    panic!("simulated failure inside a spawned task");
});
set.spawn(async { 2 + 2 });

while let Some(result) = set.join_next().await {
    match result {
        Ok(value) => println!("finished: ok({value})"),
        Err(err) => println!("finished: panicked = {}", err.is_panic()),
    }
}
```

Running the full file (`examples/04-joinset-task-panics.rs`) — one run:

```text
finished: ok(2)

thread 'tokio-rt-worker' (20840) panicked at phase2-intermediate\09-async-in-practice\01-spawn-joinset-structured-concurrency\examples\04-joinset-task-panics.rs:19:9:
simulated failure inside a spawned task
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
finished: ok(4)
finished: panicked = true
```

The panic message from inside the task, on stderr, lands right in the middle of the three `finished: ...` lines — the panicking task never actually goes away without that message printing first; `tokio` just catches the panic and turns it into that `Err(JoinError)` you see, instead of taking down the whole program. (The thread id — `(20840)` here — changes every run, same as the similar numbers in [2.8.6](../../08-concurrency/06-tokio-basics/README.md).) I'm also not promising the exact order of those three `finished: ...` lines — none of these three tasks ever `.await`s anything, so all three finish on their very first poll, and which one reaches `join_next` first isn't something this API decides for you; the only thing you can count on is that all three eventually show up — one of them `panicked = true`.

### Structured concurrency: a task's lifetime, tied to its scope

Now the real question: when `set` itself goes away, what happens to the tasks still inside it?

```rust
let ran = Arc::new(AtomicBool::new(false));
{
    let flag = Arc::clone(&ran);
    let mut set = JoinSet::new();
    set.spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        flag.store(true, Ordering::SeqCst);
        println!("structured task: finished sleeping"); // never prints
    });
    // `set` drops right here — well before the 100ms sleep is over.
}
tokio::time::sleep(Duration::from_millis(200)).await;
println!("structured task ran: {}", ran.load(Ordering::SeqCst));
```

Running the full file (`examples/05-structured-drop-aborts.rs`):

```text
structured task ran: false
```

The task sleeps 100ms; the program then waits 200ms — more than enough time for the task to have finished and printed its line, if it really had. But it never printed, and `flag` never became `true`. **Dropping `set`, at that exact moment, aborted every task still alive inside it** — right where that `{ ... }` block closed, not a moment later.

Compare that with a plain `tokio::spawn` — no `JoinSet` involved at all:

```rust
let ran = Arc::new(AtomicBool::new(false));
{
    let flag = Arc::clone(&ran);
    let handle = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        flag.store(true, Ordering::SeqCst);
        println!("unstructured task: finished sleeping");
    });
    drop(handle); // dropping the HANDLE does not cancel the TASK
}
tokio::time::sleep(Duration::from_millis(200)).await;
println!("unstructured task ran: {}", ran.load(Ordering::SeqCst));
```

Running the full file (`examples/06-unstructured-spawn-detaches.rs`):

```text
unstructured task: finished sleeping
unstructured task ran: true
```

This time the task genuinely finished — even though its handle was dropped in exactly the same spot as before. That's the whole difference: dropping a `JoinHandle` only ever means "I'm no longer waiting on this" — it never touches the task itself, which stays fully independent and **detached** from whoever spawned it, carrying on with its work for as long as it takes, or until the runtime itself shuts down.

That's exactly the difference called **structured concurrency**: when a `JoinSet` (or a group of handles you held onto yourself and `.await`ed every one of) goes out of its scope, every task still owned by it has a lifetime tied to that exact scope — never longer. A lone, handle-dropped `tokio::spawn` is **unstructured**: no scope owns it, nobody is responsible for shutting it down, and it keeps running for as long as it takes or until the runtime itself dies — answering to no one.

```senpai-visual
{"kind":"ownership","labels":["JoinSet spawns task","scope ends","JoinSet drops","task aborted"]}
```

### Which one do you reach for?

Three tools, three shapes of the same problem:

- **`tokio::join!`** — a small, fixed set of futures, known right at compile time, that doesn't need to spread across several threads. No separate task, no handle.
- **`tokio::spawn` + a hand-held `JoinHandle` ([2.8.6](../../08-concurrency/06-tokio-basics/README.md))** — when you know exactly how many jobs you have up front, and want each on its own task, likely its own thread.
- **`JoinSet<T>`** — when the count is only known at run time, or when it matters to find out as soon as possible which one finished first — instead of waiting for its turn in spawn order.

## Hands on

```sh
cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 01-join-two-futures
cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 02-vec-joinhandle-spawn-order
cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 03-joinset-completion-order
cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 04-joinset-task-panics
cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 05-structured-drop-aborts
cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 06-unstructured-spawn-detaches
```

Then the two broken ones:

```sh
cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 07-forgot-await-on-join-next --features broken
cargo run -p p2-09-01-spawn-joinset-structured-concurrency --example 08-unwrap-panicked-join --features broken
```

Then try these:

1. In `03-joinset-completion-order`, change id 2's delay from 10 to 200ms (larger than the rest). How does the output order change?
2. In `05-structured-drop-aborts`, remove the final `sleep(200)` in `main`. Does the program now finish almost immediately? Does that change whether the task inside the `JoinSet` was aborted? (Hint: look at that `println!` that never prints — not at how long `main` itself stays alive.)
3. In `04-joinset-task-panics`, repeat `set.spawn(async { 1 + 1 });` a few more times. Run it several times and watch the order of the output — is it always the same?

## Errors you will meet

### `E0308` — a forgotten `.await` on `join_next`

```text
error[E0308]: mismatched types
  --> phase2-intermediate\09-async-in-practice\01-spawn-joinset-structured-concurrency\examples\07-forgot-await-on-join-next.rs:16:15
   |
16 |     while let Some(result) = set.join_next() {
   |               ^^^^^^^^^^^^   --------------- this expression has type `impl Future<Output = Option<Result<u32, JoinError>>>`
   |               |
   |               expected future, found `Option<_>`
   |
   = note: expected opaque type `impl Future<Output = Option<Result<u32, JoinError>>>`
                     found enum `Option<_>`
help: consider `await`ing on the `Future`
   |
16 |     while let Some(result) = set.join_next().await {
   |                                             ++++++

For more information about this error, try `rustc --explain E0308`.
```

**What the compiler is actually objecting to:** `.join_next()` isn't an ordinary method that hands back an `Option` directly — it's an `async fn`, so what it returns is a `Future`. The `Some(result)` pattern expects to match against an `Option`; against a `Future` that hasn't been `.await`ed yet, that has no meaning at all.

**The fix:** exactly what the compiler itself suggested — add a `.await`:

```rust
while let Some(result) = set.join_next().await {
    // ...
}
```

**Why this is the fix:** `.await` is precisely the point where this `Future` actually waits for a task to finish, and only then pulls out the real `Option<Result<T, JoinError>>` — exactly what the `Some(result)` pattern was expecting from the start.

### Panic — `unwrap()` on a `JoinError`

```text
thread 'tokio-rt-worker' (10992) panicked at phase2-intermediate\09-async-in-practice\01-spawn-joinset-structured-concurrency\examples\08-unwrap-panicked-join.rs:12:9:
simulated failure inside a spawned task
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'main' (17536) panicked at phase2-intermediate\09-async-in-practice\01-spawn-joinset-structured-concurrency\examples\08-unwrap-panicked-join.rs:16:27:
called `Result::unwrap()` on an `Err` value: JoinError::Panic(Id(17), "simulated failure inside a spawned task", ...)
```

**What the compiler is actually objecting to:** this isn't a compiler error either — two genuine run-time panics. The first is from inside the task itself (our own message). The second is here: `set.join_next().await.unwrap()` `unwrap()`s a `Result<T, JoinError>` without ever looking at it; because that `Result` was an `Err` (the task had panicked), that same `unwrap()` panics too — this time in the `main` thread. (The thread ids — `(10992)` and `(17536)` — will be different numbers on your own run.)

**The fix:** `match` on the `Result`, instead of `unwrap()`:

```rust
match set.join_next().await.unwrap() {
    Ok(value) => println!("{value}"),
    Err(err) => println!("task failed: panicked = {}", err.is_panic()),
}
```

**Why this is the fix:** a spawned task can fail — exactly what "The concept" showed you. `unwrap()`ing it assumes that never happens; `match`ing it means genuinely handling both cases.

## Exercises

### Warm up

<details>
<summary>What does this print, and roughly how long does the whole program take?</summary>

```rust
let (a, b) = tokio::join!(
    async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        "a"
    },
    async {
        tokio::time::sleep(Duration::from_millis(30)).await;
        "b"
    }
);
println!("{a} {b}");
```

</details>

<details>
<summary>Answer</summary>

```text
a b
```

And the whole program takes something close to 100ms (not 130) — both futures make progress concurrently, on the same one task; `join!` only returns once *both* are ready, and always in the exact order you wrote them — `a` before `b` — even though `b` finished first.

</details>

<details>
<summary>Does this compile?</summary>

```rust
let mut set: JoinSet<u32> = JoinSet::new();
set.spawn(async { 42 });
let first: u32 = set.join_next().await.unwrap().unwrap();
println!("{first}");
```

</details>

<details>
<summary>Answer</summary>

Yes — it compiles, and runs without panicking too, because that one task genuinely succeeds (`Ok(42)`), so both `unwrap()`s ("pull it out of the `Option`", "pull it out of the `Result`") land on a value that's actually there.

</details>

<details>
<summary>True or false: <code>JoinSet::spawn</code> returns a handle you have to hold onto yourself.</summary>

</details>

<details>
<summary>Answer</summary>

False — you don't have to hold onto anything: `set` itself becomes the task's owner, and you get the result later through `.join_next().await`. `.spawn()` does actually return a value, an `AbortHandle` you could use to cancel that one task individually, but nothing forces you to keep it, and this lesson's examples never do.

</details>

### Repair

Fix both broken examples:

1. Fix `examples/07-forgot-await-on-join-next.rs` so it compiles, without changing what it does.
2. Fix `examples/08-unwrap-panicked-join.rs` so it no longer panics — instead of `unwrap()`ing the result, print `"task failed"` when the task panicked, and print the value when it succeeded.

### Implement

Two functions in `src/lib.rs`:

- `fetch_simulated(id: u32, delay_ms: u64) -> String` — genuinely `.await`s for `delay_ms` milliseconds, then returns exactly the string `"item-{id}"` — e.g. for `id = 7`, the string `"item-7"`.
- `fetch_all_via_joinset(requests: Vec<(u32, u64)>) -> Vec<String>` — for every `(id, delay_ms)` pair in `requests`, spawns one task onto a `JoinSet` (each calling `fetch_simulated`), returning every result in the order the tasks actually **finish** — not the order `requests` lists them in.

```sh
cargo test -p p2-09-01-spawn-joinset-structured-concurrency
```

### Build

Write `pub async fn count_task_outcomes(ids: Vec<u32>, panics_at: Vec<u32>) -> (usize, usize)`: for every id in `ids`, spawn one task onto a `JoinSet`; a task whose id is in `panics_at` should panic, every other one should finish without incident. Using a `join_next` loop, count how many came back `Ok` and how many `Err`, and return `(successes, panics)`.

### Challenge (optional)

Write `pub async fn first_ok_of(ids: Vec<u32>, delay_ms: u64, fail_ids: Vec<u32>) -> Option<String>`: spawn one task per id — ids in `fail_ids` panic immediately, every other one calls `fetch_simulated(id, delay_ms)`. Return the first successful result to arrive (`Some`), and the moment it does, cancel every other task still left inside the `JoinSet` with `.abort_all()` — you no longer need to wait for them. If every task panicked, return `None`.

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `tokio::join!` | Waits on several fixed futures at once, on the same one task, returning them in the exact order written | A small, known-up-front batch of concurrent work |
| `JoinSet<T>` | A growable collection of spawned tasks; hands results back in completion order | A count only known at run time |
| `.join_next().await` | Waits for *whichever* task inside a `JoinSet` finishes first; `None` once it's empty | The main loop for collecting results |
| `Result<T, JoinError>` | A task's outcome — `Err` from either a panic or a cancellation | Telling failure apart from success |
| Structured concurrency | A task's lifetime tied to the scope that spawned it; dropping that scope cancels the task too | `JoinSet` vs. a lone, handle-dropped `tokio::spawn` |

### What you now know

- Why `tokio::join!` never creates a separate task, and how that differs from `tokio::spawn`.
- Why a `Vec<JoinHandle<T>>` hands results back in spawn order, not completion order — and how `JoinSet` fixes that.
- `.spawn()`, `.join_next().await`, `.len()`, `.is_empty()` on `JoinSet<T>`.
- Exactly how a task shows up as `Err(JoinError)` from `.join_next()`, and what `is_panic()` tells apart.
- Why dropping a `JoinSet` aborts every task still inside it, and why a lone `tokio::spawn` doesn't — and why we call the first "structured" and the second "unstructured."

### What comes back later

- **Deliberately, safely cancelling a task, with `select!`** — [2.9.2 — `select!` and cancellation safety](../02-select-and-cancellation-safety/README.md). This lesson only mentioned that a `JoinError` can come from a cancellation too, not only a panic; that lesson teaches you how to actually trigger one on purpose.

### Can you explain?

- Why does `tokio::join!` never take you onto more than one OS thread, even under the `multi_thread` runtime?
- Why does a `Vec<JoinHandle<T>>` always hand results back in spawn order, never completion order?
- What exactly does `.join_next().await` return, and why does it have two layers (`Option` and `Result`)?
- If you drop a `JoinSet` while three tasks are still running inside it, what happens to all three?
- Why do we call a lone `tokio::spawn` "unstructured"?

## Going further

- [`JoinSet` docs](https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html) — the full signature of `.spawn()`, `.join_next()`, `.abort_all()`, and a few more methods this lesson didn't get to.
- [`tokio::join!` docs](https://docs.rs/tokio/latest/tokio/macro.join.html) — the same macro [2.8.6](../../08-concurrency/06-tokio-basics/README.md) previewed in its "Challenge."
- [`JoinError` docs](https://docs.rs/tokio/latest/tokio/task/struct.JoinError.html) — `is_panic()`, `is_cancelled()`, and `into_panic()` for recovering the original panic message.
- [The "Spawning" chapter of the official tokio tutorial](https://tokio.rs/tokio/tutorial/spawning) — the same task/thread distinction [2.8.6](../../08-concurrency/06-tokio-basics/README.md) leaned on, with more examples.
