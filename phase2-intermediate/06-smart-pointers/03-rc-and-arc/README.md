# 2.6.3 — `Rc` and `Arc`

## At a glance

After this lesson you can:

- Explain why `Rc<T>` lets a heap value have more than one real, simultaneous owner — something Phase 1 never allowed at all — and say exactly when that value is actually freed.
- Show the real difference between `.clone()` on an `Rc<T>` and `.clone()` on the data it wraps, with concrete proof — not just a claim.
- Choose between `Rc<T>` and `Arc<T>` for a real piece of code, and say in one sentence why `Rc` is never allowed to cross a thread boundary at all.

**Time:** ~60 minutes · **Prerequisites:**
[2.6.1 — `Box` and heap allocation](../01-box-and-heap-allocation/README.md),
[2.6.2 — Recursive types and boxed trait objects](../02-recursive-types-and-trait-objects/README.md)

---

## Why this matters

2.3.7 ended with a promise, not a plain wrap-up: "`Rc`/`Arc` for shared
ownership — the next step once a single owner (even a `Box<dyn Trait>` one)
isn't enough." That lesson built a `Vec<Box<dyn Summarize>>` holding two
genuinely different structs side by side — but every `Box`, however smart,
still had exactly one owner: the `Vec` itself. Today that promise gets paid
in full — a heap value that genuinely has more than one owner, not as a
metaphor, as a fully measurable fact.

Phase 1 repeated one rule constantly: every value has exactly one owner,
always. [2.6.1](../01-box-and-heap-allocation/README.md) gave you `Box<T>` —
the simplest smart pointer — and left that rule untouched; `Box` only added
a layer of indirection, the owner was still exactly one.
[2.6.2](../02-recursive-types-and-trait-objects/README.md) pushed one step
further, into recursive types, and the rule still held: every tree node had
exactly one owner, just this time that owner was itself sitting behind a
`Box`.

Now consider this: an application's configuration — an `AppConfig` — loaded
exactly once, when the program starts up; and then three completely
independent parts of the program need it: a request handler, a background
job, and a logger. Which one "owns" this config? The honest answer: none of
them, alone. Hand the config to the handler, and there's nothing left for
the logger — exactly the familiar Phase 1 error for moving a value twice.
`Box` doesn't help either; `Box` only changes *where* ownership lives, never
*how many* owners there can be.

This is where the lesson introduces a genuine exception, not a smaller
version of ownership. `Rc<T>` gives you **shared ownership**: it lets more
than one variable be a real, simultaneous owner of the same heap value at
once — no `unsafe`, no working around the rule, just the rule itself
changing. Instead of "zero or one owner," Rust now counts "how many live
owners exist right now," and only frees the value once that count reaches
zero.

---

## The concept

### `Rc<T>`: real, simultaneous owners

```rust
use std::rc::Rc;

struct AppConfig {
    app_name: String,
    max_connections: u32,
}

let config = Rc::new(AppConfig {
    app_name: "senpai-api".to_string(),
    max_connections: 100,
});
let for_handler = Rc::clone(&config);
let for_logger = Rc::clone(&config);
```

```rust
println!("handler sees:           {}", for_handler.app_name);
println!("logger sees:            {}", for_logger.app_name);
println!("original still usable:  {}", config.app_name);
println!("owners right now:       {}", Rc::strong_count(&config));
println!("max_connections too:    {}", for_handler.max_connections);
```

```text
handler sees:           senpai-api
logger sees:            senpai-api
original still usable:  senpai-api
owners right now:       3
max_connections too:    100
```

Three variables — `config`, `for_handler`, `for_logger` — are all, at the
same time, real owners of the same one `AppConfig` on the heap. None of them
"moved"; try this with `Box<AppConfig>` instead and that second `let` would
pull the value away from the first, and the compiler would answer with
`E0382` — exactly what Phase 1 taught you. Here it's the opposite:
`Rc::clone(&config)` produces a **new** owner without invalidating the one
that already existed. And none of these three is any more "original" than
the others — from `Rc`'s point of view, `config`, `for_handler`, and
`for_logger` are exactly equal.

```senpai-visual
{"kind":"ownership","labels":["AppConfig on heap","Rc: config","Rc: for_handler","Rc: for_logger"]}
```

### The count: exactly when the value is actually freed

Every `Rc<T>` keeps a **reference count** alongside its data — a number
saying how many handles are alive right now for this heap allocation. Every
`Rc::clone` bumps it up by one; every drop brings it down by one. To watch
exactly when the value is truly freed, give it a `Drop` impl — the same
thing [1.2.5](../../../phase1-fundamentals/02-ownership-and-memory/05-drop-and-raii/README.md)
taught you: `Drop` runs at the exact moment its owner's scope ends.

```rust
struct AppConfig {
    app_name: String,
}

impl Drop for AppConfig {
    fn drop(&mut self) {
        println!("  {} actually freed now", self.app_name);
    }
}

let a = Rc::new(AppConfig {
    app_name: "senpai-api".to_string(),
});
```

```rust
println!("after Rc::new:      count = {}", Rc::strong_count(&a));
let b = Rc::clone(&a);
println!("after first clone:  count = {}", Rc::strong_count(&a));
let c = Rc::clone(&a);
println!("after second clone: count = {}", Rc::strong_count(&a));
drop(b);
println!("after dropping one: count = {}", Rc::strong_count(&a));
drop(c);
println!("after dropping two: count = {}", Rc::strong_count(&a));
println!("dropping the last owner:");
drop(a);
```

```text
after Rc::new:      count = 1
after first clone:  count = 2
after second clone: count = 3
after dropping one: count = 2
after dropping two: count = 1
dropping the last owner:
  senpai-api actually freed now
```

The `actually freed now` message prints exactly once, and exactly when the
count hits zero — not sooner, when `b` or `c` went, because neither of them
was the last handle. No owner is "special"; what actually decides when the
data is freed is only the count itself.

```senpai-visual
{"kind":"concept","labels":["count = 1","count = 2","count = 3","count = 2","count = 1","freed at 0"]}
```

### `.clone()` on `Rc` is cheap — not a data copy

There's a real trap here: `some_rc.clone()` and `some_string.clone()` read
identically but do opposite things. Instead of just asserting that, prove
it — by comparing addresses:

```rust
#[derive(Clone)]
struct AppConfig {
    app_name: String,
}

let original = Rc::new(AppConfig {
    app_name: "senpai-api".to_string(),
});
let rc_clone = Rc::clone(&original);
let data_clone: AppConfig = (*original).clone();
```

```rust
println!(
    "Rc::as_ptr(&original) == Rc::as_ptr(&rc_clone):             {}",
    Rc::as_ptr(&original) == Rc::as_ptr(&rc_clone)
);
println!(
    "original.app_name.as_ptr() == data_clone.app_name.as_ptr(): {}",
    original.app_name.as_ptr() == data_clone.app_name.as_ptr()
);
```

```text
Rc::as_ptr(&original) == Rc::as_ptr(&rc_clone):             true
original.app_name.as_ptr() == data_clone.app_name.as_ptr(): false
```

`Rc::as_ptr` returns the address of the exact `AppConfig` on the heap that
an `Rc` points at. For `original` and `rc_clone`, that address is **the
same** — `Rc::clone` made no new allocation at all, it only bumped the
count and handed back a second, cheap pointer to the same spot. But
`(*original).clone()` genuinely copied the `AppConfig` — a fresh `String`
was built for `app_name`, with its own bytes, on completely separate
memory — which is exactly why you see `false`. This is why the convention
in real code is to write `Rc::clone(&x)` instead of `x.clone()`: right at
the call site it says "this is only a cheap pointer clone," without making
the reader go check what type `x` even is first. And for the exact
`Rc::as_ptr(&a) == Rc::as_ptr(&b)` check above, there is a built-in
shorthand that says the same thing more directly: `Rc::ptr_eq(&a, &b)`.

### Why `Rc` is single-threaded only

The count inside an `Rc<T>` is a completely ordinary integer — nothing
special. "Add one" on an ordinary integer, at the processor level, is not
one indivisible operation: you have to read the number first, add one, then
write the result back. If two threads did exactly those three steps "at the
same time" on the same count, both might read the same old value, both
write back the same old-plus-one, and one increment is simply lost. The
result is a data race: the count reaches zero too early, the value gets
freed, while a handle to it is still alive somewhere — exactly the class of
bug the aliasing rule exists to rule out.

Rust does not leave this to run-time luck; the compiler stops it today:
`Rc<T>` does not implement the `Send` marker trait, so any attempt to send
it to another thread fails right at compile time — not months later, in
production. The exact mechanics of crossing a thread boundary, and `Send`
itself, belong to module 8; today all you need is this fact: `Rc`
deliberately stays inside one thread.

### `Arc<T>`: same API, an atomic count

`Arc<T>` ("**a**tomically **r**eference **c**ounted") solves the exact same
problem — with a count that goes up and down through **atomic** CPU
instructions instead of a plain increment. An atomic operation, unlike the
plain increment above, is genuinely indivisible even when several cores
touch the same memory at once — which is exactly what makes `Arc<T>` safe
to share across threads. That safety isn't free: an atomic operation has to
coordinate with the other cores, so it's measurably slower than `Rc`'s
plain increment. The API, though, is identical letter for letter:

```rust
use std::sync::Arc;

struct AppConfig {
    app_name: String,
    max_connections: u32,
}

let config = Arc::new(AppConfig {
    app_name: "senpai-api".to_string(),
    max_connections: 100,
});
let for_handler = Arc::clone(&config);
```

```rust
println!("handler sees:     {}", for_handler.app_name);
println!("owners right now: {}", Arc::strong_count(&config));
println!("max_connections:  {}", config.max_connections);
```

```text
handler sees:     senpai-api
owners right now: 2
max_connections:  100
```

Letter for letter the same code as `Rc` above, just `Arc` in place of `Rc`.
Nothing here actually crosses a thread boundary — that part belongs to
module 8. The rule of thumb is already clear: default to `Rc` for
single-threaded code; the moment a value genuinely has to reach another
thread, reach for `Arc`.

### When you actually need shared ownership — and when you don't

The default is still what Phase 1 taught you: single ownership and
ordinary borrowing (`&T`/`&mut T`). It's simpler, fully checked at compile
time, and costs nothing at run time. `Rc`/`Arc` are for that genuine
exception, not a habit — two real shapes you'll actually run into:

- **Data loaded once that several independent parts need** — the same
  `AppConfig` above, shared by a handler, a background job, and a logger.
- **A tree or graph node with more than one parent** — say, a genre tag
  that two different anime entries both reference, without copying the
  genre's text twice.

| Type | Owners | Count | For |
|---|---|---|---|
| `Box<T>` | exactly one | none | a value that must live on the heap; still exactly one owner |
| `Rc<T>` | several, at once | ordinary, non-atomic | the same value, genuinely several owners, all in one thread |
| `Arc<T>` | several, at once | atomic | the same thing as `Rc`, when owners might be in different threads |

One important limitation, stated honestly: `Rc`/`Arc` alone only allow
**sharing**, not mutation. Every owner you get from an `Rc<T>` only gets
`&T` — because letting more than one simultaneous owner have `&mut T` would
break exactly the aliasing rule Phase 1 taught you. If you genuinely need to
mutate the shared value too, you need a different tool —
[2.6.5](../05-refcell-and-interior-mutability/README.md) solves exactly
that. And shared ownership itself has a real failure mode — when two
values each end up owning an `Rc` pointing at the other — which
[2.6.4](../04-weak-and-reference-cycles/README.md) covers in full.

---

## Hands on

```sh
cargo run -p p2-06-03-rc-and-arc --example 01-multiple-owners
cargo run -p p2-06-03-rc-and-arc --example 02-count-and-drop-order
cargo run -p p2-06-03-rc-and-arc --example 03-clone-cost-comparison
cargo run -p p2-06-03-rc-and-arc --example 04-arc-same-api
```

Then the three broken ones:

```sh
cargo run -p p2-06-03-rc-and-arc --example 05-forgot-to-clone-broken --features broken
cargo run -p p2-06-03-rc-and-arc --example 06-cannot-mutate-through-rc-broken --features broken
cargo run -p p2-06-03-rc-and-arc --example 07-rc-arc-mismatch-broken --features broken
```

Then try these:

1. In `01-multiple-owners.rs`, add a third owner (say, `for_background_job`)
   and print `Rc::strong_count` again — what does it say now?
2. In `02-count-and-drop-order.rs`, swap the order of `drop(b)` and
   `drop(c)` — does `actually freed now` still print exactly once, and only
   at the end?
3. In `04-arc-same-api.rs`, take a second `Arc::clone` and print
   `Arc::strong_count` again — does it behave exactly like the `Rc` version
   above?

---

## Errors you will meet

### `E0382` — moving the `Rc` itself, not its data

```text
error[E0382]: the type `Rc` does not implement `Copy`
  --> phase2-intermediate\06-smart-pointers\03-rc-and-arc\examples\05-forgot-to-clone-broken.rs:29:17
   |
24 |     let config = Rc::new(AppConfig {
   |         ------ this move could be avoided by cloning the original `Rc`, which is inexpensive
...
28 |     announce(config);
   |              ------ value moved here
29 |     log_startup(config);
   |                 ^^^^^^ value used here after move
   |
   = note: consider using `Rc::clone`
note: consider changing this parameter type in function `announce` to borrow instead if owning the value isn't necessary
  --> phase2-intermediate\06-smart-pointers\03-rc-and-arc\examples\05-forgot-to-clone-broken.rs:15:21
   |
15 | fn announce(config: Rc<AppConfig>) {
   |    --------         ^^^^^^^^^^^^^ this parameter takes ownership of the value
   |    |
   |    in this function
help: clone the value to increment its reference count
   |
28 |     announce(config.clone());
   |                    ++++++++

For more information about this error, try `rustc --explain E0382`.
```

**What the compiler is objecting to:** an `Rc<AppConfig>` is still a
perfectly ordinary owned value. Handing that same `config` to two functions
that each take ownership moves it away the first time, exactly like any
other value that isn't `Copy` — `Rc` is not an exception, only what it
points at is cheap.

**The fix:** instead of handing over `config` itself, make a fresh clone for
each call:

```rust
announce(Rc::clone(&config));
log_startup(Rc::clone(&config));
```

**Why this is the fix:** the compiler says it outright — "this move could be
avoided by cloning the original `Rc`, which is inexpensive." `Rc::clone`
only bumps the count and hands back a fresh handle; now both functions have
their own owner, and `config` is still valid after both calls too.

### `E0596` — cannot mutate through a shared `Rc`

```text
error[E0596]: cannot borrow data in an `Rc` as mutable
  --> phase2-intermediate\06-smart-pointers\03-rc-and-arc\examples\06-cannot-mutate-through-rc-broken.rs:26:5
   |
26 |     config.raise_limit(50);
   |     ^^^^^^ cannot borrow as mutable
   |
   = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Rc<AppConfig>`

For more information about this error, try `rustc --explain E0596`.
```

**What the compiler is objecting to:** `Rc<T>` implements `Deref` — the
thing that lets `config.app_name` or `config.raise_limit(...)` work without
manual dereferencing — but not `DerefMut`. There is no way to reach `&mut
T` through an `Rc`, even when the count happens to be exactly `1`, because
the compiler has to decide right now, without knowing the count at run
time, whether this code is safe.

**The fix:** with only this lesson's own tools, there is no correct fix —
and that's the point. A value that needs to be mutated through a shared
reference needs a different type, not a harder attempt at dereferencing an
`Rc`.

**Why this "fix" is right:** because this limit isn't accidental. If
`Rc<T>` allowed `&mut T`, two owners could get two simultaneous `&mut`s to
the same data — exactly the aliasing rule Phase 1 said must never break.
`Rc`/`Arc` deliberately only ever hand out `&T`; the tool that works around
this with a run-time rule instead of a compile-time one belongs to
[2.6.5](../05-refcell-and-interior-mutability/README.md).

### `E0308` — `Rc<T>` and `Arc<T>` are not interchangeable

```text
error[E0308]: mismatched types
  --> phase2-intermediate\06-smart-pointers\03-rc-and-arc\examples\07-rc-arc-mismatch-broken.rs:25:18
   |
25 |     spawn_worker(config);
   |     ------------ ^^^^^^ expected `Arc<AppConfig>`, found `Rc<AppConfig>`
   |     |
   |     arguments to this function are incorrect
   |
   = note: expected struct `Arc<AppConfig>`
              found struct `Rc<AppConfig>`
note: function defined here
  --> phase2-intermediate\06-smart-pointers\03-rc-and-arc\examples\07-rc-arc-mismatch-broken.rs:16:4
   |
16 | fn spawn_worker(config: Arc<AppConfig>) {
   |    ^^^^^^^^^^^^ ----------------------

For more information about this error, try `rustc --explain E0308`.
```

**What the compiler is objecting to:** `Rc<AppConfig>` and `Arc<AppConfig>`
are two completely separate, unrelated structs that merely happen to share
method names. Having an identical API — `::new`, `::clone`,
`::strong_count` — doesn't make them one type, and Rust performs no
implicit conversion between them.

**The fix:** if a value is going to end up somewhere that wants `Arc`,
build it with `Arc::new` from the start:

```rust
let config = Arc::new(AppConfig {
    app_name: "senpai-api".to_string(),
});
spawn_worker(config);
```

**Why this is the fix:** there is no cheap way to go from `Rc` to `Arc` or
back — converting means building a genuinely new allocation with the right
kind of counter. The simplest approach is to decide up front whether this
value is going to stay single-threaded, and pick its type from there.

---

## Exercises

### Warm up

<details>
<summary>After this code, what does <code>Rc::strong_count(&a)</code> return?</summary>

```rust
let a = Rc::new(5);
let b = Rc::clone(&a);
let c = Rc::clone(&a);
drop(b);
```

</details>

<details>
<summary>Answer</summary>

```text
2
```

`a` and `c` are still alive; only `b` is gone. The count goes up by one on
every `Rc::clone` and down by one on every `drop` — never more, never less.

</details>

<details>
<summary>Does this compile?</summary>

```rust
struct AppConfig { app_name: String }
fn announce(config: Rc<AppConfig>) {}
fn log_startup(config: Rc<AppConfig>) {}

let config = Rc::new(AppConfig { app_name: "x".to_string() });
announce(config);
log_startup(config);
```

</details>

<details>
<summary>Answer</summary>

No — `E0382`. `config` is a perfectly ordinary owned value; handing it to
`announce` moves it away, and there's nothing left for `log_startup`. You'd
need `Rc::clone(&config)` for each one.

</details>

<details>
<summary>Does this compile?</summary>

```rust
struct AppConfig { max_connections: u32 }
impl AppConfig {
    fn raise_limit(&mut self, by: u32) { self.max_connections += by; }
}

let config = Rc::new(AppConfig { max_connections: 100 });
config.raise_limit(50);
```

</details>

<details>
<summary>Answer</summary>

No — `E0596`. `Rc<T>` only has `Deref`, not `DerefMut`; you can never reach
`&mut T` through it, even when the count is `1`.

</details>

<details>
<summary>True or false: <code>some_rc.clone()</code> on an <code>Rc&lt;String&gt;</code> copies that <code>String</code>'s bytes.</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

False. `Rc<T>: Clone` implements `Clone` on `Rc` itself, so
`some_rc.clone()` always goes to that implementation — it only bumps the
count, never touches the data inside.

</details>

### Repair

Fix all three broken examples:

1. Fix `examples/05-forgot-to-clone-broken.rs` by cloning the `Rc` for each
   call.
2. Fix `examples/06-cannot-mutate-through-rc-broken.rs` so it no longer
   tries to mutate through the `Rc` — for instance, set the final value
   when `AppConfig` is built, before wrapping it in an `Rc`.
3. Fix `examples/07-rc-arc-mismatch-broken.rs` by building `config` with
   `Arc::new` instead of `Rc::new`.

### Implement

Five functions in `src/lib.rs`, on the same `AppConfig` that's already
fully written:

```sh
cargo test -p p2-06-03-rc-and-arc
```

- `share_config` — wrap an `AppConfig` in an `Rc`.
- `add_owner` — hand back a fresh handle to the same allocation, without
  copying the data.
- `owner_count` — how many handles are alive right now.
- `same_allocation` — whether two `Rc`s point at the same heap allocation,
  not merely equal data.
- `share_config_across_threads` — the same idea as `share_config`, with
  `Arc`.

The exact specification for each — including concrete examples — is in the
doc comment above the function.

### Build

Pick a real shape for shared ownership of your own — data loaded once that
several independent parts need, or a small node (say, a genre or an
author) that two other structs both reference. Build it with `Rc`, and use
`owner_count` or `same_allocation` to prove it's genuinely one allocation,
not two. In a comment, say why single ownership wouldn't have worked for
this scenario.

### Challenge (optional)

Look up `Rc::get_mut(&mut self) -> Option<&mut T>` in the standard docs —
it only returns `Some` when the count is exactly `1`. Build an `Rc<i32>`,
call `Rc::get_mut` (it should give `Some`), take a clone and call it again
(this time it should give `None`), drop the clone and call it once more
(it should give `Some` again). In a sentence or two, say why this is
exactly Phase 1's aliasing rule — just this time the compiler decides by
counting owners instead of tracking a borrow to the end of its scope.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Shared ownership | more than one variable, at once, is a real owner of one value | when no single owner makes sense |
| `Rc<T>` | the reference-counted smart pointer, single-threaded only | shared, read-only data, in one thread |
| `Arc<T>` | the same as `Rc`, with an atomic count | the same data, when owners might be in different threads |
| Reference count (`strong_count`) | how many handles are alive right now | knowing exactly when a value is actually freed |
| `Rc::ptr_eq` | do two handles point at the same allocation? | telling "the same value" apart from "an equal value" |

### What you now know

- `Rc<T>` lets more than one variable be a real, simultaneous owner of one
  heap value — a formal exception to Phase 1's "exactly one owner" rule,
  not a smaller version of it.
- `Rc::clone` never copies data — it only bumps the count and hands back a
  cheap second pointer to the same allocation; the address comparison via
  `Rc::as_ptr` proved it.
- The underlying value is only actually freed once the count reaches zero
  — exactly when the last `Rc` is also dropped.
- `Rc`'s count is a plain, non-atomic integer, so `Rc<T>` deliberately
  isn't `Send` and can never cross a thread boundary; `Arc<T>` gives the
  same API with an atomic count, at the cost of a slower clone.
- `Rc<T>` and `Arc<T>` share an API but are two completely separate types —
  there is no implicit conversion between them.
- Single ownership is still the right default; `Rc`/`Arc` are for that
  genuine exception — data with several independent owners, or a node with
  more than one parent — and both only ever hand out `&T`, never `&mut T`.

### What comes back later

- **`RefCell` and interior mutability, for when you genuinely need to
  mutate through a shared reference** — [2.6.5 — `RefCell`, `Cell`, and the run-time panic trade](../05-refcell-and-interior-mutability/README.md)
- **`Weak` and reference cycles — the real failure mode of shared ownership** — [2.6.4 — `Weak` and reference cycles](../04-weak-and-reference-cycles/README.md)
- **`thread::spawn` and actually crossing a thread boundary** — [2.8.1 — Threads, `Mutex`, `Arc`](../../08-concurrency/01-threads-mutex-arc/README.md)
- **The `Send` trait, and exactly why `Rc<T>` doesn't have it** — [2.8.4 — `Send` and `Sync`](../../08-concurrency/04-send-and-sync/README.md)

### Can you explain?

- Why can two `Rc<T>`s be real, simultaneous owners of one value, when
  Phase 1 said every value has exactly one owner?
- What does `Rc::clone` actually do to the underlying data, and how did the
  address comparison prove it?
- Why is `Rc<T>` restricted to one thread — what specifically would break
  if it weren't?
- `Rc<T>` and `Arc<T>` share an API but aren't the same type — which
  compiler error proved that?
- Name one real shape of shared ownership, and say why a single owner
  wasn't enough there.
- `Rc`/`Arc` alone only hand out `&T` — if the shared value genuinely needs
  to change, what's still missing?

---

## Going further

- [The Rust Book — `Rc<T>`, the Reference Counted Smart Pointer](https://doc.rust-lang.org/book/ch15-04-rc.html) — the same subject, from the Rust team itself.
- [`std::rc::Rc` documentation](https://doc.rust-lang.org/std/rc/struct.Rc.html) — the full list of its methods, including `get_mut` and `ptr_eq` from today.
- [`std::sync::Arc` documentation](https://doc.rust-lang.org/std/sync/struct.Arc.html) — the same API, this time with an atomic count.
- [`std::sync::atomic` documentation](https://doc.rust-lang.org/std/sync/atomic/index.html) — for whenever you get curious exactly how an atomic operation itself works.
