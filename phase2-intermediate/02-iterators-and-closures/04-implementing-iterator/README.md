# 2.2.4 — Implementing `Iterator` and `IntoIterator` for your own type

## At a glance

After this lesson you can:

- Explain exactly what the `Iterator` trait requires — one method, `next(&mut self) -> Option<Self::Item>` — and why that one method is enough to make every adapter and consumer [2.2.2](../02-iterator-adapters/README.md) and [2.2.3](../03-consuming-and-collecting/README.md) taught you work on your own type for free.
- Build your own custom iterator that carries real state — a type whose `next()` reads what the struct itself is holding, computes the next value from it, and advances that same data for the following call.
- Implement `IntoIterator` for your own type so `for x in your_value` compiles, tell its three usual forms apart (by value / by shared reference / by mutable reference), and explain why `for x in v` moves `v` — the same move semantics [1.2.2](../../../phase1-fundamentals/02-ownership-and-memory/02-move-semantics/README.md) already taught you.

**Time:** ~75 minutes · **Prerequisites:** [2.2.3 — Consuming and collecting](../03-consuming-and-collecting/README.md), and in particular [2.2.2 — Iterator adapters](../02-iterator-adapters/README.md) and [1.2.2 — Move semantics](../../../phase1-fundamentals/02-ownership-and-memory/02-move-semantics/README.md)

---

## Why this matters

Up to now you have only ever been a **consumer** of `Iterator`, never its author. [2.2.2](../02-iterator-adapters/README.md) taught you to chain `.map()`, `.filter()`, `.take()`, and the rest. [2.2.3](../03-consuming-and-collecting/README.md) taught you to drive that chain to a final result with `.collect()`, `.fold()`, `.sum()`. Both of those were about things the standard library had already made `Iterator` for you — `Vec`, `HashMap`, slices.

Real code constantly needs something that isn't already a `Vec`: a sequence generator (a Fibonacci generator, a counter); a layer over an API's results that arrive page by page instead of all at once in memory; a custom type of your own — a watch queue, an event log — that you want behaving exactly like `Vec` does: filterable, loopable, collectible.

The good news: that power isn't free, but you only pay for it once and it's yours forever after. `Iterator` asks for exactly **one** method from you. Just the one. Every adapter and every consumer [2.2.2](../02-iterator-adapters/README.md) and [2.2.3](../03-consuming-and-collecting/README.md) taught you — dozens of methods — is already written on top of that one method. Write it, and every single one of them works on your own type too, no exceptions.

---

## The concept

### The `Iterator` trait: one method, `next`

Build a Fibonacci generator — a type that hands back the next number in the sequence each time it's asked. The state that takes — the current value, and the next one — lives right there, inside the struct itself:

```rust
struct Fibonacci {
    current: u64,
    next: u64,
}
impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let value = self.current;
        let new_next = self.current + self.next;
        self.current = self.next;
        self.next = new_next;
        Some(value)
    }
}
```

Now call it by hand, eight times:

```rust
let mut fib = Fibonacci { current: 0, next: 1 };
for _ in 0..8 {
    println!("{:?}", fib.next());
}
```

```text
Some(0)
Some(1)
Some(1)
Some(2)
Some(3)
Some(5)
Some(8)
Some(13)
```

That was all it took. `Iterator` asks for exactly two things from an implementer: an **associated type** named `Item` — it says what type each call to `next` produces — and one required method, `next(&mut self) -> Option<Self::Item>`. The associated type does not make `Iterator` itself generic; each implementer just says "mine is this," and that is enough. [2.3.5](../../03-traits-and-generics/05-associated-types/README.md) opens this mechanism up in full; one paragraph is enough for today.

Take `&mut self` seriously — `next` has to *change* the state, or the next call would produce the same thing again; with `&self` it does not compile at all (the exact error is in "Errors you will meet"). One small note in passing: this struct has a field named `next` *and* a method named `next()`. Rust keeps the two completely separate — `self.next` is the field, `self.next()` is the method call — but if that reads oddly, name your own field whatever you like; the trait does not care what you call your fields.

> **Python bridge:** if you're coming from Python, you've written this once before under a different name: the `__iter__`/`__next__` protocol. `__next__` either returns the next value or *raises* `StopIteration` to say it's done. Rust has the same idea with one real difference: finishing is not an exception, it's an ordinary value — `None`, the same thing the type system already forces you to handle. Nothing exceptional is happening; it's just an `Option` that came back empty.

### A `for` loop is nothing but `loop` and `match` on `next`

Since `Fibonacci` never returns `None` (its sequence never ends), bound it with `.take(6)` first. Now pull the same six values by hand, with a `loop`:

```rust
let mut manual = Fibonacci { current: 0, next: 1 }.take(6);
loop {
    match manual.next() {
        Some(value) => println!("{value}"),
        None => break,
    }
}
```

And the same six values with `for`:

```rust
for value in (Fibonacci { current: 0, next: 1 }).take(6) {
    println!("{value}");
}
```

```text
0
1
1
2
3
5
```

Both produce exactly the same output — because both do exactly the same thing. `for value in iter { BODY }` is nothing but shorthand for that same `loop`/`match` above: the compiler writes it for you, every single time you type `for`. There is no magic in it, only a `.next()` call repeating itself until it gets a `None`.

```senpai-visual
{"kind":"concept","labels":["state: current, next","next() called","value is read","new state is stored","Some(value) returned"]}
```

### Everything free: adapters and consumers

`Fibonacci` has only ever written `next()`. Now try everything [2.2.2](../02-iterator-adapters/README.md) and [2.2.3](../03-consuming-and-collecting/README.md) taught you on it — `.take()`, `.filter()`, `.map()`, `.collect()`, `.sum()`, `.enumerate()`:

```rust
let first_ten: Vec<u64> = (Fibonacci { current: 0, next: 1 }).take(10).collect();
let evens: Vec<u64> = (Fibonacci { current: 0, next: 1 })
    .take(10)
    .filter(|n| n % 2 == 0)
    .collect();
let sum: u64 = (Fibonacci { current: 0, next: 1 }).take(10).sum();
```

```text
first 10:      [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
even, of those: [0, 2, 8, 34]
sum of first 10: 88
```

None of these were written for `Fibonacci`. `.take()`, `.filter()`, `.collect()`, `.sum()` are all methods with a default implementation, on the `Iterator` trait itself, built on top of the one method you wrote. The standard library wrote them once, for *every* type that has a `next()` — which is exactly the promise the previous section made.

### The `IntoIterator` trait: what makes `for` legal

`Iterator` says how a type produces values. `IntoIterator` is a different thing: the trait `for x in value` actually calls. Without it, that line does not compile at all — no matter how much `value` looks like a collection.

Build a custom type — a watch queue, wrapping a `Vec<String>` — and implement `IntoIterator` for it:

```rust
struct WatchList(Vec<String>);

impl IntoIterator for WatchList {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
```

Two associated types were needed: `Item` (what gets produced — `String` here, not `&String`, because `self` was taken without `&`) and `IntoIter` (the iterator type itself — here, the exact thing `Vec<String>::into_iter()` already returns; no extra work, just hand it back). Now:

```rust
let queue = WatchList(vec!["Frieren".to_string(), "Bocchi the Rock!".to_string()]);
for title in queue {
    println!("now watching: {title}");
}
```

```text
now watching: Frieren
now watching: Bocchi the Rock!
```

That line did not compile before. It does now, purely because `IntoIterator` was implemented.

### The three-forms convention: by value, by shared reference, by mutable reference

`WatchList` above implemented only **one** shape of `IntoIterator`: by value. But a well-behaved type — exactly like `Vec` — implements this trait **three times**, one for each kind of access. You've been using `Vec`'s three since [1.1.6](../../../phase1-fundamentals/01-foundations/06-vec-and-string-basics/README.md), without needing the formal name for any of them:

```rust
let titles = vec!["Frieren".to_string(), "Bocchi the Rock!".to_string()];

for t in &titles {
    println!("{t}");
}
println!("titles still usable, {} entries", titles.len());
```

```text
Frieren
Bocchi the Rock!
titles still usable, 2 entries
```

| You write | It calls | `Item` is | `titles` afterward |
|---|---|---|---|
| `for t in &titles` | `IntoIterator for &Vec<T>` | `&T` | still usable |
| `for t in &mut titles` | `IntoIterator for &mut Vec<T>` | `&mut T` | still usable |
| `for t in titles` | `IntoIterator for Vec<T>` | `T` | **gone** — moved |

Three separate `impl` blocks, on three separate types (`Vec<T>` itself, `&Vec<T>`, `&mut Vec<T>`) — not one `impl` covering all three cases. `WatchList` above only has the third row; the first two were deliberately left out — the next section says why, and "Errors you will meet" shows you exactly what that absence looks like.

```senpai-visual
{"kind":"ownership","labels":["you write `for x in v`","IntoIterator::into_iter(v) is called","it takes self without & — ownership","v is moved","v is no longer usable"]}
```

### Why `for x in v` moves `v`

Look at that table again: the third row's `fn into_iter(self) -> ...` takes `self` — not `&self`, not `&mut self`. The exact same signature `WatchList` wrote above. `self` without `&` is exactly what [1.2.2](../../../phase1-fundamentals/02-ownership-and-memory/02-move-semantics/README.md) already taught you: ownership transfers, it isn't borrowed. `for x in v` has no special syntax of its own — it is only an ordinary call to `IntoIterator::into_iter(v)`, and that function takes `v` by value. After that line, `v` is exactly as gone as it would be after any other function that took its argument by value.

That is exactly why `for t in &titles` is a different thing entirely: there, the argument is `&titles`, not `titles` — a reference gets passed, `into_iter(self: &Vec<T>)` (the table's first row) is called, and that `self` only borrows. The three-forms convention from the previous section is exactly what decides this: which `impl` gets called is exactly what determines whether `v` is still yours afterward.

---

## Hands on

```sh
cargo run -p p2-02-04-implementing-iterator --example 01-fibonacci-manual-next
cargo run -p p2-02-04-implementing-iterator --example 02-for-loop-is-next-in-a-loop
cargo run -p p2-02-04-implementing-iterator --example 03-fibonacci-adapters-and-consumers-free
cargo run -p p2-02-04-implementing-iterator --example 04-watchlist-into-iterator-by-value
cargo run -p p2-02-04-implementing-iterator --example 05-vec-three-forms-recap
```

Then the four broken ones:

```sh
cargo run -p p2-02-04-implementing-iterator --example 06-missing-next-method --features broken
cargo run -p p2-02-04-implementing-iterator --example 07-next-wrong-self-mutability --features broken
cargo run -p p2-02-04-implementing-iterator --example 08-use-after-move --features broken
cargo run -p p2-02-04-implementing-iterator --example 09-reference-not-into-iterator --features broken
```

Then try these:

1. In `01-fibonacci-manual-next.rs`, change the starting values from `current: 0, next: 1` to `current: 1, next: 1`. How does the sequence change? Is it still genuinely Fibonacci?
2. In `03-fibonacci-adapters-and-consumers-free.rs`, add a line that squares the first five values with `.map(|n| n * n)` and `.collect()`s them. Did you need to write anything new to make that work?
3. In `05-vec-three-forms-recap.rs`, right before the final loop (`for t in titles`), try also printing `titles.len()`. Does it compile? Why does the exact same question have a different answer for the *first* loop (`for t in &titles`)?

---

## Errors you will meet

### `E0046` — a trait item is missing: `next`

```rust
struct Fibonacci {
    current: u64,
    next: u64,
}
impl Iterator for Fibonacci {
    type Item = u64;
}
```

```text
error[E0046]: not all trait items implemented, missing: `next`
  --> phase2-intermediate\02-iterators-and-closures\04-implementing-iterator\examples\06-missing-next-method.rs:23:1
   |
23 | impl Iterator for Fibonacci {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `next` in implementation
   |
   = help: implement the missing item: `fn next(&mut self) -> Option<<Self as Iterator>::Item> { todo!() }`

For more information about this error, try `rustc --explain E0046`.
```

**What the compiler is objecting to:** the associated type `Item` was written, but `Iterator` asks for one more thing — the one required method — and it is missing. Unlike adapters such as `.map()`, which ship with a default implementation, `next` has no default at all; there is nothing to build a default *on*, since it is the foundation everything else is built on.

**The fix:** write `next`.

**Why this is the fix:** the compiler's own help line hands you the exact signature — `fn next(&mut self) -> Option<<Self as Iterator>::Item>`. Add that one method, with a body that actually advances the state (not just the `todo!()` it suggested), and the error goes away.

### `E0053` — `next`'s signature does not match the trait

```rust
impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&self) -> Option<u64> {
        Some(self.current)
    }
}
```

```text
error[E0053]: method `next` has an incompatible type for trait
  --> phase2-intermediate\02-iterators-and-closures\04-implementing-iterator\examples\07-next-wrong-self-mutability.rs:26:13
   |
26 |     fn next(&self) -> Option<u64> {
   |             ^^^^^ types differ in mutability
   |
   = note: expected signature `fn(&mut Fibonacci) -> Option<_>`
              found signature `fn(&Fibonacci) -> Option<_>`
help: change the self-receiver type to match the trait
   |
26 |     fn next(&mut self) -> Option<u64> {
   |              +++

For more information about this error, try `rustc --explain E0053`.
```

**What the compiler is objecting to:** `Iterator` defines an exact signature for `next` — `&mut self`, not `&self` — and this implementation does not match it. This is a very ordinary slip: forgetting `&mut`, of all places, exactly where the state needs to change.

**The fix:** write `&mut self`.

**Why this is the fix:** `next` cannot advance the sequence at all without the power to change state — the second call would return exactly what the first one did. `&mut self` is exactly the permission `self.current = ...` needs; it is exactly what the first part of "The concept" already stressed.

### `E0382` — borrow of a moved value: `queue`

```rust
let queue = WatchList(vec!["Frieren".to_string(), "Bocchi the Rock!".to_string()]);

for title in queue {
    println!("now watching: {title}");
}

println!("queue again: {queue:?}");
```

```text
error[E0382]: borrow of moved value: `queue`
   --> phase2-intermediate\02-iterators-and-closures\04-implementing-iterator\examples\08-use-after-move.rs:28:29
    |
 22 |     let queue = WatchList(vec!["Frieren".to_string(), "Bocchi the Rock!".to_string()]);
    |         ----- move occurs because `queue` has type `WatchList`, which does not implement the `Copy` trait
 23 |
 24 |     for title in queue {
    |                  ----- `queue` moved due to this implicit call to `.into_iter()`
...
 28 |     println!("queue again: {queue:?}");
    |                             ^^^^^ value borrowed here after move
    |
note: `into_iter` takes ownership of the receiver `self`, which moves `queue`
   --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\iter\traits\collect.rs:312:18
    |
312 |     fn into_iter(self) -> Self::IntoIter;
    |                  ^^^^

For more information about this error, try `rustc --explain E0382`.
```

**What the compiler is objecting to:** the message names the mechanism itself — "`queue` moved due to this implicit call to `.into_iter()`." This is exactly what "Why `for x in v` moves `v`" explained, this time on your own type instead of `Vec`. `for title in queue` called `WatchList::into_iter(queue)`, that method took `self` by value, and `queue` moved. The final line wants to read from `queue` again — but there is nothing left to read.

**The fix:** get whatever you need from `queue` before or during the loop — not after:

```rust
println!("queue before: {queue:?}");
for title in queue {
    println!("now watching: {title}");
}
```

**Why this is the fix:** `queue` is entirely yours right up until the loop; only after it does it disappear. Reordering — or collecting whatever you need into a fresh variable during the loop — is exactly the rule [1.2.2](../../../phase1-fundamentals/02-ownership-and-memory/02-move-semantics/README.md) already taught you: after a move, only the new destination is valid.

### `E0277` — `&WatchList` is not an iterator

```rust
let queue = WatchList(vec!["Frieren".to_string()]);

for title in &queue {
    println!("now watching: {title}");
}
```

```text
error[E0277]: `&WatchList` is not an iterator
  --> phase2-intermediate\02-iterators-and-closures\04-implementing-iterator\examples\09-reference-not-into-iterator.rs:24:18
   |
24 |     for title in &queue {
   |                  ^^^^^^ `&WatchList` is not an iterator
   |
   = help: the trait `Iterator` is not implemented for `&WatchList`
   = note: required for `&WatchList` to implement `IntoIterator`
help: consider removing the leading `&`-reference
   |
24 -     for title in &queue {
24 +     for title in queue {
   |

For more information about this error, try `rustc --explain E0277`.
```

**What the compiler is objecting to:** this is the "three-forms convention" lesson meeting reality. `impl IntoIterator for WatchList` was written — the table's third row — but `impl IntoIterator for &WatchList` (the first row) never was. The two `impl` blocks are entirely separate; writing one does not bring the other along. `&WatchList` is a different type, and nobody wrote its trait.

**The fix:** work with what was actually implemented — by value, not by reference:

```rust
for title in queue {
    println!("now watching: {title}");
}
```

**Why this is the fix:** the compiler suggested exactly this — "consider removing the leading `&`-reference." Writing `impl IntoIterator for &WatchList` is also a valid fix, but it needs a tool this lesson has not given you yet: a named lifetime on the `impl` itself, because now you have to say how long the borrowed values stay valid. [2.4.1](../../04-lifetimes-and-conversion/01-lifetime-basics-and-elision/README.md) hands you exactly that tool.

---

## Exercises

### Warm up

<details>
<summary>Picture a struct <code>Doubler { value: u32 }</code> with this implementation — <code>fn next(&mut self) -> Option&lt;u32&gt; { self.value *= 2; Some(self.value) }</code>. If you build <code>Doubler { value: 3 }</code> and call <code>.next()</code> three times, what comes back?</summary>

Write your answer down before reading on.

</details>

<details>
<summary>Answer</summary>

```text
Some(6)
Some(12)
Some(24)
```

Each call doubles the value and returns it — and since it never produces a `None`, this iterator, exactly like `Fibonacci`, is infinite.

</details>

<details>
<summary>Same <code>Doubler</code>, but this time <code>fn next(&mut self) -> u32 { self.value *= 2; self.value }</code> — no <code>Option</code> at all. Does it compile?</summary>

Write your answer down before reading on.

</details>

<details>
<summary>Answer</summary>

No — `E0053`, the same family of error you saw in "Errors you will meet," this time about the return type instead of `self`. `Iterator::next` must return exactly `Option<Self::Item>`; returning the bare value, with no `Option`, does not match the signature.

</details>

<details>
<summary><code>let v = vec![1, 2, 3]; for x in v {} println!("{v:?}");</code> — does this compile?</summary>

Write your answer down before reading on.

</details>

<details>
<summary>Answer</summary>

No — `E0382`. `for x in v` calls `IntoIterator::into_iter(v)`, which takes `self` by value; `v` is moved, and the `println!` line after it has nothing left to read.

</details>

<details>
<summary>Same code, but <code>for x in &v {}</code> instead of <code>for x in v {}</code>. Now what?</summary>

Write your answer down before reading on.

</details>

<details>
<summary>Answer</summary>

Yes, it compiles. `&v` calls `IntoIterator for &Vec<T>`, which only borrows `self`; `v` is entirely yours again once the loop ends.

</details>

<details>
<summary>True or false: for <code>.filter()</code> to work on your own type, you have to write <code>.filter()</code> for it yourself.</summary>

Write your answer down before reading on.

</details>

<details>
<summary>Answer</summary>

False. The only thing you have to write is `next()`. `.filter()` — like `.map()`, `.take()`, `.collect()`, and dozens of others — has a default implementation the standard library wrote once, on the `Iterator` trait itself; any type with a `next()` gets all of them for free.

</details>

### Repair

Fix all four broken examples:

1. `examples/06-missing-next-method.rs` — add the missing method; it should produce the same Fibonacci sequence examples 01 and 02 did.
2. `examples/07-next-wrong-self-mutability.rs` — match `next`'s signature to the trait, and fill in a body that genuinely advances the state, not just returns `current`.
3. `examples/08-use-after-move.rs` — without dropping whatever the final `println!` was showing, restructure the code so it compiles. (Hint: print what you need before the loop, or collect it during the loop.)
4. `examples/09-reference-not-into-iterator.rs` — change the call site to work with what `WatchList` actually implements.

### Implement

Two types in `src/lib.rs`, each exercising one of this lesson's two traits:

```sh
cargo test -p p2-02-04-implementing-iterator
```

`Collatz` needs the `Iterator` trait (just `next`; the struct and `new` are already written for you), and `EpisodeLog` needs `IntoIterator` (just `into_iter`). Each doc comment states exactly what behavior is required — don't guess at anything.

### Build

Add a brand-new custom iterator of your own design to `src/lib.rs` — anything you like (powers of two, a counter that steps by some amount, whatever you want), with two requirements: (1) it must genuinely be a state machine — the next value has to be computed from something the struct stores, not out of nowhere; (2) in its doc comment, in a sentence or two, say what sequence it produces and why the state needs to persist between calls. Then add a `#[test]` that calls at least one adapter and one consumer on it that you did not write yourself — so you can see with your own eyes that they work for free.

### Challenge (optional)

Implement `IntoIterator for &EpisodeLog` so that `for title in &log` also compiles, without moving `log`. One real difference from everything you've written so far: its signature will need to look something like `impl<'a> IntoIterator for &'a EpisodeLog` — a named lifetime on the `impl` itself. That tool is formally [2.4.1](../../04-lifetimes-and-conversion/01-lifetime-basics-and-elision/README.md)'s subject; if you try it now, the compiler's messages, followed one at a time, will mostly show you the way. If you'd rather wait, that lesson hands you the tool properly and you can come back to this afterward.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `Iterator` | a trait with one required method, `next` | building your own type that `map`/`filter`/`collect` work on |
| `next(&mut self) -> Option<Self::Item>` | the only method you have to write | every custom iterator |
| associated type (`type Item`) | a type the trait's implementer fills in | saying what `next` produces |
| `IntoIterator` | the trait that makes `for x in value` legal | writing a type usable in a `for` loop |
| the three-forms convention | by value / by shared reference / by mutable reference | deciding how `for` behaves on your own type |

### What you now know

- `Iterator` needs exactly one required method, `next(&mut self) -> Option<Self::Item>`, plus one associated type, `Item`; those two things alone bring every adapter and consumer [2.2.2](../02-iterator-adapters/README.md) and [2.2.3](../03-consuming-and-collecting/README.md) taught you, for free.
- `for value in iter { BODY }` is nothing but a `loop` that calls `.next()` and breaks on the first `None` — the compiler writes this for you.
- `IntoIterator` is a separate trait: the thing `for x in value` actually calls. Having `Iterator` does not automatically give you `IntoIterator`; it has to be implemented on its own.
- A well-behaved type implements `IntoIterator` three times — by value, by shared reference, by mutable reference — on three separate types (`T`, `&T`, `&mut T`); implementing one does not bring the others along.
- `for x in v` moves `v` because that loop calls `IntoIterator::into_iter(v)`, and that method takes `self` by value — the same move semantics [1.2.2](../../../phase1-fundamentals/02-ownership-and-memory/02-move-semantics/README.md) already taught you, now with the exact name of the mechanism behind it.

### What comes back later

- **Associated types, in full** — [2.3.5 — Associated types vs. generic parameters](../../03-traits-and-generics/05-associated-types/README.md)
- **Writing your own trait** (not just implementing one that already exists) — [2.3.1 — Defining and implementing traits](../../03-traits-and-generics/01-defining-and-implementing-traits/README.md)
- **Lifetimes, for implementing `IntoIterator for &EpisodeLog`** — [2.4.1 — Lifetime basics and elision](../../04-lifetimes-and-conversion/01-lifetime-basics-and-elision/README.md)
- **Blanket implementations, formally — the exact reason every `Iterator` is already an `IntoIterator` too** — [2.3.6 — Supertraits, blanket impls, the orphan rule](../../03-traits-and-generics/06-supertraits-blanket-impls-orphan-rule/README.md)
- **Iterator laziness and performance, in depth** — [2.2.5 — Laziness and iterator performance](../05-laziness-and-performance/README.md)

### Can you explain?

- Why do methods like `.map()`, `.filter()`, and `.collect()` work on your own type just from writing `next()`? What is actually happening inside Rust?
- What does `IntoIterator` legalize that `Iterator` alone did not?
- Why does `for x in v` move `v`, but `for x in &v` does not? Which method and which signature does that trace back to?
- Why doesn't implementing `IntoIterator` for `WatchList` automatically give you `IntoIterator` for `&WatchList` too?
- What does `type Item` do that keeps `Iterator` itself from being generic?

---

## Going further

- [`Iterator` trait documentation](https://doc.rust-lang.org/std/iter/trait.Iterator.html) — the full list of the dozens of methods you got for free today, just by writing one `next()`.
- [`IntoIterator` trait documentation](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html) — including the blanket implementation that makes every `Iterator` an `IntoIterator` too.
- [The `std::iter` module](https://doc.rust-lang.org/std/iter/index.html) — the official explanation of what you saw in "A `for` loop is nothing but `loop` and `match` on `next`."
- [The Rust Book, ch. 13.2 — Processing a Series of Items with Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html)
