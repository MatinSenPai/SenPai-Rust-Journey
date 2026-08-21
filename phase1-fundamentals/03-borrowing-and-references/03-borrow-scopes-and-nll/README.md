# 1.3.3 — Borrow scopes and NLL

## At a glance

After this lesson you can:

- Point at the exact line a borrow ends on, and say why it's there and not at the closing brace.
- Read the third label of an `E0502` and know the compiler is telling you where the borrow ends.
- Decide whether a borrow error wants a line moved, a `{ }` block, or the code restructured.
- Say why `items.push(items.len())` compiles, when at first glance it shouldn't.

**Time:** ~45 minutes · **Prerequisites:** [1.3.2 — The rules of the borrow checker](../02-borrow-checker-rules/README.md)

---

## Why this matters

The last lesson gave you the rule: **at any point in the program** you have either many shared references or one mutable reference, never both.

Stop there and that rule appears to ban half of all ordinary code. "Read a thing, then change that thing" is something you do every day. So why does it still work?

Because "at any point" is more precise than it looks. A borrow is alive until its **last use**, not until the end of the block it was created in. Between the last use and the closing brace, that borrow no longer exists and conflicts with nothing.

It wasn't always this way. Before the 2018 edition the compiler measured a borrow by its block, and perfectly sound code was rejected. People learned reflexes — "add braces" — and those reflexes live on in blog posts and Stack Overflow answers. Most of them do nothing today.

The difference between "a borrow lives until the closing brace" and "a borrow lives until its last use" is the difference between fighting the borrow checker and reading it.

---

## The concept

### The same code, one line moved

Three statements, in this order:

```rust
let mut names = vec![String::from("Matin")];

let peek = &names;
println!("before the push: {} name(s)", peek.len());

names.push(String::from("Sora"));
println!("after the push:  {} name(s)", names.len());
```

```text
before the push: 1 name(s)
after the push:  2 name(s)
```

Now move one line: the `println!` that reads `peek` goes below the `push`. Nothing else changes — no name, no type, no statement added or removed:

```rust
let peek = &names;

names.push(String::from("Sora"));
println!("before the push: {} name(s)", peek.len());
```

```text
error[E0502]: cannot borrow `names` as mutable because it is also borrowed as immutable
  --> examples\05-used-after-the-push.rs:12:5
   |
10 |     let peek = &names;
   |                ------ immutable borrow occurs here
11 |
12 |     names.push(String::from("Sora"));
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
13 |     println!("before the push: {} name(s)", peek.len());
   |                                             ---- immutable borrow later used here
```

Same `let`, same `push`, same `println!`. The only thing that changed is **where `peek` is used last**.

### The compiler names the line the borrow ends on

That error carries three labels, and you need all three:

| Label | What it marks |
|---|---|
| `immutable borrow occurs here` | where the borrow **starts** |
| `mutable borrow occurs here` | the thing it conflicts with |
| `immutable borrow later used here` | where the borrow **ends** |

The third label is the one nobody reads and the one that does the work. `later used here` is not saying "this line is also wrong". It is saying: **this line is what kept the borrow alive.**

Which gives you a mechanical procedure for every borrow error:

1. Read label one: where the borrow starts.
2. Read label three: where it ends.
3. The thing the compiler rejected fell between them. Move the use earlier, move the change later, or don't take the borrow at all.

### "Later" means later in the run, not lower in the file

Look at this:

```rust
let mut totals = vec![10, 20, 30];
let view = &totals;

for _ in 0..2 {
    println!("still {} items", view.len());
    totals.push(40);
}
```

```text
error[E0502]: cannot borrow `totals` as mutable because it is also borrowed as immutable
  --> examples\06-borrowed-across-a-loop.rs:14:9
   |
10 |     let view = &totals;
   |                ------- immutable borrow occurs here
...
13 |         println!("still {} items", view.len());
   |                                    ---- immutable borrow later used here
14 |         totals.push(40);
   |         ^^^^^^^^^^^^^^^ mutable borrow occurs here
```

Look at the line numbers. `later used here` is on line **13** and the error is on line **14**. The "later use" is *above* the error.

That is not a display bug. The loop goes back round: `view.len()` on line 13 runs **again**, this time after `totals.push(40)` on line 14. So on the path the program actually takes, that use really is later.

This is where the mental model has to change: a borrow's scope is not an interval of **text**, it is a set of points in the program's **control flow**. Anywhere execution can reach while still needing the reference is inside it.

If you're coming from Python: the nearest familiar thing is that mutating a list while looping over it behaves strangely. The similarity stops there — Python's problem is about the **iterator**, shows up at run time, and does so silently; this is about **any reference at all**, and the compiler stops it before the program runs.

### Now let's name it

The old model was **lexical**: a borrow's life was tied to the block it was written in. Today's model is tied to the uses, and its name is **non-lexical lifetimes** — which everybody shortens to NLL.

NLL arrived with the 2018 edition (compiler version 1.31) and was switched on for the 2015 edition too in 1.36. That means you cannot reproduce the old behaviour on a modern compiler; that strictness is genuinely gone. What survives is the folklore about it.

```senpai-visual
{"kind":"borrowing","labels":["borrow created","last use","borrow over","mutation allowed"]}
```

### A borrow nobody uses ends where it started

If the scope is defined by the uses, a borrow with no uses should have an empty scope. It does:

```rust
let mut totals = vec![10, 20, 30];

let view = &totals;
let counted = view.len();

totals.push(40);

let unused = &totals;

totals.push(50);
```

```text
warning: unused variable: `unused`
  --> examples\02-where-a-borrow-ends.rs:16:9
   |
16 |     let unused = &totals;
   |         ^^^^^^ help: if this is intentional, prefix it with an underscore: `_unused`
   |
   = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

first look: 3 items
last look:  5 items
```

`unused` is a shared reference created exactly one line above a `push` that needs `&mut` — and nobody objected. The only thing the compiler said was that the variable is never read.

The precise conclusion: **the `let` line doesn't put the borrow in play; the use does.** Creating a reference on its own locks nothing.

### Ending a borrow early, on purpose

A `{ }` block forces the borrow inside it to end at the closing brace:

```rust
let mut totals = vec![10, 20, 30];
let counted = {
    let view = &totals;
    view.len()
};
totals.push(40);
```

```text
with a block: counted 3, 4 items now
```

But the same code without those two braces compiles as well — `examples/03-ending-a-borrow-early.rs` runs both versions side by side and their output is identical. Here the block is decoration, not the fix.

So **when** is a block right?

- When the borrow genuinely is a **phase** of the function's work and you want that to stay true after the next edit. The block makes it impossible for somebody to add a use underneath six months from now.
- When the thing holding the borrow has a `Drop` — a `Mutex` lock, a `RefCell` borrow — because then the release itself is a use, and it happens at the closing brace. Those arrive in [Phase 2](../../../phase2-intermediate/05-smart-pointers/03-refcell-and-interior-mutability/README.md), and there the block stops being decoration.

And when is it a smell? When you added braces because the error went away, without reading which line label three pointed at. Usually what you actually wanted was this:

```rust
let mut scores = vec![90, 80];
let first = scores[0];
scores.push(first);
```

```text
no borrow:    [90, 80, 90]
```

`i32` is `Copy` ([1.2.3](../../02-ownership-and-memory/03-clone-and-copy/README.md)), so you can take the value instead of a reference. Then there is no borrow left to end at all. That is the restructure the braces were hiding.

### Two-phase borrows

Three calls that at first glance should all be `E0502`:

```rust
let mut items = vec![10, 20, 30];
items.push(items.len());

let mut names = vec![String::from("Matin"), String::from("Sora")];
names.push(names[0].clone());

let mut text = String::from("hi");
text.push_str(&text.len().to_string());
```

```text
push(items.len()):      [10, 20, 30, 3]
push(names[0].clone()): ["Matin", "Sora", "Matin"]
push_str(own length):   hi2
```

All three compile. Why should that be strange? Because `items.push(items.len())` is really this:

`Vec::push(&mut items, items.len())`

The `&mut items` is built first, then the argument is evaluated — and the argument reads `items`. A mutable reference and a read, at the same time.

The answer is a **two-phase borrow**. The `&mut` the compiler writes for you at a method call is born **reserved**: while it is reserved it behaves like a shared reference, so other reads are allowed. It is **activated** — becomes a real exclusive borrow — only at the moment of the call itself. Two phases; hence the name.

It has two limits worth knowing:

- **It only applies to the `&mut` the compiler writes.** Write that same reference yourself and give it a name, and it is exclusive from the first line. That's `examples/09-a-borrow-you-named-yourself.rs`, and its error is in the next section.
- **Reserved allows reads, not writes.** A second `&mut` inside the argument is still `E0499` — the challenge makes you try it.

### The rule, stated properly

> A borrow is alive from where it is created to its last use — along **every path** execution can take. Two borrows conflict only when some point exists that is inside both scopes and one of them is mutable.

| | Lexical (pre-2018) | NLL (today) |
|---|---|---|
| a borrow lives until | the closing brace of its block | its last use |
| first fix for a conflict | wrap it in `{ }` | move the use, or don't borrow |
| what a scope is | an interval of program text | a set of points in the control flow |

---

## Hands on

```sh
cargo run -p p1-03-03-borrow-scopes-and-nll --example 01-one-line-moved
cargo run -p p1-03-03-borrow-scopes-and-nll --example 02-where-a-borrow-ends
cargo run -p p1-03-03-borrow-scopes-and-nll --example 03-ending-a-borrow-early
cargo run -p p1-03-03-borrow-scopes-and-nll --example 04-two-phase-borrows
```

Then the five broken ones:

```sh
cargo run -p p1-03-03-borrow-scopes-and-nll --example 05-used-after-the-push --features broken
cargo run -p p1-03-03-borrow-scopes-and-nll --example 06-borrowed-across-a-loop --features broken
cargo run -p p1-03-03-borrow-scopes-and-nll --example 07-two-mutable-borrows --features broken
cargo run -p p1-03-03-borrow-scopes-and-nll --example 08-assign-while-borrowed --features broken
cargo run -p p1-03-03-borrow-scopes-and-nll --example 09-a-borrow-you-named-yourself --features broken
```

Then try:

1. In `01-one-line-moved`, move the `peek.len()` line below the push. Now open `05` and see that it's the same file.
2. In `06-borrowed-across-a-loop`, move `let view = &totals;` **inside** the loop. Does it compile? Why?
3. In `07-two-mutable-borrows`, delete only the `first.len()` line, nothing else. What happens now, and what had the third label been telling you?

---

## Errors you will meet

### `E0502` — reading while a `&mut` is still alive

```text
error[E0502]: cannot borrow `items` as immutable because it is also borrowed as mutable
  --> examples\09-a-borrow-you-named-yourself.rs:12:17
   |
11 |     let handle = &mut items;
   |                  ---------- mutable borrow occurs here
12 |     handle.push(items.len());
   |                 ^^^^^ immutable borrow occurs here
13 |
14 |     println!("{handle:?}");
   |                ------ mutable borrow later used here
```

**What the compiler is objecting to:** `handle` is a mutable reference you wrote and gave a name to, and line 14 uses it again. So its scope stretches to line 14, and `items.len()` on line 12 landed right in the middle of it.

Notice this is the same `E0502` you met in the concept section, from the opposite direction: there a read was alive and the `&mut` was rejected; here a `&mut` is alive and the read is rejected. One rule, two phrasings.

**The fix:** don't name the reference — let the compiler make it:

```rust
let mut items = vec![10, 20, 30];
items.push(items.len());
println!("{items:?}");
```

**Why that's the fix:** the `&mut` the compiler builds for a method call gets a two-phase borrow and allows reads while reserved. The one you built with `let` is exclusive from its first line. The difference isn't about "at the same time" — it's about which of them is two-phase.

### `E0499` — two mutable references whose scopes overlap

```text
error[E0499]: cannot borrow `scores` as mutable more than once at a time
  --> examples\07-two-mutable-borrows.rs:13:18
   |
10 |     let first = &mut scores;
   |                 ----------- first mutable borrow occurs here
...
13 |     let second = &mut scores;
   |                  ^^^^^^^^^^^ second mutable borrow occurs here
...
16 |     println!("{}", first.len());
   |                    ----- first borrow later used here
```

**What the compiler is objecting to:** `first` is created on line 10 and used on line 16, so it is alive until line 16. `second` is created on line 13 — right in the middle of that stretch.

**The fix:** finish each borrow before creating the next:

```rust
let first = &mut scores;
first.push(70);
println!("{}", first.len());

let second = &mut scores;
second.push(60);
println!("{}", second.len());
```

**Why that's the fix:** no line was deleted and no brace was added — the use of `first` just moved up. `first`'s scope now ends on the third line and `second` starts after it. Two scopes end to end, not on top of each other.

### `E0506` — assigning to something that is still borrowed

```text
error[E0506]: cannot assign to `level` because it is borrowed
  --> examples\08-assign-while-borrowed.rs:11:5
   |
 9 |     let watcher = &level;
   |                   ------ `level` is borrowed here
10 |
11 |     level = 7;
   |     ^^^^^^^^^ `level` is assigned to here but it was already borrowed
12 |
13 |     println!("watcher saw {watcher}, level is now {level}");
   |                            ------- borrow later used here
```

**What the compiler is objecting to:** there is no `&mut` anywhere in that file. A plain assignment is enough — because `watcher` is looking at `level`, and on line 13 it is still looking. If the assignment were allowed, `watcher` would be showing a value that isn't there any more.

**The fix:** finish using `watcher` before the assignment:

```rust
let watcher = &level;
println!("watcher saw {watcher}");

level = 7;
println!("level is now {level}");
```

**Why that's the fix:** the third label said exactly this. `borrow later used here` was on line 13; move that use up and the borrow's scope ends before the assignment. That one repair is all it needs — no `clone`, no braces, no extra `mut`.

---

## Exercises

### Warm up

None of these need typing. Answer, then open.

**1.** Does this compile?

```rust
let mut names = vec![String::from("a")];
let peek = &names;
names.push(String::from("b"));
println!("{}", peek.len());
```

<details>
<summary>Answer</summary>

No. `E0502`. `peek` is used on the last line, so its scope reaches that far and the `push` fell inside it.

</details>

**2.** And this?

```rust
let mut n = 5;
let r = &mut n;
*r += 1;
n += 1;
println!("{n}");
```

<details>
<summary>Answer</summary>

Yes, and it prints `7`. `r`'s last use is the `*r += 1` line; after that `n` is free again.

</details>

**3.** And this?

```rust
let mut v = vec![10, 20, 30];
v[0] = v.len();
println!("{v:?}");
```

<details>
<summary>Answer</summary>

Yes, and it prints `[3, 20, 30]`. This one isn't two-phase: in an assignment Rust evaluates the right-hand side first, so `v.len()` has finished before the `&mut` for `v[0]` is taken at all. Move the read into the brackets — `v[v.len() - 1] = 99;` — and it becomes `E0502`, because there the read happens while the `&mut` is being built.

</details>

**4.** And this?

```rust
let mut items = vec![10, 20, 30];
let handle = &mut items;
handle.push(items.len());
println!("{handle:?}");
```

<details>
<summary>Answer</summary>

No. `E0502`. Two-phase borrows apply only to the `&mut` the compiler makes, not to one you named with a `let`.

</details>

<details>
<summary>What does the <code>later used here</code> label tell you?</summary>

The line that stretched the borrow's scope that far — that is, where the borrow ends. The fix is almost always to move that line.

</details>

<details>
<summary>Why does a <code>let</code> that makes a reference and never reads it lock nothing?</summary>

Because the scope is defined by the uses. With no uses the scope is empty, so it shares no point with any other borrow.

</details>

### Repair

Fix all five broken files — and for each one, say which line the third label points at **before** you touch the code.

1. `examples/05-used-after-the-push.rs` — by moving one line.
2. `examples/06-borrowed-across-a-loop.rs` — two ways: once by moving `let view` inside the loop, once by removing `view` altogether. Which is better, and why?
3. `examples/07-two-mutable-borrows.rs` — without deleting any `println!`.
4. `examples/08-assign-while-borrowed.rs` — with no `clone` and no braces.
5. `examples/09-a-borrow-you-named-yourself.rs` — then say why the unnamed version compiles.

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p1-03-03-borrow-scopes-and-nll
```

All five read a value and then change that same value. The question is never whether that's allowed; it's where you put the read. Two of them fit on one line thanks to two-phase borrows and three of them don't — work out which is which.

### Build

Write `with_longest_repeated` again, this time called `longest_repeated_by_index`, so that it **holds no reference to an element at all**: find the index of the longest string first, then copy from the index.

Then do three things:

1. In both versions, say where each borrow's last use is.
2. Delete the `.clone()` from the by-index version and read the error you get. It is not a borrow error at all — which earlier lesson did you first meet that code in?
3. Write one sentence on why taking an index is simpler than holding a reference here — and one sentence on why that isn't true everywhere.

### Challenge (optional)

**Part one.** Run this in a scratch file:

```rust
let mut v = vec![10, 20, 30];
v.push(v.remove(0));
```

You get `E0499`, not `E0502`. Explain why, using what you read about "reserved". Then read the compiler's own two-part help and do what it says.

**Part two.** A lot of people think this is how you end a borrow:

```rust
let mut totals = vec![10, 20, 30];
let view = &totals;
drop(view);
totals.push(40);
```

It compiles, but the compiler emits a warning. Read the warning and say why that line is useless — and why deleting `drop(view)` leaves it compiling anyway.

**Part three.** (Reaches forward.) RFC 2094, the document that defined NLL, has a section called "Problem case #3" which NLL does *not* solve. Find it and read it. Solving it is the job of the next analysis, which is called Polonius. You don't need to understand it; it's enough to know where the edges are — and that [lifetimes](../../../phase2-intermediate/04-error-handling-and-lifetimes/03-lifetime-basics-and-elision/README.md) in Phase 2 continue this same conversation.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| borrow scope | from creation to last use | every borrow error |
| last use | the final execution point that reads the reference | the `later used here` label |
| lexical | the old model: until the closing brace | only in old blog posts |
| NLL | today's model: until the last use | since the 2018 edition |
| two-phase borrow | a method's automatic `&mut`, reserved first | `v.push(v.len())` |
| `E0506` | assigning to something that is borrowed | arrives with no `&mut` in sight |

### What you now know

- A borrow lives until its last use, not until the closing brace.
- The `later used here` label names the line the borrow ends on; that line is usually the one to move.
- "Later" means later along the execution path — a loop can make a line above count as the next one.
- Creating a reference locks nothing; using it does.
- A `{ }` block ends a borrow early, but since NLL it is usually unnecessary.
- `v.push(v.len())` compiles because of two-phase borrows, and the same code with a named reference does not.

### What comes back later

- **Slices, which are borrows of part of a collection** — [1.3.4 — Slices](../04-slices/README.md)
- **`RefCell`, and borrows checked at run time** — [Phase 2 — `RefCell`](../../../phase2-intermediate/05-smart-pointers/03-refcell-and-interior-mutability/README.md)
- **Locks, where the closing brace matters again** — [Phase 2 — `Mutex` and threads](../../../phase2-intermediate/07-concurrency-and-async/01-threads-mutex-arc/README.md)
- **Writing lifetimes out by hand, when the compiler can't work them out** — [Phase 2 — Lifetimes](../../../phase2-intermediate/04-error-handling-and-lifetimes/03-lifetime-basics-and-elision/README.md)

### Can you explain?

- When does a borrow end?
- What do the three labels of an `E0502` each say?
- How can the "later use" be above the line the error is reported on?
- Why does an unused reference produce no error?
- When is a `{ }` block the right fix, and when is it cover-up?
- Why does `items.push(items.len())` compile when its named version doesn't?

---

## Going further

- [The Rust Book — References and borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html) — the same ground, officially.
- [RFC 2094 — non-lexical lifetimes](https://rust-lang.github.io/rfcs/2094-nll.html) — the document that defined the change. Its introduction opens with real examples and is far more readable than you'd expect.
- [rustc dev guide — two-phase borrows](https://rustc-dev-guide.rust-lang.org/borrow_check/two_phase_borrows.html) — exactly how "reserved" and "activated" work.
- [`rustc --explain E0502`](https://doc.rust-lang.org/error_codes/E0502.html) — run this on every code you meet. It's a good ten-second habit.
