# 2.6.5 — `RefCell`, `Cell`, and the run-time panic trade

## At a glance

After this lesson you can:

- Explain exactly what rule interior mutability breaks, and why `RefCell` doesn't actually get around that rule — it only moves where it's enforced, from compile time to run time.
- Choose between `Cell` and `RefCell` for a real field, and say precisely why the one you picked is enough.
- Read a genuine "already borrowed" panic, say which two borrows collided, and use that same combination (`Rc<RefCell<T>>`) for shared, mutable state.

**Time:** ~55 minutes · **Prerequisites:**
[2.6.3 — Rc and Arc](../03-rc-and-arc/README.md) ·
[1.3.1 — Shared and mutable references](../../../phase1-fundamentals/03-borrowing-and-references/01-shared-and-mutable-refs/README.md)

---

## Why this matters

2.6.3 broke one rule on purpose: `Rc` let several places in a program own the same value at once, instead of just one. But that same lesson said something important and moved past it quickly: every owner you get from an `Rc` is only `&T` — a read-only view. If that `Rc<Config>` is a fixed settings object you only ever read, no problem. But if you want to build a shared counter, or a node that several places in the program point at and need to change later, you hit a real wall: the `&T` `Rc` hands you is exactly as unwritable, as far as the compiler is concerned, as any other `&T` has ever been. The aliasing rule you know from [1.3.1](../../../phase1-fundamentals/03-borrowing-and-references/01-shared-and-mutable-refs/README.md) is still sitting right where it was: any number of shared borrows, or exactly one mutable borrow — never both at once.

This lesson is exactly where that wall gets solved — not by setting the rule aside, but by moving where it's enforced. (If you remember, [1.2.5](../../../phase1-fundamentals/02-ownership-and-memory/05-drop-and-raii/README.md) showed you a `RefCell<Vec<String>>` from a distance — purely as that lesson's own test scaffolding, for recording the order things dropped in. The idea of "mutating from behind a shared reference" was never explained there — you only saw the type's name, not its mechanism. Here is where it actually gets explained.)

---

## The concept

### The tension: mutating from behind a shared reference

Say you have a page that has to count each visit:

```rust
struct PageViews {
    count: u32,
}
```

Now you want to write a method that only takes `&self` — because this method isn't supposed to change who owns anything, just bump a number — and inside it, write `self.count += 1;`.

This doesn't compile. `record_view` only has a shared reference to `self`, and the aliasing rule says exactly this: from behind a `&T` you can read, not write. The full error — `E0594` — is in "Errors you will meet"; `examples/05-mutate-through-shared-self.rs` builds exactly this.

The naive fix is to change the signature to `&mut self`. But that isn't always possible: if you got `PageViews` from behind an `Rc` and more than one owner genuinely exists, every owner only has `&PageViews` — `Rc::get_mut` exists ([2.6.3](../03-rc-and-arc/README.md) had you look it up), but it only returns `Some` when the strong count is exactly `1`, which is precisely the case that doesn't apply here. With real sharing in play, there's no `&mut` to be had, no matter what the method's signature says. This is exactly where interior mutability comes in: mutating a value from behind a shared reference — the very thing the aliasing rule normally forbids outright, at compile time. This lesson doesn't set that rule aside; it just moves where it's enforced.

### `RefCell<T>`: the same rule, moved to run time

```rust
use std::cell::RefCell;

struct WatchLog {
    entries: RefCell<Vec<String>>,
}

impl WatchLog {
    fn log(&self, title: &str) {
        self.entries.borrow_mut().push(title.to_string());
    }
}
```

`log` also only takes `&self`, but `.borrow_mut()` returns a `RefMut<Vec<String>>` — a guard, not the `Vec` itself. As long as that guard is alive (not yet dropped), `RefCell` knows a mutable borrow is in progress, and lets you write to the `Vec` through the guard. `.borrow()` does the same thing for reading, returning a `Ref<Vec<String>>`.

```rust
let log = WatchLog {
    entries: RefCell::new(Vec::new()),
};
log.log("Frieren");
log.log("Bocchi the Rock!");

let peek = log.entries.borrow(); // Ref<Vec<String>>: shared, read-only
println!("so far: {peek:?}");
drop(peek); // must end before borrow_mut, or the next log() panics

log.log("Made in Abyss");
println!("now:    {:?}", log.entries.borrow());
```

```sh
cargo run -p p2-06-05-refcell-and-interior-mutability --example 02-watch-log-refcell
```

```text
so far: ["Frieren", "Bocchi the Rock!"]
now:    ["Frieren", "Bocchi the Rock!", "Made in Abyss"]
```

The important line is `drop(peek)`. `peek` is a shared borrow; as long as it's alive, `RefCell` won't accept a fresh `borrow_mut()`. Here it's dropped explicitly, and that's not incidental — a `Ref` guard is an ordinary owned value with a real `Drop` impl, so it follows [1.2.5](../../../phase1-fundamentals/02-ownership-and-memory/05-drop-and-raii/README.md)'s block-scope cleanup rule (dropped at the closing brace), not [1.3.3](../../../phase1-fundamentals/03-borrowing-and-references/03-borrow-scopes-and-nll/README.md)'s last-use rule for a plain `&`/`&mut` reference. If `peek` had been taken inside a smaller block, it would have been dropped the moment that block closed — but had we left it running to the end of `main` with its last real use sitting earlier, it would still be "alive" and still block a `borrow_mut()`, which is exactly why the explicit `drop(peek)` was needed here at all.

### When you break the rule: a run-time panic

Phase 1's compiler caught this exact mistake right then, before the program ever ran. `RefCell` doesn't give up that guarantee — it just applies it later. If a `RefMut` is still alive (not dropped by hand, not dropped by its scope ending) and you take another `borrow_mut()` at that same moment:

```senpai-visual
{"kind":"borrowing","labels":["borrow_mut(): a RefMut is alive","second borrow_mut() attempted","panic: RefCell already borrowed","the first RefMut is dropped","borrow_mut() succeeds this time"]}
```

the program panics: it compiles, it runs, and it stops right at the point where two `borrow_mut()`s would be alive at once. `examples/07-double-borrow-mut-panics.rs` builds exactly this; the full panic message is in "Errors you will meet".

None of this is `unsafe` in any way. It's completely safe Rust — it just checks, at run time, a rule Phase 1 checked at compile time. If `RefCell` didn't panic here and instead let two incompatible accesses stay alive at once, you'd get exactly what Phase 1 made impossible in the first place — a write happening at the same time as another read or write, to the same memory. A panic, without exception, beats that.

### `Cell<T>`: the simpler sibling, with no borrowing at all

`Cell<T>` solves the same problem with a much simpler tool: there's no borrowing involved at all, so there's no guard, no tracking of live borrows, and nothing that can break and panic.

```rust
use std::cell::Cell;

struct PageViews {
    count: Cell<u32>,
}

impl PageViews {
    fn record_view(&self) {
        self.count.set(self.count.get() + 1);
    }
}
```

```rust
let page = PageViews {
    count: Cell::new(0),
};
page.record_view();
page.record_view();
page.record_view();
println!("views: {}", page.count.get());
```

```sh
cargo run -p p2-06-05-refcell-and-interior-mutability --example 01-page-views-cell
```

```text
views: 3
```

`Cell`'s secret is that it never hands out a reference to what's inside it. `.get()` copies the value out; `.set()` takes a new value and swaps it in for the old one — completely, in one move. No real `&u32` or `&mut u32` ever leaks out for anything to have to keep track of, so there's nothing to violate the aliasing rule, and nothing that needs checking at run time. Notice `page` above isn't even `mut` — `record_view` only wants `&self`, and `.set()`/`.get()` both take `&self` too, not `&mut self`; the variable itself never needed to be mutable, exactly as much as `record_view` didn't.

The price of that simplicity: `.get()` only works on `Copy` types — a number, a `bool`, anything small and cheap to copy. Build a `Cell<String>` and call `.get()`, and it won't compile; the full error (`E0599`) is in "Errors you will meet", and it says exactly why: `String` isn't a `Copy` type. For a type like `String` you can either use `Cell` with `.replace()`/`.take()` (which move the value rather than copy it — no `Copy` bound needed), or reach for `RefCell<String>` instead.

### When `Cell` is enough, when you need `RefCell`

The practical rule is short. If what's inside is `Copy` — a counter, a `bool` flag, a small enum — `Cell` is usually the better pick: cheaper, no borrow tracking, no panic risk at all. If what's inside isn't `Copy` (a `String`, a `Vec`, a struct of your own), or you genuinely need a `&mut T` that lasts more than one line — say, you want to walk a `Vec` that lives inside a cell while also pushing to it at the same moment — that's when you want `RefCell`; `Cell` doesn't give you that kind of temporary access, only copying or swapping the whole value.

### The honest trade: `Rc<RefCell<T>>` for shared, mutable state

Now go back to the lesson's opening problem. 2.6.3 said every owner of an `Rc<T>` only gets `&T`. Wrap that `T` in a `RefCell` and that limit disappears — every owner, working through their own clone of the `Rc`, can call `.borrow_mut()` and genuinely change the shared value:

```rust
let hype = Rc::new(RefCell::new(0));
let a = Rc::clone(&hype);
let b = Rc::clone(&hype);

*a.borrow_mut() += 10;
*b.borrow_mut() += 5;
```

```sh
cargo run -p p2-06-05-refcell-and-interior-mutability --example 03-shared-hype-rc-refcell
```

```text
hype via a: 15
hype via b: 15
owners:     3
```

```senpai-visual
{"kind":"ownership","labels":["Rc::clone once","Rc::clone again","handle a: borrow_mut()","one shared RefCell<i32>","handle b sees the new value"]}
```

Three separate handles (`hype`, `a`, `b`), one `RefCell<i32>` on the heap. Any of them can change the value as often as it wants, and all of them see the same, latest value — because all three point at one address, not three copies.

`RefCell` doesn't have to wrap the whole type; sometimes it only needs to wrap the one field that actually has to change:

```rust
struct Series {
    title: String,
    episodes_watched: RefCell<u32>,
}
```

```sh
cargo run -p p2-06-05-refcell-and-interior-mutability --example 04-catalog-shared-series
```

```text
Frieren: 2 episodes watched (3 owners share it)
```

`title` stays fixed once it's built; only `episodes_watched` is mutable, and `Rc<Series>` shares that one field between every place holding a reference to it — exactly the shape a real tree, or anywhere a value is seen and changed from several places in a program, needs.

This combination — `Rc` for shared ownership, `RefCell` for shared mutability — is the most common reason you reach for `RefCell` at all. But look at its trade honestly. What you gain is a pattern the compiler alone can't express: multiple long-lived owners who all genuinely need to change the shared value. What you lose is that a whole class of bug — two incompatible accesses to one value — turns from a compile-time error into a run-time panic: something that used to never even compile now compiles, and only breaks once the wrong path is actually taken at run time. That's why `Rc<RefCell<T>>` should be a deliberate choice, not a reflex: reach for it when you genuinely have multiple long-lived owners who all need to mutate and there's no way to name a single owner instead — not in place of a plain `&mut` that would have done the job.

One last warning: if two of these point at each other — say, a parent and a child both linked with a strong `Rc` — neither one's count ever reaches zero, and their memory is never freed. This is called a reference cycle, and the tool that fixes it — `Weak` — is 2.6.4's subject.

---

## Hands on

```sh
cargo run -p p2-06-05-refcell-and-interior-mutability --example 01-page-views-cell
cargo run -p p2-06-05-refcell-and-interior-mutability --example 02-watch-log-refcell
cargo run -p p2-06-05-refcell-and-interior-mutability --example 03-shared-hype-rc-refcell
cargo run -p p2-06-05-refcell-and-interior-mutability --example 04-catalog-shared-series
```

Then the three broken ones:

```sh
cargo run -p p2-06-05-refcell-and-interior-mutability --example 05-mutate-through-shared-self --features broken
cargo run -p p2-06-05-refcell-and-interior-mutability --example 06-cell-get-requires-copy --features broken
cargo run -p p2-06-05-refcell-and-interior-mutability --example 07-double-borrow-mut-panics --features broken
```

Then try:

1. In `02-watch-log-refcell`, delete the `drop(peek);` line and run it again. Read the panic message — exactly which borrow collides with which?
2. In `03-shared-hype-rc-refcell`, add a `println!("{}", Rc::strong_count(&hype));` right after `hype` is built, before `a` and `b` exist. What number do you expect, and does it match what prints at the end of the program?
3. In `01-page-views-cell`, change `Cell<u32>` to `Cell<i64>` and adjust the arithmetic to match. Does anything besides the type change? Why doesn't `Cell` care what `T` actually is, as long as it's `Copy`?

---

## Errors you will meet

### `E0594` — you can't write from behind a shared reference

```text
error[E0594]: cannot assign to `self.count`, which is behind a `&` reference
  --> phase2-intermediate\06-smart-pointers\05-refcell-and-interior-mutability\examples\05-mutate-through-shared-self.rs:11:9
   |
11 |         self.count += 1;
   |         ^^^^^^^^^^^^^^^ `self` is a `&` reference, so it cannot be written to
   |
help: consider changing this to be a mutable reference
   |
10 |     fn record_view(&mut self) {
   |                     +++

For more information about this error, try `rustc --explain E0594`.
error: could not compile `p2-06-05-refcell-and-interior-mutability` (example "05-mutate-through-shared-self") due to 1 previous error
```

**What the compiler is objecting to:** `record_view` is defined on `&self` — a shared reference. `self.count += 1` is an assignment, and assigning to a field from behind a `&` is exactly what the aliasing rule forbids. The compiler even offers its own suggestion — changing the signature to `&mut self` — but that suggestion isn't always something you can act on.

**The fix:** wrap `count` in a `Cell`, and leave the signature untouched:

```rust
struct PageViews {
    count: Cell<u32>,
}

impl PageViews {
    fn record_view(&self) {
        self.count.set(self.count.get() + 1);
    }
}
```

**Why that's the fix:** `record_view` still only takes `&self` — exactly the signature an `Rc<PageViews>` would let you call too. `Cell::set` itself takes `&self`, not `&mut self`; the compiler no longer has to check "only one write at a time," because `Cell` never hands out a reference to its contents for anything to collide with in the first place.

### `E0599` — `.get()` on a `Cell` only works for `Copy` types

```text
error[E0599]: the method `get` exists for struct `Cell<String>`, but its trait bounds were not satisfied
   --> phase2-intermediate\06-smart-pointers\05-refcell-and-interior-mutability\examples\06-cell-get-requires-copy.rs:9:22
    |
  9 |     let title = slot.get();
    |                      ^^^
    |
   ::: C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\alloc\src\string.rs:353:1
    |
353 | pub struct String {
    | ----------------- doesn't satisfy `String: Copy`
    |
    = note: the following trait bounds were not satisfied:
            `String: Copy`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `p2-06-05-refcell-and-interior-mutability` (example "06-cell-get-requires-copy") due to 1 previous error
```

**What the compiler is objecting to:** `.get()` genuinely exists on `Cell<T>` — the message even says "the method `get` exists." The problem is elsewhere: `.get()` is only available when `T: Copy`, and `String` doesn't have that bound. The compiler points straight at the `pub struct String` line inside the standard library that its missing `Copy` comes from.

**The fix:** for a non-`Copy` type, reach for `RefCell`:

```rust
use std::cell::RefCell;

fn main() {
    let slot: RefCell<String> = RefCell::new(String::from("Frieren"));
    let title = slot.borrow();
    println!("{title}");
}
```

**Why that's the fix:** `.get()` had to copy the whole value to hand it back — exactly what `Copy` is required for. `RefCell::borrow()` doesn't have that problem, because it never copies at all; it just hands out a temporary, read-only view (`Ref<String>`) of the same original value. If you'd rather stay with `Cell<String>`, `.replace()` or `.take()` also work — because they move the value instead of copying it — but neither one is "just look, don't touch"; both always swap something in. The Repair exercise walks through that difference hands-on.

### A run-time panic — "RefCell already borrowed"

```text
thread 'main' (10384) panicked at phase2-intermediate\06-smart-pointers\05-refcell-and-interior-mutability\examples\07-double-borrow-mut-panics.rs:12:24:
RefCell already borrowed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**What the compiler is objecting to:** this isn't even a compiler error — the program built, it ran, and it panicked exactly where line 12 (the second `hype.borrow_mut()`) executed. Line 11 took a `RefMut<i32>` and never released it (not by hand, not by its scope ending — both live until the end of `main`); line 12 wants a second mutable borrow on that same `RefCell`. `RefCell` detects this right at that moment, and rather than let two incompatible accesses stay alive at once, it panics.

**The fix:** make sure the first borrow is genuinely dropped before the second one is taken:

```rust
let hype = RefCell::new(0);
{
    let _first = hype.borrow_mut();
} // _first is dropped here, before the next borrow starts
let _second = hype.borrow_mut();
```

**Why that's the fix:** now `_first` is dropped exactly when its small block closes — before the next line even starts taking `_second`. At the moment of the second `borrow_mut()`, `RefCell` only asks "right now, is there a live borrow this would collide with?" and the answer is no longer "yes." No new tool was needed — just the same scope-based discipline RAII and [1.2.5](../../../phase1-fundamentals/02-ownership-and-memory/05-drop-and-raii/README.md) already taught you.

---

## Exercises

### Warm up

<details>
<summary>What does this print?</summary>

```rust
use std::cell::Cell;

let count = Cell::new(1);
count.set(count.get() + 1);
count.set(count.get() + 1);
println!("{}", count.get());
```

</details>

<details>
<summary>Answer</summary>

```text
3
```

Each `.set()` replaces the value with the fresh one `.get()` just produced. Three steps: 1, then 2, then 3.

</details>

<details>
<summary>Does <code>let cell = Cell::new(5); cell.set(10);</code> — with no <code>let mut cell</code> — compile?</summary>

Yes. `Cell::set` is defined on `&self`, not `&mut self`; calling a `&self` method never requires the variable itself to be `mut`.

</details>

<details>
<summary>Does this compile, or does it panic?</summary>

```rust
use std::cell::RefCell;

let cell = RefCell::new(vec![1, 2, 3]);
cell.borrow().push(4);
```

</details>

<details>
<summary>Answer</summary>

It doesn't compile — it never even gets a chance to panic. `.borrow()` returns a `Ref<Vec<i32>>`; `Ref` only has `Deref`, not `DerefMut` — the same trait pair you know from [2.4.3](../../04-lifetimes-and-conversion/03-deref-asref-borrow/README.md). `.push()` needs a `&mut Vec<i32>`, and there's no way to get one from behind a `Ref`. The error code is `E0596` — the same one you saw there too.

</details>

<details>
<summary>You take a <code>.borrow()</code> inside a small block and never <code>drop</code> it by hand. After that block closes, you take a <code>.borrow_mut()</code>. Does it panic?</summary>

No. The end of the block drops `peek` exactly as much as a manual `drop(peek)` would — the same borrow-scope rule you know from Phase 1. By the time `borrow_mut()` is taken after that point, no live borrow is left for it to collide with.

</details>

### Repair

Fix all three broken examples:

1. Fix `examples/05-mutate-through-shared-self.rs` so `record_view` still takes `&self`, not `&mut self` — meaning use exactly the tool this lesson showed you.
2. Fix `examples/06-cell-get-requires-copy.rs` **two** ways: once by changing `Cell<String>` to `RefCell<String>` and reading with `.borrow()`; once by keeping `Cell<String>` but changing `.get()` to `.take()`. After each, what's different about `slot`'s own value afterward?
3. Fix `examples/07-double-borrow-mut-panics.rs` so it no longer panics — without deleting the second `borrow_mut()` itself.

### Implement

Two types in `src/lib.rs` — one built on `Cell` and `RefCell`, one built on `Rc<RefCell<T>>`:

```sh
cargo test -p p2-06-05-refcell-and-interior-mutability
```

`ClubStats` has two fields, and all five of its methods take only `&self` — exactly the pattern you saw in "The concept." `SharedHypeMeter` is that same classic `Rc<RefCell<T>>` combo; `.clone()` should give a second handle to the same shared value, not an independent copy. Implement each method exactly to the doc comment above it.

### Build

Design a small struct, for a domain of your choosing, that needs interior mutability — a shopping cart total, a play counter, a settings flag, a simple log. If every field is `Copy`, reach for `Cell`; if even one isn't (a `String`, a `Vec`, an enum that carries data), reach for `RefCell`. Give it at least one method that writes from behind `&self` and one that reads from behind `&self`.

### Challenge (optional)

**Part one.** Alongside `borrow_mut()`, `RefCell` also has `try_borrow_mut()`, which returns a `Result<RefMut<T>, BorrowMutError>` instead of panicking. Write a `try_hype_up(&self, amount: i32) -> bool` method on `SharedHypeMeter` that uses `try_borrow_mut()`: if it gets the borrow, change the value and return `true`; if it doesn't, no panic happens and it returns `false`. Write a test that builds two shared handles, deliberately keeps a `RefMut` alive through one of them, and confirms that `try_hype_up` on the other one returns `false` at that same moment.

**Part two.** (This one looks ahead.) Build a `Rc<RefCell<i32>>` and try using it inside a `std::thread::spawn` closure. It won't compile — the compiler says exactly that `Rc<RefCell<i32>>` cannot be safely sent between threads. That same pattern, across multiple threads instead of one, is [Threads, Mutex, and Arc](../../08-concurrency/01-threads-mutex-arc/README.md)'s subject.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Interior mutability | Mutating a value from behind a `&T` | Whenever you only have a shared reference but need to write |
| `Cell<T>` | `.get()`/`.set()` with no borrowing at all; `.get()` needs `Copy` | A simple counter or flag |
| `RefCell<T>` | The aliasing rule, checked at run time | A `Vec`, `String`, or struct that must change from behind `&self` |
| `Ref<T>` / `RefMut<T>` | The guards `.borrow()`/`.borrow_mut()` return | Only `RefMut` also has `DerefMut` |
| "already borrowed" | `RefCell`'s panic when two incompatible borrows are alive at once | The bug Phase 1 caught at compile time |
| `Rc<RefCell<T>>` | Shared ownership plus shared mutability | Multiple long-lived owners who all need to write |

### What you now know

This lesson closes the smart-pointers module, and from a distance it's one continuous story. `Box` moves a value onto the heap — for when ownership is still just one place's business, but the value's size or recursiveness makes living on the stack a problem. `Rc`/`Arc` break the "exactly one owner" rule and share ownership across several places in a program — at the cost of every owner only getting to read. `Weak` points at that same shared value without counting as one of its owners, exactly for the place where a cycle would otherwise never get freed. And `RefCell` — together with its simpler sibling `Cell` — is the last piece: once you need mutability alongside that shared ownership, this is where the rule moves, from compile time to run time.

From this lesson specifically:

- Interior mutability means writing from behind a `&T`; it doesn't set the aliasing rule aside, it just moves where the rule gets enforced.
- `RefCell<T>` moves that rule to run time: `.borrow()`/`.borrow_mut()` return guards, and if two incompatible borrows are alive at once, you get a panic, not a silent crash.
- `Cell<T>` never hands out a reference to what's inside it, so it needs no guard, no tracking, and no panic — at the cost of `.get()` only working on `Copy` types.
- Which one, when: `Copy` and simple — `Cell`; not `Copy`, or a temporary `&mut` is genuinely needed — `RefCell`.
- `Rc<RefCell<T>>` combines `Rc`'s shared ownership with `RefCell`'s mutability — exactly what multiple long-lived owners need in order to write.
- That trade isn't free: something the compiler used to reject at compile time now compiles, and only panics if the wrong path is actually taken at run time.

### What comes back later

- **The same rule, across multiple threads instead of one — `Mutex` and `Arc<Mutex<T>>`** — [Threads, Mutex, and Arc](../../08-concurrency/01-threads-mutex-arc/README.md)

### Can you explain?

- Why doesn't `self.count += 1` compile inside a `&self` method, but the same thing compiles from behind a `Cell` or `RefCell`?
- What's the exact difference between `Cell` and `RefCell`, and which question do you ask yourself to choose between them?
- Why won't `Ref<T>` let you call `.push()` (or any other method that needs `&mut`) on it, even when there's no competing borrow around at all?
- What exactly does the "run-time panic trade" gain, and what does it cost?
- In `Rc<RefCell<T>>`, what is `Rc` doing and what is `RefCell` doing? What would you lose if you only had one of them?

---

## Going further

- [The Rust Book — `RefCell<T>` and the Interior Mutability Pattern](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html) — the same ground, officially.
- [`std::cell::RefCell`](https://doc.rust-lang.org/std/cell/struct.RefCell.html) — the type's full documentation, including `try_borrow`/`try_borrow_mut`.
- [`std::cell::Cell`](https://doc.rust-lang.org/std/cell/struct.Cell.html) — the type's full documentation, including `.replace()`, `.take()`, and `.into_inner()`.
- [`std::cell` — module overview](https://doc.rust-lang.org/std/cell/index.html) — why these types are sound at all, for whenever you get curious how deep it goes.
