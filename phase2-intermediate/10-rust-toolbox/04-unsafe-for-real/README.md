# 2.10.4 — `unsafe`, for real

## At a glance

After this lesson you can:

- Name the exact five operations `unsafe` unlocks, and explain why entering an `unsafe` block does not turn off the borrow checker.
- Tell a raw pointer (`*const T`/`*mut T`) apart from a reference (`&T`/`&mut T`), and say why raw pointers are exempt from the aliasing rule — which is exactly why they exist.
- Explain why a raw pointer field makes a type `!Send`/`!Sync` by default, and grant it back by hand with `unsafe impl` — saying precisely what that promise obligates you to have verified.
- Build a small owning wrapper around a raw pointer, on top of `Box::into_raw`/`Box::from_raw`, and say what `PhantomData<T>` actually does for it.

**Time:** ~75 minutes · **Prerequisites:**
[2.8.4 — `Send` and `Sync`](../../08-concurrency/04-send-and-sync/README.md),
[2.6.1 — `Box` and heap allocation](../../06-smart-pointers/01-box-and-heap-allocation/README.md)

---

## Why this matters

2.8.4 handed you two promises and deliberately left both open. One: "Raw pointers — `*const T` and `*mut T` — are also neither `Send` nor `Sync` by default, for the same general reason: the compiler has no way to know whether whatever they point at stays safe under access from several threads, so it assumes the cautious answer. Real work with raw pointers is 2.10's job." The other: "A type's author can hand it either trait by hand, with `unsafe impl Send for X {}` (or `Sync`), when they've personally verified it's genuinely safe. That's a real `unsafe` promise — exactly the kind of commitment 2.10 opens up properly."

Today both get paid off. Every time `unsafe` has crossed your path so far — inside the standard library's own source, inside a third-party crate — you've moved past it and told yourself you'd understand it later. This is the last lesson of the toolbox module, and the last lesson of Phase 2 as a whole: the point where that keyword stops being a vague red flag and becomes a specific, bounded tool — along with exactly the responsibility it hands you.

---

## The concept

### `unsafe` opens exactly five operations

Inside an `unsafe` block, ownership, borrowing, and lifetimes are still fully enforced by the compiler. `unsafe` opens only these five operations, and nothing else:

1. Dereferencing a **raw pointer** (`*const T` / `*mut T`).
2. Calling an `unsafe fn` (including C functions via FFI).
3. Accessing or modifying a mutable `static` variable.
4. Implementing an `unsafe trait`.
5. Accessing a field of a `union`.

That's it. Inside `unsafe { ... }`, the compiler drops exactly the one invariant each of these five operations relies on — it stops checking nothing else:

```rust
let mut value = 10;
let ptr: *mut i32 = &mut value;
let doubled = unsafe { *ptr * 2 };
println!("{doubled}");
```

```text
20
```

Creating `ptr` needed no `unsafe` at all — operation #1 only shows up the moment you actually read through the pointer.

### Raw pointers vs. references

Unlike `&T`/`&mut T`, raw pointers can be null, dangling, or unaligned — the compiler checks none of that when you *create* one, only when you *dereference* it. And they're exempt from the aliasing rule: you can have as many `*mut T` pointers to the same data at once as you like — which is exactly what justifies them existing at all:

```rust
let mut data = [1, 2, 3];
let a: *mut i32 = &mut data[0];
let b: *mut i32 = &mut data[0];
unsafe {
    *a += 10;
    *b += 100;
}
println!("{data:?}");
```

```text
[111, 2, 3]
```

Two `*mut i32` pointing at the same slot at once — something two `&mut i32` never would have compiled. The compiler let this through because raw pointers simply don't obey "aliasing XOR mutability."

```senpai-visual
{"kind":"concept","labels":["a: *mut i32 points at data[0]","b: *mut i32 points at the same data[0]","the compiler allows both to exist at once","unsafe { *a += 10; *b += 100; } — both write through","you, not the compiler, are the one vouching this is fine"]}
```

### Worked example: why `split_at_mut` can't be written the obvious way

Suppose you want a simple `split_at_mut`: split one mutable slice into two independent halves at index `mid`. The obvious first try is `(&mut slice[..mid], &mut slice[mid..])` — two `&mut` borrows of `slice`, both alive through the end of the function. This does not compile; the borrow checker has no way to see that these two ranges provably never overlap, it only sees "two mutable borrows of the same variable" — the same family of error you've already met, this time on two pieces of one slice. The full diagnostic is in "Errors you will meet."

This is exactly where raw pointers earn their place: you, the programmer, know the two ranges are disjoint; a raw pointer is how you tell the compiler "trust me here." The toolkit is small — `.as_mut_ptr()` gives a `*mut T` to the first element, and `.add(n)` moves a pointer `n` elements forward; both are safe to *compute*, only dereferencing the result needs `unsafe`:

```rust
let mut data = [1, 2, 3, 4, 5];
let first = data.as_mut_ptr();
unsafe {
    *first *= 10;
    *first.add(4) *= 10;
}
println!("{data:?}");
```

```text
[10, 2, 3, 4, 50]
```

`std::slice::from_raw_parts_mut(ptr, len)` goes one step further: it builds a real `&mut [T]` out of a raw pointer and a length — its safety contract is exactly what you'd expect, the pointer must be valid for `len` elements, and nothing else may reference that memory at the same time. Those two tools — a pointer to the slice's start, and `from_raw_parts_mut` — are everything `split_at_mut_demo` needs in the Implement exercise.

### Why a raw pointer blocks `Send` by default

Let's see exactly what 2.8.4 promised. Build a small type with a raw pointer field:

```text
struct RawHolder<T> {
    ptr: *mut T,
}
```

With no extra work at all, `RawHolder<i32>` is neither `Send` nor `Sync` — even though `i32` itself is both. That's not the compiler looking at `T` and rejecting it; it's that the type `*mut T` itself, for *any* `T`, simply isn't `Send`/`Sync` — the compiler can't see past the pointer to know what's on the other side of it, so it stays cautious. Trying to use it gives: `` `*mut i32` cannot be sent between threads safely `` — the full transcript is in "Errors you will meet."

### Granting `Send` by hand, and the promise it actually makes

This is where a type's author steps in:

```rust
struct RawHolder<T> {
    ptr: *mut T,
}

unsafe impl<T: Send> Send for RawHolder<T> {}

fn assert_send<T: Send>() {}

fn main() {
    assert_send::<RawHolder<i32>>();
    println!("RawHolder<i32> is Send");
}
```

```text
RawHolder<i32> is Send
```

This is the exact `unsafe impl` 2.8.4 said "belongs to a different lesson" — today is that lesson. Writing this line is you personally guaranteeing something the compiler can't prove on its own: that moving ownership of `RawHolder<T>` to another thread, whenever `T` itself is `Send`, is genuinely safe. That's a real contract, not a formality — an author who gets it wrong introduces undefined behavior for every caller, invisibly. For `RawHolder<T>` the promise is easy to keep, because its one field is a raw pointer nothing else touches at the same time — exactly as safe as a `Box<T>`, which is already `Send` whenever `T` is; this `unsafe impl` isn't asserting anything beyond what `Box` already guarantees.

### `PhantomData<T>`: telling the compiler this type owns a `T`

Suppose you write a type whose parameter appears in no field at all:

```rust
use std::marker::PhantomData;

struct Tag<T> {
    id: u32,
    _marker: PhantomData<T>,
}
```

```rust
let t: Tag<String> = Tag { id: 7, _marker: PhantomData };
println!("tag id: {}", t.id);
```

```text
tag id: 7
```

Drop that `_marker` field, and the compiler's complaint is short: `` error[E0392]: type parameter `T` is never used ``. `PhantomData<T>` is exactly the fix for that: a zero-sized field that stores no actual `T` at runtime, but tells the compiler "treat this type as if it holds a `T`."

```senpai-visual
{"kind":"ownership","labels":["OwnedBox<T>","ptr: raw pointer to T","compiler sees no owned T","plus PhantomData<T>","compiler treats it as owning a T"]}
```

Now the honest part. The `OwnedBox<T>` you're about to build in "Build" already has a `ptr: *mut T` field — meaning it already, textually, mentions `T`. Delete `_marker: PhantomData<T>` from it right now and it still compiles; you won't even get `E0392`, because `*mut T` already does the job of "uses `T`" as far as the compiler is concerned. So why keep it? Not because of a compile it would break today, but because it's the exact signal every real raw-pointer-based type in the standard library — `Box`, `Vec`, `Rc` — carries: it says this pointer means ownership, not merely a borrow. A habit worth keeping even here, where the compiler had already reached the same conclusion on its own.

---

## Hands on

```sh
cargo run -p p2-10-04-unsafe-for-real --example 01-raw-pointer-basics
cargo run -p p2-10-04-unsafe-for-real --example 02-box-into-raw-and-from-raw
```

Then the three broken ones:

```sh
cargo run -p p2-10-04-unsafe-for-real --example 03-split-at-mut-naive-broken --features broken
cargo run -p p2-10-04-unsafe-for-real --example 04-deref-outside-unsafe-broken --features broken
cargo run -p p2-10-04-unsafe-for-real --example 05-raw-pointer-not-send-broken --features broken
```

And the fix:

```sh
cargo run -p p2-10-04-unsafe-for-real --example 06-raw-pointer-send-fixed
```

Then try these:

1. In `01-raw-pointer-basics`, after the last section, build a third `*mut i32` pointing at `more[2]` and multiply it by 10 too. How does the printed output change?
2. In `05-raw-pointer-not-send-broken`, change `RawHolder<i32>` to `RawHolder<std::rc::Rc<i32>>`. After adding file 06's `unsafe impl` to this file too, does it compile now? Why or why not?

---

## Errors you will meet

### `E0499` — cannot borrow a slice as mutable twice at once

```text
error[E0499]: cannot borrow `*slice` as mutable more than once at a time
  --> phase2-intermediate\10-rust-toolbox\04-unsafe-for-real\examples\03-split-at-mut-naive-broken.rs:10:30
   |
 9 | fn split_at_mut_naive<T>(slice: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
   |                                 - let's call the lifetime of this reference `'1`
10 |     (&mut slice[..mid], &mut slice[mid..])
   |     -------------------------^^^^^--------
   |     |     |                  |
   |     |     |                  second mutable borrow occurs here
   |     |     first mutable borrow occurs here
   |     returning this value requires that `*slice` is borrowed for `'1`
   |
   = help: use `.split_at_mut(position)` to obtain two mutable non-overlapping sub-slices

For more information about this error, try `rustc --explain E0499`.
```

**What the compiler is actually objecting to:** `&mut slice[..mid]` and `&mut slice[mid..]` are both alive through the end of the function, and both borrow `*slice` mutably. The borrow checker only sees "two simultaneous `&mut` borrows of the same variable" — it has no concept of "these two ranges are provably disjoint."

**The fix:** exactly what the compiler itself suggests — `std::slice::split_at_mut`, which the standard library already wrote with raw pointers, or your own version from "Implement."

**Why this is the fix:** raw pointers are exempt from the aliasing rule, so you can tell the compiler by hand that the two ranges are disjoint — something it can't prove on its own just by looking at the indices.

### `E0133` — dereferencing a raw pointer outside `unsafe`

```text
error[E0133]: dereference of raw pointer is unsafe and requires unsafe function or block
  --> phase2-intermediate\10-rust-toolbox\04-unsafe-for-real\examples\04-deref-outside-unsafe-broken.rs:11:19
   |
11 |     let doubled = *ptr * 2;
   |                   ^^^^ dereference of raw pointer
   |
   = note: raw pointers may be null, dangling or unaligned; they can violate aliasing rules and cause data races: all of these are undefined behavior

For more information about this error, try `rustc --explain E0133`.
```

**What the compiler is actually objecting to:** creating `ptr` was fine — the line before is entirely safe. The problem is `*ptr * 2`: it reads through a raw pointer, outside any `unsafe` block.

**The fix:** put that same read inside an `unsafe { ... }` block.

**Why this is the fix:** operation #1 from the list above is exactly this — dereferencing a raw pointer. Until you're inside `unsafe`, the compiler has no way to know you've personally checked that this pointer is valid, aligned, and alive.

### `E0277` — a raw pointer isn't `Send` on its own, even with `T: Send`

```text
error[E0277]: `*mut i32` cannot be sent between threads safely
  --> phase2-intermediate\10-rust-toolbox\04-unsafe-for-real\examples\05-raw-pointer-not-send-broken.rs:16:19
   |
16 |     assert_send::<RawHolder<i32>>();
   |                   ^^^^^^^^^^^^^^ `*mut i32` cannot be sent between threads safely
   |
   = help: within `RawHolder<i32>`, the trait `Send` is not implemented for `*mut i32`
note: required because it appears within the type `RawHolder<i32>`
  --> phase2-intermediate\10-rust-toolbox\04-unsafe-for-real\examples\05-raw-pointer-not-send-broken.rs:9:8
   |
 9 | struct RawHolder<T> {
   |        ^^^^^^^^^
note: required by a bound in `assert_send`
  --> phase2-intermediate\10-rust-toolbox\04-unsafe-for-real\examples\05-raw-pointer-not-send-broken.rs:13:19
   |
13 | fn assert_send<T: Send>() {}
   |                   ^^^^ required by this bound in `assert_send`

For more information about this error, try `rustc --explain E0277`.
```

**What the compiler is actually objecting to:** `RawHolder<i32>` has one field, `ptr: *mut i32`. The type `*mut i32` itself — regardless of how harmless `i32` is — isn't `Send`, and one non-`Send` field is enough to disqualify the whole struct.

**The fix:** the type's author has to say explicitly that this pointer is safe — the same `unsafe impl<T: Send> Send for RawHolder<T> {}` you saw in the concept section (file 06).

**Why this is the fix:** the compiler can't see past the pointer to know what's on the other side, so it stays cautious unless someone who actually knows the type says otherwise. That's exactly what `unsafe impl` is for.

---

## Exercises

### Warm up

<details>
<summary>Does entering an <code>unsafe</code> block turn off the borrow checker?</summary>

Write your answer down before opening this.

</details>

<details>
<summary>Answer</summary>

No. Ownership, borrowing, and lifetimes are still fully enforced inside `unsafe { ... }`. `unsafe` only opens five specific operations — the rest of Rust is still ordinary safe Rust.

</details>

<details>
<summary>Does creating a raw pointer (<code>let p: *mut i32 = &amp;mut x;</code>) need <code>unsafe</code>, or only dereferencing one?</summary>

Write your answer down before opening this.

</details>

<details>
<summary>Answer</summary>

Only dereferencing one. Creating a raw pointer is always safe — even a null or dangling one; the danger starts exactly when you read or write through it.

</details>

<details>
<summary>Does this compile?</summary>

```rust
use std::rc::Rc;

struct RawHolder<T> {
    ptr: *mut T,
}

unsafe impl<T: Send> Send for RawHolder<T> {}

fn assert_send<T: Send>() {}

fn main() {
    assert_send::<RawHolder<Rc<i32>>>();
}
```

</details>

<details>
<summary>Answer</summary>

No — `E0277`. `unsafe impl<T: Send> Send for RawHolder<T> {}` only makes `RawHolder<T>` `Send` when `T` itself is `Send`. `Rc<i32>` is one of the rare types that isn't, so the bound fails.

</details>

<details>
<summary>True or false: <code>PhantomData&lt;T&gt;</code> stores an actual value of type <code>T</code> at runtime, so adding one to a struct grows its size by <code>T</code>'s size.</summary>

Write your answer down before opening this.

</details>

<details>
<summary>Answer</summary>

False. `PhantomData<T>` is always zero-sized, no matter what `T` is — it adds no bytes to the struct at all. Its job is signaling to the compiler, not storage.

</details>

### Repair

Fix `examples/04-deref-outside-unsafe-broken.rs` so it compiles — put the read through `ptr` inside an `unsafe { ... }` block, and change nothing else.

### Implement

Complete `split_at_mut_demo` in `src/lib.rs`:

```sh
cargo test -p p2-10-04-unsafe-for-real
```

Its specification is exactly what the doc comment above the function says — the same two tools from the concept section, `.as_mut_ptr()` and `std::slice::from_raw_parts_mut`, are all it needs.

### Build

Complete `OwnedBox<T>` in `src/lib.rs` — a small owning wrapper around a heap-allocated `T`, built on `Box::into_raw`/`Box::from_raw`:

- `new(value)` must move `value` onto the heap and take ownership of it.
- `get(&self)` must return a shared reference to the value inside.
- `Drop::drop` must free the wrapped value exactly once — no leak, no double free.
- Right after the struct definition (not inside any `impl` block), also add `unsafe impl<T: Send> Send for OwnedBox<T> {}` — the same promise you saw made for `RawHolder<T>`. Its full spec is in the doc comment above the struct.

```sh
cargo test -p p2-10-04-unsafe-for-real
```

Nobody writes this in real code — `Box<T>` already does exactly this, correctly. Building it once by hand is the only way this mechanism stops being a black box.

### Challenge (optional)

**Part one.** Add a `get_mut(&mut self) -> &mut T` method to `OwnedBox<T>` that returns a mutable reference to the value inside.

**Part two.** Reason about it — no code required: would adding `unsafe impl<T: Sync> Sync for OwnedBox<T> {}` be just as safe? Hint: imagine `OwnedBox<T>` also had a method like `set`, taking only `&self` but writing through `ptr` — such a method is entirely legal to write on this type, because the borrow checker has nothing to say about what happens behind a raw pointer. If `OwnedBox<T>` were `Sync` today, what door would a method like that open later?

---

## Wrapping up

This closes Phase 2 — ten modules, one throughline. `collections` took you from "an array that grows" to real `Vec`/`HashMap`/`HashSet`; `iterators and closures` turned loops into lazy, composable pipelines; `traits and generics` gave you code that works over many types without paying for it at runtime; `lifetimes and conversion` made the borrow checker's promises explicit instead of inferred, plus the honest `From`/`TryFrom` family; `error handling` replaced ad hoc failure with `Result`, `?`, and real error types; `smart pointers` opened the heap properly — `Box`, `Rc`/`Arc`, `RefCell`, and the ownership question each one answers; `project structure and testing` turned a single file into a real, tested crate; `concurrency` gave you real OS threads, `Arc<Mutex<T>>`, and — two lessons ago — `Send`/`Sync` themselves; `async in practice` layered a scheduler on top of that same borrowing model; and this module, the toolbox, closed with three tools every other module quietly assumed you'd eventually reach for: pattern matching in depth, macros, feature flags, and today, `unsafe` itself.

None of it was academic. [Phase 3 — Backend foundations](../../../phase3-backend-foundations/README.md) starts from raw TCP, in [3.1.1 — TCP echo server](../../../phase3-backend-foundations/01-networking-and-http-from-scratch/01-tcp-echo-server/README.md), and everything above is what it's built on.

| Term | What it means | Where you'll use it |
|---|---|---|
| `unsafe` | opens five specific operations; every other rule still applies | raw pointers, FFI, `static mut`, `unsafe trait`, `union` |
| Raw pointer (`*const T`/`*mut T`) | a reference with no null/dangling/aliasing guarantee; safe to create, `unsafe` to dereference | structures the borrow checker can't prove safe on its own |
| `Box::into_raw` / `Box::from_raw` | hand a `Box`'s ownership off to a raw pointer, and back | building your own owning type on a raw pointer |
| `PhantomData<T>` | a zero-sized field saying this type owns a `T` | raw-pointer-based types signaling ownership |
| `unsafe impl Send for X {}` | the type author's personally-verified promise | a type with a raw pointer that behaves like `Box<T>` |

### What you now know

- `unsafe` opens exactly five operations, and the rest of the language — ownership, borrowing, lifetimes — stays fully enforced.
- A raw pointer, unlike `&T`/`&mut T`, can be null or dangling and is exempt from the aliasing rule; creating one is safe, only dereferencing needs `unsafe`.
- `split_at_mut` is the classic "safe Rust can't express this, even though it's obviously safe in practice" example — and raw pointers are exactly what closes that gap.
- A raw pointer, for any `T`, isn't `Send`/`Sync` by default; a type's author can grant it back with `unsafe impl`, but that's a real contract — getting it wrong introduces undefined behavior.
- `PhantomData<T>` tells the compiler a type owns a `T`; in `OwnedBox<T>` it doesn't change what compiles today (since `ptr: *mut T` already mentions `T`), but it's the same honest habit every raw-pointer-based type in the standard library keeps.

### What comes back later

- **FFI, `static mut`, and `union` fields** — this lesson only named them; this course does not come back to them. Their depth is in "Going further."
- **Variance and drop-check** — the sharper detail of exactly where `PhantomData<T>` does change things, which this lesson deliberately stayed clear of; fully in "Going further."

### Can you explain?

- What are the exact five operations `unsafe` opens, and what stays fully checked inside an `unsafe` block?
- Why is creating a raw pointer always safe, but dereferencing one is not?
- Why can't safe Rust write `split_at_mut` directly? What exactly can't the borrow checker see?
- Why does a raw pointer field make a struct `!Send`, even when `T` itself is `Send`?
- What exactly does `unsafe impl<T: Send> Send for OwnedBox<T> {}` promise? What happens to callers if that promise turns out to be wrong?
- What does `PhantomData<T>` do, in your own words? Why does `OwnedBox<T>` keep it when `ptr: *mut T` alone already compiles without it?

---

## Going further

- [The Rustonomicon — Working with Unsafe](https://doc.rust-lang.org/nomicon/working-with-unsafe.html) — these same five operations, from the official reference on unsafe code.
- [The Rustonomicon — PhantomData](https://doc.rust-lang.org/nomicon/phantom-data.html) — variance and drop-check, in more depth than this lesson went into.
- [The Rustonomicon — Send and Sync](https://doc.rust-lang.org/nomicon/send-and-sync.html) — the same chapter 2.8.4 pointed to; writing `unsafe impl` in more detail.
- [`std::boxed::Box::into_raw`](https://doc.rust-lang.org/std/boxed/struct.Box.html#method.into_raw) and [`Box::from_raw`](https://doc.rust-lang.org/std/boxed/struct.Box.html#method.from_raw) — their full safety contracts.
- [`std::marker::PhantomData`](https://doc.rust-lang.org/std/marker/struct.PhantomData.html)
