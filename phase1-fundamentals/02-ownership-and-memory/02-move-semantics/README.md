# 1.2.2 — Move semantics

## At a glance

After this lesson you can:

- Say exactly what `let b = a;` does when `a` is a `String` — and why `a` is unusable afterwards.
- See `E0382` and know what to look at, instead of sprinkling `.clone()` over it.
- Say why `let b = a;` is perfectly fine when `a` is an `i32`.
- Say why `for x in v` swallows the vector and `for x in &v` doesn't.

**Time:** ~50 minutes · **Prerequisites:** [1.2.1 — Stack and heap](../01-stack-and-heap/README.md)

---

## Why this matters

This is the lesson that makes Rust Rust.

In Python this is entirely safe:

```python
first = ["a", "b"]
second = first
second.append("c")
print(first)   # ['a', 'b', 'c']  ← the first one changed too
```

Both names refer to one list. That's fine if you expected it and a source of bugs if you didn't, and Python offers no help telling those apart.

In C++ that same line makes a full deep copy — safe, but potentially quietly expensive; or, if you wrote the class yourself and got it wrong, **two objects become responsible for one buffer and both free it.** That's the double free from [1.2.1](../01-stack-and-heap/README.md).

Rust gives a third answer that no widely-used language had made the default before: **the value moves.** `second` becomes the owner and `first` stops being one. No copy, no sharing, no ambiguity. And if you use `first` afterwards, the program doesn't compile.

This is the one part of Rust you genuinely have to *learn* rather than memorise. Get this module and the rest of the language follows behind it.

---

## The concept

### What `let b = a;` really does

```rust
let first = String::from("hello");
let second = first;
```

From [1.2.1](../01-stack-and-heap/README.md) you know `first` is three words on the stack pointing at a buffer on the heap.

That assignment **copies those three words.** The buffer doesn't move. You now have two sets of three words pointing at one buffer.

And there's the problem: both `first` and `second` would be freed at the closing brace. Two frees for one buffer. **A double free.**

Rust solves it with a decision that stands out for its bluntness: **after that assignment, `first` is no longer a valid binding.** Using it is a compile error. One buffer, one owner, one free.

That's a **move**. What moved wasn't the data — it was the **responsibility**.

```text
second:     hello
length:     5
third:      hello
its buffer: 0x22a8d281890
```

### A move is cheap, and always the same price

```rust
let large = vec![0_u8; 10_000_000];
let also_large = large;
```

```text
ten MB moved, 24 bytes copied
```

Ten megabytes "moved" and what actually got copied was twenty-four bytes. **A move's cost doesn't depend on the size of the data**, because the data doesn't go anywhere. Three words.

That's an important thing to internalise early: handing a large `Vec` to a function is not an expensive copy. It's free. What's expensive is `.clone()`, and you have to write that out.

### What moves and what doesn't

```rust
let a = 5_i32;
let b = a;
println!("a: {a}, b: {b}");     // both work
```

```text
a: 5, b: 5
bool:  true true
char:  س س
f64:   1.5 1.5
array: [1, 2, 3] [1, 2, 3]
tuple: (1, true) (1, true)
```

`a` still works afterwards. Why?

Because an `i32` owns nothing. The whole value is those four bytes; copying them copies the entire thing, and there's no buffer for anyone to be responsible for. Two owners of nothing is not a problem.

That's exactly where the line falls:

| Copies | Moves |
|---|---|
| `i32`, `u8`, `usize`, `f64` | `String` |
| `bool`, `char` | `Vec<T>` |
| `&T` | anything containing one of those |
| arrays and tuples where everything inside copies | |

And that last row matters:

```rust
let labelled = (1_i32, String::from("owned"));
let labelled_again = labelled;
// println!("{labelled:?}");   // <- E0382
```

A tuple with a `String` in it moves, because now there's a buffer someone has to be responsible for.

> The trait that decides which column a type is in is called `Copy`, and [1.2.3](../03-clone-and-copy/README.md) does it properly. For now the working rule is: **does it own something on the heap? Then it moves.**

### Now that `&` from 1.1.6 makes sense

```rust
for line in &lines {
    total += line.len();
}
```

```text
total bytes: 14
still here:  ["alpha", "beta", "gamma"]
```

Against:

```rust
for line in lines {
    // ...
}
// println!("{lines:?}");   // <- E0382
```

Without the `&`, the loop **takes ownership of the vector**. Afterwards `lines` doesn't exist.

With it, the loop only borrows. It looks at the elements and the vector stays yours.

Which do you write? **Lend unless you mean to consume.** The consuming form is right when you genuinely want the elements themselves — not just facts about them.

And one trap worth seeing:

```rust
let numbers = vec![1, 2, 3];
for n in numbers {
    sum += n;
}
// numbers is gone
```

`i32` copies, so why did `numbers` go? Because **the `Vec` moved, not the `i32`s.** The `Vec` owns its buffer, and the loop took that ownership. Whether its elements happen to be copyable has nothing to do with it.

### Why Rust chose this

Worth seeing plainly, because it explains why the language is this shape and not another.

| Approach | What it does | The price |
|---|---|---|
| **share** (Python, Java) | both names refer to one object | changing one changes the other; nobody can say when to free it |
| **deep copy** (C++ default) | a complete duplicate | expensive and invisible; write it wrong and you get a double free |
| **move** (Rust) | transfer of responsibility | the source becomes unusable |

Move is the only option that's both cheap and safe. Its price is that you have to keep track of who owns what — and keeping track of that for you is exactly the compiler's job.

---

## Hands on

```sh
cargo run -p p1-02-02-move-semantics --example 01-a-move
cargo run -p p1-02-02-move-semantics --example 02-what-moves
cargo run -p p1-02-02-move-semantics --example 03-loops-and-moves
```

Then the two broken ones:

```sh
cargo run -p p1-02-02-move-semantics --example 04-use-after-move --features broken
cargo run -p p1-02-02-move-semantics --example 05-move-out-of-a-vec --features broken
```

Then try:

1. In `01-a-move`, uncomment the `println!("first: ...")` line. Read the whole error — it points at three different things.
2. In `02-what-moves`, uncomment the `labelled` line. Why does a tuple containing a `String` behave differently from `(1, true)`?
3. In `03-loops-and-moves`, add a `&` to the second loop. Does it compile now? What else has to change?

---

## Errors you will meet

### `E0382` — use after move

This is the commonest Rust error after the semicolon, and the one that drives people away from the language. Read it properly and you'll see it less.

```text
error[E0382]: borrow of moved value: `first`
  --> examples\04-use-after-move.rs:11:24
   |
 6 |     let first = String::from("hello");
   |         ----- move occurs because `first` has type `String`, which does not implement the `Copy` trait
 7 |     let second = first;
   |                  ----- value moved here
...
11 |     println!("first:  {first}");
   |                        ^^^^^ value borrowed here after move
   |
help: consider cloning the value if the performance cost is acceptable
   |
 7 |     let second = first.clone();
   |                       ++++++++
```

**What the compiler is objecting to:** the error points at three separate places and all three are needed:

1. **Line 6** — why it moved at all: `String` doesn't implement `Copy`.
2. **Line 7** — where it moved.
3. **Line 11** — where it was used afterwards.

Together they're a complete account: made here, given away here, wanted here.

**The fix:** it depends on what you actually meant, and **that's the question to ask yourself**:

| If you wanted | Write |
|---|---|
| just to look at the value | `let second = &first;` — [1.3.1](../../03-borrowing-and-references/01-shared-and-mutable-refs/README.md) |
| two genuinely independent ones | `let second = first.clone();` |
| only one | reorder the code so you don't need `first` after the move |

**Why that's the "fix":** look at that `help` and at the phrase inside it — "*if the performance cost is acceptable*". The compiler is suggesting `.clone()` **and simultaneously telling you it isn't free.**

Take that seriously. `.clone()` will silence any `E0382`, and that's what makes it dangerous: beginners learn to sprinkle it over every error and then wonder why their Rust is slower than their Python. **Before every `.clone()`, ask whether you really need two independent copies.** Nine times in ten the answer is no and what you wanted was a `&`.

### `E0507` — moving out of a `Vec`

```text
error[E0507]: cannot move out of index of `Vec<String>`
  --> examples\05-move-out-of-a-vec.rs:10:17
   |
10 |     let first = lines[0];
   |                 ^^^^^^^^ move occurs because value has type `String`, which does not implement the `Copy` trait
   |
help: consider borrowing here
   |
10 |     let first = &lines[0];
   |                 +
help: consider cloning the value if the performance cost is acceptable
   |
10 |     let first = lines[0].clone();
   |                         ++++++++
```

**What the compiler is objecting to:** `lines[0]` wants to take that `String` out. But then the `Vec` would have a hole in it — a position holding no valid value — and there's no such thing. Every position from 0 to `len` must be valid.

**The fix:** again, it depends what you want:

| If you wanted | Write |
|---|---|
| a look at it | `&lines[0]` |
| a copy of your own | `lines[0].clone()` |
| **actually to take it out** | `lines.remove(0)` |

**Why that's the fix:** `remove(0)` is legal because it repairs the hole: it gives you the element and shifts everything else down, so the `Vec` is one shorter and still has no holes.

Notice the compiler didn't suggest `remove` — it suggested its two safer options. Compiler help is excellent but it isn't mind-reading; **deciding what you meant is your job.**

---

## Exercises

### Warm up

<details>
<summary>What does <code>let b = a;</code> do when <code>a</code> is a <code>String</code>?</summary>

Copies the three stack words and invalidates `a`. The heap buffer doesn't move; only responsibility for it transfers.

</details>

<details>
<summary>How many bytes are copied when you move a <code>Vec</code> of ten million items?</summary>

Twenty-four. Three words, like every other move. The size of the data is irrelevant.

</details>

<details>
<summary>Why is <code>let b = a;</code> fine when <code>a</code> is an <code>i32</code>?</summary>

Because an `i32` owns nothing. The whole value is those four bytes and copying them copies the whole thing. There's no buffer for two people to be responsible for.

</details>

<details>
<summary>What's the difference between <code>(1, true)</code> and <code>(1, String::new())</code>?</summary>

The first copies, because everything in it copies. The second moves, because that `String` has a heap buffer.

</details>

<details>
<summary>What's the difference between <code>for x in v</code> and <code>for x in &amp;v</code>?</summary>

The first takes ownership of `v` and afterwards `v` doesn't exist. The second borrows it and `v` survives.

</details>

<details>
<summary>Why doesn't <code>lines[0]</code> compile on a <code>Vec&lt;String&gt;</code>?</summary>

Because taking that element out would leave a hole in the `Vec`, and a `Vec` can't have holes. `remove(0)` is legal because it closes the hole at the same time.

</details>

<details>
<summary>When you get an <code>E0382</code>, what's the first question you ask?</summary>

"Do I actually need two independent copies?" If not — and usually not — what you wanted was a `&`, not a `.clone()`.

</details>

### Repair

Fix `examples/04-use-after-move.rs` **three** different ways:

1. With `.clone()`.
2. With `&`, no clone at all.
3. By reordering the lines, with neither clone nor `&`.

Then say which is best and why. Your answer should contain the word "allocation".

Then fix `examples/05-move-out-of-a-vec.rs` so `first` really is an owned `String` — not a reference and not a clone.

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p1-02-02-move-semantics
```

All five take ownership of something and hand back something new. That shape isn't accidental: it's what Rust code looks like when values move.

One of the five needs to be `mut` to do its job, and the signature doesn't say so. Work out which, and why you're allowed to add it yourself.

### Build

Write a `pub fn drain_into(source: Vec<String>, mut target: Vec<String>) -> Vec<String>` that moves everything from `source` onto the end of `target` and returns `target`.

Then answer this: **how many allocations happened?** Experiment with `Vec::with_capacity` and see whether you can get it to zero.

Then change the signature to `drain_into(source: Vec<String>, target: Vec<String>)` — no `mut`. Does it still work? Why?

### Challenge (optional)

**Part one.** Does this compile? If not, which line exactly, and why?

```rust
let a = String::from("x");
let b = a;
let a = b;
println!("{a}");
```

**Part two.** What about this?

```rust
let mut a = String::from("x");
let b = a;
a = String::from("y");
println!("{a} {b}");
```

If that surprises you, work out why. Hint: a moved-from binding isn't destroyed, it's **uninitialised**.

**Part three.** Run this and explain the addresses:

```rust
let first = String::from("hello");
println!("{:p}", first.as_ptr());
let second = first;
println!("{:p}", second.as_ptr());
let third = second;
println!("{:p}", third.as_ptr());
```

Three moves — and what did the address do? Now say in one sentence what a move actually is.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| move | ownership transfers; the source is invalidated | every assignment or hand-off of an owning type |
| owner | the one binding responsible for freeing | everywhere |
| `Copy` | types that copy instead of moving | numbers, `bool`, `char` |
| `E0382` | use after move | this module's commonest error |
| `E0507` | moving out of a collection | `vec[0]` on a `Vec<String>` |
| `.clone()` | a genuinely independent duplicate | only when you really want two |
| `remove(i)` | takes an element out and shifts the rest | legal removal from a `Vec` |
| `mut` on a parameter | I'll modify it inside the body | not part of the signature |

### What you now know

- Assignment transfers responsibility for owning types, not data.
- A move is always three words, however large the data.
- The source is invalidated on purpose; that's what prevents a double free.
- Types that own nothing on the heap copy rather than move.
- `for x in v` consumes and `for x in &v` borrows.
- `E0382` points at three places, and `.clone()` is not the default answer.

### What comes back later

- **The `Copy` trait and what `.clone()` really costs** — [1.2.3](../03-clone-and-copy/README.md)
- **Moving into and out of functions** — [1.2.4](../04-ownership-across-functions/README.md)
- **Code that runs on the final move** — [1.2.5 — `Drop`](../05-drop-and-raii/README.md)
- **`&`, in full** — [1.3.1 — References](../../03-borrowing-and-references/01-shared-and-mutable-refs/README.md)
- **When you genuinely need two owners** — [Phase 2 — `Rc` and `Arc`](../../../phase2-intermediate/05-smart-pointers/02-rc-and-arc/README.md)

### Can you explain?

- What does `let b = a;` do when `a` is a `String`?
- Why is moving a ten-megabyte vector cheap?
- Why doesn't an `i32` move?
- Why is the source invalidated, and what would happen if it weren't?
- What's the difference between `for x in v` and `for x in &v`?
- When you see `E0382`, what three questions do you ask?

---

## Going further

- [The Rust Book — Ownership](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#variables-and-data-interacting-with-move) — the same ground, officially, with diagrams.
- [`rustc --explain E0382`](https://doc.rust-lang.org/error_codes/E0382.html) — run it in the terminal too. Get in the habit: every error code has one.
- [The Rustonomicon — Ownership](https://doc.rust-lang.org/nomicon/ownership.html) — heavier and more precise. Not now; know it exists.
