# 1.3.2 — The rules of the borrow checker

## At a glance

After this lesson you can:

- State the aliasing rule in one sentence and apply it to a real piece of code.
- Read an `E0502` or an `E0499` and point at the label that says which borrow is still alive.
- Restructure a "push while you iterate" loop until the checker accepts it — without reaching for `.clone()`.

**Time:** ~50 minutes · **Prerequisites:**
[1.3.1 — Shared and mutable references](../01-shared-and-mutable-refs/README.md) ·
[1.1.6 — Vec and String basics](../../01-foundations/06-vec-and-string-basics/README.md)

---

## Why this matters

The last lesson stated the rule in a sentence: any number of shared references, or exactly one mutable one, never both. There it was a claim. Here you find out what it has been holding back.

Write this in Python and run it:

```python
names = ["Matin", "Sara"]
for name in names:
    if len(name) > 4:
        names.append("someone")
print(names)
```

It never prints. The loop is walking the same list it keeps growing, so every turn manufactures another element for the next turn, and the program spins until you kill it. No exception, no warning — just a service that never comes up and an empty log.

In C++ the same shape fails worse: the vector's buffer moves, and the iterator in your hand now points at memory that isn't yours any more. The program might work, might crash, and might quietly serve wrong data six months later.

Rust turns all three of those into one compile error. **The aliasing rule isn't bureaucracy; it's the compile-time version of the bugs other languages let you find at run time.** This lesson is that rule, the four errors you see when you break it, and the handful of moves that get you past them.

---

## The concept

### The rule, in one line

Three shared borrows of one `Vec`, all alive at once:

```rust
let first = &scores;
let second = &scores;
let third = &scores;
println!("three readers: {first:?} {second:?} {third:?}");
```

```text
three readers: [10, 20, 30] [10, 20, 30] [10, 20, 30]
```

None of them can write, so none of them can surprise the other two. Now one exclusive borrow, after those three are done with:

```rust
let writer = &mut scores;
writer.push(40);
println!("one writer:    {writer:?}");
```

```text
one writer:    [10, 20, 30, 40]
```

Both pieces compile, because neither one was "both at once". The rule that measures this is one line long:

> **At any point in the program, for any one value: either any number of shared borrows (`&T`), or exactly one mutable borrow (`&mut T`). Never both.**

Its short name is **aliasing XOR mutability**. An *alias* is a second way to reach the same value. The rule says: have as many read-only ways in as you like, or one way in that can write.

### Why only one writer: the buffer can move

```rust
let mut readings = vec![1, 2, 3];
println!("start  len/cap {}/{}", readings.len(), readings.capacity());
println!("       buffer @ {:p}", readings.as_ptr());
readings.push(4);
println!("push4  len/cap {}/{}", readings.len(), readings.capacity());
println!("       buffer @ {:p}", readings.as_ptr());
```

```text
start  len/cap 3/3
       buffer @ 0x1fb4c1c9650
push4  len/cap 4/6
       buffer @ 0x1fb4c1c97b0
```

Room for three, three in it. That `push` had to ask the allocator for a bigger block, copy everything across, and hand the old block back. **The address changed.**

Now the same call, one line later:

```rust
readings.push(5);
println!("push5  len/cap {}/{}", readings.len(), readings.capacity());
println!("       buffer @ {:p}", readings.as_ptr());
```

```text
push5  len/cap 5/6
       buffer @ 0x1fb4c1c97b0
```

This time there was room, so nothing moved. **Same method, same type, two different consequences** — and which one you get depends on the capacity at run time, not on anything visible in the code.

So picture holding a reference to the first element across that:

```rust
let mut readings = vec![1, 2, 3];
let first = &readings[0];
readings.push(4);
println!("first is {first}");
```

```text
error[E0502]: cannot borrow `readings` as mutable because it is also borrowed as immutable
 --> src\main.rs:4:5
  |
3 |     let first = &readings[0];
  |                  -------- immutable borrow occurs here
4 |     readings.push(4);
  |     ^^^^^^^^^^^^^^^^ mutable borrow occurs here
5 |     println!("first is {first}");
  |                         ----- immutable borrow later used here
```

If the compiler let that through, `first` would be pointing into a block that had already been handed back. The program would probably print the right number — freshly freed memory is usually still intact — and then one day it wouldn't. **That "probably" is exactly what Rust removes.** "It worked on my laptop" is not evidence here, so the compiler rejects the shape itself, every time.

```senpai-visual
{"kind":"borrowing","labels":["read","read","read","borrows end","write"]}
```

### The classic failure: pushing while you iterate

```rust
for name in &names {
    if name.len() > 4 {
        names.push(String::from("someone"));
    }
}
```

```text
error[E0502]: cannot borrow `names` as mutable because it is also borrowed as immutable
```

`&names` stays borrowed for the whole loop, because the loop pulls the next element out of it on every turn. `push` wants the `Vec` to itself. This is the code that spun forever in Python — in Rust it neither spins nor runs.

The name for the bug is **iterator invalidation**, and every language with collections and iterators has it. The difference is that Rust finds it at compile time. The full diagnostic gets read apart in "Errors you will meet".

### Two writers at once

Swapping the two ends of a `Vec`, written the way you'd say it out loud:

```rust
let front = &mut scores[0];
let back = &mut scores[2];
let keep = *front;
*front = *back;
*back = keep;
```

```text
error[E0499]: cannot borrow `scores` as mutable more than once at a time
```

Slot 0 and slot 2 are different things, but `scores[0]` and `scores[2]` both borrow through **`scores` itself**, and the compiler does not follow the number inside the brackets. What it sees is two mutable borrows of one `Vec`, and that is enough to refuse.

This is where people decide the borrow checker is stupid. It isn't — it's conservative: every program it accepts has to be safe, so it necessarily rejects some safe ones too. The right way to write this is in the errors section, and the compiler itself suggests it.

### `mut` is not one permission, it's two

```rust
let total = 0;
total = total + 10;

let scores = vec![10, 20];
let writer = &mut scores;
writer.push(30);
```

```text
error[E0384]: cannot assign twice to immutable variable `total`
error[E0596]: cannot borrow `scores` as mutable, as it is not declared as mutable
```

One missing keyword, two different errors — because `mut` on a `let` grants two separate permissions:

| Permission | What you see without it |
|---|---|
| assign to the name again | `E0384` |
| let anybody take a `&mut` to it | `E0596` |

The first is about the binding and you met it in [1.1.1](../../01-foundations/01-variables-mutability-shadowing/README.md). The second is about borrowing and belongs to this lesson. **Until the owner is `mut`, nobody can borrow it mutably** — you cannot hand out a permission you don't have yourself.

### The rule is about one value, not about the program

```rust
let mut names = vec![String::from("Matin")];
let other = &mut names;
other.push(String::from("Sara"));
println!("another value: {other:?}");
println!("scores again:  {scores:?}");
```

```text
another value: ["Matin", "Sara"]
scores again:  [10, 20, 30, 40]
```

`other` is a live mutable borrow, and `scores` is being read in the same breath. No conflict, because **they are different values**. The checker isn't counting how many `&mut` you have in a function; it keeps the books per value.

### When does a borrow end?

```rust
let mut readings = vec![1, 2, 3];
let first = &readings[0];
readings.push(4);
println!("{readings:?}");
```

```text
[1, 2, 3, 4]
```

That is the same code from a few sections ago with one thing removed — the `println!` that read `first` — and it compiles. **A borrow doesn't live to the end of the block; it lives to its last actual use.** Here `first` was never used, so there was no borrow to be in the way.

That one sentence explains most "why did this one pass and that one not" moments. The mechanism and its exact edges are the next lesson: [1.3.3 — Borrow scopes and NLL](../03-borrow-scopes-and-nll/README.md). For now hold on to this: **last use, not last brace.**

### Three ways to restructure

All three do the job that broken example 04 couldn't. None of them fight the rule; each one arranges the code so the rule isn't in the way.

**One — two passes.** Read through a shared borrow and write down what you want to add; then let that borrow finish and do the adding.

```rust
let mut additions = Vec::new();
for name in &names {
    let mut polite = name.clone();
    polite.push_str("-san");
    additions.push(polite);
}
for polite in additions {
    names.push(polite);
}
```

```text
two passes: ["Matin", "Sara", "Matin-san", "Sara-san"]
```

**Two — walk by index.** An index is a number, not a borrow, so nothing stays borrowed between one turn and the next. The length is read once, up front, or the loop chases its own tail.

```rust
let original = scores.len();
for index in 0..original {
    let doubled = scores[index] * 2;
    scores.push(doubled);
}
```

```text
by index:   [10, 20, 30, 20, 40, 60]
```

**Three — copy the value out.** `&scores[0]` is a borrow and stays alive as long as you keep it. `scores[0]` on a `Copy` type is a number that belongs to nobody, and the borrow that produced it is over.

```rust
let front = scores[0];
scores.push(front);
```

```text
copied out: [10, 20, 30, 20, 40, 60, 10]
```

They aren't three tricks; they're one habit: **read, finish, write.** Most borrow errors in real code are that shape and go away in that order.

---

## Hands on

```sh
cargo run -p p1-03-02-borrow-checker-rules --example 01-many-readers-one-writer
cargo run -p p1-03-02-borrow-checker-rules --example 02-why-one-writer
cargo run -p p1-03-02-borrow-checker-rules --example 03-restructuring
```

Then the three broken ones. Running them is a deliberate act, which is why they sit behind a feature:

```sh
cargo run -p p1-03-02-borrow-checker-rules --example 04-push-while-iterating --features broken
cargo run -p p1-03-02-borrow-checker-rules --example 05-two-mutable-borrows --features broken
cargo run -p p1-03-02-borrow-checker-rules --example 06-mut-does-two-jobs --features broken
```

Then try:

1. In `02-why-one-writer`, change `Vec::with_capacity(64)` to `Vec::with_capacity(1)`. Which pushes move the address now?
2. In `04-push-while-iterating`, replace the `push` with a `println!`. Does it compile? Why?
3. In `05-two-mutable-borrows`, delete the three swapping lines but keep both `let`s. Is the error still there? Now delete only `back`.

---

## Errors you will meet

The four errors in this lesson are four faces of one rule. What makes them worth reading is their shape: each one marks **more than one place** in your code, and you haven't understood the error until you've read all of them.

### `E0502` — mutable while a shared borrow is alive

```text
error[E0502]: cannot borrow `names` as mutable because it is also borrowed as immutable
  --> examples\04-push-while-iterating.rs:13:13
   |
11 |     for name in &names {
   |                 ------
   |                 |
   |                 immutable borrow occurs here
   |                 immutable borrow later used here
12 |         if name.len() > 4 {
13 |             names.push(String::from("someone"));
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
```

**What the compiler is objecting to:** it has put three labels on the code, and all three are needed.

1. **Line 11, first label** — `immutable borrow occurs here`: the shared borrow starts here.
2. **Line 11, second label** — `immutable borrow later used here`: and it is used here again, because the loop reads from it every turn. So the borrow is alive until the loop ends.
3. **Line 13** — `mutable borrow occurs here`: and in the middle of that span, `push` wants a mutable borrow.

That both of the first two labels sit on the same `&names` is not an accident: the compiler is saying "this borrow was created here and is still needed here", which is how you know its span covers line 13.

**The fix:** one of the three restructurings above. For this loop, two passes usually reads best:

```rust
let mut additions = Vec::new();
for name in &names {
    if name.len() > 4 {
        additions.push(String::from("someone"));
    }
}
for extra in additions {
    names.push(extra);
}
```

**Why that's the fix:** the first loop only reads and the second only writes, so the two spans never overlap. And look at what that code now guarantees: `additions` is a fixed size by the time the second loop starts, so Python's infinite loop isn't expressible any more. **The restructuring that satisfied the compiler is the same restructuring that removed the bug.**

### `E0499` — more than one mutable borrow

```text
error[E0499]: cannot borrow `scores` as mutable more than once at a time
  --> examples\05-two-mutable-borrows.rs:12:21
   |
11 |     let front = &mut scores[0];
   |                      ------ first mutable borrow occurs here
12 |     let back = &mut scores[2];
   |                     ^^^^^^ second mutable borrow occurs here
13 |
14 |     let keep = *front;
   |                ------ first borrow later used here
   |
   = help: use `.split_at_mut(position)` to obtain two mutable non-overlapping sub-slices
```

**What the compiler is objecting to:** three places again, and this time their order tells the story.

1. **Line 11** — the first mutable borrow of `scores`. Notice the underline is under `scores`, not under `scores[0]`; what got borrowed is the whole `Vec`.
2. **Line 12** — the second borrow, still of the same `scores`. This is where the error is reported.
3. **Line 14** — and the first one is still used afterwards, so you can't argue it was already finished.

Take line 14 away and there is no error at all — back to "when does a borrow end?".

**The fix:** take the values out, then put them back:

```rust
let front = scores[0];
let back = scores[2];
scores[0] = back;
scores[2] = front;
```

**Why that's the fix:** `scores[0]` on an `i32` produces a copy and the borrow ends on the spot. Now only one short borrow is alive at any moment.

And read that `help` line: `split_at_mut` is the standard library's way of getting two `&mut` into two non-overlapping pieces of one collection. Slices are [1.3.4](../04-slices/README.md); for now, notice that when the compiler knows a way, it names it.

### `E0596` — the owner isn't `mut`

```text
error[E0596]: cannot borrow `scores` as mutable, as it is not declared as mutable
  --> examples\06-mut-does-two-jobs.rs:14:18
   |
14 |     let writer = &mut scores;
   |                  ^^^^^^^^^^^ cannot borrow as mutable
   |
help: consider changing this to be mutable
   |
13 |     let mut scores = vec![10, 20];
   |         +++
```

**What the compiler is objecting to:** this one isn't about aliasing, it's about permission. Line 14 asks for something line 13 never granted. Notice the error marker is on line 14 while the suggested repair is on line 13 — **the problem is somewhere the error isn't.**

**The fix:** exactly the `+++` it drew for you: `let mut scores = vec![10, 20];`.

**Why that's the fix:** `&mut` means "I'm borrowing permission to write". If the owner never had that permission, there's nothing to lend. That direction matters, and you'll meet it again the moment a function wants `&mut` and the caller's variable isn't `mut`.

### `E0384` — assigning twice to an immutable binding

```text
error[E0384]: cannot assign twice to immutable variable `total`
  --> examples\06-mut-does-two-jobs.rs:10:5
   |
 9 |     let total = 0;
   |         ----- first assignment to `total`
10 |     total = total + 10;
   |     ^^^^^^^^^^^^^^^^^^ cannot assign twice to immutable variable
   |
help: consider making this binding mutable
   |
 9 |     let mut total = 0;
   |         +++
```

**What the compiler is objecting to:** it remembered where the name first got a value and is telling you that assignment was final. Nothing to do with borrowing — same file, same missing keyword, different error.

**The fix:** either `let mut total = 0;`, or, if what you really wanted was a fresh value, `let total = total + 10;`, which is shadowing and makes a new binding.

**Why that's the fix:** this error stands next to `E0596` to make one point: **`mut` opens two doors and you are usually only thinking about one of them.** When you see either error, ask which door you meant.

---

## Exercises

### Warm up

Answer without running anything, then check if you like.

<details>
<summary>Three <code>&scores</code> alive at once: does it compile?</summary>

Yes. Any number of shared borrows is allowed, because none of them can write.

</details>

```rust
let mut values = vec![1, 2, 3];
let reader = &values;
let writer = &mut values;
writer.push(4);
println!("{reader:?}");
```

<details>
<summary>Which error is this, and which line is to blame?</summary>

`E0502`. The line taking `&mut` is reported — but only because `reader` is still used afterwards. Delete that `println!` and it compiles.

</details>

```rust
let mut left = vec![1];
let mut right = vec![2];
let a = &mut left;
let b = &mut right;
a.push(10);
b.push(20);
```

<details>
<summary>Two <code>&mut</code> alive at once. Why is this not an error?</summary>

Because they borrow two different values. The rule is kept per value, not per function.

</details>

<details>
<summary>What does <code>len/cap 4/6</code> tell you about the push that just happened?</summary>

That capacity had run out and the buffer moved. If there had been room, the capacity would not have changed.

</details>

```rust
let mut readings = vec![1, 2, 3];
let first = &readings[0];
readings.push(4);
println!("{readings:?}");
```

<details>
<summary>This compiles. What if we print <code>first</code> as well?</summary>

Then it's `E0502`. A borrow is alive until its last use, and here there wasn't one. The details are [1.3.3](../03-borrow-scopes-and-nll/README.md).

</details>

<details>
<summary><code>let scores = vec![1];</code> and then <code>&mut scores</code>: which error?</summary>

`E0596`. The owner isn't `mut`, so it has no write permission to lend.

</details>

### Repair

Fix all three broken examples, each in the way asked for:

1. `examples/04-push-while-iterating.rs` **two** ways: once with two passes, once by walking with an index. Then say which reads better here, and why.
2. `examples/05-two-mutable-borrows.rs` without ever holding two `&mut` at the same time.
3. `examples/06-mut-does-two-jobs.rs` with the smallest change possible. Then count: how many words did you add, and how many errors went away?

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p1-03-02-borrow-checker-rules
```

They all take a `&mut Vec<...>`, so for the length of the call you are the single writer. The work is arranging each body so you never need a second borrow while the first is still alive.

Only one of them needs `.clone()` — the one whose specification says a copy of a `String` comes out. If you want to clone anywhere else, you're probably buying your way out of an error instead of restructuring.

One warning to ignore while the bodies are still `todo!()`: clippy will suggest `&mut [i32]` instead of `&mut Vec<i32>`. It cannot see that your finished bodies call `push` and `remove`, and slices are [1.3.4](../04-slices/README.md) anyway. It goes quiet once the functions do something.

### Build

Write a `pub fn insert_after_each(values: &mut Vec<i32>, extra: i32)` that puts one `extra` after **every** current element: `[1, 2]` becomes `[1, extra, 2, extra]`. An empty `Vec` stays empty.

Then answer three things:

1. Did you work forwards with `insert`, or build a fresh `Vec` and replace at the end? Which one allocated less?
2. If you *were* allowed to `insert` into `values` while iterating it, what would happen? Write it in one sentence.
3. That sentence is exactly what `E0502` prevented. Make sure you can say it without looking at the code.

### Challenge (optional)

**Part one.** This compiles. Why?

```rust
let mut counts = vec![1, 2, 3];
counts.push(counts.len() as i32);
```

It needs both a `&mut counts` and a read of `counts`. Hint: arguments are evaluated before the call happens. Search for "two-phase borrow" and read about it — this one reaches forward, and doesn't fully settle until [1.3.3](../03-borrow-scopes-and-nll/README.md).

**Part two.** Rewrite the Python loop from the top of this lesson using `while` and an index so that it genuinely never ends, and let it spin for a few seconds. Now say precisely which step of it Rust made impossible.

**Part three.** Open the docs for [`Vec::swap`](https://doc.rust-lang.org/std/primitive.slice.html#method.swap). It changes two slots at once, while your own code couldn't hold two `&mut`. How? Look at its signature and count the borrows it takes. (This also runs into [1.3.4](../04-slices/README.md) and into `unsafe` in Phase 2.)

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| aliasing | two ways to reach the same value | anywhere two references exist |
| aliasing rule | many `&T`, or one `&mut T`, not both | the whole borrow checker |
| iterator invalidation | changing a collection while walking it | `push` inside a loop |
| data race | two overlapping accesses, one of them writing | Phase 2, but this is the rule |
| `E0502` | mutable while a shared borrow is alive | the number-one borrow error |
| `E0499` | two mutable borrows of one value | swapping two slots |
| `E0596` | the owner isn't `mut` | `&mut` on a plain `let` |
| `E0384` | reassigning an immutable binding | the same `mut`, the other door |

### What you now know

- The rule is: many `&T`, or one `&mut T`, never both — and it's kept per value.
- `push` may move the buffer, and whether it does depends on capacity at run time.
- Which is why the rule is enforced at compile time: "it worked on my machine" is not evidence of safety.
- Pushing while iterating is an infinite loop in Python, freed memory in C++, and an `E0502` in Rust.
- Borrow errors mark more than one place; the "later used here" label is the one holding the span open.
- `mut` is two separate permissions: reassignment (`E0384`) and mutable borrowing (`E0596`).
- A borrow is alive until its last use, not until the end of the block.
- Most borrow errors are solved by "read, finish, write", not by `.clone()`.

### What comes back later

- **Exactly when a borrow ends, and why** — [1.3.3 — Borrow scopes and NLL](../03-borrow-scopes-and-nll/README.md)
- **`split_at_mut` and slices** — [1.3.4 — Slices](../04-slices/README.md)
- **The same rule with two threads in play** — [Phase 2 — Threads, `Mutex` and `Arc`](../../../phase2-intermediate/07-concurrency-and-async/01-threads-mutex-arc/README.md)
- **When the check genuinely has to move to run time** — [Phase 2 — `RefCell` and interior mutability](../../../phase2-intermediate/05-smart-pointers/03-refcell-and-interior-mutability/README.md)
- **Naming how long borrows last, in a signature** — [Phase 2 — Lifetime basics](../../../phase2-intermediate/04-error-handling-and-lifetimes/03-lifetime-basics-and-elision/README.md)

### Can you explain?

- State the aliasing rule in one sentence.
- Why can a `Vec` change address when you push to it, and what has that got to do with the rule?
- In an `E0502`, what does the "later used here" label prove?
- What is the difference between `E0499` and `E0596`?
- What two permissions does `mut` grant on a `let`?
- Name the three ways to restructure a "push while iterating" loop.
- Why is `.clone()` usually the wrong answer to a borrow error?

---

## Going further

- [The Rust Book — the rules of references](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html) — the same ground, officially.
- [`rustc --explain E0502`](https://doc.rust-lang.org/error_codes/E0502.html) — every code in this lesson has one of these. Get in the habit of running it on your next error.
- [`std::vec::Vec` — capacity and reallocation](https://doc.rust-lang.org/std/vec/struct.Vec.html#capacity-and-reallocation) — the promise the standard library makes about the buffer moving, in its own words.
