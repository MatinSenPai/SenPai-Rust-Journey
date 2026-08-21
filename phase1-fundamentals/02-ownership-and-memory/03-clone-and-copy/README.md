# 1.2.3 — `Clone` and `Copy`

## At a glance

After this lesson you can:

- Say exactly what `.clone()` costs — and how many allocations it makes for a `Vec<String>`.
- Say why a `String` can never be `Copy`, however short it is.
- State the difference between "copied" and "cloned", and which one *you* write.
- Look at an `E0382` and decide whether `.clone()` is the answer or just a way to silence it.

**Time:** ~45 minutes · **Prerequisites:** [1.2.2 — Move semantics](../02-move-semantics/README.md)

---

## Why this matters

The last lesson ended with a warning: `.clone()` will silence any `E0382`, and that's what makes it dangerous.

This lesson turns that warning into a number.

The usual path for a new Rust programmer goes: borrow error, add `.clone()`, error gone, repeat. Six months later they have a codebase with thousands of `.clone()` calls scattered through it that's slower than the Python it replaced, and they conclude Rust wasn't worth it.

The problem wasn't `.clone()`. The problem was never measuring it.

So this lesson measures. And while it's at it, it clears up a distinction people stay confused about for months: **`Copy` is not a fast `Clone`. It's a different promise.**

---

## The concept

### What `.clone()` actually does

```rust
let original = String::from("hello");
let copy = original.clone();
```

```text
original:   hello
copy:       hello
original @: 0x1c3fea31920
copy     @: 0x1c3fea31940
```

Look at those two addresses. **They're different.**

The last lesson showed a move leaving the address alone: three moves, one address. A clone changes it, because a clone asked the allocator for a **second** buffer and copied the contents into it.

That's the whole difference, in two lines of output:

| | Address | Allocations |
|---|---|---|
| move | stays the same | none |
| clone | changes | one (at least) |

### The cost scales with the depth of the data

```rust
let lines = vec![
    String::from("alpha"),
    String::from("beta"),
    String::from("gamma"),
];
let copied = lines.clone();
```

```text
vec     @: 0x1c3fea32160
its copy@: 0x1c3fea31f80
line 0  @: 0x1c3fea31960
its copy@: 0x1c3fea319c0

that clone made 4 allocations: the Vec, and one per String
```

The Vec's own buffer is new, and so is every string's. **Cloning a `Vec<String>` of `n` elements is `n + 1` allocations.**

That's the number to keep in your head. `.clone()` on a `Vec<String>` holding a thousand lines goes to the allocator a thousand and one times. Inside a loop, multiply by the number of turns.

### One useful side effect

```text
roomy len/cap:  3/100
clone len/cap:  3/3
```

**Cloning doesn't preserve capacity.** It allocates exactly the length. So `.clone()` is one way to shrink an over-sized buffer — though `shrink_to_fit()` says your intent better and doesn't briefly hold both buffers at once.

### `Copy` — and why it isn't a fast clone

```rust
let a = 5_i32;
let b = a;
println!("a: {a}, b: {b}");
```

```text
a: 5, b: 5
```

No method call. Nothing written by you. `a` still works.

Types with that property implement the **`Copy`** trait, and what it means is one sentence:

> **Copying this value's bytes copies the whole value, and there's nothing left to clean up.**

For an `i32` that's obvious: four bytes, done. For a `String` it's false: copying those three words creates two things responsible for one buffer. That's why `String` can't be `Copy` — and how short the string is has nothing to do with it.

| `Copy` | not `Copy` |
|---|---|
| `i32`, `u8`, `usize`, `f64` | `String` |
| `bool`, `char` | `Vec<T>` |
| `&T` | anything containing one of those |
| arrays and tuples where every part is `Copy` | anything with a `Drop` |

### How the two relate

This is where the misunderstanding always is, so plainly:

```rust
let c = a.clone();   // works on an i32
```

**Every `Copy` type is also `Clone`.** The language requires it. So `.clone()` on an `i32` is allowed — and does precisely what the assignment did, with a longer name. clippy will point it out with `clone_on_copy`.

**The reverse doesn't hold.** `String` is `Clone` and will never be `Copy`.

So the real difference isn't which is faster. It's this:

| | Who writes it | When it happens |
|---|---|---|
| `Copy` | nobody — it's implicit | on every assignment |
| `Clone` | you, explicitly | only where you write `.clone()` |

**`Copy` changes what assignment does by default. `Clone` adds a method.** That's why Rust makes you type `.clone()`: an allocation should be visible in the code.

### A reference is `Copy`

Easy to skim past and confusing later:

```rust
let owned = String::from("hello");
let first = &owned;
let second = first;
println!("both refs work: {first} / {second}");
```

```text
both refs work: hello / hello
```

`first` still works after the assignment, because `&String` is `Copy`. **Copying the arrow doesn't copy what's at the end of it.** You have a second arrow to the same thing, and neither of them owns anything.

That also sets a trap you'll see in the errors section: `.clone()` on something that is a reference may clone the reference rather than its target.

### So when should you clone?

Three situations, three answers:

```rust
// 1. You only need to read it → borrow
println!("length: {}", length_of(&name));

// 2. You keep one and give one away → clone once, deliberately
let for_them = template.clone();

// 3. You're done with it → move it
let consumed = consume(finished);
```

```text
length:     5
still ours: Matin
kept:       report-
given:      report-
consumed:   4
```

And the mistake to recognise:

```rust
for line in &lines {
    let copy = line.clone();     // allocates
    wasteful += copy.len();
}                                 // and frees, having learnt nothing
```

```text
same answer: 14 / 14
first version allocated 3 times for nothing
```

Same answer, three wasted allocations. `.len()` only needed to look.

> **Working rule:** before every `.clone()`, ask "do I actually need two independent copies?" If the answer is no, what you wanted was a `&`. And if the answer is yes, clone and don't feel bad about it — one honest clone beats a complicated architecture built to avoid it.

### `Clone` on your own types

When you get to [1.5.1](../../05-your-own-types/01-structs-and-methods/README.md) and write your own type, you'll write this:

```rust
#[derive(Clone)]
struct Reading { /* ... */ }
```

`derive` means "write the obvious implementation for me": clone every field. And if every field is `Copy`, you can write `#[derive(Clone, Copy)]`. If one field is a `String`, you can't, and that error is in the next section.

---

## Hands on

```sh
cargo run -p p1-02-03-clone-and-copy --example 01-what-clone-costs
cargo run -p p1-02-03-clone-and-copy --example 02-copy-types
cargo run -p p1-02-03-clone-and-copy --example 03-when-to-clone
```

Then the two broken ones:

```sh
cargo run -p p1-02-03-clone-and-copy --example 04-copy-needs-no-heap --features broken
cargo run -p p1-02-03-clone-and-copy --example 05-cloning-a-reference --features broken
```

Then try:

1. In `01-what-clone-costs`, raise the number of strings to ten. How many allocations does that clone make now?
2. In `02-copy-types`, keep `let c = a.clone();` and run `cargo clippy`. What does it say?
3. In `03-when-to-clone`, print `copy.as_ptr()` inside the "wasteful" loop. Is it the same each turn or different?

---

## Errors you will meet

### `E0204` — `Copy` on something that owns memory

```text
error[E0204]: the trait `Copy` cannot be implemented for this type
  --> examples\04-copy-needs-no-heap.rs:9:8
   |
 8 | #[derive(Clone, Copy)]
   |                 ---- in this derive macro expansion
 9 | struct Reading {
   |        ^^^^^^^
10 |     value: i32,
11 |     label: String,
   |     ------------- this field does not implement `Copy`
```

**What the compiler is objecting to:** `Copy` means "copying the bytes copies the whole value". That `String` has a heap buffer, so copying its bytes would create two things responsible for one buffer. The promise can't be made.

**The fix:** drop `Copy` and derive only `#[derive(Clone)]`. Duplication is now possible but you have to write `.clone()`.

**Why that's the fix:** notice how precise the error is — it underlines **the exact field** at fault. On a twenty-field struct that's the difference between one second and ten minutes.

And it isn't a punishment. Not being `Copy` means "duplicating this thing costs something and should be visible in the code" — which, for anything that owns memory, is true.

### `E0308` — cloning the arrow instead of the target

```text
error[E0308]: mismatched types
  --> examples\05-cloning-a-reference.rs:11:25
   |
11 |     let first: String = lines.first().clone();
   |                ------   ^^^^^^^^^^^^^^^^^^^^^ expected `String`, found `Option<&String>`
   |                |
   |                expected due to this
   |
   = note: expected struct `String`
                found enum `Option<&String>`
```

**What the compiler is objecting to:** `lines.first()` doesn't give you the string; it gives you a **look** at it, wrapped in an `Option` in case there isn't one. `.clone()` on that clones the wrapper — and since a reference is `Copy`, cloning it produces the same reference again. No string was copied.

**The fix:**

```rust
let first: Option<String> = lines.first().cloned();
```

`.cloned()` — with the `d` — is "clone what's inside", not "clone this".

**Why that's the fix:** this error usually means you're cloning too early. `lines.first()` is already the look you wanted; if you only meant to read its length, no clone was needed at all.

And get used to the `.clone()` versus `.cloned()` distinction, because it recurs throughout Rust. The first works on *this value*, the second on *the value inside this wrapper*. `Option` gets its full lesson in [1.6.1](../../06-absence-and-failure/01-option-and-null-safety/README.md).

---

## Exercises

### Warm up

<details>
<summary>What does <code>.clone()</code> on a <code>String</code> change that a move didn't?</summary>

The buffer's address. A clone allocates a second buffer and copies the contents into it; a move leaves the buffer exactly where it was.

</details>

<details>
<summary>How many allocations does cloning a <code>Vec&lt;String&gt;</code> of a thousand elements make?</summary>

A thousand and one. One for the Vec and one per string.

</details>

<details>
<summary>Why can't <code>String</code> be <code>Copy</code>?</summary>

Because copying its three words would make two things responsible for one buffer, and both would free it. How long the string is doesn't come into it.

</details>

<details>
<summary>Is every <code>Clone</code> type <code>Copy</code>? The other way round?</summary>

No and yes. Every `Copy` type must be `Clone` (the language requires it), but plenty of `Clone` types will never be `Copy`.

</details>

<details>
<summary>The main difference between <code>Copy</code> and <code>Clone</code>, in one sentence?</summary>

`Copy` changes what assignment does automatically; `Clone` is a method you call yourself. Which means every allocation is visible in the code.

</details>

<details>
<summary>Why does <code>let second = first;</code> leave both alive when <code>first</code> is a <code>&amp;String</code>?</summary>

Because `&T` is `Copy`. The arrow was copied, not the thing at the end of it.

</details>

### Repair

Fix `examples/04-copy-needs-no-heap.rs` **two** ways:

1. By removing `Copy` from the derive.
2. By changing the field's type so that `Copy` genuinely becomes possible.

Then say which is more likely right in a real program, and why the question can't be answered until you know what that `label` is for.

Then fix `examples/05-cloning-a-reference.rs` two ways: once with `.cloned()`, once with no clone at all.

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p1-02-03-clone-and-copy
```

Two of them need a clone. Two don't — and would still pass if you added one, which is exactly the problem. One needs precisely `n - 1` clones rather than `n`.

Read `array_survives` carefully. It does something that was `E0382` in the last lesson, and why it's allowed here is the whole of today.

### Build

Write a `pub fn cheapest_join(parts: Vec<String>, separator: char) -> String` doing the same job as `joined` from [1.1.6](../../01-foundations/06-vec-and-string-basics/README.md) — but this time **with no clones at all**.

Then count: how many times did your version allocate? Now try `String::with_capacity` and see whether you can get it to one.

Then write a sentence on why consuming `parts` here is cheaper than borrowing it.

### Challenge (optional)

**Part one.** How many allocations does this make? Guess, then check by printing the addresses:

```rust
let a = vec![String::from("x"), String::from("y")];
let b = a.clone();
let c = b;
let d = c.clone();
```

**Part two.** Run this and explain both lines of output:

```rust
let text = String::from("hello");

let r: &String = &text;
let cloned_ref = r.clone();
println!("{:p} {:p}", r.as_ptr(), cloned_ref.as_ptr());

let rr: &&String = &r;
let cloned_rr = rr.clone();
println!("{:p} {:p}", rr.as_ptr(), cloned_rr.as_ptr());
```

One of those pairs of addresses matches and the other doesn't. **Which, and why?** And read the warning the compiler gives on one of them — it says exactly what happened. (Hint: search for "auto-deref".)

**Part three.** Run `cargo clippy` over this whole lesson. Which lints are about cloning? Look one of them up in the documentation and read it.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `Clone` | a method producing an independent duplicate | when you genuinely want two |
| `Copy` | assignment duplicates and the source survives | numbers, `bool`, `char`, `&T` |
| `.clone()` | explicit, allocates | should be visible in the code |
| `.cloned()` | clones what's inside a wrapper | `Option<&T>` to `Option<T>` |
| `#[derive(Clone)]` | write the obvious implementation | your own types, from 1.5.1 |
| `E0204` | `Copy` on something owning memory | the offending field is underlined |
| clone depth | `n + 1` allocations for a `Vec<String>` | the number to know |

### What you now know

- A clone changes the address because it takes a second buffer; a move doesn't.
- Cloning a `Vec<String>` of `n` elements makes `n + 1` allocations.
- A clone doesn't preserve capacity; it takes exactly the length.
- `Copy` means a byte copy is enough and there's nothing to clean up.
- Every `Copy` type is `Clone`; the reverse isn't true.
- `&T` is `Copy` even when `T` isn't.
- The last use of a value should be a move, not a clone.

### What comes back later

- **Borrowing, which is the right answer to most clones** — [1.3.1 — References](../../03-borrowing-and-references/01-shared-and-mutable-refs/README.md)
- **`derive` on your own types** — [1.5.1 — Structs](../../05-your-own-types/01-structs-and-methods/README.md)
- **`.cloned()` and the `Option` family** — [1.6.1](../../06-absence-and-failure/01-option-and-null-safety/README.md)
- **`Cow`, for when you sometimes need a clone and sometimes don't** — [Phase 2 — Copy on write](../../../phase2-intermediate/04-error-handling-and-lifetimes/README.md)
- **Shared ownership, when you really do want two owners** — [Phase 2 — `Rc` and `Arc`](../../../phase2-intermediate/05-smart-pointers/02-rc-and-arc/README.md)

### Can you explain?

- What does `.clone()` produce that a move didn't?
- How many allocations does cloning a hundred-element `Vec<String>` make?
- Why can't `String` be `Copy`?
- State the difference between `Copy` and `Clone` in one sentence.
- Why is `&T` `Copy` even when `T` isn't?
- What question do you ask yourself before writing `.clone()`?

---

## Going further

- [The Rust Book — Clone and Copy](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#stack-only-data-copy) — the same ground, officially.
- [`std::marker::Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html) — its documentation has a whole section called "When can my type be `Copy`?". Worth reading.
- [`clippy::redundant_clone`](https://rust-lang.github.io/rust-clippy/master/#redundant_clone) — the lint that finds pointless clones. Run `cargo clippy` regularly.
