# 2.8.3 — Channels and message passing

## At a glance

After this lesson you can:

- Explain why threads that pass messages to each other need no locking at all, and state the slogan "do not communicate by sharing memory; instead, share memory by communicating" in your own words, with an example.
- Build a small worker pool: several worker threads each sending their own result back through one channel to a single collecting thread, closing the channel so the receiver's iteration ends cleanly exactly when it should, instead of hanging forever.
- Choose between an unbounded `channel()` and a bounded `sync_channel(n)` for a given producer/consumer speed mismatch, and say exactly what you're buying with that backpressure.

**Time:** ~65 minutes · **Prerequisites:**
[2.8.2 — `RwLock`, `Semaphore`, `OnceLock`/`LazyLock`, atomics](../02-rwlock-semaphore-oncelock-atomics/README.md)

---

## Why this matters

[2.8.1](../01-threads-mutex-arc/README.md) and [2.8.2](../02-rwlock-semaphore-oncelock-atomics/README.md) handed you a whole family of tools, all for one specific problem: several threads need the *same* piece of memory, so access to it has to be controlled with discipline and locks — `Mutex` for exclusive writes, `RwLock` for many readers and one writer, a semaphore for capping how many run at once, `OnceLock`/`LazyLock` for lazy one-time initialization, atomics for simple counters with no lock at all. Every one of them shared an assumption: the data stays in one place, and every thread that needs it takes a turn.

This lesson drops that assumption. Instead of several threads reaching into one shared box, threads **send values they own to each other** — a pattern called **message passing**. Once a value has been sent, nobody needs to lock anything around it, because no two threads ever touch it at the same time again. The idea has a name that comes from the Go community's own documentation and common parlance, and it has become a slogan repeated all over Rust too:

> "Do not communicate by sharing memory; instead, share memory by communicating."

That slogan is the frame for this entire lesson. Picture building a small worker pool where each worker processes one chunk of a bigger job and you need to gather every result in one place — you could put a shared `Vec<i32>` behind a `Mutex` and have every worker lock it, push, and unlock; or you could have every worker hand its result down a channel, with no locking at all, because no two threads ever touch the same memory at the same time. That's exactly what you build today.

---

## The concept

### `mpsc::channel()`: sender, receiver, and a one-shot handoff

A channel gives you a pair: a `Sender<T>` and a `Receiver<T>`, connected like the two ends of a pipe. `tx.send(v)` puts a value into the pipe; `rx.recv()` blocks the calling thread until a value arrives:

```rust
use std::sync::mpsc;
use std::thread;

let (tx, rx) = mpsc::channel();

thread::spawn(move || {
    let result = 6 * 7;
    tx.send(result).unwrap();
});

let answer = rx.recv().unwrap();
println!("main thread received: {answer}");
```

```text
main thread received: 42
```

`mpsc::channel()` returns a `(Sender<T>, Receiver<T>)` pair — the names `tx`/`rx` (transmit/receive) are a near-universal Rust convention, not something required. `tx.send(v)` returns a `Result<(), SendError<T>>` (`Err` only once no `Receiver` is left alive); `rx.recv()` returns a `Result<T, RecvError>` and genuinely blocks the calling thread until either a value arrives or it becomes clear nothing ever will.

### Sending moves ownership, it does not share it

`Sender::send` takes its argument **by value** — it moves the value into the channel instead of taking a reference to it. Whatever you send is no longer yours:

```rust
let (tx, rx) = mpsc::channel();
let readings = vec![12, 47, 8, 33];

tx.send(readings).unwrap(); // `readings` moves into the channel here
// `readings` cannot be named again from this point on.

let mut received = rx.recv().unwrap();
received.push(100); // owned outright — free to mutate
println!("received, now owned and mutated: {received:?}");
```

```text
received, now owned and mutated: [12, 47, 8, 33, 100]
```

The sending side no longer has any copy of `readings` at all; the receiving side gets full, independent ownership — independent enough to mutate it. This is exactly what the slogan above means: instead of two threads looking at one `readings`, exactly one thread owns it at any given moment. Try to use `readings` again after `send`, from the same place you sent it, and the compiler stops you — you'll see that exact scenario in "Errors you will meet".

```senpai-visual
{"kind":"ownership","labels":["readings, on the sender","tx.send(readings)","gone from the sender","owned by the receiver"]}
```

### `Sender` is `Clone`: multi-producer, single-consumer

The name `mpsc` is short for exactly this: **m**ulti-**p**roducer, **s**ingle-**c**onsumer. `Sender<T>` implements `Clone`: every thread that needs to send gets its own copy of `tx`, and every copy feeds the exact same `Receiver`:

```rust
let (tx, rx) = mpsc::channel();
let tx_a = tx.clone();
let tx_b = tx.clone();
drop(tx); // only these two clones will ever send

thread::spawn(move || tx_a.send("thread A").unwrap());
thread::spawn(move || tx_b.send("thread B").unwrap());

let mut received: Vec<&str> = vec![rx.recv().unwrap(), rx.recv().unwrap()];
received.sort();
println!("received, sorted for a stable printout: {received:?}");
```

```text
received, sorted for a stable printout: ["thread A", "thread B"]
```

Which thread arrives first isn't guaranteed — the two are genuinely racing — which is why `received` is sorted before printing; the *set* of who arrived is deterministic even when the arrival order isn't. Notice `Receiver` is **not** `Clone` — there is exactly one consumer in the whole picture, no matter how many producers there are.

### Iterating the receiver directly, and a clean end when every sender is gone

This is where it all comes together. `Receiver<T>` implements `IntoIterator`: instead of calling `.recv()` one at a time, you can `for`-loop over it directly. The loop ends exactly once **every** `Sender` — the original plus every `.clone()` of it — has been dropped:

```rust
let (tx, rx) = mpsc::channel();
for chunk in minutes_watched.chunks(chunk_size) {
    let tx = tx.clone();
    let chunk = chunk.to_vec();
    thread::spawn(move || {
        let partial: i32 = chunk.iter().sum();
        tx.send(partial).unwrap();
    });
}
drop(tx); // the original — without this, the loop below never ends
let mut total = 0;
for partial in rx {
    total += partial;
}
```

```text
4 workers reported in, grand total: 326
```

This is the worker pool "Why this matters" promised: four worker threads, each summing one chunk of `minutes_watched` and sending back one `partial`; the main thread walks `for partial in rx` to build the grand total (the complete program, including counting how many partials arrived and `join()`ing every thread, is in `examples/04-worker-pool-sum.rs`).

The detail that's easy to miss: the original `tx`, the one that stayed in the parent thread and that you cloned from, is alive for the rest of its own scope — that is, until the function returns. Without that explicit `drop(tx)`, the `for partial in rx` loop would wait on that one unused `Sender`, because it has no way of knowing you'll never call `.send()` on it. The clones living inside each worker's closure drop themselves automatically the moment that closure ends — no explicit `drop` needed there at all.

```senpai-visual
{"kind":"concurrency","labels":["worker 0 sends","worker 1 sends","worker 2 sends","drop(tx) — the original","rx ends, cleanly"]}
```

### Unbounded vs. bounded: `channel()` and `sync_channel(n)`

`mpsc::channel()` is **unbounded**: `.send()` never blocks, values queue up for as long as they need to. If a producer is much faster than the consumer, the queue just keeps growing — with no warning at all. `mpsc::sync_channel(n)` is **bounded**: at most `n` values can be waiting in the queue, and once it's full, `.send()` itself blocks until the receiver makes room. That is exactly what backpressure means: a way for the system to tell the producer "slow down".

`sync_channel(0)` is the most bounded case of all — a rendezvous: there's no room to queue anything at all, so `send()` cannot return until a `recv()` is there to take the value. This example proves that with a second, ordinary channel acting as a "did send() return yet?" signal, instead of just hoping about timing:

```rust
let (tx, rx) = mpsc::sync_channel::<i32>(0);
let (signal_tx, signal_rx) = mpsc::channel::<()>();

thread::spawn(move || {
    tx.send(1).unwrap(); // blocks here until main calls rx.recv()
    signal_tx.send(()).unwrap(); // only reachable once send() returns
});

thread::sleep(Duration::from_millis(200));
println!("before recv(): signal so far = {:?}", signal_rx.try_recv());

let value = rx.recv().unwrap();
thread::sleep(Duration::from_millis(50)); // let the signal catch up
println!("after recv() got {value}: signal so far = {:?}", signal_rx.try_recv());
```

```text
before recv(): signal so far = Err(Empty)
after recv() got 1: signal so far = Ok(())
```

After 200 milliseconds — far more than enough for the worker thread to reach `tx.send(1)` — `try_recv()` on the signal channel still says `Err(Empty)`: `send(1)` has not returned yet, because nobody has called `.recv()` yet. Only once we call `rx.recv()` does `send(1)` unblock in the other thread, letting `signal_tx.send(())` run — but the OS still needs a moment to actually schedule that line, which is what the short second sleep is for; skip it and `try_recv()` can genuinely still see `Err(Empty)` right after `rx.recv()` returns, even though `send(1)` has already unblocked. That 200 milliseconds is a safety margin, not a magic number; the same rule holds at any capacity — with `sync_channel(4)`, say, the first four sends return instantly, exactly like an unbounded channel, and only the fifth one waits.

```senpai-visual
{"kind":"queue","labels":["capacity 0 (rendezvous)","tx.send(1) — blocks","rx.recv() — takes it","send() finally returns"]}
```

### Channels vs. `Mutex`: which one, when

- Does one thread just need to hand a **result** to another thread, once, when it's done? A channel (or even the return value of the `JoinHandle` you already used in [2.8.1](../01-threads-mutex-arc/README.md)) is usually simpler than building shared state — exactly what "one-shot handoff" above showed.
- Do several independent threads need to keep **feeding a stream of data** to one place, over time, without fighting each other for a lock on a shared collection? A channel — the same worker pool you just built.
- Do several threads need to both read *and* write the *same* live, continuously-changing piece of state, with every read seeing the latest write? That's where `Arc<Mutex<T>>` (or `RwLock` for many readers) from [2.8.1](../01-threads-mutex-arc/README.md)/[2.8.2](../02-rwlock-semaphore-oncelock-atomics/README.md) is the right answer — a channel doesn't naturally model "shared, continuously-updated state," only "here, take this value and own it."

Neither is inherently better. `Mutex`/`RwLock` keep data in one place and take locking discipline seriously; channels need no locking at all because they move ownership instead — at the cost of data genuinely traveling between threads, and sometimes being copied to send a piece of it at all. The choice depends on the shape of the problem itself: should the data stay in one place that everyone reaches into, or does it make more sense to hand ownership of each piece to whoever needs it next?

---

## Hands on

```sh
cargo run -p p2-08-03-channels-message-passing --example 01-one-shot-handoff
cargo run -p p2-08-03-channels-message-passing --example 02-send-consumes-the-value
cargo run -p p2-08-03-channels-message-passing --example 03-clone-sender-multiple-producers
cargo run -p p2-08-03-channels-message-passing --example 04-worker-pool-sum
cargo run -p p2-08-03-channels-message-passing --example 05-bounded-send-blocks-until-received
cargo run -p p2-08-03-channels-message-passing --example 06-forgot-to-drop-sender-trap
```

Then the two broken ones:

```sh
cargo run -p p2-08-03-channels-message-passing --example 07-recv-on-disconnected-channel-broken --features broken
cargo run -p p2-08-03-channels-message-passing --example 08-use-after-send-broken --features broken
```

Then try these:

1. In `04-worker-pool-sum`, change `worker_count` from `4` to `3` (`minutes_watched` is still length 12). How big is each chunk now, and how many times is `partial` sent?
2. In `05-bounded-send-blocks-until-received`, change `sync_channel(0)` to `sync_channel(1)`. Is the `before recv()` output still `Err(Empty)`? Why do you think it stayed the same, or changed?
3. In `06-forgot-to-drop-sender-trap`, right after the line `let worker_tx = tx.clone();`, add a line `drop(tx);`. What does `second recv`'s output become now?

---

## Errors you will meet

### `E0382` — using a value after `.send()` has moved it

```text
error[E0382]: borrow of moved value: `readings`
  --> phase2-intermediate\08-concurrency\03-channels-message-passing\examples\08-use-after-send-broken.rs:16:31
   |
13 |     let readings = vec![12, 47, 8, 33];
   |         -------- move occurs because `readings` has type `Vec<i32>`, which does not implement the `Copy` trait
14 |
15 |     tx.send(readings).unwrap();
   |             -------- value moved here
16 |     println!("still have it: {readings:?}"); // ERROR: used after move
   |                               ^^^^^^^^ value borrowed here after move
   |
help: consider cloning the value if the performance cost is acceptable
   |
15 |     tx.send(readings.clone()).unwrap();
   |                     ++++++++

For more information about this error, try `rustc --explain E0382`.
```

**What the compiler is actually objecting to:** `tx.send(readings)` moved ownership of `readings` into the channel. The next line, inside `println!`, refers to `readings` again — but there's nothing left to refer to. This is exactly the same rule that governs every other move in Rust ([1.2.2](../../../phase1-fundamentals/02-ownership-and-memory/02-move-semantics/README.md)); the only thing new here is that the destination of the move is a channel instead of a function.

**The fix:** either don't touch `readings` again after `send`, or, if you genuinely need a local copy too, clone it before sending:

```rust
tx.send(readings.clone()).unwrap();
println!("still have it: {readings:?}");
```

**Why this is the fix:** `.send()` has no "just borrow it" mode — that is precisely what this lesson is teaching: message passing **moves** ownership, it does not share it. If both sides genuinely need a copy, you have to build two copies on purpose; the compiler won't hide that cost from you.

### Panic — `.recv().unwrap()` when every `Sender` was dropped without sending

```text

thread 'main' (21512) panicked at phase2-intermediate\08-concurrency\03-channels-message-passing\examples\07-recv-on-disconnected-channel-broken.rs:16:27:
called `Result::unwrap()` on an `Err` value: RecvError
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**What the compiler is actually objecting to:** nothing — this isn't a compiler error. The program built and ran. The problem is at run time: `drop(tx)` dropped the only `Sender` right there, without ever calling `.send()`. `rx.recv()` notices, and returns `Err(RecvError)` immediately — **without blocking**, since there's no hope left that anything will ever arrive. `.unwrap()` on that `Err` is what actually panics.

**The fix:** answer both cases with `match` or `if let`:

```rust
match rx.recv() {
    Ok(value) => println!("{value}"),
    Err(_) => println!("no Sender left, and nothing arrived"),
}
```

**Why this is the fix:** `Err(RecvError)` is itself a message: "no `Sender` is alive any more." Ignoring it with `.unwrap()` assumes something always arrives — an assumption this scenario broke. This exact behavior is also what lets `for v in rx` end cleanly instead of hanging forever: once every sender is gone, the iterator sees that same `Err` from its own internal `recv()` and stops the loop.

### No error, no panic — forgetting to drop one extra `Sender` clone leaves `recv()` waiting forever

```rust
let (tx, rx) = mpsc::channel::<i32>();
let worker_tx = tx.clone();

thread::spawn(move || worker_tx.send(1).unwrap())
    .join()
    .unwrap();

println!("first recv: {:?}", rx.recv());
// `tx`, the original, is still alive here — nobody dropped it.
println!(
    "second recv: {:?}",
    rx.recv_timeout(Duration::from_millis(200))
);
```

```text
first recv: Ok(1)
second recv: Err(Timeout)
```

**What the compiler is actually objecting to:** nothing — this is the most dangerous kind of bug, the kind neither the compiler nor a panic ever flags. `worker_tx` (the clone) dropped itself automatically after sending `1`, but `tx`, the original it was cloned from, is still alive in `main`'s scope. So the second `recv()` never gets the `Err` that says "nobody will ever send again" — it stays hopeful, forever. `recv_timeout` is used here to prove that "forever" without actually hanging this example: after 200 milliseconds, instead of a value, we get `Err(Timeout)` — proof that nothing was ever really on its way.

**The fix:** explicitly drop the original `tx` the moment you no longer need it:

```rust
let worker_tx = tx.clone();
drop(tx);
```

**Why this is the fix:** a `Receiver`'s iteration (whether via `for` or manual `.recv()`) has exactly one rule: as long as *one* `Sender` — even one nobody ever calls `.send()` on — is still alive, the receiver does not give up. The fix isn't a smarter compiler; it's keeping exactly as many `Sender`s alive as you actually need, not one more.

---

## Exercises

### Warm up

<details>
<summary>Does this compile?</summary>

```rust
let (tx, _rx) = mpsc::channel();
let name = String::from("Yui");
tx.send(name).unwrap();
let greeting = format!("hi {name}");
println!("{greeting}");
```

</details>

<details>
<summary>Answer</summary>

No — `E0382` ("borrow of moved value: `name`"). `tx.send(name)` took ownership of `name`; using it again inside `format!` afterward is a use after move, exactly the scenario in "Errors you will meet".

</details>

<details>
<summary>Three worker threads each hold their own <code>tx.clone()</code> and finish after one <code>send()</code>. You never wrote <code>drop(tx)</code> yourself, but the original <code>tx</code> isn't kept alive anywhere else either. Does <code>for v in rx</code> on the main thread eventually end?</summary>

Write down your reasoning.

</details>

<details>
<summary>Answer</summary>

Yes. An explicit `drop` is only needed for a `Sender` that lives in a scope you're deliberately keeping alive. The clone inside each worker's closure drops itself automatically the moment that closure ends — and if the original `tx` isn't kept alive anywhere either, it drops too, at the end of its own scope. Once they're all gone, `for v in rx` ends.

</details>

<details>
<summary>A <code>Sender</code> you never call <code>send</code> on, dropped immediately. What does <code>rx.recv()</code> return afterward?</summary>

Write down your answer.

</details>

<details>
<summary>Answer</summary>

`Err(RecvError)` — immediately, with no blocking. `recv()` recognizes that no `Sender` is left to send anything, so it doesn't wait.

</details>

<details>
<summary>Does this compile?</summary>

```rust
let (_tx, rx) = mpsc::channel::<i32>();
let rx2 = rx.clone();
```

</details>

<details>
<summary>Answer</summary>

No — `E0599` ("no method named `clone` found for struct `std::sync::mpsc::Receiver<T>` in the current scope"). `Sender` is `Clone`, `Receiver` is not: exactly one consumer is allowed.

</details>

### Repair

Fix all three examples:

1. Fix `examples/08-use-after-send-broken.rs` so it compiles — either stop touching `readings` after `send`, or clone it first if you genuinely need both copies.
2. Fix `examples/07-recv-on-disconnected-channel-broken.rs` so it no longer panics — answer both cases of the `Result` with `match` or `if let` instead of `.unwrap()`.
3. Fix `examples/06-forgot-to-drop-sender-trap.rs` so `second recv` actually receives a value instead of `Err(Timeout)` — add the missing `drop(tx)`.

### Implement

Two functions in `src/lib.rs`, using only this lesson's own tools: `mpsc::channel`, `.clone()` on a `Sender`, `drop`, `thread::spawn`, `.join()`.

```sh
cargo test -p p2-08-03-channels-message-passing
```

### Build

Write a `pub fn first_to_finish(a: fn() -> i32, b: fn() -> i32) -> i32`: spawn two threads, one running `a` and the other `b`, both sending their result through (clones of) one shared channel, and have the function return whichever value arrives **first** — with a single `.recv()`. In the function's doc comment, say what you did with the thread whose result arrives second (let it finish in the background, or wait for it to `join`) and when the other choice would matter more.

### Challenge (optional)

`Receiver` is not `Clone` — so if you want a **pool** of workers all pulling from the exact same job queue (rather than each knowing its own work in advance), you also need last lesson's toolkit: wrap the `Receiver` in `Arc<Mutex<Receiver<i32>>>` so several threads can take turns locking it and calling `.recv()`.

Write a `pub fn run_job_pool(worker_count: usize, jobs: Vec<i32>) -> Vec<i32>`: every number in `jobs` is a job; running a job produces `job * 2`, sent through a **second**, separate channel (the results channel). Each job runs on whichever of the `worker_count` workers gets to it first. The function returns every result, sorted ascending (arrival order across workers is not guaranteed).

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Message passing | threads send owned values to each other instead of sharing memory | the alternative to `Arc<Mutex<T>>` |
| Channel (`mpsc::channel`) | a `Sender`/`Receiver` pair; `.send()` queues, `.recv()` dequeues | handing back a result, collecting output from several threads |
| `Sender::clone()` | one copy of the sender per thread that needs to send | multi-producer, single-consumer |
| `for v in rx` | iterating the receiver directly; ends once every `Sender` is gone | collecting an unknown number of messages |
| `channel()` / `sync_channel(n)` | unbounded vs. bounded with capacity `n` | when you want the producer to wait on the consumer |
| Backpressure | a full channel makes `.send()` block until there's room | stopping a producer from unboundedly outrunning its consumer |

### What you now know

- Channels are a genuinely different model from shared memory: instead of locking one fixed value, they move ownership of values between threads.
- `tx.send(v)` **moves** the value — you're no longer the owner at the send site afterward, unless you cloned it first.
- `Sender` is `Clone` (multi-producer); `Receiver` is not (exactly one consumer).
- `for v in rx` ends cleanly once every `Sender` — the original plus every clone — has been dropped; as long as even one stays alive, it keeps waiting.
- `mpsc::channel()` is unbounded, `.send()` never blocks; `mpsc::sync_channel(n)` is bounded and provides backpressure.
- Neither channels nor `Mutex`/`RwLock` are inherently better — the choice depends on the shape of the problem: live shared state, or a transfer of ownership.

### What comes back later

- **`Send` and `Sync`, exactly which types are safe to cross a thread boundary** — [2.8.4 — `Send` and `Sync`](../04-send-and-sync/README.md)
- **Coordinating concurrent work once it's async tasks instead of OS threads, not plain threads** — [Module 2.9 — Async in practice](../../09-async-in-practice/README.md)

### Can you explain?

- Explain the slogan "do not communicate by sharing memory; instead, share memory by communicating" in your own words, over a real example.
- Why does `tx.send(v)` take ownership of `v`? What would go wrong if it only borrowed it instead?
- Why does `for v in rx` end on its own, with no explicit `drop`, once every worker finishes — but that same original `Sender`, if you keep it alive somewhere, blocks exactly that ending?
- Explain the difference between `mpsc::channel()` and `mpsc::sync_channel(n)`, and exactly where backpressure sits in that difference.
- Give one real scenario where `Arc<Mutex<T>>` is the right answer instead of a channel, and one where it's the other way around.

---

## Going further

- [The Rust Book — Chapter 16.2, message passing](https://doc.rust-lang.org/book/ch16-02-message-passing.html) — the same subject, from the official source.
- [`std::sync::mpsc` docs](https://doc.rust-lang.org/std/sync/mpsc/index.html) — the full method list for `Sender`/`Receiver`, including `try_recv`, `recv_timeout`, and the error types.
- [`crossbeam-channel`](https://docs.rs/crossbeam-channel/) — for when you go beyond the standard library later: a faster channel with a `Clone`-able `Receiver` and a real `select!` — something `std::sync::mpsc` deliberately doesn't offer.
