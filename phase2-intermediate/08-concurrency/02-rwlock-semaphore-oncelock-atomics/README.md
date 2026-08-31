# 2.8.2 — `RwLock`, `Semaphore`, `OnceLock`/`LazyLock`, atomics

## At a glance

After this lesson you can:

- Explain exactly what `RwLock` splits apart that `Mutex` doesn't — and when that split actually changes a program's real throughput, not just its safety story.
- Build a working counting semaphore out of `Mutex` and `Condvar`, say why `std::sync` has no such type of its own, and explain why an RAII permit beats calling release by hand.
- Choose among `OnceLock`, `LazyLock`, and a plain atomic for a real problem — and explain why a `Mutex`/`RwLock` would have been overkill there.

**Time:** ~75 minutes · **Prerequisites:**
[2.8.1 — Threads, `Mutex`, `Arc`](../01-threads-mutex-arc/README.md)

---

## Why this matters

2.8.1 gave you one tool for making any data shared across threads safe: `Mutex<T>`. Its rule was simple — at any moment, at most one thread may touch the value, whether it only wants to look at it or wants to change it. That rule is always safe. It is not always the fastest thing you could have written.

Picture a cache or a piece of app configuration that dozens of concurrent threads — say, dozens of request handlers in a web server — read constantly, and that one separate thread updates every few minutes. With a `Mutex`, even when every one of those threads only wants to read — an operation that cannot conflict with another read at all — they still have to queue up one at a time, because `Mutex` draws no distinction between "I just want to look" and "I want to change this." A thousand simultaneous reads that could never have hurt each other get serialized anyway, by a lock built to protect writes.

This lesson fills exactly that gap with `RwLock<T>`: a lock that distinguishes "read-only" from "read-and-write," and lets any number of readers proceed at the same time. Three more tools follow, each solving a different concurrency problem: a counting semaphore for bounding how many threads may touch a limited resource at once (which you build yourself — `std::sync` has no such type); `OnceLock`/`LazyLock` for safe one-time initialization, even when several threads race to be first; and atomics for when a single small number is enough and reaching for a full lock would be overkill.

All four come from the same two toolkits 2.8.1 introduced — `std::sync` and real OS threads — they just ask a more precise version of "who may touch this, and when."

---

## The concept

### `RwLock<T>`: any number of readers, or exactly one writer

`RwLock<T>` ("read-write lock") does the same job `Mutex<T>` did — wrap a value so two threads can't touch it in an unruly way — with a more precise rule: **any number of simultaneous read borrows, or exactly one exclusive write borrow, never both at once.** That is exactly the aliasing rule from [1.3.1](../../../phase1-fundamentals/03-borrowing-and-references/01-shared-and-mutable-refs/README.md) — this time enforced at run time, across threads, instead of by the compiler within one function.

```rust
use std::sync::RwLock;

fn main() {
    let value = RwLock::new(10);

    let r1 = value.read().unwrap();
    let r2 = value.read().unwrap(); // a second reader, at the same time, no problem
    println!("two readers at once: {} and {}", *r1, *r2);
```

`.read()` returns an `RwLockReadGuard<T>` — just like `MutexGuard` from 2.8.1, a smart pointer that derefs to the inner value and releases the borrow when it goes out of scope. The difference from `MutexGuard` is that you can keep several of these alive at once: `r1` and `r2` above are two perfectly valid read borrows of the same value, exactly the way two ordinary `&T`s could be alive at the same time.

```rust
    let busy = value.try_write();
    println!("try_write while readers are alive: {}", busy.is_err());

    drop(r1);
    drop(r2);

    let mut w = value.write().unwrap();
    *w += 1;
    println!("after write: {w}");
}
```

`.try_write()` does what `.write()` does, but instead of blocking it returns `Err` immediately if the lock isn't available — used here to prove, without needing a second thread, that writing is impossible while `r1`/`r2` are alive. `.write()` (without `try_`) hands back an `RwLockWriteGuard<T>` that has both `Deref` and `DerefMut` — since this borrow is exclusive, it's also allowed to write.

```sh
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 01-rwlock-concurrent-reads
```

```text
two readers at once: 10 and 10
try_write while readers are alive: true
after write: 11
```

```senpai-visual
{"kind":"concurrency","labels":["reader A calls read()","reader B calls read() too","both readers proceed together","writer calls write(), must wait","both readers drop their guards","writer proceeds alone"]}
```

### Why this split actually changes throughput: a read-heavy cache

The classic example — and the one that makes this distinction actually pay off — is exactly the scenario "Why this matters" described: a shared cache or config that is read constantly and written rarely. Make several reader threads and one writer thread real:

```rust
use std::sync::{Arc, RwLock};
use std::thread;

fn main() {
    let cache = Arc::new(RwLock::new(vec![
        String::from("Frieren"),
        String::from("Bocchi the Rock!"),
    ]));

    let before: Vec<_> = (0..3)
        .map(|id| {
            let cache = Arc::clone(&cache);
            thread::spawn(move || (id, cache.read().unwrap().len()))
        })
        .collect();
```

The same `Arc` 2.8.1 taught you — because `RwLock<T>`, like `Mutex<T>`, still has exactly one owner, and every thread needs its own copy of that ownership.

```rust
    for handle in before {
        let (id, count) = handle.join().unwrap();
        println!("reader {id} saw {count} cached titles");
    }
```

All three readers are done. Now the writer:

```rust
    let writer_cache = Arc::clone(&cache);
    thread::spawn(move || {
        writer_cache
            .write()
            .unwrap()
            .push(String::from("Made in Abyss"));
    })
    .join()
    .unwrap();

    let after = cache.read().unwrap().len();
    println!("after the writer, the cache holds {after} titles");
}
```

```sh
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 02-rwlock-cache-readers-and-writer
```

```text
reader 0 saw 2 cached titles
reader 1 saw 2 cached titles
reader 2 saw 2 cached titles
after the writer, the cache holds 3 titles
```

If you rewrote this with `Mutex<Vec<String>>`, the code would stay nearly identical — `.lock()` everywhere instead of `.read()`/`.write()` — but its meaning would change: the three readers above, even though none of them ever intended to write, would have to queue up and take turns holding the lock, exactly as if they had wanted to write. With `RwLock`, all three can hold `.read()` at the same time; only once the writer arrives and wants `.write()` does anything actually queue. The more reading dominates writing — exactly the shape of a cache or app config — the more this difference pays off.

### Building one `std::sync` doesn't have: a counting semaphore

The next question has a different shape: not "who may write," but "how many threads may touch a limited resource at once." A connection pool with a fixed capacity, say — you want at most 2 threads using it at a time, no more, but any number of threads may queue up waiting for their turn. The classic tool for this is a **counting semaphore**.

**`std::sync::Semaphore` does not exist** — this type is not in Rust's standard library, and that's deliberate, not a gap you forgot to check. The good news: with one tool you already have from 2.8.1 — `Mutex` — plus one new one called `Condvar` ("condition variable"), you can build a small, correct semaphore yourself.

```rust
use std::sync::{Condvar, Mutex};

struct Semaphore {
    available: Mutex<usize>,
    changed: Condvar,
}
```

The idea is simple: `available` holds the number of free permits, behind an ordinary `Mutex`. `Condvar` does something a `Mutex` alone cannot: it lets a thread *wait* until some condition ("at least one permit is free") becomes true, without holding the lock for everyone else while it waits — and it lets another thread that just made that condition true wake it up explicitly.

```rust
struct Permit<'a> {
    semaphore: &'a Semaphore,
}

impl Drop for Permit<'_> {
    fn drop(&mut self) {
        let mut available = self.semaphore.available.lock().unwrap();
        *available += 1;
        self.semaphore.changed.notify_one();
    }
}
```

This return-value guard is built before `acquire` itself, for the same reason 2.8.1 showed you `MutexGuard`: if `acquire`/`release` were two separate, hand-called functions, any early return or panic that skipped `release()` would leak that permit forever — and every other thread waiting on it would wait forever too. Wrapping the permit in a type with [Drop and RAII](../../../phase1-fundamentals/02-ownership-and-memory/05-drop-and-raii/README.md) makes releasing it automatic and guaranteed — exactly the same trade `MutexGuard` already gave you.

```rust
impl Semaphore {
    fn new(permits: usize) -> Self {
        Semaphore {
            available: Mutex::new(permits),
            changed: Condvar::new(),
        }
    }
}
```

And now `acquire` itself:

```rust
impl Semaphore {
    fn acquire(&self) -> Permit<'_> {
        let mut available = self.available.lock().unwrap();
        available = self
            .changed
            .wait_while(available, |count| *count == 0)
            .unwrap();
        *available -= 1;
        Permit { semaphore: self }
    }
}
```

`.wait_while(guard, condition)` is exactly what a semaphore needs: it borrows the lock, and as long as `condition` on its contents returns `true` (here: "zero permits are free"), it releases the lock and sleeps; every time a `notify_one()`/`notify_all()` wakes it, it checks the condition again rather than blindly assuming it's now satisfied. (This matters: a lone `.wait()` plus an `if` instead of a `while` could accept a **spurious wakeup** and wrongly decrement a permit count of zero; `wait_while` writes that loop for you.) Once the condition is no longer true, `wait_while` hands back the `MutexGuard`, the same way a plain `.lock()` would have.

```senpai-visual
{"kind":"queue","labels":["acquire() — condition false, sleep","another thread's Permit drops","notify_one() wakes the sleeper","condition re-checked — now true","permit taken, acquire() returns"]}
```

```rust
let sem = Semaphore::new(2);
{
    let _a = sem.acquire();
    let _b = sem.acquire();
    println!(
        "acquired 2 permits, available: {}",
        *sem.available.lock().unwrap()
    );
} // both permits are released right here, as `_b` then `_a` drop

println!(
    "after scope ends, available: {}",
    *sem.available.lock().unwrap()
);
```

```sh
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 03-semaphore-permits-and-raii-release
```

```text
acquired 2 permits, available: 0
after scope ends, available: 2
```

Nothing here ever called `.release()` by hand — closing the block dropped both `Permit`s and they returned their permits themselves, exactly what two `MutexGuard`s do at the end of a block.

Now the real use case: bounding how many threads touch a limited resource at once — the connection-pool scenario above, made concrete. It reaches for `AtomicUsize` a little early — "Atomics" below is where that type actually gets explained — but here it's only ever verification scaffolding, not the point of the example, so a one-line preview is enough: it's a small integer several threads can safely add to and subtract from with no `Mutex` at all.

```rust
let semaphore = Arc::new(Semaphore::new(2));
let in_flight = Arc::new(AtomicUsize::new(0));

let handles: Vec<_> = (0..4)
    .map(|id| {
        let semaphore = Arc::clone(&semaphore);
        let in_flight = Arc::clone(&in_flight);
        thread::spawn(move || {
            let _permit = semaphore.acquire();
```

By this point `acquire()` has either returned immediately or waited for a permit to free up. Now that this thread is holding one:

```rust
            let now = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
            assert!(now <= 2, "more than 2 workers held the resource at once");
            thread::sleep(Duration::from_millis(20));
            in_flight.fetch_sub(1, Ordering::SeqCst);
            id
        })
    })
    .collect();
```

Four threads, but only 2 permits — so at least two of them are always waiting on `acquire()` for one of the first two to give its permit back (by dropping `_permit`). `in_flight` and that `assert!` exist purely to prove that rule; what "Hands on" actually prints is join order, not a racy count — that number could differ run to run, but the result of this `assert!` never does.

```sh
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 04-semaphore-bounds-worker-threads
```

```text
worker 0 acquired and released the resource
worker 1 acquired and released the resource
worker 2 acquired and released the resource
worker 3 acquired and released the resource
never more than 2 workers held the resource at the same time
```

That last line only prints if no `assert!` ever panicked — meaning the semaphore genuinely never let more than 2 threads hold a permit at once.

> The code above is entirely synchronous, running on real OS threads. If you were writing the same idea in **async** code — code written with `async fn` — you'd reach for `tokio::sync::Semaphore` instead of this hand-rolled one; [2.8.6](../06-tokio-basics/README.md) introduces the real async runtime. Until then, this `Condvar`-based version is the right tool.

### `OnceLock`: initialize exactly once, even under a race

The next problem: you have a value that's expensive to build — parsing a config file, building a lookup table — and several threads need to read it. You want it built exactly once, no matter how many threads race to be the first to want it.

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::thread;

static INIT_COUNT: AtomicUsize = AtomicUsize::new(0);
static CONFIG: OnceLock<String> = OnceLock::new();

fn config() -> &'static str {
    CONFIG.get_or_init(|| {
        INIT_COUNT.fetch_add(1, Ordering::SeqCst);
        String::from("max_connections=64")
    })
}
```

`OnceLock<T>::new()` builds an empty slot. `.get_or_init(closure)` has exactly one rule: if the slot is still empty, it runs `closure` and stores the result there; if it's already full, it never calls `closure` at all and just hands back what's already there. When several threads call `.get_or_init()` at the same time, `OnceLock` guarantees only one of them actually runs `closure` — the rest wait for that one to finish, then get the same result. `INIT_COUNT` here exists only to prove that promise; it's not part of the pattern itself.

```rust
fn main() {
    let handles: Vec<_> = (0..8).map(|_| thread::spawn(config)).collect();
    for handle in handles {
        handle.join().unwrap();
    }
    println!("config: {}", config());
    println!(
        "init closure ran {} time(s)",
        INIT_COUNT.load(Ordering::SeqCst)
    );
}
```

```sh
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 05-oncelock-init-once
```

```text
config: max_connections=64
init closure ran 1 time(s)
```

Eight threads called `config()` at once, and the building closure still ran exactly once. Run this repeatedly — that last number always stays 1, no matter what order those 8 threads actually happened to run in.

```senpai-visual
{"kind":"concept","labels":["8 threads call config()","the first one wins the race","it alone runs the init closure","the rest just wait briefly","OnceLock stores the result","every thread gets the same value"]}
```

### `LazyLock`: the same promise, spelled as a `static`

[1.1.1](../../../phase1-fundamentals/01-foundations/01-variables-mutability-shadowing/README.md) promised that once you reached shared state across threads, `static` would resurface. `LazyLock<T>` is exactly that moment: a `static` that builds its value only on first access, wherever in the program that access happens.

```rust
use std::sync::LazyLock;
use std::thread;

static SQUARES: LazyLock<Vec<u32>> = LazyLock::new(|| (0..10).map(|n| n * n).collect());

fn main() {
    let handles: Vec<_> = (0..4)
        .map(|id| thread::spawn(move || (id, SQUARES[id])))
        .collect();
    for handle in handles {
        let (id, value) = handle.join().unwrap();
        println!("SQUARES[{id}] = {value}");
    }
    println!("table has {} entries", SQUARES.len());
}
```

```sh
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 06-lazylock-static-lookup-table
```

```text
SQUARES[0] = 0
SQUARES[1] = 1
SQUARES[2] = 4
SQUARES[3] = 9
table has 10 entries
```

`SQUARES[id]` and `SQUARES.len()` work with no explicit `.get()` or `*` at all, because `LazyLock<T>` has a `Deref<Target = T>` — the same implicit reference conversion you know from [Deref and AsRef](../../04-lifetimes-and-conversion/03-deref-asref-borrow/README.md). The practical difference between `OnceLock` and `LazyLock` is exactly that: with `OnceLock`, every place you want the value, you explicitly call `.get_or_init(closure)` yourself; with `LazyLock`, the recipe is written once, in the `static`'s own definition, and from then on you use it like an ordinary `static` — the machinery underneath (who got there first, what everyone else did) is exactly what `OnceLock` has.

### Atomics: a single small value, no lock at all

Sometimes what's shared across threads is small enough — a counter, a true/false flag — that taking out a `Mutex` is pure overkill. `std::sync::atomic` has types like `AtomicUsize`, `AtomicBool`, and others: every operation on one (reading, writing, adding) happens as a single, indivisible hardware instruction — not a lock that does "take it, read, write, release," but an operation that is safe entirely on its own.

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    let counter = Arc::new(AtomicUsize::new(0));
```

Four threads, each hammering `fetch_add` a thousand times:

```rust
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let counter = Arc::clone(&counter);
            thread::spawn(move || {
                for _ in 0..1000 {
                    counter.fetch_add(1, Ordering::SeqCst);
                }
            })
        })
        .collect();
```

Join them all and read the final total:

```rust
    for handle in handles {
        handle.join().unwrap();
    }

    println!("final count: {}", counter.load(Ordering::SeqCst));
}
```

```sh
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 07-atomics-lock-free-counter
```

```text
final count: 4000
```

Four threads, each calling `fetch_add(1, ...)` 1000 times — and the final answer is always exactly 4000, every time you run it, no matter what real order those 4000 additions happened in across the four threads. `fetch_add` returns the value *before* the add; if you want the value *after* the add, you add one to it yourself — exactly what you'll want for `HitCounter::hit` in the exercises.

Three basic methods cover most of what you need: `.load(ordering)` reads the current value (as `counter.load(Ordering::SeqCst)` did above), `.store(value, ordering)` writes a new value unconditionally (no reading the old one, no return value), and `.fetch_add(amount, ordering)` does both at once, as one atomic operation.

Every method on an atomic type takes an `Ordering` — everywhere here, we wrote `Ordering::SeqCst` ("sequentially consistent"). The rule for this lesson is simple: **`SeqCst` is always the safe, easy-to-reason-about default.** Weaker orderings (`Relaxed`, `Acquire`, `Release`) exist, for advanced performance work this curriculum does not get into — Rust's full atomic memory model is a subject for a whole book on its own, not a subsection. Until you have a specific reason for something else, write `SeqCst`.

The second common pattern, when instead of adding you need to ask "only if it's still this value, change it," is `compare_exchange`:

```rust
use std::sync::atomic::{AtomicBool, Ordering};

fn main() {
    let started = AtomicBool::new(false);
    let first = started.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst);
    let second = started.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst);
    println!("first attempt:  {first:?}");
    println!("second attempt: {second:?}");
}
```

```text
first attempt:  Ok(false)
second attempt: Err(true)
```

`compare_exchange(current, new, success_order, failure_order)` asks: "if the current value is exactly `current`, make it `new` — and do that whole check-and-swap as one move, with no other thread able to sneak in between the check and the swap." If it really was `current`, it swaps and returns `Ok(the_previous_value)`. If it wasn't (someone else got there first), it changes nothing and returns `Err(the_actual_current_value)`. Above: the first `compare_exchange` moves the value from `false` to `true` and gets `Ok(false)`; the second, since the value is already `true` and not `false`, changes nothing and gets `Err(true)`. This is exactly a "one-shot flag": any number of threads can call this, but only one of them will ever get `Ok`.

```rust
let claimed = Arc::new(AtomicBool::new(false));
let winners = Arc::new(AtomicUsize::new(0));
let handles: Vec<_> = (0..6)
    .map(|_| {
        let claimed = Arc::clone(&claimed);
        let winners = Arc::clone(&winners);
        thread::spawn(move || {
```

Each thread tries the exact same `compare_exchange` from above:

```rust
            let won = claimed
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok();
            if won {
                winners.fetch_add(1, Ordering::SeqCst);
            }
        })
    })
    .collect();
```

```sh
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 08-atomics-compare-exchange-flag
```

```text
threads that won the race to initialize: 1
```

Six threads genuinely race on the same `AtomicBool`, and the winner count is still always exactly 1 — the same guarantee `compare_exchange` makes, this time with real threads instead of two calls back to back. This is the same problem `OnceLock` also solves; the difference here is that you're working with the raw flag directly, not a type that has hidden this loop for you.

```senpai-visual
{"kind":"concurrency","labels":["load the current value","compute value minus 1","compare_exchange(current, new)","someone else changed it: retry","no one changed it: success","value updated, resource claimed"]}
```

### Choosing among them

- **`Mutex`** — general exclusive access; when reads and writes are both common, or equally likely.
- **`RwLock`** — when reads vastly outnumber writes and you want readers to proceed together.
- **A semaphore (`Mutex` + hand-rolled `Condvar`)** — when you need to bound how many threads touch a limited resource at once.
- **`OnceLock` / `LazyLock`** — safe one-time setup; a value that's expensive to build but only ever read afterward.
- **Atomics** — a single small value (a counter, a flag) where a full lock would be overkill.

---

## Hands on

```sh
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 01-rwlock-concurrent-reads
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 02-rwlock-cache-readers-and-writer
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 03-semaphore-permits-and-raii-release
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 04-semaphore-bounds-worker-threads
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 05-oncelock-init-once
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 06-lazylock-static-lookup-table
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 07-atomics-lock-free-counter
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 08-atomics-compare-exchange-flag
```

Then the four broken ones:

```sh
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 09-rwlock-move-without-arc --features broken
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 10-write-through-read-guard --features broken
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 11-oncelock-set-twice-panics --features broken
cargo run -p p2-08-02-rwlock-semaphore-oncelock-atomics --example 12-atomics-no-direct-equality --features broken
```

Then try these:

1. In `04-semaphore-bounds-worker-threads`, change the semaphore's capacity from 2 to 1 and raise the thread count to 6. Does the program still finish correctly? Why does it take a bit longer to run?
2. In `07-atomics-lock-free-counter`, replace every `Ordering::SeqCst` with `Ordering::Relaxed`. Is the final count still 4000? (This deliberately goes past what this lesson teaches — it's just to see that the compiler won't stop you.)
3. In `05-oncelock-init-once`, move `INIT_COUNT.fetch_add` from before the `String::from(...)` line to after it. Does the final answer change? Should it?

---

## Errors you will meet

### `E0382` — `RwLock` needs the same `Arc` treatment `Mutex` did

```text
error[E0382]: use of moved value: `cache`
  --> phase2-intermediate\08-concurrency\02-rwlock-semaphore-oncelock-atomics\examples\09-rwlock-move-without-arc.rs:14:23
   |
12 |     let cache = RwLock::new(String::from("v1"));
   |         ----- move occurs because `cache` has type `std::sync::RwLock<String>`, which does not implement the `Copy` trait
13 |     for _ in 0..3 {
   |     ------------- inside of this loop
14 |         thread::spawn(move || {
   |                       ^^^^^^^ value moved into closure here, in previous iteration of loop
15 |             let guard = cache.read().unwrap();
   |                         ----- use occurs due to use in closure

For more information about this error, try `rustc --explain E0382`.
error: could not compile `p2-08-02-rwlock-semaphore-oncelock-atomics` (example "09-rwlock-move-without-arc") due to 1 previous error
```

**What the compiler is actually objecting to:** `move` hands the closure full ownership of `cache` every time the loop body runs. The first iteration did exactly that and took ownership; by the time the second iteration runs, `cache` has nothing left to give — exactly what the message says: "value moved into closure here, in previous iteration of loop."

**The fix:** exactly what 2.8.1 taught you to do with `Mutex` — wrap it in an `Arc` and `move` a clone of that `Arc` into each thread, not the `RwLock` itself:

```rust
let cache = Arc::new(RwLock::new(String::from("v1")));
for _ in 0..3 {
    let cache = Arc::clone(&cache);
    thread::spawn(move || {
        let guard = cache.read().unwrap();
        println!("{guard}");
    });
}
```

**Why this is the fix:** `Arc::clone` doesn't copy the heap value — it just bumps an owner count and hands back a fresh handle to the same `RwLock`. Now every iteration still `move`s something, but what gets moved is a fresh `Arc`, not the original `cache` — so the original ownership never runs out.

### `E0594` — you can't write through a read guard

```text
error[E0594]: cannot assign to data in dereference of `std::sync::RwLockReadGuard<'_, i32>`
  --> phase2-intermediate\08-concurrency\02-rwlock-semaphore-oncelock-atomics\examples\10-write-through-read-guard.rs:12:5
   |
12 |     *guard += 1;
   |     ^^^^^^^^^^^ cannot assign
   |
   = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `std::sync::RwLockReadGuard<'_, i32>`

For more information about this error, try `rustc --explain E0594`.
error: could not compile `p2-08-02-rwlock-semaphore-oncelock-atomics` (example "10-write-through-read-guard") due to 1 previous error
```

**What the compiler is actually objecting to:** `counter.read()` hands back an `RwLockReadGuard<i32>` — a guard with only `Deref`, no `DerefMut`, for the exact same reason `RefCell`'s `Ref<T>` (from [2.6.5](../../06-smart-pointers/05-refcell-and-interior-mutability/README.md)) only had `Deref`: this guard promised "read-only," and the compiler holds it to that promise.

**The fix:** ask for `.write()`, not `.read()`, when you intend to write:

```rust
let counter = RwLock::new(0);
let mut guard = counter.write().unwrap();
*guard += 1;
println!("{guard}");
```

**Why this is the fix:** `.write()` hands back an `RwLockWriteGuard<i32>` with both `Deref` and `DerefMut` — because its promise is "only me, exclusively," not "anyone who wants to look." If another read borrow had been alive at that moment, `.write()` would have waited for it to end; but when you genuinely mean to write, this is the right tool, not trying to write through a read guard.

### Run-time panic — `OnceLock`'s second `.set()` is always `Err`

```text
thread 'main' (22608) panicked at phase2-intermediate\08-concurrency\02-rwlock-semaphore-oncelock-atomics\examples\11-oncelock-set-twice-panics.rs:12:40:
called `Result::unwrap()` on an `Err` value: "second"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**What the compiler is actually objecting to:** this isn't even a compiler error — the program built and ran. `OnceLock::set(value)` returns `Result<(), T>`: if the slot was empty, it fills it and returns `Ok(())`; if it was already full (exactly the case here, since the line before just filled it), it changes nothing and returns `Err(the_value_you_passed)`. `.unwrap()` on that `Err("second")` panics.

**The fix:** if one-time initialization is what you actually want, reach for `.get_or_init()`, not `.set()`:

```rust
let config: OnceLock<String> = OnceLock::new();
config.get_or_init(|| String::from("first"));
config.get_or_init(|| String::from("second")); // ignored, does not panic
println!("{:?}", config.get());
```

**Why this is the fix:** `.set()` is for when you're certain this is the first time and want to see it fail if you're wrong (say, during startup). `.get_or_init()` is the more common shape — "the first time anything needs this, build it; every time after, hand back the same one" — and it never panics on a harmless second call.

### `E0369` — you can't compare two atomics directly

```text
error[E0369]: binary operation `==` cannot be applied to type `Atomic<usize>`
   --> phase2-intermediate\08-concurrency\02-rwlock-semaphore-oncelock-atomics\examples\12-atomics-no-direct-equality.rs:13:10
    |
 13 |     if a == b {
    |        - ^^ - Atomic<usize>
    |        |
    |        Atomic<usize>
    |
note: `Atomic<usize>` does not implement `PartialEq`
   --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\sync\atomic.rs:366:1
    |
366 | pub struct Atomic<T: AtomicPrimitive> {
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `Atomic<usize>` is defined in another crate

For more information about this error, try `rustc --explain E0369`.
error: could not compile `p2-08-02-rwlock-semaphore-oncelock-atomics` (example "12-atomics-no-direct-equality") due to 1 previous error
```

(On this toolchain, the message names `AtomicUsize` by its real underlying name — `Atomic<usize>` — because the standard library now defines `AtomicUsize` as an alias for that same generic type; the `AtomicUsize` you actually wrote is the very same thing, the compiler is just answering with its real definition.)

**What the compiler is actually objecting to:** atomic types deliberately don't implement `PartialEq`. If they did, `a == b` would silently perform two separate `.load()`s — one for `a`, one for `b` — with any other thread free to change either value in between, so your comparison would be measuring two things that were never actually observed at the same moment.

**The fix:** `.load()` explicitly yourself, then compare the plain numbers:

```rust
let a = AtomicUsize::new(1);
let b = AtomicUsize::new(1);
if a.load(Ordering::SeqCst) == b.load(Ordering::SeqCst) {
    println!("equal");
}
```

**Why this is the fix:** now it's explicit that two separate reads happened at two separate moments — exactly what was always true — instead of letting `==` hide that behind familiar-looking syntax.

---

## Exercises

### Warm up

<details>
<summary>What does this print?</summary>

```rust
use std::sync::RwLock;

let v = RwLock::new(5);
let a = v.read().unwrap();
let b = v.read().unwrap();
println!("{}", *a + *b);
```

</details>

<details>
<summary>Answer</summary>

```text
10
```

`RwLock` allows any number of simultaneous read borrows; `a` and `b` are two perfectly valid borrows of the same value, both 5.

</details>

<details>
<summary>Does this compile?</summary>

```rust
let counter = RwLock::new(0);
let guard = counter.read().unwrap();
*guard += 1;
```

</details>

<details>
<summary>Answer</summary>

No. `.read()` hands back an `RwLockReadGuard`, which has `Deref` but not `DerefMut` — writing through it is exactly the `E0594` you saw in "Errors you will meet." Writing requires asking for `.write()` instead.

</details>

<details>
<summary>How many permits are free in the semaphore after this block?</summary>

```rust
let sem = Semaphore::new(2);
{
    let _a = sem.acquire();
    let _b = sem.acquire();
}
```

</details>

<details>
<summary>Answer</summary>

```text
2
```

Both `Permit`s drop when the block closes — `_b` first, then `_a` — and each `Drop` returns its permit. The semaphore is back to its starting capacity.

</details>

<details>
<summary>How many times does "computing..." print?</summary>

```rust
let cache: OnceLock<u32> = OnceLock::new();
cache.get_or_init(|| {
    println!("computing...");
    42
});
cache.get_or_init(|| {
    println!("computing...");
    42
});
println!("{:?}", cache.get());
```

</details>

<details>
<summary>Answer</summary>

```text
computing...
Some(42)
```

Only once. The second `.get_or_init()` call sees the slot is already full and never runs its closure at all.

</details>

<details>
<summary>What's the final value?</summary>

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

let counter = AtomicUsize::new(0);
counter.fetch_add(5, Ordering::SeqCst);
counter.fetch_add(3, Ordering::SeqCst);
println!("{}", counter.load(Ordering::SeqCst));
```

</details>

<details>
<summary>Answer</summary>

```text
8
```

Each `fetch_add` adds the given amount to the current value: 0 + 5 = 5, then 5 + 3 = 8.

</details>

### Repair

Fix all four broken examples:

1. Fix `examples/09-rwlock-move-without-arc.rs` with an `Arc<RwLock<String>>` — exactly the pattern 2.8.1 taught you with `Mutex`.
2. Fix `examples/10-write-through-read-guard.rs` by changing `.read()` to `.write()`.
3. Fix `examples/11-oncelock-set-twice-panics.rs` so it no longer panics — without removing the second call.
4. Fix `examples/12-atomics-no-direct-equality.rs` by `.load()`ing both sides before `==`.

### Implement

Three types in `src/lib.rs`:

```sh
cargo test -p p2-08-02-rwlock-semaphore-oncelock-atomics
```

`LazyGreeting` holds an `OnceLock<String>` as a field — not a `static`, but one per instance of `LazyGreeting`. `HitCounter` is just one `AtomicUsize`. `ResourcePool` implements the same bounding idea a semaphore does with a `compare_exchange` loop instead — no `Condvar`, no waiting; if nothing is free, it returns `false` immediately. Implement each method exactly from its doc comment above it.

### Build

Design a small cache of your own — a phrase-translation lookup, a leaderboard snapshot, a settings blob, whatever you like — backed by an `RwLock<T>`. Give it a "read" method callable from several threads at once and a "write" method that changes the value. Spawn at least 3 reader threads and 1 writer thread to prove it both compiles and behaves the way you expect.

### Challenge (optional)

**Part one.** Add a `try_acquire(&self) -> Option<Permit<'_>>` method to the `Semaphore` you built in "The concept" — one that never waits: if a permit is free, do what `acquire` does and return `Some(permit)`; if not, return `None` without touching anything. (Hint: you need a plain `Mutex::lock()`, not `wait_while`.)

**Part two.** (This one looks ahead.) In this lesson, threads coordinated by sharing *access* to a value — an `RwLock`, a semaphore. [2.8.3](../03-channels-message-passing/README.md) shows a completely different model: instead of sharing access, threads send *owned values* to each other. For a job queue where several worker threads each grab one task at a time, guess — would you reach for a semaphore, or that new model? Write down your reasoning, then check it against 2.8.3.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `RwLock<T>` | any number of simultaneous read borrows, or exactly one exclusive write borrow | a read-heavy cache, rarely-changed config |
| `RwLockReadGuard` / `RwLockWriteGuard` | the guards `.read()`/`.write()` return | only the second has `DerefMut` |
| `Condvar` | "wait until some condition is true, without holding the lock while you wait" | underneath any wait-based tool, like a semaphore |
| counting semaphore | bounding how many threads may touch a resource at once | `std::sync` doesn't have one; build it from `Mutex`+`Condvar` |
| `OnceLock<T>` | `.get_or_init()`: the closure runs exactly once, even under a race | an expensive value read by several threads |
| `LazyLock<T>` | the same as `OnceLock`, spelled as a declarative `static` | a lookup table or computed global value |
| atomic type (`AtomicUsize`, ...) | read/write/add as one operation, no lock | a single small shared value |
| `compare_exchange` | "if it's still this value, change it — all in one move" | lock-free counters, a one-shot flag |
| `Ordering::SeqCst` | the safe default memory ordering | always, unless you have a specific reason for something else |

### What you now know

- `RwLock<T>` allows any number of simultaneous read borrows, but writing is still exclusive — and that split genuinely changes the throughput of a read-heavy workload, not just how it looks on paper.
- `std::sync::Semaphore` doesn't exist; a real counting semaphore can be built from a `Mutex<usize>` and a `Condvar`, and an RAII permit gives the same automatic-release guarantee `MutexGuard` did.
- `OnceLock::get_or_init` guarantees its building closure runs exactly once, no matter how many threads race for it; `LazyLock` gives the same guarantee spelled as a `static`.
- Atomics make a single small value safe with no lock at all; `fetch_add` is for lock-free counters, `compare_exchange` is for "only if it's still this value, change it."
- `Ordering::SeqCst` is always the safe default; the weaker orderings are for advanced performance work this curriculum doesn't get into.
- You now have five tools — `Mutex`, `RwLock`, a semaphore, `OnceLock`/`LazyLock`, atomics — and a rule for choosing among them, not just a list of names.

### What comes back later

- **Why some types can't cross a thread boundary at all, and what `Send`/`Sync` actually guarantee** — [2.8.4 — `Send` and `Sync`](../04-send-and-sync/README.md)
- **Sending owned values between threads, instead of sharing access** — [2.8.3 — Channels and message passing](../03-channels-message-passing/README.md)
- **Working with these same shapes of problem once code is async instead of thread-based** — [Module 2.9 — Async in practice](../../09-async-in-practice/README.md)

### Can you explain?

- Why can `RwLock` allow several simultaneous readers, when `Mutex` never can?
- Why doesn't `std::sync::Semaphore` exist, and what two pieces is your own semaphore built from?
- What does an RAII permit guarantee that a manual `release()` wouldn't?
- What's the practical difference between `OnceLock` and `LazyLock`?
- Why can't you compare two atomic values directly with `==`?
- If you have a single small shared value, how do you know it's time to drop `Mutex` and reach for an atomic instead?

---

## Going further

- [The Rust book — `RwLock<T>`](https://doc.rust-lang.org/std/sync/struct.RwLock.html) — full docs, including `try_read`/`try_write`.
- [`std::sync::Condvar`](https://doc.rust-lang.org/std/sync/struct.Condvar.html) — `wait`, `wait_while`, `notify_one`, `notify_all`.
- [`std::sync::OnceLock`](https://doc.rust-lang.org/std/sync/struct.OnceLock.html) and [`std::sync::LazyLock`](https://doc.rust-lang.org/std/sync/struct.LazyLock.html) — full docs for both types.
- [`std::sync::atomic` — module overview](https://doc.rust-lang.org/std/sync/atomic/index.html) — the full list of types and methods.
- [The Rustonomicon — atomics](https://doc.rust-lang.org/nomicon/atomics.html) — for when you're curious what `Relaxed`/`Acquire`/`Release` actually mean; it agrees that `SeqCst` is the safe default when you're unsure.
