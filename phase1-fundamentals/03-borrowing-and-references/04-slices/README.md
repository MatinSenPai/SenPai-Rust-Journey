# 1.3.4 — Slices

## At a glance

After this lesson you can:

- Take a borrowed look at part of a `Vec` or an array with `&v[1..4]`, and say why it allocates nothing.
- Write one function that accepts an array, a `Vec`, and a piece of either — by taking `&[i32]` instead of `&Vec<i32>`.
- Say which two things a slice is made of, and why it's twice the size of a plain reference.
- Read the run-time panic you get from a range that leaves the slice, and say what produced it.

**Time:** ~50 minutes · **Prerequisites:** [1.3.3 — Borrow scopes and NLL](../03-borrow-scopes-and-nll/README.md) · [1.1.3 — Compound types](../../01-foundations/03-compound-types-and-destructuring/README.md)

---

## Why this matters

The arrays lesson ended at a dead end. `[i32; 4]` and `[i32; 5]` are different types, so a function that takes an array is stuck at one length forever. That lesson said the way out is a different thing entirely, called a slice. This is that lesson.

But the problem is bigger than array lengths. You have three things that all hold a contiguous run of numbers — an array, a `Vec`, and a piece of either — and without slices you'd write a separate function for each.

In Python you don't think about this at all:

```python
readings = [10, 20, 30, 40, 50]
middle = readings[1:4]
```

And because you don't think about it, you also don't notice that Python just built a brand-new list and copied three elements into it. For five numbers that doesn't matter. For a ten-megabyte file you're slicing inside a loop, that's the thing that made your program slow.

In Rust, `&readings[1..4]` builds nothing. It takes **one address and one count** and hands you that. It isn't cheaper — it costs nothing at all.

---

## The concept

### A view, not a copy

```rust
let readings = vec![10, 20, 30, 40, 50];
let middle = &readings[1..4];

println!("readings:      {readings:?}");
println!("middle:        {middle:?}");
println!("readings[1] @: {:p}", &readings[1]);
println!("middle[0]   @: {:p}", &middle[0]);
```

```text
readings:      [10, 20, 30, 40, 50]
middle:        [20, 30, 40]
readings[1] @: 0x1cd46c1c224
middle[0]   @: 0x1cd46c1c224
```

Look at those two addresses. **They're the same.**

`middle[0]` *is* the Vec's second element, not a copy of it. Nothing was allocated and nothing was moved. All that happened is that something now points into the middle of that buffer.

What you built is a **slice**: a borrowed view of a *contiguous* run of elements inside a larger collection.

And because it's a borrow, everything from the last three lessons still applies: `readings` can't be changed while `middle` is alive, and `middle` stops existing at its last use.

### Two words: where it starts, and how many

```rust
println!("&i32:   {} bytes", std::mem::size_of::<&i32>());
println!("&[i32]: {} bytes", std::mem::size_of::<&[i32]>());
println!("&str:   {} bytes", std::mem::size_of::<&str>());
```

```text
&i32:   8 bytes
&[i32]: 16 bytes
&str:   16 bytes
```

A `&i32` is eight bytes: one address. A `&[i32]` is sixteen — **two words**: the address of the first element, and how many elements there are.

That's all of it. A slice knows those two things and nothing else. It doesn't know whether it came from a Vec or an array, it doesn't know that buffer's capacity, and it doesn't know what comes after it.

```senpai-visual
{"kind":"borrowing","labels":["heap buffer","start address","length","slice"]}
```

This two-word shape is called a **fat pointer**, and it's what gives `&str` sixteen bytes too. That isn't a coincidence, and we come back to it at the end of this lesson.

### Writing the range

```rust
let days = [11, 12, 13, 14, 15, 16, 17];

println!("&days[..]    {:?}", &days[..]);
println!("&days[2..]   {:?}", &days[2..]);
println!("&days[..3]   {:?}", &days[..3]);
println!("&days[1..4]  {:?}", &days[1..4]);
println!("&days[1..=4] {:?}", &days[1..=4]);
println!("&days[3..3]  {:?}", &days[3..3]);
```

```text
&days[..]    [11, 12, 13, 14, 15, 16, 17]
&days[2..]   [13, 14, 15, 16, 17]
&days[..3]   [11, 12, 13]
&days[1..4]  [12, 13, 14]
&days[1..=4] [12, 13, 14, 15]
&days[3..3]  []
```

These are the same ranges you already write inside a `for`. Here, instead of counting, they index.

| Written | Means |
|---|---|
| `&v[..]` | all of it |
| `&v[2..]` | from index 2 to the end |
| `&v[..3]` | from the start up to but not including index 3 |
| `&v[1..4]` | indexes 1, 2 and 3 |
| `&v[1..=4]` | indexes 1 to 4, this time including 4 |
| `&v[3..3]` | the empty slice — not an error |

The start is in and the end is out: `1..4` is three elements, exactly as `for n in 1..4` is three turns. And `..=` pulls the last one in, just as it did in the loop.

That `&` isn't decoration either. `readings[1..4]` means "the run of elements itself", which is not something a variable can hold; `&readings[1..4]` means "a look at that run". Its error is in the errors section.

### Why `&[i32]` is the right parameter type

```rust
fn total(values: &[i32]) -> i32 {
    let mut sum = 0;
    for value in values {
        sum += value;
    }
    sum
}

let fixed = [1, 2, 3, 4];
let grown = vec![10, 20, 30, 40, 50];
println!("array:       {}", total(&fixed));
println!("vec:         {}", total(&grown));
println!("part of vec: {}", total(&grown[1..3]));
```

```text
array:       10
vec:         150
part of vec: 50
```

One function, three completely different shapes of caller. `total` never named a length, a Vec, or an array.

Write that same function as `&Vec<i32>` and all three of those calls turn into two errors: an array isn't a Vec, and a piece of a Vec isn't a Vec either. You'd have narrowed the parameter type and got nothing back for it.

> **Working rule:** if your function only wants to *read* a run of numbers, take `&[i32]`, not `&Vec<i32>`. If it wants to make the run longer or shorter, then it really does need `&mut Vec<i32>` — because `push` is the Vec's job, not the slice's.

clippy says the same thing; the lint is called `ptr_arg`. And the rule carries over to text exactly: take `&str`, not `&String`.

### What a slice will tell you

```rust
let week = &days[..];

println!("len:          {}", week.len());
println!("is_empty:     {}", week.is_empty());
println!("first:        {:?}", week.first());
println!("last:         {:?}", week.last());
println!("contains(14): {}", week.contains(&14));

let (front, back) = week.split_at(3);
println!("split_at(3):  {front:?} and {back:?}");
```

```text
len:          7
is_empty:     false
first:        Some(11)
last:         Some(17)
contains(14): true
split_at(3):  [11, 12, 13] and [14, 15, 16, 17]
```

Look at what `first` and `last` answered: `Some(11)`, not `11`. A slice is allowed to be empty, so both of them wrap their answer — the same wrapper you got from `.get()` back in the arrays lesson. It gets its full lesson in [1.6.1](../../06-absence-and-failure/01-option-and-null-safety/README.md).

`split_at` is worth noticing too: it turns one view into two and still allocates nothing. Two windows onto the same seven numbers.

### `&mut [T]` — a window you can write through

```rust
let mut readings = vec![10, 20, 30, 40, 50];
let window = &mut readings[1..4];

window[0] = 99;
window[2] = 77;

println!("window: {window:?}");
println!("whole:  {readings:?}");
```

```text
window: [99, 30, 77]
whole:  [10, 99, 30, 77, 50]
```

Writing through the window wrote into the Vec. Of course it did: there's only one buffer, and the window points at it.

But notice what you **can't** do. A `&mut [i32]` can change elements, swap them, sort them — it cannot `push` or remove. The slice's length was fixed the moment it was made. Growing and shrinking is the buffer owner's business, not the business of whoever is looking through a window at it.

And the aliasing rule from two lessons ago hasn't changed: a `&mut` slice is exclusive for as long as it's alive, even when the other window doesn't overlap it at all. That's `E0502`, and it's below.

### Running off the end is a run-time panic

```rust
// examples/08-off-the-end.rs
let readings = vec![10, 20, 30, 40, 50];

// `end` could just as easily have come from a file or a request.
let end = 3 + 5;
let off = &readings[2..end];
println!("off:  {off:?}");
```

```text
thread 'main' (13660) panicked at examples\08-off-the-end.rs:17:24:
range end index 8 out of range for slice of length 5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

That code **compiles with no error at all**. The range is checked when the slice is made, not when the code is compiled, because `end` comes from somewhere the compiler can't see in advance.

It's the same split you saw in the arrays lesson between `readings[99]` and `readings.get(99)`: `&v[a..b]` panics, and `v.get(a..b)` hands back an answer that has "maybe there wasn't one" built into it.

There's a second shape of this panic worth seeing. Change `3 + 5` to `3 - 2` in that same file and the range runs backwards:

```text
thread 'main' (31084) panicked at examples\08-off-the-end.rs:17:24:
slice index starts at 2 but ends at 1
```

### `&str` is a string slice

```rust
let owned = String::from("hello world");
let hello = &owned[..5];
let world = &owned[6..];

println!("owned: {owned}");
println!("hello: {hello}");
println!("world: {world}");
```

```text
owned: hello world
hello: hello
world: world
```

The same notation, the same meaning, the same two words in memory. `&str` isn't a separate thing you'll have to learn one day: **it is a string slice** — a borrowed view of a contiguous run of bytes.

The parameter rule carries over too. `&str` is to `String` exactly what `&[i32]` is to `Vec<i32>`: the type that accepts an owned string, a piece of one, and a literal baked into the binary, all three.

There's one important difference, and here we only name it: the indexes of a `&str` are **bytes**, not characters. While the text is English those look like the same thing, but `&name[0..1]` on a Persian string lands in the middle of a character and the program panics:

```text
thread 'main' (22948) panicked at not-a-char-boundary.rs:3:20:
end byte index 1 is not a char boundary; it is inside 'س' (bytes 0..2 of string)
```

That output came from a three-line scratch file rather than this lesson's `examples/`, because the trap belongs to a later lesson — the challenge at the end has you reproduce it. It has a lesson of its own — [1.4.4 — Slicing text safely](../../04-text-and-strings/04-slicing-text-safely/README.md) — and until then, keep your text slicing to English text.

---

## Hands on

```sh
cargo run -p p1-03-04-slices --example 01-a-view-not-a-copy
cargo run -p p1-03-04-slices --example 02-range-forms
cargo run -p p1-03-04-slices --example 03-one-parameter-type
cargo run -p p1-03-04-slices --example 04-mutable-views
```

Then the four broken ones. They sit behind a cargo feature so that running one is a decision you make:

```sh
cargo run -p p1-03-04-slices --example 05-vec-parameter --features broken
cargo run -p p1-03-04-slices --example 06-forgot-the-ampersand --features broken
cargo run -p p1-03-04-slices --example 07-two-views-one-mutable --features broken
cargo run -p p1-03-04-slices --example 08-off-the-end --features broken
```

Then try:

1. In `01-a-view-not-a-copy`, change `&readings[1..4]` to `&readings[..]`. What happens to the addresses?
2. In `02-range-forms`, print `&days[7..]`. Guess before you run it.
3. In `04-mutable-views`, add a `readings.push(60)` after `window` is made. What does the compiler say, and why?
4. In `08-off-the-end`, change `3 + 5` to `3 - 2` and read the new message.

---

## Errors you will meet

### `E0277` — you left the `&` off

```text
error[E0277]: the size for values of type `[{integer}]` cannot be known at compilation time
  --> examples\06-forgot-the-ampersand.rs:11:9
   |
11 |     let middle = readings[1..4];
   |         ^^^^^^ doesn't have a size known at compile-time
   |
   = help: the trait `Sized` is not implemented for `[{integer}]`
   = note: all local variables must have a statically known size
help: consider borrowing here
   |
11 |     let middle = &readings[1..4];
   |                  +
```

**What the compiler is objecting to:** `readings[1..4]` has the type `[i32]` — "a run of `i32`s" with no count attached. The compiler doesn't know how many bytes to set aside on the stack for it, because the length isn't part of that type. Its second note says exactly that: every local variable must have a size known at compile time.

**The fix:**

```rust
let middle = &readings[1..4];
```

**Why that's the fix:** `&[i32]` does have a known size — the two words we counted above. With the `&` you're no longer holding the run itself, you're holding its address and its length, and those are sixteen bytes however long the slice is.

And look at `help: consider borrowing here`: the compiler pointed at the one missing character with a `+`.

### `E0308` — `&Vec<i32>` refuses everything that isn't a Vec

```text
error[E0308]: mismatched types
  --> examples\05-vec-parameter.rs:13:33
   |
13 |     println!("array: {}", total(&fixed));
   |                           ----- ^^^^^^ expected `&Vec<i32>`, found `&[{integer}; 4]`
   |                           |
   |                           arguments to this function are incorrect
   |
   = note: expected reference `&Vec<i32>`
              found reference `&[{integer}; 4]`
note: function defined here
  --> examples\05-vec-parameter.rs:17:4
   |
17 | fn total(values: &Vec<i32>) -> i32 {
   |    ^^^^^ -----------------
```

**What the compiler is objecting to:** an array is not a Vec. A Vec is a three-word structure on the stack owning a buffer on the heap; an array *is* the elements. There's no automatic conversion between them, because no conversion would make sense.

That file gives a second error which says even more: `total(&grown[1..])` is rejected too, with `found reference &[i32]`. The function won't take a piece of the very Vec it just accepted.

**The fix:**

```rust
fn total(values: &[i32]) -> i32 {
```

**Why that's the fix:** a `&Vec<i32>` is two hops to reach the data, and in exchange for that restriction it gives the function nothing — `total` never pushes and never asks about capacity. With `&[i32]` all three calls compile and not one character of the body changes.

### `E0502` — two windows, one of them mutable

```text
error[E0502]: cannot borrow `readings` as mutable because it is also borrowed as immutable
  --> examples\07-two-views-one-mutable.rs:12:21
   |
11 |     let front = &readings[..2];
   |                  -------- immutable borrow occurs here
12 |     let back = &mut readings[3..];
   |                     ^^^^^^^^ mutable borrow occurs here
...
15 |     println!("front: {front:?}");
   |                       ----- immutable borrow later used here
   |
   = help: use `.split_at_mut(position)` to obtain two mutable non-overlapping sub-slices
```

**What the compiler is objecting to:** `&readings[..2]` and `&mut readings[3..]` share no element at all, but both borrow from `readings`, and the compiler doesn't compare ranges. To the borrow checker these are two borrows of one thing: one shared and one exclusive, at the same time.

**The fix:**

```rust
let front = &readings[..2];
println!("front: {front:?}");

let back = &mut readings[3..];
back[0] = 0;
```

**Why that's the fix:** it's the NLL from the previous lesson. If `front` reaches its last use before `back` is created, the first borrow is over and the second is free to happen. Move the lines, don't add a clone.

And take that `help` line seriously: `split_at_mut` exists precisely for when you genuinely need two mutable windows at once. The challenge goes there.

### The run-time panic — a range that leaves the slice

```text
thread 'main' (13660) panicked at examples\08-off-the-end.rs:17:24:
range end index 8 out of range for slice of length 5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**What happened:** the code compiled without a murmur. The problem arrived at **run** time, because the end of the range came from a calculation whose result the compiler didn't know in advance. The message names both numbers: you asked to go to 8, and the length was 5.

**The fix:** either make sure the range stays inside the slice first:

```rust
if end <= readings.len() {
    let off = &readings[2..end];
    println!("off:  {off:?}");
}
```

or use `readings.get(2..end)`, which hands back an answer with "maybe there wasn't one" built into it instead of panicking.

**Why that's the fix:** this panic is still there in release, exactly like the out-of-bounds index in the arrays lesson and unlike integer overflow. "It'll be fine in production" doesn't happen. The only real choice is between checking and accepting that this input takes the program down.

---

## Exercises

### Warm up

<details>
<summary>For a five-element <code>v</code>, what does <code>&amp;v[2..2]</code> give you?</summary>

An empty slice, `[]`. A range whose start and end are the same has no elements in it, and that isn't an error.

</details>

<details>
<summary>How many elements are in <code>&amp;days[1..4]</code>? And <code>&amp;days[1..=4]</code>?</summary>

Three and four. The end of `..` is excluded and the end of `..=` is included — the same range rule as in a `for`.

</details>

<details>
<summary>How many bytes is a <code>&amp;[u8]</code> on a 64-bit machine?</summary>

Sixteen. Eight bytes of address and eight of length. How long the slice is makes no difference to that number.

</details>

<details>
<summary>Does this compile? <code>let view = &amp;v[1..3]; v.push(9); println!("{view:?}");</code></summary>

No. `push` wants a mutable borrow of `v` while `view` is still alive — `E0502`. Move the `println!` above the `push` and it compiles.

</details>

<details>
<summary>Why shouldn't a function that only adds numbers up take <code>&amp;Vec&lt;i32&gt;</code>?</summary>

Because it ties itself to one shape of caller for no reason. With `&[i32]` the same function takes an array, a Vec, and a piece of either, and its body doesn't change.

</details>

<details>
<summary>What does <code>v.split_at(0)</code> give you?</summary>

An empty slice, then the whole of `v`. The empty slice starts nowhere and has no elements.

</details>

### Repair

The four broken files in this lesson are four different errors. Run each one, read the error, then fix it:

1. `examples/06-forgot-the-ampersand.rs` — it's one character short.
2. `examples/05-vec-parameter.rs` — change the signature so all three calls work. Then say how many lines of the function body you had to change.
3. `examples/07-two-views-one-mutable.rs` — without adding a `clone`, only by moving lines.
4. `examples/08-off-the-end.rs` — two ways: once by checking the range, once by asking for an answer that doesn't panic.

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p1-03-04-slices
```

None of them allocates anything. Three of them return a slice — which means their answer is a look at the caller's own numbers, not a copy of them. One of the tests checks exactly that.

Read `middle` and `window_sum` carefully. Both have a case that depends on the input's length and will panic if you don't notice it, and both doc comments say what that case must do.

### Build

Write a `pub fn chunk_totals(values: &[i32], size: usize) -> Vec<i32>` giving the sum of each consecutive group of `size` elements of `values`. The last group is allowed to be short: `chunk_totals(&[1, 2, 3, 4, 5], 2)` should give `[3, 7, 5]`.

Write it with slices and a loop. The standard library has a `chunks` method that does this job, but what it gives back is an iterator, and that's [Phase 2](../../../phase2-intermediate/02-iterators-and-closures/02-iterator-adapters/README.md).

Then write one sentence: why does the function have to handle `size == 0` separately?

### Challenge (optional)

**Part one.** The `E0502` above told you the answer in its own `help` line. Run this:

```rust
let mut readings = vec![10, 20, 30, 40, 50];
let (front, back) = readings.split_at_mut(2);

front[0] = 1;
back[0] = 2;

println!("{front:?} {back:?}");
```

Two mutable slices of one Vec, at the same time, and the compiler is content. **Why is that allowed when writing two separate `&mut` borrows wasn't?** The answer is that `split_at_mut` guarantees the two pieces don't overlap — a guarantee that is itself written with `unsafe`, which [Phase 2](../../../phase2-intermediate/08-rust-toolbox/README.md) goes into.

**Part two.** Run this and read the panic:

```rust
let name = String::from("سلام");
let cut = &name[0..1];
println!("{cut}");
```

Then change `0..1` to `0..2`. Now it works. **Why 2 and not 1?** That's what [1.4.2](../../04-text-and-strings/02-utf8-bytes-chars-graphemes/README.md) and [1.4.4](../../04-text-and-strings/04-slicing-text-safely/README.md) finish.

**Part three.** Run `cargo clippy` over a scratch file containing `fn total(values: &Vec<i32>)` and read the `ptr_arg` lint. Then look up `windows` and `chunks` in the `slice` documentation and see what they hand back.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| slice | a borrowed view of a contiguous run of elements | every function that reads a sequence |
| `&[T]` | shared slice — read only | the right parameter type |
| `&mut [T]` | mutable slice — write, swap, sort | changing in place, with no allocation |
| fat pointer | an address plus a length, two words | `&[T]` and `&str` both |
| `&v[a..b]` | the slice from `a` up to but not including `b` | panics when it leaves the slice |
| `split_at` | one view into two | dividing without copying |
| `&str` | a string slice | the same machinery, over bytes |
| `E0277` | the `&` was left off | a slice without `&` has no size |

### What you now know

- A slice is a view, not a copy: its first element's address *is* the address of that element in the original.
- A slice is two words, an address and a length, which is why `&[i32]` is twice the size of `&i32`.
- `&v[1..4]`, `&v[..]`, `&v[2..]` and `&v[1..=4]` are the same familiar `for` ranges, used as an index.
- `&[T]` is the right parameter type because it accepts arrays, Vecs, and pieces of either.
- `&mut [T]` can change elements but cannot change the length.
- Slicing out of range isn't a compile error; it's a run-time panic, and it's still there in release.
- `&str` is a string slice — this lesson, over bytes instead of numbers.

### What comes back later

- **`Option`, the wrapper `first` and `.get()` put their answers in** — [1.6.1 — `Option` and null safety](../../06-absence-and-failure/01-option-and-null-safety/README.md)
- **Bytes and characters, and why index 1 was inside a character** — [1.4.2 — UTF-8](../../04-text-and-strings/02-utf8-bytes-chars-graphemes/README.md)
- **Slicing text without panicking** — [1.4.4 — Slicing text safely](../../04-text-and-strings/04-slicing-text-safely/README.md)
- **`windows`, `chunks`, and the rest of the methods that hand back iterators** — [Phase 2 — Iterators](../../../phase2-intermediate/02-iterators-and-closures/02-iterator-adapters/README.md)
- **How the compiler knows how long a returned slice stays valid** — [Phase 2 — Lifetimes and elision](../../../phase2-intermediate/04-error-handling-and-lifetimes/03-lifetime-basics-and-elision/README.md)

### Can you explain?

- What is a slice made of, and why is it twice the size of a plain reference?
- Why does `&readings[1..4]` allocate nothing when `readings[1:4]` in Python does?
- Why is `&[i32]` a better parameter type than `&Vec<i32>`?
- What can a `&mut [i32]` do, and what can't it do?
- When is an out-of-range slice caught, and why not sooner?
- What does `&str` have to do with `&[u8]`?

---

## Going further

- [The Rust Book — The Slice Type](https://doc.rust-lang.org/book/ch04-03-slices.html) — the same ground, officially.
- [`std::slice`](https://doc.rust-lang.org/std/primitive.slice.html) — the full list of what a slice can do. Read it top to bottom once; half the things you think you have to write yourself are in there.
- [`clippy::ptr_arg`](https://rust-lang.github.io/rust-clippy/master/#ptr_arg) — the lint that finds `&Vec<i32>` in a signature.
