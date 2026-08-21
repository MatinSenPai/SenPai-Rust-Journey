# 1.1.3 — Compound types and destructuring

## At a glance

After this lesson you can:

- Build a tuple and an array, and say what each one is for.
- Explain why `(u32, f64)` and `(f64, u32)` are different types, and why `[i32; 4]` isn't `[i32; 5]`.
- Take a compound value apart by writing its shape, instead of pulling out one field at a time.
- Say what happens when you run off the end of an array — and why that panic, unlike overflow, is still there in release.

**Time:** ~45 minutes · **Prerequisites:** [1.1.2 — Scalar types and overflow](../02-scalar-types-and-overflow/README.md)

---

## Why this matters

Every variable you've made so far has been one thing: a number, a `bool`. Real data isn't like that. A sensor sample is a time *and* a temperature *and* a humidity — three numbers that mean something together and nearly nothing apart.

In Python you solve this without thinking:

```python
sample = (1700000000, 21.5, 48.0)
timestamp, celsius, humidity = sample
```

And in Python that `sample` can quietly grow a fourth element later, or have a string in position zero, and nobody finds out until it explodes.

In Rust you write the same two lines — the syntax is nearly identical — but **how many elements there are, and what type each one is, is part of the value's type.** A function that wants a three-field sample can never be handed a two-field one, and whoever adds a field tomorrow gets *every* place that must change handed to them as compile errors.

And a second thing that has no Python equivalent: **a Rust array does not grow.** Its length is known up front and stays. That sounds like a restriction at first; it's the restriction that lets the compiler put the array on the stack and skip heap allocation entirely. You do get a growable list — it's called `Vec`, and [1.1.6](../06-vec-and-string-basics/README.md) is about it.

---

## The concept

Rust has two built-in ways to stick several values together, and the difference is one sentence:

| | count | types | example |
|---|---|---|---|
| **tuple** | fixed | may differ | `(1700000000, 21.5, true)` |
| **array** | fixed | all the same | `[12, 7, 19, 3, 14]` |

Both are fixed size. The difference is that a tuple groups *different* things and an array groups *alike* things.

### Tuples

A tuple's type is written exactly the way its value is built:

```rust
let sample: (u32, f64, bool) = (1_700_000_000, 21.5, true);
```

You reach fields by position, counting from zero:

```rust
println!("timestamp: {}", sample.0);
println!("celsius:   {}", sample.1);
println!("verified:  {}", sample.2);
```

```text
timestamp: 1700000000
celsius:   21.5
verified:  true
```

The point that's easy to miss: **position is part of the type.**

```rust
let pair: (u32, f64) = (3, 1.5);
let flipped: (f64, u32) = (1.5, 3);
```

These aren't "two numbers". They're two entirely different types. A function that wants `(u32, f64)` will not take `(f64, u32)`. If you're used to passing tuples around freely in Python, this is where Rust gets strict — and where it deletes a whole class of bug from your life.

Tuples nest, and the access chains:

```rust
let reading = ((10, 20), true);
println!("nested .0.1: {}", reading.0.1);
```

```text
nested .0.1: 20
```

Two special cases, worth seeing now:

```rust
let single = (7,);      // a one-element tuple — the trailing comma is required
let nothing: () = ();   // a zero-element tuple
```

```text
single:    (7,)
unit:      ()
```

Without that trailing comma, `(7)` is just the number 7 in brackets. And that empty `()` has a name of its own: **the unit type**. It's what a function that returns nothing actually returns. It won't matter much yet; but when you meet `Result<(), Error>` in [1.6.3](../../06-absence-and-failure/03-result-and-question-mark/README.md) — meaning "it worked and has nothing to say" — you'll want to know where that came from.

### What tuples are mostly for

**Returning more than one value from a function.**

```rust
fn divide(total: u32, per_box: u32) -> (u32, u32) {
    (total / per_box, total % per_box)
}
```

```text
47 in 6s:  7 boxes, 5 left over
```

In a language like C you'd have declared a `struct` for this, or passed an out-pointer. Here you just return the pair. This is by far the commonest reason you'll see a tuple in real code.

### Arrays

```rust
let readings: [i32; 5] = [12, 7, 19, 3, 14];
```

Read the type as "this type, this many of them". And **that number is part of the type** — in exactly the sense that position was part of a tuple's:

```rust
let four: [i32; 4] = [1, 2, 3, 4];
let five: [i32; 5] = [1, 2, 3, 4, 5];
// let same: [i32; 4] = five;   // ← E0308
```

If you want one value repeated, there's a shorthand:

```rust
let zeroed = [0_u8; 8];
```

```text
zeroed:    [0, 0, 0, 0, 0, 0, 0, 0]
```

You get the length from `.len()` and reach elements by index — again from zero:

```rust
println!("length:    {}", readings.len());
println!("first:     {}", readings[0]);
println!("last:      {}", readings[readings.len() - 1]);
```

```text
length:    5
first:     12
last:      14
```

> Get in the habit of writing the last element as `readings[readings.len() - 1]` rather than `readings[4]`. Both are correct here, because the length is written into the type. But the same code as a `Vec` turns `[4]` into a panic waiting for a short input.

### Every index is checked

Here Rust does something C does not. In C, reading `readings[5]` from a five-element array hands you whatever happened to be next in memory — silently — and writing there is where a security advisory starts. In Rust the program stops.

And there are two different moments at which it stops.

**If the compiler can work it out, it refuses to compile at all:**

```text
error: this operation will panic at runtime
 --> out-of-bounds.rs:3:20
  |
3 |     println!("{}", readings[5]);
  |                    ^^^^^^^^^^^ index out of bounds: the length is 5 but the index is 5
  |
  = note: `#[deny(unconditional_panic)]` on by default
```

**If the index is only known at run time, the program panics:**

```text
length:  5
safely:  None

thread 'main' (12124) panicked at examples\04-out-of-bounds.rs:20:29:
index out of bounds: the len is 5 but the index is 5
```

**And now the difference that matters.** In [1.1.2](../02-scalar-types-and-overflow/README.md) you saw the overflow panic disappear in release. This one does not:

```text
=== release ===
length:  5
safely:  None

thread 'main' (24804) panicked at examples\04-out-of-bounds.rs:20:29:
index out of bounds: the len is 5 but the index is 5
```

Identical. **Bounds checking is not an optimisation you can switch off; it's part of the definition of the language.** Yes, it costs something — one comparison per access — and Rust pays that gladly, because what it buys is that a wrong index can never reach the memory next door. (The compiler can often prove the check is unnecessary and remove it, particularly in loops. But that's its decision, not yours.)

If you'd rather the program didn't stop, don't demand — **ask**:

```rust
println!("get(2):    {:?}", readings.get(2));
println!("get(99):   {:?}", readings.get(99));
```

```text
get(2):    Some(19)
get(99):   None
```

`.get()` hands back an `Option` instead of panicking: `Some(value)` or `None`. The same thing you saw from `checked_add`. Its full lesson is [1.6.1](../../06-absence-and-failure/01-option-and-null-safety/README.md).

### Destructuring — writing the shape instead of picking fields out

This is the heart of the lesson.

```rust
let (timestamp, celsius, verified) = sample;
```

The left of the `=` is not three separate statements. **It's one pattern with the same shape as the value**, and the names in it get filled in. If you know Python you've seen this. The difference is that in Rust the compiler checks the shape reads correctly — the count and the types, right there.

`_` means "there's a field here and I don't want a name for it":

```rust
let (_, only_celsius, _) = sample;
```

```text
only:      21.5
```

Note that `_` doesn't excuse you from counting. The pattern still has to cover the whole shape; those two `_`s are you saying "three fields, and I know it".

Arrays come apart the same way, and the count is checked:

```rust
let corners = [1, 2, 3, 4];
let [top_left, top_right, bottom_left, bottom_right] = corners;
```

Patterns nest as deep as the value does:

```rust
let route = ((35.7, 51.4), (32.6, 51.7));
let ((from_lat, from_lon), (to_lat, to_lon)) = route;
```

```text
from:      35.7, 51.4
to:        32.6, 51.7
```

And because the right-hand side is fully evaluated before anything is bound, swapping two variables needs no temporary:

```rust
let mut a = 1;
let mut b = 2;
(a, b) = (b, a);
```

```text
before:    a=1 b=2
after:     a=2 b=1
```

> **Take this seriously:** patterns aren't a small feature of `let`. This is the same machinery behind `match`, behind `if let`, behind function arguments, and behind unwrapping `Option` and `Result`. Every pattern you meet for the rest of this course is this rule with a more interesting shape.

### Which one, when

| Situation | Take |
|---|---|
| two or three values travelling together briefly | tuple |
| returning several values from a function | tuple |
| several values of one type, count known up front | array |
| a list that grows | `Vec` — [1.1.6](../06-vec-and-string-basics/README.md) |
| anything whose fields want names | `struct` — [1.5.1](../../05-your-own-types/01-structs-and-methods/README.md) |

That last row is the important one. A tuple is bearable up to about three fields. Past that, `thing.3` becomes a puzzle and you owe the fields names.

---

## Hands on

```sh
cargo run -p p1-01-03-compound-types-and-destructuring --example 01-tuples
cargo run -p p1-01-03-compound-types-and-destructuring --example 02-arrays
cargo run -p p1-01-03-compound-types-and-destructuring --example 03-destructuring
```

Then see the bounds panic yourself, in both builds:

```sh
cargo run -p p1-01-03-compound-types-and-destructuring --example 04-out-of-bounds --features broken
cargo run --release -p p1-01-03-compound-types-and-destructuring --example 04-out-of-bounds --features broken
```

**Run both.** The output is the same, and that's exactly the point — compare it with the last lesson.

Then try:

1. In `01-tuples`, change `{sample:?}` to `println!("{sample}")`. Read the error and see what it suggests.
2. In `02-arrays`, uncomment `let same: [i32; 4] = five;`.
3. In `03-destructuring`, delete one of the names in the `corners` pattern and see what the compiler says.

---

## Errors you will meet

### `E0308` — the pattern wanted three fields and you wrote two

```text
error[E0308]: mismatched types
 --> examples\05-wrong-arity.rs:9:9
  |
9 |     let (timestamp, celsius) = sample;
  |         ^^^^^^^^^^^^^^^^^^^^   ------ this expression has type `(u32, f64, bool)`
  |         |
  |         expected a tuple with 3 elements, found one with 2 elements
  |
  = note: expected tuple `(u32, f64, bool)`
             found tuple `(_, _)`
```

**What the compiler is objecting to:** your pattern has two slots and the value has three fields. Rust won't decide for you which one to throw away.

**The fix:** name the third, or if you don't need it, put a `_` there: `let (timestamp, celsius, _) = sample;`

**Why that's the fix:** the `_` writes down your intent. The difference from forgetting is that six months later a reader knows the third field exists and was skipped deliberately — and, more usefully, if a fourth field is added tomorrow you get an error right here and have to look at it.

### `E0527` — the same thing, for arrays

```text
error[E0527]: pattern requires 3 elements but array has 4
  --> examples\05-wrong-arity.rs:14:9
   |
14 |     let [a, b, c] = corners;
   |         ^^^^^^^^^ expected 4 elements
```

**What the compiler is objecting to:** the same problem, but arrays get their own error code for it.

**The fix:** write all four slots, or if you only want the first few, ignore the rest with `..`: `let [a, b, ..] = corners;`

### `E0308` — the array lengths don't match

```text
error[E0308]: mismatched types
 --> array-lengths.rs:3:26
  |
3 |     let same: [i32; 4] = five;
  |               --------   ^^^^ expected an array with a size of 4, found one with a size of 5
  |               |
  |               expected due to this
  |
help: consider specifying the actual array length
  |
3 -     let same: [i32; 4] = five;
3 +     let same: [i32; 5] = five;
  |
```

**What the compiler is objecting to:** `[i32; 4]` and `[i32; 5]` are different types. There's no conversion between them, because no conversion would make sense.

**The fix:** write the right length — or, if you genuinely want a function that accepts any length, what you need isn't an array, it's a **slice**. Slices arrive in [1.3.4](../../03-borrowing-and-references/04-slices/README.md) and exist for exactly this.

### `E0277` — a tuple won't print with `{}`

```text
error[E0277]: `(u32, f64, bool)` doesn't implement `std::fmt::Display`
 --> print-a-tuple.rs:3:20
  |
3 |     println!("{}", sample);
  |               --   ^^^^^^ `(u32, f64, bool)` cannot be formatted with the default formatter
  |
  = help: the trait `std::fmt::Display` is not implemented for `(u32, f64, bool)`
  = note: in format strings you may be able to use `{:?}` (or {:#?} for pretty-print) instead
```

**What the compiler is objecting to:** `{}` means "show this to a user", and Rust doesn't know how a tuple should look to a user — commas? spaces? field names? It has no answer, so it doesn't invent one.

**The fix:** write `{:?}`. And if it's nested, `{:#?}`, which prints it over several tidy lines.

**Why that's the fix:** those are two different outputs, not two skins on one. `{}` is for the *user* and `{:?}` is for *you*. `Display` and `Debug` are separate traits and [Phase 2](../../../phase2-intermediate/03-generics-and-traits/README.md) finishes the story.

### The run-time panic — which is not a compile error

```text
thread 'main' (12124) panicked at examples\04-out-of-bounds.rs:20:29:
index out of bounds: the len is 5 but the index is 5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**What happened:** the program compiled with no error at all. The problem arrived at **run** time, because the index came from somewhere the compiler couldn't work out in advance.

**The fix:** either check the index is valid first, or take `.get()` and handle the `None`.

**Why that's the fix:** unlike overflow, this panic is still there in release, so "it'll be fine in production" doesn't happen. The only real choice is between panicking and writing down what absence means.

---

## Exercises

### Warm up

<details>
<summary>Why aren't <code>(u32, f64)</code> and <code>(f64, u32)</code> the same?</summary>

Because position is part of the type. A tuple isn't "a bag of two numbers"; it's "a `u32` at position zero and an `f64` at position one". Swapping the order makes a different type, and the compiler won't let you pass one for the other.

</details>

<details>
<summary>What type is <code>(7)</code>? What about <code>(7,)</code>?</summary>

`(7)` is an integer — the brackets are just grouping. `(7,)` is a one-element tuple. The trailing comma is the only thing separating them.

</details>

<details>
<summary>A function wants <code>[i32; 5]</code>. You hand it a four-element array. When do you find out?</summary>

At compile time, with `E0308`. The length is part of the type, so this is a type error rather than a run-time check.

</details>

<details>
<summary>What does <code>readings[99]</code> do on a five-element array? And <code>readings.get(99)</code>?</summary>

The first panics — in debug and in release alike. The second gives `None` and the program carries on. Wherever the index comes from outside data, take `.get()`.

</details>

<details>
<summary>In <code>let (_, celsius, _) = sample;</code>, what do those two <code>_</code> do?</summary>

They bind nothing. They only count the positions so the pattern covers the whole shape. `_` isn't a variable and you can't refer to it later.

</details>

### Repair

Change `examples/04-out-of-bounds.rs` so it no longer panics and instead prints something sensible when the index isn't valid — without touching the value of `wanted`.

Then run it in release too. Both builds now give the same output and neither stops. **That's the point of the exercise.**

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p1-01-03-compound-types-and-destructuring
```

One of those tests matters more than the others: `splitting_and_rebuilding_gets_back_where_it_started`. It doesn't check three examples; it checks that your two functions are **inverses of each other**. If the rest pass and that one doesn't, one of the pair is half right.

### Build

Write a `pub fn stats(readings: [i32; 5]) -> (i32, i32, i32)` returning the minimum, maximum and sum, in that order.

Write it without a loop — you haven't met loops yet and you don't need one. (`a.min(b)` and `a.max(b)` work on any integer and can be chained.)

Then write a sentence on why the same function couldn't return `(i32, i32, i32)` for a `Vec`. If you find the answer, you've understood half of [1.6.1](../../06-absence-and-failure/01-option-and-null-safety/README.md) before you get there.

### Challenge (optional)

Run this:

```rust
let grid = [[1, 2, 3], [4, 5, 6]];
println!("{:?}", grid);
println!("{}", grid.len());
println!("{}", grid[0].len());
println!("{}", grid[1][2]);
```

Then answer:

1. What exactly is `grid`'s type? (Write it out in full, with both numbers.)
2. Why are `grid.len()` and `grid[0].len()` two different things?
3. Write a single pattern that gives all six numbers six names in one line.
4. Now try `[[1, 2, 3], [4, 5]]`. Which error do you get, and why is that error exactly the one you wanted?

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| tuple | a fixed count of values, types may differ | returning several values |
| array `[T; N]` | a fixed count of values of one type | buffers, fixed tables |
| unit type `()` | the empty tuple; "no value" | `Result<(), E>` |
| destructuring | taking a value apart by writing its shape | `let`, `match`, `if let` |
| pattern | the shape the compiler matches against | everywhere, from here on |
| `_` in a pattern | "a field is here, I want no name" | skipping fields |
| bounds checking | every index is verified before the read | every indexed access |
| `.get()` | indexing without risking a panic | outside input |

### What you now know

- Tuples group different things and arrays group alike things; both are fixed size.
- Position in a tuple and length in an array are both part of the **type**.
- A tuple is the usual way to return several values from a function.
- Destructuring is writing the value's shape, and `_` counts a position without naming it.
- Every index is checked — sometimes at compile time, otherwise with a run-time panic that's still there in release.
- `.get()` gives you an `Option` instead of a panic.

### What comes back later

- **The `Option` you saw from `.get()`** — [1.6.1 — `Option` and null safety](../../06-absence-and-failure/01-option-and-null-safety/README.md)
- **A list that grows** — [1.1.6 — `Vec` and `String`](../06-vec-and-string-basics/README.md)
- **Slices, for when the length isn't known up front** — [1.3.4 — Slices](../../03-borrowing-and-references/04-slices/README.md)
- **Patterns at full strength** — [1.5.4 — `match` in depth](../../05-your-own-types/04-match-in-depth/README.md)
- **Naming fields instead of counting them** — [1.5.1 — Structs and methods](../../05-your-own-types/01-structs-and-methods/README.md)
- **`Display` versus `Debug`** — [Phase 2 — Generics and traits](../../../phase2-intermediate/03-generics-and-traits/README.md)

### Can you explain?

- What's the difference between a tuple and an array, in one sentence?
- Why can't you put `[i32; 4]` where `[i32; 5]` is wanted?
- What separates `(7)` from `(7,)`?
- In `let (a, _, c) = triple;`, what does the `_` do and what does it not do?
- What happens when you run off the end of an array, and how does that differ from the overflow panic in the last lesson?
- When is a tuple enough, and when is it time to write a struct?

---

## Going further

- [The Rust Book — Data Types](https://doc.rust-lang.org/book/ch03-02-data-types.html#compound-types) — the same ground, officially.
- [The Rust Reference — Patterns](https://doc.rust-lang.org/reference/patterns.html) — the complete list of every pattern form there is. You won't recognise most of it yet; look once so you know how big it gets.
- [`std::primitive::array`](https://doc.rust-lang.org/std/primitive.array.html) — everything an array can do.
