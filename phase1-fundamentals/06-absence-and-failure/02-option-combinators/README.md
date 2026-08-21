# 1.6.2 — `Option` combinators

## At a glance

After this lesson you can:

- Say what `.map()` and `.and_then()` do differently, and why using the wrong one gives you a nested `Option<Option<T>>`.
- Write the same piece of logic as both a `match` and a combinator chain, and honestly say which one reads better here.
- Say, with real evidence rather than a guess, why `.unwrap_or_else(|| expensive())` can be strictly cheaper than `.unwrap_or(expensive())`.
- Pick the right one out of roughly ten `Option` methods — `.filter()`, `.or()`, `.take()`, `.zip()` and the rest — for a given job.

**Time:** ~70 minutes · **Prerequisites:** [1.6.1 — `Option` and null safety](../01-option-and-null-safety/README.md)

---

## Why this matters

The last lesson gave you one tool: `match` on `Option`. It can do anything — the compiler forces you to cover both `Some` and `None`, and that's exactly what makes it safe.

But try a small experiment: write code that takes an `Option<u32>`, adds one to it if present, and returns zero otherwise. With `match` that's three lines — `Some(x) => x + 1`, `None => 0`, and the braces. Now imagine five of those back to back: fetch, filter, transform, substitute a default. Pure `match` nests you into the ground.

Real Rust code does this with a chain of **combinators** instead — methods like `.filter()`, `.map()` and `.unwrap_or()` that stack one after another: `opt.filter(...).map(...).unwrap_or(...)`. Shorter, and — used correctly — more readable too. But there are two common places to trip, and this lesson exists for exactly those:

First, **`.map()` and `.and_then()` are not interchangeable**, and mixing them up produces something the compiler rejects — an `Option` inside another `Option`. This is the single most common snag newcomers hit with this part of the language.

Second, some of these methods evaluate their argument **every time**, whether it's needed or not, and others only **when it's actually needed**. That difference has a real cost, and it's hidden right there in the method signature — you have to know which is which.

So this lesson gives you two things: the vocabulary of the combinators, and the judgment for when to reach for them and when not to.

---

## The concept

### Recap: `match` on `Option`

```rust
let age: Option<u32> = Some(25);
let described = match age {
    Some(a) => format!("{a} years old"),
    None => "unknown age".to_string(),
};
println!("{described}");
```

```text
25 years old
```

This is exactly what you learned in 1.6.1: two branches, both mandatory. It's entirely correct. The question this lesson asks isn't whether `match` is right — it always is — but whether it's the best tool for *this particular* small transform.

### Closures, in one sentence

Before the combinators, you need one small thing: every combinator you're about to meet takes a **closure** as its argument.

```rust
let double = |x: i32| x * 2;
println!("double(21) = {}", double(21));
```

```text
double(21) = 42
```

A closure is just an inline function you pass around as a value — no name, no separate definition. `|x: i32| x * 2` means "a function that takes an `i32` and returns double it." That's all you need for this lesson; closures have their own rules — what they borrow from their surroundings, what they take ownership of — and those get a full lesson of their own in Phase 2.

### `.map()` — transforming the value inside `Some`

```rust
let n: Option<i32> = Some(4);
let doubled: Option<i32> = n.map(|x| x * 2);
println!("{doubled:?}");
```

```text
Some(8)
```

`.map()` runs your closure *only if* the value is `Some`, puts the result back in `Some`, and hands back `None` untouched if that's what it got. It's exactly what you saw in 1.6.1 — this time, instead of writing the `match` yourself, you call it by name.

### When the transform itself might fail

The trouble starts when the function you hand to `.map()` itself returns an `Option`:

```rust
fn double_if_positive(n: i32) -> Option<i32> {
    if n > 0 { Some(n * 2) } else { None }
}

let n: Option<i32> = Some(4);
let mapped: Option<Option<i32>> = n.map(double_if_positive);
println!("{mapped:?}");
```

```text
Some(Some(8))
```

Look at that: **`Some(Some(8))`**. An `Option` inside another `Option`. `.map()` did exactly its job — it ran the closure and put the result in `Some` — but this time the closure's result was already an `Option`, so you got an extra layer.

`.and_then()` is the fix:

```rust
let chained: Option<i32> = n.and_then(double_if_positive);
println!("{chained:?}");
```

```text
Some(8)
```

`.and_then()` does exactly what `.map()` did — runs the closure on the value inside `Some` — with one difference: instead of wrapping the result in `Some` again, it hands the result back as-is. Because your function already returned an `Option`, there's nothing left to wrap. This is called **flattening**: `.and_then()` removes a layer of nesting that `.map()` would have created.

```senpai-visual
{"kind":"concept","labels":["Some(n)",".map(f)","Some(Some(_))",".and_then(f)","Some(_)"]}
```

The practical rule is to look at the signature of the function you're calling:

| If your function returns | use |
|---|---|
| a plain value (`T`) | `.map()` |
| an `Option<T>` itself | `.and_then()` |

If you're unsure, the declared type tells you: write `let x: Option<i32> = ...` and if the compiler complains it found `Option<Option<i32>>`, you have your answer right there.

### `.filter()` — keeping the value conditionally

```rust
let n: Option<i32> = Some(8);
println!("{:?}", n.filter(|v| *v % 2 == 0));
println!("{:?}", n.filter(|v| *v % 2 != 0));
```

```text
Some(8)
None
```

`.filter()` takes a predicate. If the value is `Some` and the predicate returns `true` on it, the same `Some` comes back; otherwise — whether it was already `None` or the predicate said `false` — the result is `None`.

One subtlety: `.filter()`'s closure doesn't receive the value, it receives a **reference** to it — `v` here has type `&i32`, not `i32`. That's why we wrote `*v % 2`. This is the same `Option<&T>` you met in 1.6.1: `.filter()` has to be able to still hold the value if the predicate rejects it, so it only lets you look.

### `.or()` and `.or_else()` — falling back to a different `Option`

```rust
let primary: Option<i32> = None;
println!("{:?}", primary.or(Some(99)));
println!("{:?}", primary.or_else(|| Some(100)));
```

```text
Some(99)
Some(100)
```

If `primary` had already been `Some`, that same value would have come back and the argument would never have mattered. These two are siblings of `.unwrap_or()`/`.unwrap_or_else()`, coming up next: those substitute a *value* when you're `None`; these substitute a whole *other `Option`*.

### `.unwrap_or()`, `.unwrap_or_else()` and `.unwrap_or_default()` — eager versus lazy

This is where carelessness has a real cost. Compare these two lines:

```rust
fn expensive_default() -> i32 {
    println!("    ... expensive_default() ran ...");
    42
}

let present: Option<i32> = Some(7);
let a = present.unwrap_or(expensive_default());
println!("unwrap_or:      {a}");
let b = present.unwrap_or_else(expensive_default);
println!("unwrap_or_else: {b}");
```

```text
    ... expensive_default() ran ...
unwrap_or:      7
unwrap_or_else: 7
```

Both lines gave the right answer: `7`. But look at that first line of output — it printed exactly once, and it happened for `unwrap_or`, not `unwrap_or_else`. `present` was already `Some(7)`; neither line actually needed the default.

The reason is in the method signatures:

| method | argument | runs when |
|---|---|---|
| `.unwrap_or(x)` | a **value** | always — before `.unwrap_or` is even called |
| `.unwrap_or_else(f)` | a **closure** | only if the `Option` is actually `None` |

When Rust sees `present.unwrap_or(expensive_default())`, it has to run `expensive_default()` first just to have the argument — that's how function calls always work, whether the result ends up being thrown away or not. This is called **eager** evaluation. `.unwrap_or_else()` instead takes the *function itself*, not its result, and only calls it if it actually sees `None` — **lazy** evaluation.

For `0` or `""` you won't feel the difference. For something that queries a database, reads a file, or just builds a large `Vec`, it's a real one — and the compiler gives you no warning either way, because both are perfectly valid code.

> **Rule of thumb:** if your default is a plain literal (`0`, `""`, `Vec::new()`), `.unwrap_or()` is fine. If it's behind a function call, write `.unwrap_or_else(|| ...)`.

There's a third sibling, for when the default is whatever the type itself considers default:

```rust
let n: Option<i32> = None;
println!("{}", n.unwrap_or_default());
```

```text
0
```

`.unwrap_or_default()` takes no argument at all — it reaches for the type's `Default` implementation (for `i32`, that's zero). It's neither eager nor lazy in the sense above; there's nothing to run.

### `.ok_or()` and `.ok_or_else()` — the bridge to `Result`

Just a glance, without going into the details:

```text
Some(8).ok_or("missing")            -> Ok(8)
None.ok_or("missing")               -> Err("missing")
None.ok_or_else(|| "missing".into()) -> Err("missing")
```

`.ok_or()` turns an `Option<T>` into a `Result<T, E>`: `Some` becomes `Ok`, and `None` becomes `Err` carrying whatever you gave it. `.ok_or_else()` is its lazy twin — exactly the same `unwrap_or`/`unwrap_or_else` pattern you just saw. The full story of `Result` belongs to [1.6.3](../03-result-and-question-mark/README.md); for now, just know the bridge exists.

### `.take()` and `.replace()` — mutating in place

These two want a `&mut Option<T>`, not an `Option<T>` — they act on the variable itself:

```rust
let mut slot: Option<String> = Some("draft".to_string());
let taken = slot.take();
println!("taken={taken:?} slot={slot:?}");
```

```text
taken=Some("draft") slot=None
```

`.take()` pulls out whatever was in `slot` and leaves `None` behind. `.replace()` is the mirror: it puts a new value in and hands back whatever was there before:

```rust
let mut counter: Option<u32> = Some(1);
let previous = counter.replace(2);
println!("previous={previous:?} counter={counter:?}");
```

```text
previous=Some(1) counter=Some(2)
```

Neither one clones — they're pure ownership moves, the same thing you saw back in 1.2.2.

### `.as_ref()`, `.as_mut()`, `.cloned()` and `.copied()` — looking without taking ownership

```rust
let owned: Option<String> = Some("hello".to_string());
let length: Option<usize> = owned.as_ref().map(|s| s.len());
println!("length={length:?} owned still usable: {owned:?}");
```

```text
length=Some(5) owned still usable: Some("hello")
```

`owned.as_ref()` gives you an `Option<&String>` — a look inside, without moving the `String` out. That's why you can still use `owned` after the `.map()`. This is the same `Option<&T>` introduced in 1.6.1; here you see it put to work.

`.as_mut()` does the same thing with mutable access:

```rust
let mut editable: Option<String> = Some("hi".to_string());
if let Some(s) = editable.as_mut() {
    s.push('!');
}
println!("{editable:?}");
```

```text
Some("hi!")
```

And when you have an `Option<&T>` but want an owned `Option<T>` back, `.cloned()` (which clones) and `.copied()` (which copies, for `Copy` types) run that path in reverse:

```rust
let cloned: Option<String> = owned.as_ref().cloned();
println!("{cloned:?}");
```

```text
Some("hello")
```

### `.zip()` — combining two `Option`s

```rust
let x: Option<i32> = Some(3);
let y: Option<i32> = Some(4);
println!("{:?}", x.zip(y));
println!("{:?}", x.zip(None::<i32>));
```

```text
Some((3, 4))
None
```

`.zip()` pairs two `Option`s into a tuple, but only if **both** are `Some`. If either side is `None`, the whole result is `None` — no nesting involved here, because neither side is a function that itself returns an `Option`.

### `.is_some_and()` — a condition without unwrapping

```rust
let n: Option<i32> = Some(8);
println!("{}", n.is_some_and(|v| v > 5));
println!("{}", n.is_some_and(|v| v > 100));
```

```text
true
false
```

`.is_some_and()` gives you a `bool`: `true` only if the value is `Some` *and* the predicate holds on it. Shorter than `matches!(n, Some(v) if v > 5)` when all you need is a `bool`, not the value itself.

### `match` or chain? The judgment is yours

Now that you have the vocabulary, the real question remains. These two functions do exactly the same thing:

```rust
fn greeting_match(name: Option<&str>) -> String {
    match name {
        Some(n) if !n.is_empty() => format!("Hello, {n}!"),
        _ => "Hello, stranger!".to_string(),
    }
}

fn greeting_combinator(name: Option<&str>) -> String {
    name.filter(|n| !n.is_empty())
        .map(|n| format!("Hello, {n}!"))
        .unwrap_or_else(|| "Hello, stranger!".to_string())
}
```

```text
name=Some("Sam") -> match="Hello, Sam!" combinator="Hello, Sam!" (equal: true)
name=Some("") -> match="Hello, stranger!" combinator="Hello, stranger!" (equal: true)
name=None -> match="Hello, stranger!" combinator="Hello, stranger!" (equal: true)
```

Here the chain wins: each line is exactly one step — filter, transform, default — and it reads in one glance.

But look at this one:

```rust
fn shipping_cost_combinator(weight_kg: Option<f64>, express: bool) -> f64 {
    weight_kg
        .map(|w| {
            if w <= 0.0 {
                0.0
            } else if express {
                w * 2.5 + 10.0
            } else {
                w * 1.2
            }
        })
        .unwrap_or(5.0)
}
```

Three different rules — zero weight, express, standard — all crammed inside one closure, and `express` still has to be captured from outside. Here's the `match` version:

```rust
fn shipping_cost_match(weight_kg: Option<f64>, express: bool) -> f64 {
    match weight_kg {
        Some(w) if w <= 0.0 => 0.0,
        Some(w) if express => w * 2.5 + 10.0,
        Some(w) => w * 1.2,
        None => 5.0,
    }
}
```

```text
weight=Some(0.0), express=false -> match=0 combinator=0 (equal: true)
weight=Some(2.0), express=true -> match=15 combinator=15 (equal: true)
weight=Some(2.0), express=false -> match=2.4 combinator=2.4 (equal: true)
weight=None, express=true -> match=5 combinator=5 (equal: true)
```

Both give the same answer. But in the `match` version, every rule sits right next to its condition — one glance and you know what happens when. The chain just relocated the branching, it didn't simplify it. **When you have several independent rules, especially if one of them needs something from outside the `Option`, `match` wins.**

The general rule: one transform with one default → chain. Several separate rules, or branches that genuinely differ in kind → `match`. If you're not sure, write both and keep whichever is shorter *and* clearer.

---

## Hands on

```sh
cargo run -p p1-06-02-option-combinators --example 01-map-and-and-then
cargo run -p p1-06-02-option-combinators --example 02-match-vs-combinator
cargo run -p p1-06-02-option-combinators --example 03-eager-vs-lazy
cargo run -p p1-06-02-option-combinators --example 04-mutating-and-borrowing
```

Then the two broken ones:

```sh
cargo run -p p1-06-02-option-combinators --example 05-map-instead-of-and-then --features broken
cargo run -p p1-06-02-option-combinators --example 06-map-moves-the-option --features broken
```

Then try these:

1. In `03-eager-vs-lazy`, add one more line: call `absent.unwrap_or_else(expensive_default)` and see whether the message prints this time, and why.
2. In `04-mutating-and-borrowing`, instead of `x.zip(None::<i32>)`, write `x.zip(y).zip(Some(true))`. What's the type of the result?
3. In `02-match-vs-combinator`, rewrite `shipping_cost_combinator` to use two `.filter()` calls back to back instead of the nested `if`/`else if`. Is it more readable now?

---

## Errors you will meet

### `E0308` — `.map()` where `.and_then()` was needed

```text
error[E0308]: mismatched types
  --> phase1-fundamentals\06-absence-and-failure\02-option-combinators\examples\05-map-instead-of-and-then.rs:22:32
   |
22 |     let timeout: Option<i32> = find_setting("timeout").map(double_if_positive);
   |                  -----------   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Option<i32>`, found `Option<Option<i32>>`
   |                  |
   |                  expected due to this
   |
   = note: expected enum `Option<i32>`
              found enum `Option<Option<i32>>`
help: consider using `Option::expect` to unwrap the `Option<Option<i32>>` value, panicking if the value is an `Option::None`
   |
22 |     let timeout: Option<i32> = find_setting("timeout").map(double_if_positive).expect("REASON");
   |                                                                               +++++++++++++++++
```

**What the compiler is objecting to:** the type annotation says `timeout` must be `Option<i32>`. On the right, `find_setting("timeout")` gives an `Option<i32>`, and `.map(double_if_positive)` runs on it — but `double_if_positive` itself returns `Option<i32>`, not `i32`. So `.map()` wraps that `Option<i32>` inside another `Some`, and the final result is `Option<Option<i32>>`. The two types don't match.

**Fix 1:** change `.map()` to `.and_then()` — exactly what you saw a few sections ago.

**Fix 2:** keep `.map()`, but add a `.flatten()` after it — a method that does exactly this: turns an `Option<Option<T>>` into an `Option<T>`.

**Why these are the fix:** the `help` the compiler offers — `.expect("REASON")` — is not the right move; it just silences the compiler and hides a panic where the real problem was. The compiler has no way to know *why* you ended up with a nested `Option`, only how to get rid of one. The actual decision — `.and_then()` or `.flatten()` — is yours, because only you know what `double_if_positive` was supposed to do.

### `E0382` — `.map()` takes ownership of the `Option`

```text
error[E0382]: borrow of moved value: `name`
    --> phase1-fundamentals\06-absence-and-failure\02-option-combinators\examples\06-map-moves-the-option.rs:8:22
     |
   6 |     let name: Option<String> = Some("Sam".to_string());
     |         ---- move occurs because `name` has type `Option<String>`, which does not implement the `Copy` trait
   7 |     let length: Option<usize> = name.map(|s| s.len());
     |                                      ---------------- `name` moved due to this method call
   8 |     println!("name: {name:?}, length: {length:?}");
     |                      ^^^^ value borrowed here after move
     |
note: `Option::<T>::map` takes ownership of the receiver `self`, which moves `name`
help: consider calling `.as_ref()` to borrow the value's contents
     |
   7 |     let length: Option<usize> = name.as_ref().map(|s| s.len());
     |                                     +++++++++
help: consider calling `.as_mut()` to mutably borrow the value's contents
     |
   7 |     let length: Option<usize> = name.as_mut().map(|s| s.len());
     |                                     +++++++++
help: you can `clone` the value and consume it, but this might not be your desired behavior
     |
   7 |     let length: Option<usize> = name.clone().map(|s| s.len());
     |                                     ++++++++
```

(A `note` pointing at your local copy of the standard library's source file was trimmed here — the path is different on every machine, but the message itself is right above.)

**What the compiler is objecting to:** this is the same `E0382` — use after move — you met in [1.2.2](../../02-ownership-and-memory/02-move-semantics/README.md), except this time a method is the culprit, not an assignment. `Option::map` takes ownership of `self`. When you called `name.map(...)`, `name` **moved** into that call, and there was nothing left for `println!` to read.

**Fix:** the exact same table from 1.2.3 applies here too — and the compiler suggested all three options itself:

| if you wanted to | write |
|---|---|
| just look, keep ownership | `name.as_ref().map(...)` |
| edit in place | `name.as_mut().map(...)` |
| a separate copy to consume | `name.clone().map(...)` |

**Why this is the fix:** notice this error is already applying exactly what you read a few sections ago — `.as_ref()`, for when you still need the original value. That's not a coincidence: `.map()` on an `Option<String>` (or any non-`Copy` type) always takes ownership, and that's exactly the moment `.as_ref()` exists for.

---

## Exercises

### Warm up

<details>
<summary>What type does <code>n.map(f)</code> return when <code>f</code> itself returns <code>Option&lt;T&gt;</code>?</summary>

`Option<Option<T>>`. `.map()` wraps whatever the closure returns back into `Some`, even if it was already an `Option`.

</details>

<details>
<summary>Why does <code>opt.filter(|v| *v > 0)</code> need <code>*v</code> instead of just <code>v</code>?</summary>

Because `.filter()`'s closure receives a reference (`&T`), not the value itself — it needs to still hold the value if the predicate rejects it.

</details>

<details>
<summary>In <code>present.unwrap_or(compute())</code>, does <code>compute()</code> run even if <code>present</code> is already <code>Some</code>?</summary>

Yes, always. `.unwrap_or()` takes a value, not a closure, so Rust has to build that argument before the method is even called — used or not.

</details>

<details>
<summary>Does this compile?

```rust
let a: Option<i32> = Some(5);
let b = a.map(|x| x + 1);
println!("{a:?} {b:?}");
```
</summary>

Yes. `i32` is `Copy`, so `.map()` takes a copy of the value inside `a` rather than moving `a` itself. `a` still works afterward — unlike the `Option<String>` case in the errors section.

</details>

<details>
<summary>Why can <code>.zip()</code> never produce a nested <code>Option</code>?</summary>

Because neither side is a function that itself returns an `Option` — `.zip()` puts the two values straight into a tuple. The nesting problem only shows up when the closure you pass to a combinator itself already returns an `Option`.

</details>

### Repair

Fix `examples/05-map-instead-of-and-then.rs` **two** ways:

1. By changing `.map()` to `.and_then()`.
2. By keeping `.map()` and adding a `.flatten()` after it.

Then fix `examples/06-map-moves-the-option.rs` with `.as_ref()`, so the `println!` line can reach both `name` and `length`.

### Implement

Six functions in `src/lib.rs`:

```sh
cargo test -p p1-06-02-option-combinators
```

Each one is naturally a single combinator (or a short chain of them), not a `match`. Look closely at `safe_half`: you have to choose between `.map()` and `.and_then()`, and the wrong choice won't compile — exactly the error you met above.

### Build

Write a `pub fn describe_stock(count: Option<u32>) -> String`, using a combinator chain (not `match`):

- If `count` is absent or zero: `"out of stock"`.
- If it's between 1 and 5: `"low stock: N left"` (with the real number in place of `N`).
- Otherwise: `"in stock"`.

Then write one sentence: would you have written this one as a `match` too? Why here and not in `greeting_match`/`greeting_combinator` above?

### Challenge (optional)

**Part one.** Write a `pub fn access_level(name: Option<&str>) -> String`: if `name` is absent or empty it returns `"guest"`, otherwise `"welcome, NAME"`. Pull the default from a separate function that prints when it's called — and see with your own eyes that it only runs for the absent/empty cases, not for every input.

**Part two.** Take any one function from `src/lib.rs` and write it both as a chain (what you just wrote) and as a `match`. Say in one sentence which one you'd keep in a real code review, and why.

**Part three.** Guess first, then run it: what does `Some(3).and_then(|x| Some(x).filter(|v| *v > 5)).or(Some(0))` evaluate to? It's three combinators chained — trace each step separately.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| closure | an inline function you pass as a value | the argument to every combinator |
| combinator | a method that builds `Option`/`Result` behavior without `match` | `.map()`, `.and_then()` and the rest |
| `.map()` | transforms, wraps back in `Some` | when the function returns a plain value |
| `.and_then()` | transforms, doesn't nest | when the function itself returns an `Option` |
| flattening | removing one layer of `Option<Option<T>>` | what `.and_then()`/`.flatten()` do |
| eager | the argument always runs | `.unwrap_or()`, `.or()` |
| lazy | the argument only runs when needed | `.unwrap_or_else()`, `.or_else()`, `.ok_or_else()` |
| `E0382` on `.map()` | the method took ownership, not just an assignment | `.as_ref()`/`.as_mut()`/`.clone()` are the fixes |

### What you now know

- `.map()` wraps the closure's result in `Some`; `.and_then()` doesn't, because its closure already wrapped it.
- A closure is the inline function you pass to combinators — `Fn`/`FnMut`/`FnOnce` and its borrowing rules belong to Phase 2.
- `.unwrap_or(x)` always builds `x`; `.unwrap_or_else(f)` only calls `f` if you're actually `None`.
- `.filter()`'s closure takes a `&T`, not a `T`.
- `.take()`/`.replace()` act on the variable itself (`&mut Option<T>`), not a copy.
- `.as_ref()`/`.as_mut()` look without taking ownership; `.cloned()`/`.copied()` run that path in reverse.
- `.ok_or()` is the bridge from `Option` to `Result` — its full story finishes in 1.6.3.
- One transform with one default usually wants a chain; several independent rules usually want `match`.

### What comes back later

- **`Result<T, E>` and the `?` operator** — [1.6.3](../03-result-and-question-mark/README.md)
- **Closures, `Fn`/`FnMut`/`FnOnce`, and borrowing the environment** — [Phase 2 — Closures and the `Fn` traits](../../../phase2-intermediate/02-iterators-and-closures/01-closures-and-fn-traits/README.md)
- **`.map()`/`.filter()` on iterators** (same method names, different job) — [Phase 2 — Iterator adapters](../../../phase2-intermediate/02-iterators-and-closures/02-iterator-adapters/README.md)
- **`.parse()` for turning a string into a number** — [1.6.3](../03-result-and-question-mark/README.md)
- **Panic policy: when `panic!` is the right call and when it isn't** — [1.6.4](../04-panic-vs-result/README.md)

### Can you explain?

- Why does `n.map(f)` produce an `Option<Option<T>>` when `f` itself returns an `Option`?
- What's a closure, in one sentence?
- Why does `present.unwrap_or(compute())` build the value of `compute()` even if it doesn't need it?
- Why does `.filter()`'s closure take a `&T` instead of a `T`?
- Name a situation where you'd prefer `match` over a chain, and say why.
- What does `.ok_or()` convert, into what?

---

## Going further

- [`std::option::Option` documentation](https://doc.rust-lang.org/std/option/enum.Option.html) — the full method list, with each one's exact signature.
- [The Rust Book — Closures](https://doc.rust-lang.org/book/ch13-01-closures.html) — you just saw the preview; the full version is Phase 2, but there's no harm reading ahead.
- [`clippy::manual_map`](https://rust-lang.github.io/rust-clippy/master/#manual_map) and [`clippy::unnecessary_lazy_evaluations`](https://rust-lang.github.io/rust-clippy/master/#unnecessary_lazy_evaluations) — two lints that automate exactly the judgment calls from today.
