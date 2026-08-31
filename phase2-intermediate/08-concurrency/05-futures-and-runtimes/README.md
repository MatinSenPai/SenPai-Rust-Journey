# 2.8.5 — Futures and runtimes: what `async fn` desugars to

## At a glance

After this lesson you can:

- Explain why calling an `async fn` neither spawns a thread nor runs any of its body — and say exactly what value it hands you instead.
- Write the `Future` trait's `poll` signature from memory, explain what `Poll::Ready` and `Poll::Pending` each mean, and implement `poll` by hand for a small type of your own.
- Say why a bare `Future` sitting in a variable does nothing on its own, and explain — mechanically, not just by naming it — what job a runtime (an executor) actually does.

**Time:** ~80 minutes · **Prerequisites:**
[2.8.1 — Threads, `Mutex`, `Arc`](../01-threads-mutex-arc/README.md),
[2.8.4 — `Send` and `Sync`: what they are and why your type isn't `Send`](../04-send-and-sync/README.md)

---

## Why this matters

From the start of this module to right here, everything you've learned has turned around one fixed axis: the OS thread. [2.8.1](../01-threads-mutex-arc/README.md) gave you `thread::spawn` and `Mutex`/`Arc`. [2.8.2](../02-rwlock-semaphore-oncelock-atomics/README.md) added the finer-grained `RwLock` and atomics. [2.8.3](../03-channels-message-passing/README.md) showed you how to pass messages through a channel instead of a lock. [2.8.4](../04-send-and-sync/README.md) told you how the compiler knows which types are even allowed to move between threads. All four lessons shared one assumption: the work you're running in parallel is **CPU work** — a real computation that genuinely keeps a core busy.

This lesson drops that assumption. Most of what a real backend service — exactly what Phase 3 is heading toward — spends its time doing isn't computing at all; it's **waiting**: for a database to answer, for an HTTP request to a downstream service to come back, for a timer to elapse. A thread sitting there waiting on the network isn't computing anything — but it's still occupying its multi-megabyte stack, and still holding a slot in the OS scheduler. If a service needs to handle ten thousand simultaneous slow connections, ten thousand threads is a bad plan — and by the end of this lesson you'll see, in real numbers, why.

That's exactly the problem `async` was built for: work that spends most of its time simply waiting. This lesson pivots the whole module's axis — from threads to `async` — and it's more mechanical and less intuitive than any other lesson in this module, which is exactly why we go slowly: first `async fn` gets taken apart into exactly what it really is (not a thread, not immediate execution), then you implement the `Future` trait by hand, and only at the very end do we get to why you need a runtime at all. Writing real async programs — with `tokio` — is next lesson's job; today is only about understanding what's underneath it.

---

## The concept

### Threads versus async: two tools for two different problems

[2.8.1](../01-threads-mutex-arc/README.md) gave you `thread::spawn`: a genuine OS thread, scheduled **preemptively** — meaning the operating system can interrupt it mid-instruction, at any moment it sees fit, without asking the code's permission. That's exactly what makes real parallelism possible: two threads can genuinely be running at the same literal instant, on two separate cores. But that power isn't free: every thread reserves its own stack — on the order of megabytes — and every switch between threads is a real, comparatively expensive trip through the kernel's scheduler.

`async` is a different tool for a different problem. An async **task** is managed by **cooperative** scheduling instead: not the operating system, but a userspace library — a **runtime** you choose yourself — decides when each task runs. "Cooperative" means a task only ever gives up control at the exact points it explicitly marks (`.await` points, a few paragraphs down), never interrupted mid-line the way a thread can be. Because nobody has to reserve a whole OS thread's stack for a task, a task's state can be as small as whatever it actually holds onto — often a few dozen or a few hundred bytes, not megabytes. That's why you can realistically have hundreds of thousands of concurrent async tasks running on a machine where even a few thousand OS threads would already be a heavy cost, in both memory and scheduling.

The simple rule: work that genuinely keeps a CPU busy — real computation — belongs to threads ([2.8.1](../01-threads-mutex-arc/README.md)). Work that is mostly waiting — network, database, timers — belongs to async, from this lesson on.

```senpai-visual
{"kind":"concurrency","labels":["thread: OS-scheduled, megabytes of stack","task: runtime-scheduled, bytes of state","a few thousand threads: already costly","hundreds of thousands of tasks: still fine"]}
```

### Calling an `async fn` does nothing — until it's polled

Look at this:

```rust
async fn greet(name: &str) -> String {
    println!("  (inside greet: actually running now)");
    format!("hello, {name}")
}

fn main() {
    println!("calling greet(\"Matin\")...");
    greet("Matin");
    println!("...called it. did \"inside greet\" print above?");
}
```

If you expected the first line printed to be "actually running now," you're not alone — this is exactly where most programmers' intuition trips the first time. Here's what the compiler and the program actually print, together (you'll also see a handful of unrelated `unused` warnings first, from this lesson's still-`todo!()` exercise code lower in `src/lib.rs` — ignore those, the one below is the one that matters):

```text
warning: unused implementer of `Future` that must be used
  --> examples/01-calling-does-nothing.rs:12:5
   |
12 |     greet("Matin");
   |     ^^^^^^^^^^^^^^
   |
   = note: futures do nothing unless you `.await` or poll them
   = note: `#[warn(unused_must_use)]` (part of `#[warn(unused)]`) on by default

calling greet("Matin")...
...called it. did "inside greet" print above?
```

The "inside greet" line never printed. Calling `greet("Matin")` didn't run a single line of its body — it just, immediately, returned a value: something of an unnamed type that implements the `Future` trait. That value sat there, unused, until the end of scope, and then got dropped. The compiler is telling you exactly this itself: the `Future` trait is marked so that dropping one of its values without `.await`ing or polling it is suspicious enough to warn about.

So an `async fn`, when you call it, is precisely two things it is **not**:

- It doesn't spawn a thread — no `thread::spawn` happens anywhere.
- It doesn't run its body immediately either — unlike a plain `fn`, whose call always drives its body right then.

Instead of either of those, it just returns a `Future` value — a motionless description of work that needs to happen, not the work itself, in progress. If you've used Python's `asyncio`, this will feel familiar: calling an `async def` function doesn't run its body either — it hands back an inert coroutine object that does nothing until something `await`s it or schedules it. Here's where the analogy stops: Python's event loop (`asyncio.run`) is, whatever else it is, single-threaded and cooperative — two tasks never actually run at the same literal instant on two cores, GIL or no GIL. A Rust runtime like `tokio` (next lesson) can do both at once: cooperative scheduling of a huge number of tasks, and — if you ask for its multi-threaded flavor — genuinely parallel execution of several tasks across cores at the same instant.

### The `Future` trait itself: `poll`, and the two `Poll` variants

So what exactly is a `Future`? Look at the trait itself — this is genuinely `std::future::Future`, with nothing simplified away:

```rust
pub trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
```

`Output` is an **associated type** — exactly what [2.3.5](../../03-traits-and-generics/05-associated-types/README.md) gave you, playing exactly the role `Item` plays for `Iterator`: the type of value this computation eventually hands back, once it's actually done. The `poll` method is the only way to push this computation one step forward — you call it once, and it gives back exactly one of two things:

- `Poll::Ready(value)` — done; here's the value.
- `Poll::Pending` — not yet; try again later.

For now, just read the `self: Pin<&mut Self>` parameter as "a mutable reference to the same state machine this async computation is built from" — a few sections down we say exactly why `Pin` is needed there, not a plain `&mut Self`. The `cx: &mut Context<'_>` parameter also carries a `Waker` handle along with it; the last section of this lesson finishes what that means.

Let's try this by hand, with no runtime at all — a small type that returns `Pending` the first time and `Ready` the second:

```rust
struct FlipOnce {
    polled_before: bool,
}

impl Future for FlipOnce {
    type Output = &'static str;
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.polled_before {
            Poll::Ready("ready on the second poll")
        } else {
            self.polled_before = true;
            Poll::Pending
        }
    }
}
```

```rust
let mut task = FlipOnce {
    polled_before: false,
};
let waker = Waker::noop();
let mut cx = Context::from_waker(waker);

let first = Pin::new(&mut task).poll(&mut cx);
println!("first poll:  {first:?}");

let second = Pin::new(&mut task).poll(&mut cx);
println!("second poll: {second:?}");
```

```text
first poll:  Pending
second poll: Ready("ready on the second poll")
```

That's it. We called `poll` twice, by hand — got `Pending` once, `Ready` once. There's no magic here: a `Future`, at rock bottom, is just a `poll` method that either has an answer or it doesn't.

```senpai-visual
{"kind":"async","labels":["call the async fn","get back a Future — nothing has run yet","poll it","Pending: not ready yet","poll it again","Ready: the value is here"]}
```

### `.await`: poll it, and if it's `Pending`, give control back instead of blocking

`.await` isn't a method — it's special syntax the compiler only recognizes inside an `async fn` or `async {}` (why only there, a few lines down). Semantically, `something.await` roughly means what this pseudocode shows:

```text
// what `let v = something.await;` roughly means:
loop {
    match something.poll(cx) {
        Poll::Ready(v) => break v,               // carry on from here
        Poll::Pending  => return Poll::Pending,   // suspend right here instead
    }                                              // control goes back to the executor
}
```

If `poll` returns `Ready(v)`, the value of the `.await` expression itself is `v`, and execution just continues normally. But if it returns `Pending`, here's the important part: the entire surrounding `async fn` also suspends itself, right at this exact spot — not by locking the current thread and sitting there waiting, but by returning control to whatever polled it in the first place (which, ultimately, is the runtime). This is exactly the difference the whole lesson has been building toward: a blocking call (like `std::thread::sleep`) locks the thread for the entire wait — nothing else on that thread can make progress. `.await`ing a `Pending` `Future` instead **gives the thread back** to the runtime, so the runtime can advance whatever else is ready to make progress on that same thread, for exactly as long as this task is waiting.

### Why a `Future` must not move once it starts being polled: `Pin`

Now go back to that `self: Pin<&mut Self>` parameter we promised to explain. The compiler turns an `async fn`'s body into a state machine — an unnamed struct holding whatever stayed alive between two `.await` points. If that state machine holds a reference, between two of its own suspension points, into one of its **own** local variables — say, a string slice pointing at a `String` that lives on that same state machine — then that value is **self-referential**. Moving such a value in memory (which Rust is normally free to do whenever it likes) breaks that internal reference: the address it's holding no longer points at the right place. `Pin<&mut Self>` is exactly the compiler's promise that this can't happen: once `poll` has been called on a value a single time, that value is never allowed to move in memory again.

Where does that actually bite? In `FlipOnce` above, `self.polled_before = true` worked with no ceremony at all through that same `Pin<&mut Self>` — because `FlipOnce` holds no internal references, so Rust automatically gives it the `Unpin` trait, and for an `Unpin` type, `Pin<&mut Self>` is, in practice, barely more than an ordinary `&mut Self`. But the state machine the compiler builds for a real `async fn` is always — whether it actually self-references or not — deliberately **not** `Unpin`. If you try to pin such a value with `Pin::new` (which only works for `Unpin` types), the compiler refuses — you'll see that exact error in "Errors you will meet." The common fix, almost always, is simple: put the value on the heap once, with `Box::pin` — and from then on, pinning it is never a problem again, because it never actually moves. (There's also a fix that doesn't allocate — the `std::pin::pin!` macro, for pinning to the current stack frame — but the full mechanics of pin-projection aren't this lesson's scope.)

```senpai-visual
{"kind":"concept","labels":["before the first poll: free to move","the first poll happens","the state machine may now reference itself","pinned: never moves again"]}
```

### Why you need a runtime at all

Back to the second section: a lone `Future`, sitting in a variable that nobody has polled, does nothing. Nothing drives it forward on its own. Rust's standard library deliberately stops right there too: it gives you the `Future` trait, but no code anywhere that actually calls `poll()` for you. That's deliberate — a web server, an embedded device, and a game engine all want genuinely different schedulers, so the language hands you the contract and lets the ecosystem supply the runtimes.

A runtime's (an executor's) job is exactly this: call `poll()`, and if it's `Pending`, arrange to call `poll()` again. The simplest possible shape of this is a `block_on` function — the very thing you implement by hand in "Exercises." To show it working, we need one more hand-rolled `Future` — this time something actually worth `.await`ing rather than just polling by hand: another shape of the same `FlipOnce` idea, this time called `YieldOnce`:

```rust
struct YieldOnce {
    done: bool,
}

impl Future for YieldOnce {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        if self.done {
            Poll::Ready(())
        } else {
            self.done = true;
            Poll::Pending
        }
    }
}
```

And here's `block_on`, complete and working:

```rust
fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = Box::pin(future);
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    let mut attempt = 1;
    loop {
        println!("executor: poll attempt {attempt}");
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(value) => return value,
            Poll::Pending => attempt += 1,
        }
    }
}
```

Now drive both of those with a genuinely real `async fn` that itself `.await`s `YieldOnce`:

```rust
async fn run() -> u32 {
    println!("  run: about to yield once");
    YieldOnce { done: false }.await;
    println!("  run: resumed after yielding");
    42
}

fn main() {
    let total = block_on(run());
    println!("total = {total}");
}
```

```text
executor: poll attempt 1
  run: about to yield once
executor: poll attempt 2
  run: resumed after yielding
total = 42
```

That `Pending` bubbles out of `YieldOnce`, out of `run` too, and all the way up to the `poll` that `block_on` called — exactly what the `.await` section above promised. `block_on` sees it, and polls again. `Waker::noop()` gives a `Waker` that does absolutely nothing when woken — which is why our `block_on` here just spins in a busy loop, polling again and again, without ever actually waiting on anything.

A real executor doesn't have that busy loop — it can't afford to: at the scale of hundreds of thousands of tasks, that would burn an entire CPU for nothing. Instead, the very `Waker` passed into every `poll` finally earns its keep: when a `Future` is genuinely waiting on something — a socket, a timer — it stashes that `Waker` away for itself, and whatever code is actually handling that real wait (down in the I/O layer) calls `wake` on that `Waker` the moment it's ready. That call is exactly the signal the executor was waiting for: "go poll this task again" — instead of blindly guessing, over and over, what might be ready.

```senpai-visual
{"kind":"async","labels":["Future returns: Pending","the Future stores the Waker","somewhere else, it waits: I/O or a timer","that code wakes the Waker","the executor polls again"]}
```

And here's the number the "Why this matters" section promised — just how small an async task's state really is in practice:

```rust
async fn one_step(x: u32) -> u32 {
    x + 1
}

async fn two_steps(x: u32) -> u32 {
    let a = one_step(x).await;
    one_step(a).await
}

fn main() {
    let simple = one_step(41);
    let bigger = two_steps(41);
    println!("size_of_val(&simple)  = {}", std::mem::size_of_val(&simple));
    println!("size_of_val(&bigger)  = {}", std::mem::size_of_val(&bigger));
}
```

```text
size_of_val(&simple)  = 8
size_of_val(&bigger)  = 16
```

(`simple` is an `async fn` with one `.await`; `bigger` is a two-step chain — both, along with every one of their local variables, fit in just a few bytes.) That's exactly what the start of the lesson said: a task's state is only as big as what it actually holds onto, not a multi-megabyte stack.

The runtime this curriculum has chosen for backend work — and the one you'll actually write code against starting next lesson — is the `tokio` crate, by a wide margin the most common choice, and the one every Phase 3 example (`axum` included) runs on top of. Structurally, `tokio` does exactly what our `block_on` did — poll, handle `Pending`, return `Ready` — just done properly: a real multi-threaded scheduler, I/O genuinely wired into the operating system so `Waker`s actually fire the moment they should, real timers. One more piece [2.8.4](../04-send-and-sync/README.md) already set up for right here: a multi-threaded runtime like `tokio`'s default one can move a task's `Future` from one worker thread to another between polls, so that `Future` has to be `Send` — exactly the trait [2.8.4](../04-send-and-sync/README.md) taught you to reason about. None of this is code you write today — [2.8.6](../06-tokio-basics/README.md) is where you actually sit down and write `tokio` code.

---

## Hands on

```sh
cargo run -p p2-08-05-futures-and-runtimes --example 01-calling-does-nothing
cargo run -p p2-08-05-futures-and-runtimes --example 02-poll-by-hand
cargo run -p p2-08-05-futures-and-runtimes --example 03-a-tiny-executor
```

Then the three broken ones:

```sh
cargo run -p p2-08-05-futures-and-runtimes --example 04-await-outside-async-broken --features broken
cargo run -p p2-08-05-futures-and-runtimes --example 05-forgot-to-await-broken --features broken
cargo run -p p2-08-05-futures-and-runtimes --example 06-pin-new-on-a-real-future-broken --features broken
```

Then try:

1. In `01-calling-does-nothing`, change `greet("Matin");` to `let _ = greet("Matin");`. Does the warning still appear? Why?
2. In `02-poll-by-hand`, call `poll` a third time. Based on `FlipOnce`'s own code, what does it return? (`Future::poll`'s documentation warns that calling `poll` again after `Ready` has no defined contract — see for yourself why right here.)
3. In `03-a-tiny-executor`, add a second `YieldOnce` to `run`, right after the first one. How many times does `block_on` now print "poll attempt"?

---

## Errors you will meet

### `E0728` — `.await` is only allowed inside `async` functions and blocks

```text
error[E0728]: `await` is only allowed inside `async` functions and blocks
  --> examples/04-await-outside-async-broken.rs:10:27
   |
 9 | fn main() {
   | --------- this is not `async`
10 |     let result = fetch(7).await;
   |                           ^^^^^ only allowed inside `async` functions and blocks

For more information about this error, try `rustc --explain E0728`.
```

**What the compiler is objecting to:** `.await` isn't just a method you can call on any value — it's syntax that only makes sense inside an `async fn` or `async {}`, because only there can the compiler build a state machine that can actually suspend right at this spot. A plain `fn main()`, exactly as the compiler says, isn't `async`.

**The fix:** move `.await` inside an `async fn`, and drive that `async fn` with a hand-rolled `block_on` (exactly like the one we built in "The concept"):

```rust
async fn run() -> u32 {
    fetch(7).await
}

fn main() {
    let result = block_on(run());
    println!("result = {result}");
}
```

**Why that's the fix:** nothing makes `fn main()` `async` for you automatically — something has to build a runtime first and then drive the async body on top of it. Here, we wrote that "something" ourselves; from [2.8.6](../06-tokio-basics/README.md) on, `#[tokio::main]` fills exactly this gap for you.

### `E0308` — forgetting to `.await`

```text
error[E0308]: mismatched types
  --> examples/05-forgot-to-await-broken.rs:10:22
   |
10 |     let value: u32 = fetch(id);
   |                ---   ^^^^^^^^^ expected `u32`, found future
   |                |
   |                expected due to this
   |
note: calling an async function returns a future
  --> examples/05-forgot-to-await-broken.rs:10:22
   |
10 |     let value: u32 = fetch(id);
   |                      ^^^^^^^^^
help: consider `await`ing on the `Future`
   |
10 |     let value: u32 = fetch(id).await;
   |                               ++++++

For more information about this error, try `rustc --explain E0308`.
```

**What the compiler is objecting to:** `fetch(id)`'s type isn't `u32` — it's a `Future<Output = u32>`. `value` is declared with the explicit type `u32`, and the compiler doesn't do any automatic conversion between those two types, exactly as it wouldn't between `Option<u32>` and `u32` ([1.6.1](../../../phase1-fundamentals/06-absence-and-failure/01-option-and-null-safety/README.md)).

**The fix:** the compiler already said it — add `.await`:

```rust
let value: u32 = fetch(id).await;
```

**Why that's the fix:** `fetch(id)` alone is exactly what the second part of "The concept" showed you — a `Future`, motionless, with nothing run yet. `.await` is what actually `poll`s it and pulls the final value out. The interesting part here is that the compiler itself knows exactly what's wrong — see the note "calling an async function returns a future" — because a lot of programmers, at least once, forget exactly this.

### `E0277` — you can't `Pin::new` a real `Future` directly

```text
error[E0277]: `{async fn body of add_one()}` cannot be unpinned
    --> examples/06-pin-new-on-a-real-future-broken.rs:18:27
     |
   9 | async fn add_one(x: u32) -> u32 {
     |                             --- within this `impl Future<Output = u32>`
...
  18 |     let result = Pin::new(&mut future).poll(&mut cx);
     |                  -------- ^^^^^^^^^^^ within `impl Future<Output = u32>`, the trait `Unpin` is not implemented for `{async fn body of add_one()}`
     |                  |
     |                  required by a bound introduced by this call
     |
     = note: consider using the `pin!` macro
             consider using `Box::pin` if you need to access the pinned value outside of the current scope
note: required because it appears within the type `impl Future<Output = u32>`
    --> examples/06-pin-new-on-a-real-future-broken.rs:9:29
     |
   9 | async fn add_one(x: u32) -> u32 {
     |                             ^^^
note: required by a bound in `Pin::<Ptr>::new`
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\pin.rs:1159:25
     |
1159 | impl<Ptr: Deref<Target: Unpin>> Pin<Ptr> {
     |                         ^^^^^ required by this bound in `Pin::<Ptr>::new`
...
1181 |     pub const fn new(pointer: Ptr) -> Pin<Ptr> {
     |                  --- required by a bound in this associated function

For more information about this error, try `rustc --explain E0277`.
```

**What the compiler is objecting to:** `Pin::new` only works for types that have themselves promised to be `Unpin` — meaning moving them in memory, even after `poll` has started, is always safe. The state machine the compiler builds for `async fn add_one` is exactly what "Why a `Future` must not move" explained: it is not `Unpin` — not because this particular one genuinely self-references, but because the compiler applies that same rule to **every** `async fn`, without exception, in case one of them ever does.

**The fix:** put it on the heap instead, with `Box::pin`, in place of `Pin::new`:

```rust
let mut future = Box::pin(add_one(41));
let result = future.as_mut().poll(&mut cx);
```

**Why that's the fix:** `Box::pin` doesn't need `Unpin` — because it guarantees on its own that the value will never move again; it lands on the heap once and stays right there. That's exactly the one difference that let our `block_on` in "The concept" accept any `Future` — hand-rolled or a genuinely real `async fn` — with no `Unpin` bound at all.

---

## Exercises

### Warm up

<details>
<summary>What does this print?</summary>

```rust
async fn double(x: u32) -> u32 {
    println!("computing...");
    x * 2
}

fn main() {
    let _future = double(21);
    println!("done");
}
```

</details>

<details>
<summary>Answer</summary>

```text
done
```

`double(21)` just builds and returns a `Future` — because binding it to `_future` doesn't `poll` or `.await` anything either, `double`'s body never runs. (This time, because it's bound to a variable rather than discarded as a bare expression statement, the `unused implementer of Future` warning doesn't appear — but nothing inside it has run either way.)

</details>

<details>
<summary>Does this compile?</summary>

```rust
async fn total(a: u32, b: u32) -> u32 {
    let sum: u32 = a + b;
    sum
}

async fn caller() -> u32 {
    let value: u32 = total(1, 2);
    value
}
```

</details>

<details>
<summary>Answer</summary>

No — `E0308`, "expected `u32`, found future". Calling `total(1, 2)` gives a `Future<Output = u32>`, not a `u32`; the compiler does no automatic conversion between those types, even here, where `caller` itself is also `async`. You need `total(1, 2).await`.

</details>

<details>
<summary>Does this compile?</summary>

```rust
fn helper() -> u32 {
    5
}

async fn caller() -> u32 {
    helper().await
}
```

</details>

<details>
<summary>Answer</summary>

No — `E0277` this time, because `helper()` doesn't return a `Future` at all (its type is `u32`, not something implementing `Future`), so `.await` on it doesn't mean anything. Being inside an `async fn` only permits writing `.await`; the value itself still has to actually be a `Future`.

</details>

<details>
<summary>Because <code>FlipOnce</code> has no internal references and is <code>Unpin</code>, what does <code>Pin&lt;&amp;mut FlipOnce&gt;</code> practically differ from a plain <code>&amp;mut FlipOnce</code> by?</summary>

Almost nothing. For an `Unpin` type, `Pin<&mut Self>` hands back the same ordinary `&mut Self` access through `DerefMut` — exactly what let `self.polled_before = true` work with no ceremony at all. `Pin`'s real restrictions only bite for `!Unpin` types — like the state machine of a genuinely real `async fn`.

</details>

### Repair

Fix all three broken examples:

1. Fix `examples/04-await-outside-async-broken.rs` so it compiles — by moving `.await` inside an `async fn` and driving it with a hand-rolled `block_on`, exactly like the one we built in "The concept."
2. Fix `examples/05-forgot-to-await-broken.rs` by adding `.await` — not by changing `value`'s declared type.
3. Fix `examples/06-pin-new-on-a-real-future-broken.rs` by replacing `Pin::new` with `Box::pin`.

### Implement

Two things in `src/lib.rs`:

```sh
cargo test -p p2-08-05-futures-and-runtimes
```

`Countdown::poll` and `block_on` — exactly the two things you saw in "The concept," this time implemented by you. The doc comments above each function spell out exactly what each one does.

### Build

Write a second hand-rolled `Future`, `AlwaysPending`, whose `poll` always — with no exception — returns `Poll::Pending`. Run it through your own `block_on`, but **don't wait for it to finish** (it never will; interrupt it with `Ctrl+C`). Then write a paragraph: what does a real executor need, beyond our busy loop, to handle a situation like this correctly? (Think about timeouts, cancelling a task, and the fact that a real program should never lock up a thread forever.)

### Challenge (optional)

**Part one.** Write a third hand-rolled `Future`, without touching `Countdown`'s own definition: a type that holds two independent `Countdown`s as plain fields and resolves, to the sum of their two totals, only once both have separately reached `Ready`. (Hint: because both fields are themselves `Unpin`, the whole struct is automatically `Unpin` too — you can poll each field directly with `Pin::new(&mut self.field)`, with no special pin-projection machinery needed.)

**Part two.** (This one looks ahead — [2.8.6](../06-tokio-basics/README.md) is where you'll actually write this.) Look up the signature of `tokio::time::sleep`. Based only on what this lesson taught about `Future`, `poll`, and `Waker`, write a few sentences: the first time something like `sleep(Duration::from_secs(1)).await` is polled, does it return `Ready` or `Pending`? What do you think has to happen, and whose job do you think it is, before a second poll can return `Ready`?

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `Future` | a value representing a computation that might not be finished yet; it does nothing on its own until `poll`ed | the return type of every `async fn` |
| `poll` | `Future`'s core method; advances it one step and returns `Poll::Ready` or `Poll::Pending` | calling it is the runtime's job, not usually yours |
| `Poll::Ready(v)` / `Poll::Pending` | done, here's the value / not yet, try again later | the return value of every `poll` |
| `Pin<&mut Self>` | guarantees a value never moves again once it has started being polled | `Future::poll`'s signature; only genuinely restrictive for `!Unpin` types |
| `Waker` | a handle a `Pending` `Future` keeps, to notify the executor once it's actually worth polling again | the `cx` parameter on every `poll` |
| runtime (executor) | whatever actually calls `poll()` and handles `Pending`/`Ready` | our hand-rolled `block_on` today; `tokio` from 2.8.6 |
| cooperative scheduling | a task only gives up control at `.await`, never whenever the OS feels like it | why async can be so much lighter than threads |

### What you now know

- Threads ([2.8.1](../01-threads-mutex-arc/README.md)) are for genuinely CPU-bound work; async is for work that's mostly waiting — and you can have hundreds of thousands of async tasks where even a few thousand threads would already cost a lot, in memory and scheduling.
- Calling an `async fn` neither spawns a thread nor runs its body — it just returns a `Future` value that stays motionless until it's `poll`ed.
- The `Future` trait has exactly one method: `poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>`, giving back either `Ready(value)` or `Pending`.
- `.await` means: poll this `Future`; if `Ready`, carry on; if `Pending`, suspend right here and hand control — not a locked thread — back to the runtime.
- The state machine of a genuinely real `async fn` is always `!Unpin`, because it might reference itself between suspension points; `Box::pin` is usually the simplest fix.
- A lone `Future` doesn't drive itself forward; an executor is needed to call `poll`, and for real I/O it waits on the `Waker` for a "try again" signal instead of guessing forever.

### What comes back later

- **Writing real async code with `tokio`, including `tokio::spawn`** — [2.8.6 — Tokio basics](../06-tokio-basics/README.md)
- **Structured concurrency with `spawn` and `JoinSet`** — [2.9.1 — `spawn`, `JoinSet`, structured concurrency](../../09-async-in-practice/01-spawn-joinset-structured-concurrency/README.md)
- **`select!` and cancellation safety** — [2.9.2 — `select!` and cancellation safety](../../09-async-in-practice/02-select-and-cancellation-safety/README.md)
- **Streams, with `futures::Stream` and `tokio-stream`** — [2.9.3 — Streams](../../09-async-in-practice/03-streams/README.md)

### Can you explain?

- Why is calling an `async fn` neither of these two things: spawning a new thread, or immediately running its body?
- Write `poll`'s signature from memory. What does each of `Poll::Ready` and `Poll::Pending` mean?
- Mechanically, what does `.await` actually do, and exactly how does that differ from a blocking call like `std::thread::sleep`?
- Why must a real `async fn`'s state machine never move once it starts being polled, and how does `Pin` guarantee that?
- Explain "a lone `Future` does nothing" in your own words — and say exactly what an executor adds to that picture.

---

## Going further

- [The Rust Book — Chapter 17: Async and Await](https://doc.rust-lang.org/book/ch17-00-async-await.html) — the same ground, official, with more detail.
- [`std::future::Future`](https://doc.rust-lang.org/std/future/trait.Future.html) — full documentation for the exact trait you saw today.
- [`std::pin`](https://doc.rust-lang.org/std/pin/index.html) — the official, complete explanation of `Pin`, for when you want to go deeper than today did.
- [Asynchronous Programming in Rust (the async book)](https://rust-lang.github.io/async-book/) — builds a `block_on` and a tiny executor the exact same way, chapter by chapter.
