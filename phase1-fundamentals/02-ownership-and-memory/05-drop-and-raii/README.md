# 1.2.5 — `Drop` and RAII

## At a glance

After this lesson you can:

- Predict the order the values in a block are cleaned up in, and say why that order is reversed.
- Write an `impl Drop` and turn cleanup from a claim into something you watch on screen.
- Release something early with `drop(value)`, and say why `value.drop()` is refused.
- Say why a type either is `Copy` or has a destructor, and never both.

**Time:** ~45 minutes · **Prerequisites:** [1.2.4 — Ownership across function boundaries](../04-ownership-across-functions/README.md)

---

## Why this matters

The last four lessons kept repeating one sentence: "when the owner goes out of scope, the value is freed."

Up to now that has been an invisible promise. Something you accepted because the compiler said so, not because you saw it.

Today you see it. You build a type that speaks up as it is being cleaned up, and then all of 1.2.1 through 1.2.4 plays out in front of you as lines of output: nested scopes, moves, ownership handed back out of a function.

And something new joins in that isn't only about memory. Files, sockets, database connections, locks — anything that has to be closed gets closed by this one mechanism.

In Python, closing is on you:

```python
f = open("data.txt")
# ... work ...
f.close()          # never runs if an exception is raised between these lines
```

Python's answer is `with`, but `with` is opt-in twice over: whoever wrote the class had to write `__enter__` and `__exit__`, and you have to remember to type it. A forgotten `with` raises nothing at all — it just leaves a file open until the garbage collector gets round to it.

In Rust, any type can say what to do as it is being cleaned up, and **the compiler puts that call in at the closing brace itself**. There is no way to forget it, because nobody wrote it down to be forgotten.

---

## The concept

### Cleanup you can watch happen

Here is a small type with one field. `struct` and `impl` get their own lesson in [1.5.1](../../05-your-own-types/01-structs-and-methods/README.md); for today they just mean "a type made of one field" and "code attached to that type".

```rust
struct Guard {
    name: String,
}

impl Guard {
    fn new(name: &str) -> Guard {
        println!("open  {name}");
        Guard {
            name: name.to_string(),
        }
    }
}
```

Nothing new so far. This is the new part:

```rust
impl Drop for Guard {
    fn drop(&mut self) {
        println!("close {}", self.name);
    }
}
```

Now run this:

```rust
println!("-- before the block");
{
    let _inner = Guard::new("inner");
    println!("-- inside the block");
}
println!("-- after the block");
```

```text
-- before the block
open  inner
-- inside the block
close inner
-- after the block
```

Nobody called that `close inner`. There is no line in the code saying "close this one". The compiler put the call in **at that closing brace** while it was building the program.

Now the names. `Drop` is a **trait** — a set of behaviour a type promises to have — and its only method is `drop`. Code that runs as a value is destroyed is called a **destructor**. Traits get their full lesson in [Phase 2](../../../phase2-intermediate/03-generics-and-traits/02-defining-and-implementing-traits/README.md); today you need exactly this one.

### The order is reversed

[1.2.1](../01-stack-and-heap/README.md) said it in one sentence: "things are released in reverse order of declaration." Now watch it:

```rust
{
    let _a = Guard::new("a");
    let _b = Guard::new("b");
    let _c = Guard::new("c");
    println!("      all three alive");
}
```

```text
open  a
open  b
open  c
      all three alive
close c
close b
close a
```

Built `a`, `b`, `c`. Cleaned up `c`, `b`, `a`.

```senpai-visual
{"kind":"ownership","labels":["build a","build b","build c","drop c","drop b","drop a"]}
```

Why reversed? Because something built later may be leaning on something built earlier — a connection reading a config, say. Forward order would pull the ground out from under the second one before dealing with it. Reverse order is the only order that is always safe.

It's the discipline of nested brackets: they close from the inside out, never the other way round.

### A collection cleans up front to back

"Reversed" isn't the whole rule. Watch what a `Vec` does:

```rust
let mut group = Vec::new();
for name in ["first", "second", "third"] {
    group.push(Guard::new(name));
}
```

```text
open  first
open  second
open  third
      the Vec owns all three
close first
close second
close third
```

Nothing is reversed. Here `group` is **one** value that gets cleaned up at the end of the block, and the `Vec`'s destructor walks its elements from index zero upwards.

So the rule is more precise than you thought:

> **A block's bindings are cleaned up in reverse declaration order; a collection's elements from first to last.**

Three separate `let`s and one three-element `Vec` give you two different orders, and that difference is real rather than an implementation detail.

### RAII — the resource *is* the value

This pattern has a name: **RAII**, *Resource Acquisition Is Initialization*. It's a bad name inherited from C++, but what it says is simple:

> **Take the resource when the value is built, give it back in `drop`. From then on the scope is responsible for closing, not your memory.**

`Guard` above is exactly that: `Guard::new` opens, `drop` closes. Every path out of the block — reaching the brace, an early `return`, a `break` out of a loop — goes through that same destructor.

|  | Python | Rust |
|---|---|---|
| When it closes | when you call `close()`, or when a `with` block ends | at the owner's closing brace |
| Who has to remember | the class author, and then everyone who uses it | the type's author, once |
| If you forget | it stays open, with no error | you can't forget |

**Where the analogy breaks:** you can see a `with` block in the code; Rust's cleanup is invisible. Which means that when you want to know when something closes, you have to ask "who owns it, and where does its scope end?" — and that question was the whole of 1.2.2 and 1.2.4.

### A move moves the cleanup

If cleanup is attached to the owner, then every move also moves *when* cleanup happens:

```rust
let given = Guard::new("given");
consume(given);
println!("      back in main, and it is already closed");

fn consume(_guard: Guard) {
    println!("      inside consume");
} // <- this function owns it now, so it closes here
```

```text
open  given
      inside consume
close given
      back in main, and it is already closed
```

`given` was built in `main` and died in `consume`. That's what [1.2.4](../04-ownership-across-functions/README.md) said, seen from the other side: a move isn't only a transfer of data, it's a transfer of **responsibility for the cleanup**.

And shadowing, from [1.1.1](../../01-foundations/01-variables-mutability-shadowing/README.md), has a surprising consequence here:

```rust
let _slot = Guard::new("shadowed");
let _slot = Guard::new("replacement");
println!("      both are still alive; the name just points elsewhere");
```

```text
open  shadowed
open  replacement
      both are still alive; the name just points elsewhere
close replacement
close shadowed
```

The first value was **not** cleaned up. Shadowing re-binds the name; it doesn't destroy the old value. That value is still there, you just have no name left to reach it by, and at the closing brace it goes in its turn — which is to say, in reverse.

### Letting go early with `drop`

Sometimes the end of the block is too late. A lock held to the end of a function keeps everyone else waiting for no reason:

```rust
let first = Guard::new("first");
let _second = Guard::new("second");
drop(first);
println!("      first is closed, second is not");
```

```text
open  first
open  second
close first
      first is closed, second is not
close second
```

`std::mem::drop` is not magic. The whole of it is an ordinary function that takes its argument by value and has an empty body. The value is cleaned up because *that function's* scope ends immediately — the same rule as always, with a very short scope.

And because it takes ownership, after `drop(x)` you no longer have `x`. That's a move like any other, and its error is in the next section.

### Why you cannot call `.drop()` yourself

`drop` is a method, it's in scope, and it does exactly what you want. So:

```rust
let guard = Guard::new("early");
guard.drop();
```

```text
error[E0040]: explicit use of destructor method
```

Refused. The full explanation is in the errors section, but the short version: the compiler has already put a call in at the closing brace. If you called one too, the cleanup would run **twice** — and a double free is one of the three bugs [1.2.1](../01-stack-and-heap/README.md) named.

### `Drop` and `Copy` cannot both be true

[1.2.3](../03-clone-and-copy/README.md)'s table had a row that looked arbitrary at the time: "anything with a `Drop`" is not `Copy`. Here's why.

```rust
#[derive(Clone, Copy)]
struct Ticket {
    id: u32,
}

impl Drop for Ticket {
    fn drop(&mut self) {
        println!("returning ticket {}", self.id);
    }
}
```

```text
error[E0184]: the trait `Copy` cannot be implemented for this type; the type has a destructor
```

Every field of `Ticket` is a `u32`, so by 1.2.3's rules that derive ought to be allowed. It's the `Drop` that makes it impossible, because the two promises contradict each other:

- `Copy` says: **a byte copy is the whole value, and there's nothing left to clean up.**
- `Drop` says: **there is precisely something to clean up.**

If both were allowed, one assignment would produce two tickets, and both would hand the same one ticket back at the end of their scopes. It's the `String` problem again, except this time you wrote it yourself.

---

## Hands on

```sh
cargo run -p p1-02-05-drop-and-raii --example 01-drop-runs
cargo run -p p1-02-05-drop-and-raii --example 02-drop-order
cargo run -p p1-02-05-drop-and-raii --example 03-a-move-moves-the-drop
cargo run -p p1-02-05-drop-and-raii --example 04-dropping-early
```

Then the three broken ones:

```sh
cargo run -p p1-02-05-drop-and-raii --example 05-calling-drop-yourself --features broken
cargo run -p p1-02-05-drop-and-raii --example 06-copy-and-drop --features broken
cargo run -p p1-02-05-drop-and-raii --example 07-used-after-drop --features broken
```

Then try:

1. In `02-drop-order`, change the `let _b` line to `let _ = Guard::new("b");`. What changes in the output, and why?
2. In `02-drop-order`, build the same three guards in three separate `let`s instead of pushing them into the `Vec`. What order do you get?
3. In `04-dropping-early`, move the `drop(first);` line up one, above `let _second`. What changes?

---

## Errors you will meet

### `E0040` — calling the destructor directly

```text
error[E0040]: explicit use of destructor method
  --> examples\05-calling-drop-yourself.rs:27:11
   |
27 |     guard.drop();
   |           ^^^^ explicit destructor calls not allowed
   |
help: consider using `drop` function
   |
27 -     guard.drop();
27 +     drop(guard);
   |
```

**What the compiler is objecting to:** the destructor is its to call, not yours. It has already placed a call at the closing brace and is counting on that call running. If `guard.drop()` were allowed, the body would run twice, and every resource it releases would be released twice.

**The fix:** `drop(guard)` instead of `guard.drop()` — and the compiler has spelled exactly that out for you with a `-` and a `+`.

**Why that's the fix:** `drop(guard)` takes ownership away from you. The value dies inside that function, the compiler knows it has died, and it does not place a second call at the brace. "Exactly once" is still guaranteed. The difference isn't which one reads nicer; it's that one of them is countable and the other isn't.

### `E0184` — `Copy` on a type with a destructor

```text
error[E0184]: the trait `Copy` cannot be implemented for this type; the type has a destructor
  --> examples\06-copy-and-drop.rs:8:8
   |
 7 | #[derive(Clone, Copy)]
   |                 ---- in this derive macro expansion
 8 | struct Ticket {
   |        ^^^^^^ `Copy` not allowed on types with destructors
   |
note: destructor declared here
  --> examples\06-copy-and-drop.rs:13:5
   |
13 |     fn drop(&mut self) {
   |     ^^^^^^^^^^^^^^^^^^
```

**What the compiler is objecting to:** two incompatible promises. `Copy` means "assignment produces a second one and the source stays valid, because there's nothing to clean up." `Drop` means "there is something to clean up." Together they mean every assignment creates one more thing responsible for the same resource.

**The fix:** pick one. Either drop `Copy` from the derive:

```rust
#[derive(Clone)]
struct Ticket {
    id: u32,
}
```

or remove the `impl Drop` and hand the ticket back somewhere else.

**Why that's the fix:** look at `note: destructor declared here` — the compiler has put both of the places involved side by side, not just the one where the error is. That's exactly what you need to decide: which of those two lines did you actually mean?

And this restriction wasn't imposed on you; you asked for it. The moment you write a destructor for a type, you have said "this thing needs releasing" — and from then on, silent duplication at every assignment is wrong.

### `E0382` — using a value after dropping it

```text
error[E0382]: borrow of moved value: `text`
  --> examples\07-used-after-drop.rs:12:16
   |
 6 |     let text = "a heap buffer".to_string();
   |         ---- move occurs because `text` has type `String`, which does not implement the `Copy` trait
...
10 |     drop(text);
   |          ---- value moved here
11 |
12 |     println!("{text}");
   |                ^^^^ value borrowed here after move
```

**What the compiler is objecting to:** nothing special about `drop` at all. It says `text` moved on line 10. `drop` is a function that takes its argument by value, so `drop(text)` is exactly the `consume(text)` you met in 1.2.2 and 1.2.4.

**The fix:** do whatever you need with `text` before dropping it:

```rust
let text = "a heap buffer".to_string();
println!("{text}");
drop(text);
```

**Why that's the fix:** look at the compiler's `help` suggestion: `drop(text.clone())`. It compiles, and it is completely pointless — it allocates a fresh buffer purely in order to free it immediately, while the original buffer still gets freed at the closing brace. That's [1.2.3](../03-clone-and-copy/README.md)'s warning in its most natural habitat: `.clone()` silences the error without answering the question.

If you still need `x` after `drop(x)`, you weren't finished with it — and the `drop` was in the wrong place, not the use.

---

## Exercises

### Warm up

<details>
<summary>In what order are three <code>let</code>s in one block cleaned up?</summary>

Reverse declaration order. The last thing built is the first thing to go.

</details>

<details>
<summary>In what order are three elements of a <code>Vec</code> cleaned up?</summary>

First to last. The `Vec` itself is one value, and its destructor walks the elements from index zero. Three separate `let`s and one three-element `Vec` give you two different orders.

</details>

<details>
<summary>When you hand a value to a function, where does its destructor run?</summary>

At that function's closing brace, unless the function hands the value back. Ownership decides when, not where the value was born.

</details>

<details>
<summary><code>let _slot = a;</code> then <code>let _slot = b;</code> — when is <code>a</code> cleaned up?</summary>

At the closing brace, after `b`. Shadowing re-binds the name but doesn't kill the old value; you just have no name left to reach it by.

</details>

<details>
<summary>How does <code>std::mem::drop</code> work?</summary>

It's an ordinary function that takes its argument by value and has an empty body. The value is cleaned up because that function's scope ends immediately. There is nothing special in it.

</details>

<details>
<summary>Why is <code>value.drop()</code> refused when <code>drop(value)</code> isn't?</summary>

Because `value.drop()` would run the destructor body without cancelling the automatic call at the closing brace — two cleanups. `drop(value)` takes ownership, so the compiler knows no second call is needed.

</details>

<details>
<summary>Why can't a type be <code>Copy</code> and have a destructor?</summary>

`Copy` promises there is nothing left to clean up; `Drop` says there is. If both were possible, every assignment would create one more thing responsible for one resource.

</details>

### Repair

Fix all three broken examples, and for each one say which rule it broke:

1. `examples/05-calling-drop-yourself.rs` — using the compiler's own suggestion.
2. `examples/06-copy-and-drop.rs` — **two** ways: once by removing `Copy`, once by removing the `impl Drop`. Then say which is more likely right in a real program, and why the answer depends on what that ticket is for.
3. `examples/07-used-after-drop.rs` — without deleting the `drop` and without writing `.clone()`.

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p1-02-05-drop-and-raii
```

At the top of the file there's a `Tracker` that writes its name into a log as it is cleaned up — the same thing `Guard` did on screen, only in a form a test can read back. That part is written for you and you never touch it.

Four of the five need nothing but a block, `Tracker::new` and `drop`. One of them does **not** come back reversed; its doc comment says so, but understanding *why* is the whole of today.

### Build

Write a `pub fn transaction(steps: Vec<String>) -> Vec<String>` that builds one `Tracker` per step and keeps them all alive to the end of one block — except for steps starting with `"skip:"`, which must be released immediately after being built. The function returns the log.

Guess before you run it: what is the log for `["a", "skip:b", "c"]`? Then run it and see whether you were right.

Then write a sentence on what would change if you had kept every step in one `Vec<Tracker>` instead.

### Challenge (optional)

**Part one.** Run these two pieces and explain why their output differs:

```rust
let _ = Guard::new("wildcard");
println!("      after the wildcard line");
```

```rust
let _named = Guard::new("underscore-name");
println!("      after the named line");
```

One of them closes immediately and the other stays alive to the end of the block. **Which, and why?** This difference is expensive in real code: a lock taken with `let _` is released on that very line and protects nothing at all.

**Part two.** Build a `Guard` and then call `std::mem::forget` on it. Does the destructor run? Read its documentation and say why that function isn't `unsafe`, given that it does precisely what this whole lesson exists to prevent. (Hint: leaking memory doesn't count as unsafe in Rust.)

**Part three.** Do destructors run if the program panics? Try it: put a `panic!("boom")` in `01-drop-runs` after `_last` is built. This reaches forward to [1.6.4](../../06-absence-and-failure/04-panic-vs-result/README.md), but the answer is exactly what makes RAII dependable in Rust.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `Drop` | a one-method trait that runs as a value is cleaned up | closing anything you opened |
| destructor | that cleanup code, as a piece of terminology | the `E0040` and `E0184` errors |
| RAII | take the resource on construction, give it back in `drop` | files, sockets, connections, locks |
| reverse order | a block's bindings are cleaned up backwards | nested scopes |
| collection order | a `Vec`'s elements go first to last | `Vec` versus several `let`s |
| `drop(value)` | a function that takes ownership and has an empty body | letting go early |
| `E0040` | calling `.drop()` directly | you wanted `drop(value)` |
| `E0184` | `Copy` on a type with a destructor | one of the two has to go |

### What you now know

- The compiler places the cleanup call at the closing brace, and no path out of the block avoids it.
- A block's bindings are cleaned up in reverse declaration order; a collection's elements first to last.
- Moving a value moves when it is cleaned up.
- Shadowing does not clean the old value up early.
- `drop(value)` is an ordinary function that takes ownership; `value.drop()` is refused because it would run the cleanup twice.
- A type either is `Copy` or has a destructor, never both.
- RAII means release is tied to scope, not to the programmer's memory.

### What comes back later

- **`struct` and `impl`, which we only borrowed today** — [1.5.1 — Structs and methods](../../05-your-own-types/01-structs-and-methods/README.md)
- **Traits in general, and the family `Drop` belongs to** — [Phase 2 — Defining and implementing traits](../../../phase2-intermediate/03-generics-and-traits/02-defining-and-implementing-traits/README.md)
- **Borrowing, and why an early drop is sometimes the only way to satisfy the borrow checker** — [1.3.1 — References](../../03-borrowing-and-references/01-shared-and-mutable-refs/README.md)
- **`RefCell`, which is what the exercise log is written with** — [Phase 2 — Interior mutability](../../../phase2-intermediate/05-smart-pointers/03-refcell-and-interior-mutability/README.md)
- **Locks, where letting go early stops being a nicety and becomes a requirement** — [Phase 2 — Threads, `Mutex` and `Arc`](../../../phase2-intermediate/07-concurrency-and-async/01-threads-mutex-arc/README.md)
- **`panic`, and what destructors do on the failure path** — [1.6.4 — Panic versus `Result`](../../06-absence-and-failure/04-panic-vs-result/README.md)

### Can you explain?

- When does the code inside `fn drop` run, and who calls it?
- Why are a block's bindings cleaned up in reverse?
- Why aren't a `Vec`'s elements cleaned up in reverse?
- State RAII in one sentence, and say how it differs from Python's `with`.
- What exactly does `std::mem::drop` do?
- Why can't `Copy` and `Drop` both apply to one type?

---

## Going further

- [The Rust Book — Running code on cleanup with `Drop`](https://doc.rust-lang.org/book/ch15-03-drop.html) — the same ground, officially.
- [`std::ops::Drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html) — its documentation has a section on the exact drop order for structs and collections.
- [`std::mem::drop`](https://doc.rust-lang.org/std/mem/fn.drop.html) — look at the whole definition. It's two lines.
- [`std::mem::forget`](https://doc.rust-lang.org/std/mem/fn.forget.html) — the deliberate way to not run a destructor, and the explanation of why that isn't unsafe.
