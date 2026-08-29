# 2.8.1 — Threads, `Mutex`, `Arc`

## At a glance

After this lesson you can:

- Spawn a real OS thread with `thread::spawn`, get its result back through a `JoinHandle` and `.join()`, and read and fix the common first-attempt compiler error — a closure borrowing a local variable without `move`.
- Share mutable state safely across threads with `Arc<Mutex<T>>`, and say in one sentence why 2.6.3's `Rc` and 2.6.5's `RefCell` have no substitute here.
- Read an `Err` caused by lock poisoning, explain exactly what "poisoned" means, and decide what your own code should do about it.

**Time:** ~65 minutes · **Prerequisites:**
[2.6.3 — `Rc` and `Arc`](../../06-smart-pointers/03-rc-and-arc/README.md) ·
[2.6.5 — `RefCell`, `Cell`, and the run-time panic trade](../../06-smart-pointers/05-refcell-and-interior-mutability/README.md)

---

## Why this matters

Remember the aliasing rule from Phase 1: any number of shared borrows, or exactly one mutable borrow — never both at once. Up to this point in the course, the compiler enforced that rule on code that always ran on a single thread, top to bottom. Today that assumption goes away: the code you write can run on several genuinely separate threads at once, on different cores of the same processor — real parallelism, not a simulation of it.

2.6.3 ended with a promise, not just a wrap-up: "The exact mechanics of crossing a thread boundary, and `Send` itself, belong to module 8; today all you need is this fact: `Rc` deliberately stays inside one thread." 2.6.5 closed the same way, with an actual experiment: build an `Rc<RefCell<i32>>` and drop it into a `std::thread::spawn` closure — it won't compile, and the compiler says outright that `Rc<RefCell<i32>>` cannot be safely sent between threads. Today is where both promises get paid: the same two types that were trustworthy inside one thread turn out to have a real reason they aren't trustworthy across several — and today you learn exactly what to reach for instead.

If you've worked with Python, you already know the shape of this problem. `threading.Thread` in CPython genuinely builds an OS-level thread — not a simulation, a real one. But the GIL (Global Interpreter Lock) guarantees only one of those threads executes Python bytecode at any instant; for CPU-bound code, that means `threading` gives you no real parallelism at all — for that you reach for `multiprocessing`, entirely separate processes with no shared memory whatsoever. I/O-bound code genuinely benefits from `threading`, because I/O releases the GIL while it waits. This is exactly where the bridge breaks: Rust has no GIL, so cores genuinely execute Rust code at the same instant, with no hidden lock serializing anything behind the scenes — which is exactly why the Rust compiler has to work harder here than it ever has before.

---

## The concept

### A real OS thread: `thread::spawn`, `JoinHandle`, `.join()`

```rust
use std::thread;

let handle = thread::spawn(|| {
    println!("hello from the spawned thread");
    2 + 2
});
```

```rust
let result = handle.join().unwrap(); // blocks until the thread finishes
println!("the spawned thread returned: {result}");
```

```text
hello from the spawned thread
the spawned thread returned: 4
```

`thread::spawn` hands your closure to the operating system, which runs it on a genuinely separate **thread** at the OS level — a real thread, scheduled by the operating system itself, capable of running truly in parallel on another core.

`spawn` returns a `JoinHandle<T>`, where `T` is whatever the closure returns — `i32` here. Calling `.join()` blocks the calling thread until the spawned one finishes, and hands back a value shaped like this:

```text
Result<T, Box<dyn Any + Send>>
```

`Ok(T)` if the closure finished normally, `Err(...)` if it **panicked**. That's why you see `.join().unwrap()` almost everywhere: it's a direct statement of "I expect this thread to succeed; if it didn't, panic loudly right here." Code that genuinely has to tolerate a worker thread failing should `match` on that `Result` instead of unwrapping it.

```senpai-visual
{"kind":"concurrency","labels":["main thread","thread::spawn","child thread runs","join() blocks","result returned"]}
```

### Why the closure passed to `spawn` must own what it uses

If the closure only borrows a local variable, with no `move`:

```text
error[E0373]: closure may outlive the current function, but it borrows `data`, which is owned by the current function
```

Full detail is in "Errors you will meet"; `examples/05-borrow-without-move-broken.rs` builds exactly this. The fix is `move` itself:

```rust
let data = vec![1, 2, 3];

let handle = thread::spawn(move || {
    println!("the spawned thread owns: {data:?}");
});
handle.join().unwrap();
```

```text
the spawned thread owns: [1, 2, 3]
```

The `move` keyword forces the closure to take full ownership of everything it references, instead of merely borrowing it — and this is a rule, not a style choice: `spawn`'s signature requires its closure to be `'static`, meaning it cannot hold any borrow that might not outlive the thread itself. Here's why: the function that called `spawn` could return, and its stack frame — exactly where a borrowed `data` would have lived — could be torn down before the spawned thread ever gets a chance to use it. The compiler has no way to prove the parent will wait, so it simply refuses to compile a `spawn` closure that borrows local data.

If you're coming from Python: `threading.Thread(target=lambda: print(data)).start()` just works without a second thought, because Python's reference-counting garbage collector keeps `data` alive for as long as anything still points to it — including that closure, even across a thread boundary. But this is exactly where the analogy breaks: Python's garbage collector only prevents **premature deallocation**, not a data race over that same shared value; if two Python threads mutate the same object at the same instant, the GIL alone does not stop every logical bug. Rust's solution — ownership plus `'static` — turns that exact "how long is this data valid?" question from a run-time accident into a compile-time proof.

### Shared ownership across threads: why `Arc`, not `Rc`

Say several threads need to see the same value. As always, every value needs exactly one owner — but here, several threads each want to be that owner. 2.6.3 solved exactly this problem: the same `AppConfig` you built there with `Rc::new` and shared between a handler and a logger with `Rc::clone` gives shared ownership — more than one simultaneous handle to one heap value. But that same lesson also stated a rule plainly: `Rc` deliberately must never cross a thread boundary, because its count is an ordinary, non-atomic integer — two threads running "add one" on that same count at the same moment can lose an increment, and the count can reach zero too early while a handle is still alive somewhere. The exact mechanism the compiler uses to reject this — the `Send` trait — is 2.8.4's job; all you need right now is the practical rule: the moment more than one thread is involved, swap `Rc` for `Arc`.

The API is identical, letter for letter — `Arc::new`, `Arc::clone`, the same `strong_count` — only its count moves up and down through **atomic** CPU instructions, which stay genuinely indivisible even when several cores touch that same memory at once. That atomicity is exactly what makes `Arc` safe to share across threads, at the cost of a slower clone than `Rc`'s.

One note: if you forget `Arc` and try to `move` the bare `Mutex` itself into two closures, you hit a familiar error — Phase 1's "use after move," this time at a `thread::spawn` boundary. The full version is in "Errors you will meet."

### Mutating that same shared data: why `Mutex`, not `RefCell`

`Arc` alone only shares, it doesn't mutate — exactly the same limit 2.6.3 stated for `Rc`: every owner you get from an `Arc<T>` is only `&T`. If the threads need to change that value too, you need interior mutability — exactly the problem 2.6.5's `RefCell` solved. But `RefCell` isn't safe here for the same reason `Rc` isn't: the borrow counts it tracks inside `Ref`/`RefMut` are not atomic either. Two threads calling `.borrow_mut()` at the same instant could both believe they hold the only live borrow — exactly the data race the aliasing rule exists to rule out. The exact mechanism is 2.8.4's job again; the practical rule is the same pattern: swap `RefCell` for `Mutex`.

`Mutex<T>` (short for mutual exclusion) wraps a value so that only one thread can access it at any given moment — not only for writing; unlike `RefCell`, which allows several simultaneous `Ref`s, `.lock()` always grants exclusive access, even for plain reading. That more familiar `RefCell` shape — many readers or one writer — exists across threads too, just under a different name: `RwLock`, 2.8.2's subject.

Ownership and mutability together: build an `Arc<Mutex<T>>`, take one `Arc::clone` per thread, and let each one mutate the value from behind the `Mutex`:

```rust
let counter = Arc::new(Mutex::new(0));
let mut handles = Vec::new();

for _ in 0..10 {
    let counter = Arc::clone(&counter);
    handles.push(thread::spawn(move || {
        let mut guard = counter.lock().unwrap();
        *guard += 1;
    }));
}
```

```rust
for handle in handles {
    handle.join().unwrap();
}
println!("final count: {}", *counter.lock().unwrap());
```

```text
final count: 10
```

Ten threads, each incrementing the counter exactly once; the exact order in which each thread happens to grab the lock varies every time you run it, but because `.join()` waits for every one of them before the result is read, the final number is always 10 — every time.

```senpai-visual
{"kind":"ownership","labels":["Mutex<i32> on heap","Arc clone: thread A","Arc clone: thread B","Arc clone: thread C"]}
```

### `.lock()` returns a guard — exactly like 2.6.5's `Ref`/`RefMut`

This should already feel familiar: `.lock()` returns a `MutexGuard<T>` (wrapped inside a `Result`, which the next section gets to) — an RAII guard, exactly the same pattern as 2.6.5's `Ref`/`RefMut`. Dereference it (`*guard`) to read or write the value inside; when it goes out of scope, `Drop` releases the lock automatically — you almost never call `.unlock()` yourself.

One real, sharp catch: because `MutexGuard` is itself a borrow (it borrows from the `Mutex`), the same borrow-scope rules apply to it. A bare tail expression that dereferences a guard at the very end of a block that owns the `Mutex` only locally runs straight into `E0597` — full detail and the fix are in "Errors you will meet."

### Lock poisoning: a panic while holding the lock

`.lock()` actually returns a `Result<MutexGuard<T>, PoisonError<MutexGuard<T>>>`, not a bare `MutexGuard<T>`. Here's why: if a thread panics at the exact moment it's holding the lock, that `Mutex` becomes **poisoned** — because the data behind it might have been left half-updated and broken. From that moment on, every later `.lock()`, on any thread, returns `Err` instead of granting the lock.

```rust
let counter = Arc::new(Mutex::new(0));
let poisoner = Arc::clone(&counter);

let handle = thread::spawn(move || {
    let mut guard = poisoner.lock().unwrap();
    *guard += 1;
    panic!("simulated failure mid-update");
});
let _ = handle.join(); // Err — it panicked; we don't propagate it here
```

```rust
let value = match counter.lock() {
    Ok(guard) => *guard,
    Err(poisoned) => *poisoned.into_inner(),
};
println!("value after recovery: {value}");
```

```text
thread '<unnamed>' (30132) panicked at phase2-intermediate\08-concurrency\01-threads-mutex-arc\examples\04-lock-poisoning-recovery.rs:17:9:
simulated failure mid-update
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
value after recovery: 1
```

(That number in parentheses is the thread's own id and changes every run; the rest of the message is always the same.)

The panic message prints to stderr because a non-main thread panicked — only that thread dies, not the whole program; the main thread carries on, and the program finishes with exit code 0. The `Err` branch shows the lock was poisoned, and `poisoned.into_inner()` hands back the protected guard anyway, poisoning notwithstanding — a legitimate choice once you're confident the data behind the lock is still usable despite that panic.

If you `.unwrap()` that same `.lock()` instead of matching on it, the poisoning cascades: the next thread panics too. `examples/08-unwrap-poisoned-lock-broken.rs` builds exactly this; the full text of that second panic is in "Errors you will meet."

```senpai-visual
{"kind":"concurrency","labels":["thread locks and panics","Mutex becomes poisoned","next lock() returns Err","poisoned.into_inner() recovers"]}
```

---

## Hands on

```sh
cargo run -p p2-08-01-threads-mutex-arc --example 01-spawn-and-join
cargo run -p p2-08-01-threads-mutex-arc --example 02-move-required
cargo run -p p2-08-01-threads-mutex-arc --example 03-shared-counter
cargo run -p p2-08-01-threads-mutex-arc --example 04-lock-poisoning-recovery
```

Then the four broken ones:

```sh
cargo run -p p2-08-01-threads-mutex-arc --example 05-borrow-without-move-broken --features broken
cargo run -p p2-08-01-threads-mutex-arc --example 06-mutex-without-arc-broken --features broken
cargo run -p p2-08-01-threads-mutex-arc --example 07-bare-tail-guard-broken --features broken
cargo run -p p2-08-01-threads-mutex-arc --example 08-unwrap-poisoned-lock-broken --features broken
```

Then try these:

1. In `03-shared-counter.rs`, change the loop count from 10 to 100 — do you still see the same final count every time?
2. In `04-lock-poisoning-recovery.rs`, comment out the `panic!` line — which branch runs this time, and what does it print?
3. In `01-spawn-and-join.rs`, change the closure to return a `String` instead of `2 + 2` — how does the type of `JoinHandle`, and of whatever `.join().unwrap()` gives back, change?

---

## Errors you will meet

### `E0373` — a closure without `move` can't borrow local data

```text
error[E0373]: closure may outlive the current function, but it borrows `data`, which is owned by the current function
  --> phase2-intermediate\08-concurrency\01-threads-mutex-arc\examples\05-borrow-without-move-broken.rs:10:32
   |
10 |     let handle = thread::spawn(|| {
   |                                ^^ may outlive borrowed value `data`
11 |         println!("{data:?}");
   |                    ---- `data` is borrowed here
   |
note: function requires argument type to outlive `'static`
  --> phase2-intermediate\08-concurrency\01-threads-mutex-arc\examples\05-borrow-without-move-broken.rs:10:18
   |
10 |       let handle = thread::spawn(|| {
   |  __________________^
11 | |         println!("{data:?}");
12 | |     });
   | |______^
help: to force the closure to take ownership of `data` (and any other referenced variables), use the `move` keyword
   |
10 |     let handle = thread::spawn(move || {
   |                                ++++

For more information about this error, try `rustc --explain E0373`.
```

**What the compiler is objecting to:** `thread::spawn`'s signature requires its closure to be `'static`. The closure above only borrows `data` (`||`, not `move ||`), and that borrow's lifetime is tied to `main`'s own scope — shorter than the lifetime of a thread that might still be running after `main` returns.

**The fix:** the compiler says it outright — add `move`:

```rust
let handle = thread::spawn(move || {
    println!("{data:?}");
});
```

**Why this is the fix:** `move` forces the closure to take full ownership of `data` instead of borrowing it. The closure no longer depends on anything outside itself — wherever it goes, `data` goes with it, so it no longer matters when `main` returns.

### `E0382` — moving a bare `Mutex` into two closures

```text
error[E0382]: use of moved value: `counter`
  --> phase2-intermediate\08-concurrency\01-threads-mutex-arc\examples\06-mutex-without-arc-broken.rs:15:19
   |
 9 |     let counter = Mutex::new(0);
   |         ------- move occurs because `counter` has type `std::sync::Mutex<i32>`, which does not implement the `Copy` trait
10 |
11 |     thread::spawn(move || {
   |                   ------- value moved into closure here
12 |         *counter.lock().unwrap() += 1;
   |          ------- variable moved due to use in closure
...
15 |     thread::spawn(move || {
   |                   ^^^^^^^ value used here after move
16 |         *counter.lock().unwrap() += 1;
   |          ------- use occurs due to use in closure

For more information about this error, try `rustc --explain E0382`.
```

**What the compiler is objecting to:** `Mutex<i32>` is no different from any other non-`Copy` value Phase 1 showed you. The first closure took full ownership of `counter` with `move`; the second wants that same ownership again, and there's nothing left to give it — exactly Phase 1's "use after move," this time at a `thread::spawn` boundary.

**The fix:** wrap `counter` in an `Arc`, and hand each closure a fresh `Arc::clone` instead of `counter` itself:

```rust
let counter = Arc::new(Mutex::new(0));
let a = Arc::clone(&counter);
let b = Arc::clone(&counter);
thread::spawn(move || *a.lock().unwrap() += 1);
thread::spawn(move || *b.lock().unwrap() += 1);
```

**Why this is the fix:** now each closure owns its own handle — a separate `Arc<Mutex<i32>>` pointing at the same underlying allocation — rather than owning the `Mutex` itself. This is exactly what "shared ownership" in "The concept" was built for.

### `E0597` — dereferencing a guard in a bare tail expression

```text
error[E0597]: `counter` does not live long enough
  --> phase2-intermediate\08-concurrency\01-threads-mutex-arc\examples\07-bare-tail-guard-broken.rs:13:6
   |
12 |     let counter = Arc::new(Mutex::new(5));
   |         ------- binding `counter` declared here
13 |     *counter.lock().unwrap()
   |      ^^^^^^^----------------
   |      |
   |      borrowed value does not live long enough
   |      a temporary with access to the borrow is created here ...
14 | }
   | -
   | |
   | `counter` dropped here while still borrowed
   | ... and the borrow might be used here, when that temporary is dropped and runs the `Drop` code for type `std::sync::MutexGuard`
   |
   = note: the temporary is part of an expression at the end of a block;
           consider forcing this temporary to be dropped sooner, before the block's local variables are dropped
help: for example, you could save the expression's value in a new local variable `x` and then make `x` be the expression at the end of the block
   |
13 |     let x = *counter.lock().unwrap(); x
   |     +++++++                         +++

For more information about this error, try `rustc --explain E0597`.
```

**What the compiler is objecting to:** `counter.lock()` builds a temporary `MutexGuard` that borrows from `counter`. Because this expression is the block's **final** tail expression, that temporary guard stays alive until the very end of the block — exactly where `counter` itself is also about to be dropped. The two can't be dropped at the same moment when one still borrows from the other.

**The fix:** the compiler suggests it directly — pour the value into a `let` first:

```rust
fn current_value() -> i32 {
    let counter = Arc::new(Mutex::new(5));
    let value = *counter.lock().unwrap();
    value
}
```

**Why this is the fix:** now the guard is dropped exactly where `let value = ...;` ends — before `counter` itself goes out of scope at the end of the block. The copied `i32` (copied because `i32` is itself `Copy`) lives completely independently of that guard, so returning it no longer depends on anything being dropped at that same instant.

### A run-time panic — a poisoned lock, blindly `unwrap`ped

```text
thread '<unnamed>' (28856) panicked at phase2-intermediate\08-concurrency\01-threads-mutex-arc\examples\08-unwrap-poisoned-lock-broken.rs:17:9:
simulated failure mid-update
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'main' (27372) panicked at phase2-intermediate\08-concurrency\01-threads-mutex-arc\examples\08-unwrap-poisoned-lock-broken.rs:21:32:
called `Result::unwrap()` on an `Err` value: PoisonError { .. }
```

(Both numbers in parentheses are thread ids and change every run; the rest of the text is fixed.)

**This isn't even a compiler error:** the program compiles and runs completely, and it panics exactly where line 21 — the second `.lock().unwrap()` — executes. The first thread had already, deliberately, panicked while holding the lock; that `Mutex` has been poisoned ever since. `.unwrap()` on a `Result::Err` — which is now what `.lock()` always returns here — panics too.

**The fix:** `match` on the `Err` explicitly (or call `.into_inner()`) instead of blindly unwrapping — exactly the pattern from `04-lock-poisoning-recovery.rs` you saw in "The concept."

**Why this is the fix:** a poisoned lock by itself doesn't mean the data behind it is genuinely broken — only that a thread panicked partway through. Code that decides for itself whether that data is still trustworthy, instead of blindly letting the first thread's panic cascade into its own, is exactly the choice `.lock()` hands you by returning a `Result` rather than a bare `MutexGuard`.

---

## Exercises

### Warm up

<details>
<summary>What does this print?</summary>

```rust
let handle = thread::spawn(|| 6 * 7);
let n = handle.join().unwrap();
println!("{n}");
```

</details>

<details>
<summary>Answer</summary>

```text
42
```

The closure borrows nothing, so it needs no `move`. `handle` is a `JoinHandle<i32>`; `.join()` waits and gives back `Ok(42)`; `.unwrap()` pulls out the `42`.

</details>

<details>
<summary>Does this compile?</summary>

```rust
let name = String::from("Frieren");
let handle = thread::spawn(|| {
    println!("{name}");
});
handle.join().unwrap();
```

</details>

<details>
<summary>Answer</summary>

No — `E0373`. The closure only borrows `name`; `spawn`'s signature requires its closure to be `'static`. The fix: `move ||`.

</details>

<details>
<summary>Does this compile?</summary>

```rust
let lock = Mutex::new(String::from("hi"));
thread::spawn(move || lock.lock().unwrap().push('!'));
thread::spawn(move || lock.lock().unwrap().push('?'));
```

</details>

<details>
<summary>Answer</summary>

No — `E0382`. The first closure took full ownership of `lock` with `move`; the second wants that same ownership again, and there's nothing left. For two simultaneous owners, `lock` needs to be inside an `Arc`.

</details>

<details>
<summary>True or false: two threads can hold a lock on the same <code>Mutex</code> at once, as long as neither is going to write.</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

False. Unlike `RefCell`, which allows several simultaneous, read-only `Ref`s, `.lock()` always grants exclusive access — even when nobody is going to write.

</details>

<details>
<summary>A thread panics while holding the lock. What does the next <code>.lock()</code> on a different thread return?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`Err` — the lock is poisoned. It doesn't block forever, and it doesn't quietly hand back an `Ok`; every later `.lock()`, on any thread, keeps returning that `Err` until you explicitly handle it.

</details>

### Repair

Fix all four broken examples:

1. Fix `examples/05-borrow-without-move-broken.rs` by adding `move` to the closure.
2. Fix `examples/06-mutex-without-arc-broken.rs` by wrapping `counter` in an `Arc` and building a separate `Arc::clone` for each closure.
3. Fix `examples/07-bare-tail-guard-broken.rs` by pouring the dereferenced value into its own `let` before returning it.
4. Fix `examples/08-unwrap-poisoned-lock-broken.rs` so the second panic no longer happens — handle the `Err` explicitly instead of blindly unwrapping.

### Implement

Two functions in `src/lib.rs` — the full specification for each is its own doc comment:

```sh
cargo test -p p2-08-01-threads-mutex-arc
```

- `sum_in_threads` — data is split across threads, but nothing is shared while they run; no `Mutex` needed.
- `count_matching_in_threads` — threads genuinely mutate one shared count at the same time; exactly what this lesson was built for.

### Build

Build a small design of your own: several threads that each record their result into a shared `Vec` or `HashMap`, behind `Arc<Mutex<...>>` — say, tallying how often a handful of words repeat, or collecting the results of several independent computations. Prove with a test or an `assert!` that every update landed exactly once — not zero times, not twice.

### Challenge (optional)

**Part one.** Right now, `count_matching_in_threads` spawns a fresh thread per item in `items` — fine for a handful of items, wasteful for a million. Write a version that spawns only `thread_count` worker threads instead (the same way `sum_in_threads` already chunks its input), while still sharing one `Arc<Mutex<i32>>` counter between them. Confirm it gives the same answer, on the same input, every single time you run the tests.

**Part two** (this one looks ahead). This is the last time you'll carry a counter this simple by hand behind `Arc<Mutex<...>>` — 2.8.2, with an atomic type, does the exact same job with no lock at all. Stay curious; you'll see why once you get there.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Thread | a real OS-level thread of execution, capable of running truly in parallel | any time work should genuinely proceed at the same time, independently |
| `JoinHandle<T>` | `thread::spawn`'s return value; `.join()` hands back the thread's result or its panic | collecting a thread's result, or waiting for it to finish |
| `Arc<T>` across threads | the same as `Rc`, with an atomic count — safe to cross a thread boundary | shared ownership of one value between several threads |
| `Mutex<T>` | exclusive access for one thread at a time, even for reading | mutating one shared value from several threads |
| `MutexGuard<T>` | the RAII guard `.lock()` returns; its `Drop` releases the lock | reading or writing the value behind a `Mutex` |
| Lock poisoning | a thread panicking while holding the lock; every later `.lock()` returns `Err` | deciding what to do about data that might be left half-updated |

### What you now know

- You spawned a real OS thread with `thread::spawn` and got its result back through `JoinHandle::join`.
- Why the closure passed to `spawn` must be `move` and stay `'static`, and you read and fixed `E0373` yourself.
- `Arc` is 2.6.3's `Rc`, just with an atomic count — safe to cross a thread boundary; `Mutex` does the same job as 2.6.5's `RefCell`, just with an OS-level lock instead of a run-time borrow count.
- `.lock()` returns an RAII guard — exactly the `Ref`/`RefMut` pattern — and because it's itself a borrow, the same borrow-scope rules apply to it (`E0597`).
- What lock poisoning means, why it happens, and how to recover from it with `match` or `.into_inner()` instead of blindly unwrapping.

### What comes back later

- **`RwLock`, many readers or one writer, and atomic types with no lock at all** — [2.8.2 — `RwLock`, `Semaphore`, `OnceLock`/`LazyLock`, atomics](../02-rwlock-semaphore-oncelock-atomics/README.md)
- **Channels: send a message instead of sharing memory** — [2.8.3 — Channels and message passing](../03-channels-message-passing/README.md)
- **The exact mechanism behind why `Arc`/`Mutex` are safe and `Rc`/`RefCell` aren't: the `Send` and `Sync` traits** — [2.8.4 — `Send` and `Sync`](../04-send-and-sync/README.md)
- **`async`, for I/O-bound work instead of CPU-bound work** — [2.8.5 — Futures and runtimes](../05-futures-and-runtimes/README.md)

### Can you explain?

- Why must the closure passed to `spawn` be `'static`, and what does `move` actually do about it?
- What exact problem do `Arc` and `Mutex` each solve, and why does sharing mutable state across threads need both together?
- Why can't `Rc`/`RefCell` substitute for `Arc`/`Mutex` once more than one thread is involved — what's the practical rule, and which lesson finishes the exact mechanism?
- What is a `MutexGuard`, and which familiar 2.6.5 pattern does it repeat?
- What does it mean for a lock to be poisoned, and what does every `.lock()` after that return?

---

## Going further

- [The Rust Book — Fearless Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html) — the same subject, from the Rust team itself.
- [`std::thread` documentation](https://doc.rust-lang.org/std/thread/index.html) — the full module, including `Builder` for controlling a thread's name and stack size.
- [`std::sync::Mutex` documentation](https://doc.rust-lang.org/std/sync/struct.Mutex.html) — full detail on poisoning.
- [`std::sync::Arc` documentation](https://doc.rust-lang.org/std/sync/struct.Arc.html) — the same API, this time with an atomic count.
