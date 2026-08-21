# 1.1.2 — Scalar types and overflow

## At a glance

After this lesson you can:

- Say what range `u8`, `i32` and `usize` each cover, and pick the right one for a job.
- Predict what happens when an integer runs past its range — and know the answer differs between debug and release.
- Explain why `0.1 + 0.2 == 0.3` is false and what to write instead.

**Time:** ~40 minutes · **Prerequisites:** [1.1.1 — Variables and shadowing](../01-variables-mutability-shadowing/README.md)

---

## Why this matters

In Python you have one integer type and it grows without limit. You have never had to think about its size.

In Rust you have eleven integer types and every one has a ceiling. That's irritating at first, and then you realise why it matters: **those ceilings were always there.** Python was hiding them, and at the moment it actually mattered — a database counter, an ID, a currency amount — the hiding turned into a bug that surfaces far too late.

There's a second trap that exists in every language and that few people take seriously: **floating-point numbers are approximate.** `0.1 + 0.2` is `0.30000000000000004` in Python too. The difference is that Rust makes you decide what to do about it.

---

## The concept

### Integer types: width and sign

Each type's name says two things: whether it's signed, and how many bits.

| | 8-bit | 16-bit | 32-bit | 64-bit | 128-bit | pointer-sized |
|---|---|---|---|---|---|---|
| **unsigned** | `u8` | `u16` | `u32` | `u64` | `u128` | `usize` |
| **signed** | `i8` | `i16` | `i32` | `i64` | `i128` | `isize` |

- **Unsigned (`u`)** holds zero and positives. `u8` is 0 to 255.
- **Signed (`i`)** holds negatives too. `i8` is −128 to 127.

```text
u8                         0 .. 255
i8                      -128 .. 127
u32                        0 .. 4294967295
i32              -2147483648 .. 2147483647
u64                        0 .. 18446744073709551615
usize                      0 .. 18446744073709551615
```

**`usize` and `isize` are the size of the machine's pointers** — 64 bits on a 64-bit system. You see `usize` anywhere sizes or indices are involved: `.len()` returns one, and indexing requires one. The reason is that no collection can be larger than the machine's address space.

### Which do you pick?

| Situation | Pick |
|---|---|
| you don't know, nothing special | `i32` — Rust's default, fast, room for negatives |
| a count, a size, an index | `usize` |
| a database ID | `i64` or `u64` — `i32` runs out sooner than you think |
| a byte | `u8` |
| money | **none of the floats.** `i64` in the smallest unit (rial, cent) |

That last row is the most important and we'll come back to it.

### Overflow — and why there are two answers

Here's something you've never seen in Python:

```rust
let mut count: u8 = 250;
count += 1;   // ... and on, past 256
```

In a **debug build** (which `cargo run` gives you by default) the program panics:

```text
start:      250
plus five:  255

thread 'main' (7560) panicked at examples\04-overflow-panics.rs:17:5:
attempt to add with overflow
```

In a **release build** (`cargo run --release`) it doesn't panic. It wraps:

```text
start:      250
plus five:  255
plus one:   0
```

**One program, two behaviours.** That's deliberate and worth understanding:

- Checking for overflow at run time costs something. In release, where speed matters, Rust removes the check.
- In debug it keeps it, because that's where you find the bug.

The practical consequence: **never rely on the overflow panic as a safety net.** It isn't there in production.

### The four explicit methods

Rust gives you four versions of every arithmetic operation. The name says which behaviour you want:

```rust
let nearly_full: u8 = 250;

nearly_full.checked_add(10)      // None — the answer doesn't fit
nearly_full.checked_add(5)       // Some(255)
nearly_full.saturating_add(10)   // 255 — stops at the ceiling
nearly_full.wrapping_add(10)     // 4 — wraps on purpose
nearly_full.overflowing_add(10)  // (4, true) — the answer, and whether it overflowed
```

```text
checked_add(10):     None
checked_add(5):      Some(255)
saturating_add(10):  255
wrapping_add(10):    4
overflowing_add(10): (4, true)
```

Which, when:

| Method | When |
|---|---|
| `checked_` | overflow is an error you must handle — **the right default in backend code** |
| `saturating_` | stopping at the limit makes sense (a progress bar, a display counter) |
| `wrapping_` | you genuinely want it to wrap (a hash, a ring counter) |
| `overflowing_` | you want both the answer and the flag |

The `None` and `Some(255)` in that output are `Option` — Rust's way of saying "there might not be a value". Its full lesson is [1.6.1](../../06-absence-and-failure/01-option-and-null-safety/README.md). For now: `checked_` gives you nothing back when the answer won't fit.

### Floating point: `f32` and `f64`

Two types. `f64` is the default and almost always what you want.

And now the trap:

```rust
let sum = 0.1 + 0.2;
println!("{sum}");
println!("{}", sum == 0.3);
```

```text
0.30000000000000004
false
```

**This is not a Rust bug.** Python does the same. So does JavaScript. The reason is that `0.1` isn't exactly representable in base two, in the same way `1/3` isn't in base ten.

What you do instead:

```rust
let close_enough = (sum - 0.3_f64).abs() < f64::EPSILON;
```

```text
within epsilon ? true
```

That is: "is the difference smaller than some threshold?" You pick the threshold — `f64::EPSILON` works for values near 1, but for very large or very small numbers you need a proportional one.

**And once more because it matters: don't use floats for money.** An `f64` can't hold "10.10" exactly. Store an integer in the smallest unit and divide only when you display it.

### `bool` and `char`

```text
bool is 1 byte
char is 4 bytes — it holds one Unicode scalar, not one byte
```

- **`bool`** is only `true` or `false`. Unlike Python it is not an integer, and `if 1` does not compile.
- **`char`** is a **Unicode scalar**, not a byte. It takes four bytes and can hold `'س'` or `'字'` or `'🦀'`.

That second one matters to you: `'س'` is one `char`, but it occupies several bytes inside a string. That distinction gets a whole lesson — [1.4.2 — UTF-8, bytes, chars](../../04-text-and-strings/02-utf8-bytes-chars-graphemes/README.md) — and it matters most of all to someone working with Persian text.

### `as` conversions — and their trap

Rust does no automatic conversion between numeric types. You can't hand a `u8` to something wanting a `u32`.

```rust
let small: u8 = 250;
let big = small as u32;      // 250 — safe, it fits
let back = 300_u32 as u8;    // 44  — silently truncated!
```

**`as` never complains.** If the value doesn't fit, it throws away the extra bits. So use `as` only where you know it fits.

The safe version is `try_into`, which tells you when it doesn't — but it hands back a `Result`, which you haven't met. [Phase 2 — `TryFrom`](../../../phase2-intermediate/08-rust-toolbox/04-tryfrom-fallible-conversions/README.md) finishes that. Until then: use `as` sparingly and deliberately.

### The readability separator

A small thing that helps a lot:

```rust
let big = 1_000_000;      // the same as 1000000, just readable
let bytes = 0xFF_u8;      // hexadecimal
let flags = 0b1010_0011;  // binary
```

The `_` has no effect on the value. It's purely for your eyes.

---

## Hands on

```sh
cargo run -p p1-01-02-scalar-types-and-overflow --example 01-widths
cargo run -p p1-01-02-scalar-types-and-overflow --example 02-overflow-guards
cargo run -p p1-01-02-scalar-types-and-overflow --example 03-float-equality
```

Then see the debug/release difference with your own eyes:

```sh
cargo run -p p1-01-02-scalar-types-and-overflow --example 04-overflow-panics --features broken
cargo run --release -p p1-01-02-scalar-types-and-overflow --example 04-overflow-panics --features broken
```

The first panics. The second doesn't, and goes quietly from 255 to 0. **Run both** — seeing it is different from reading about it.

Then try:

1. In `01-widths`, print `i128::MAX` as well. How many digits is it?
2. In `02-overflow-guards`, change `250` to `100` and see which lines change.
3. In `03-float-equality`, change `0.1 + 0.2` to `0.1 + 0.7`. Does `==` work now?

---

## Errors you will meet

### The overflow panic — which is not a compile error

```text
thread 'main' (7560) panicked at examples\04-overflow-panics.rs:17:5:
attempt to add with overflow
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**What happened:** the program compiled without any error at all. The problem arrived at **run** time.

That's different from every error you've seen so far and worth noticing: the compiler can't know what a number will be at run time. Overflow isn't a type error; it's a run-time fact.

**The fix:** if overflow is genuinely possible, choose explicitly: `checked_add` (and handle the "it didn't fit" case), or `saturating_add`, or `wrapping_add`. If it isn't possible, use a wider type.

**Why that's the fix:** the panic doesn't exist in release, so relying on it means production goes quiet and hands you the wrong number. Those four methods behave identically in both builds.

### `E0308` between numeric types

Hand a `u8` to something wanting a `u32`:

```text
error[E0308]: mismatched types
  --> src/main.rs:4:20
   |
 4 |     let total: u32 = small;
   |                ---   ^^^^^ expected `u32`, found `u8`
   |                |
   |                expected due to this
   |
help: you can convert a `u8` to a `u32`
   |
 4 |     let total: u32 = small.into();
   |                           +++++++
```

**What the compiler is objecting to:** `u8` and `u32` are different types, even when the value would fit. Rust performs no automatic conversion.

**The fix:** `.into()` when the conversion is always safe (small to large), or `as` when you're sure yourself.

**Why that's the fix:** the compiler offered `into()` because it *knows* a `u8` always fits in a `u32`. Going the other way (`u32` to `u8`) it wouldn't offer that — because that conversion can lose data, and you have to choose it knowingly.

---

## Exercises

### Warm up

<details>
<summary>A <code>u8</code> holds <code>255</code> and you add one. What happens?</summary>

It depends on the build. In debug it panics with `attempt to add with overflow`. In release it quietly becomes `0`. Which is why you don't rely on the panic as a safety net.

</details>

<details>
<summary>What does <code>0.1 + 0.2 == 0.3</code> give, and why?</summary>

`false`. `0.1` isn't exactly representable in base two, so the sum is `0.30000000000000004`. Not a Rust thing — Python does the same.

</details>

<details>
<summary>You're taking an ID from a database. <code>i32</code> or <code>i64</code>?</summary>

`i64`. `i32` tops out around 2.1 billion, which a busy table reaches sooner than you expect — and migrating once you're there is painful.

</details>

<details>
<summary>How do you store an amount of 19.99?</summary>

Not in an `f64`. As an integer in the smallest unit — `1999` — and divide only for display. Floats can't hold monetary values exactly and the errors accumulate.

</details>

### Repair

Change `examples/04-overflow-panics.rs` so that it doesn't panic and instead stops at the ceiling when it gets there — without changing `count`'s type.

Then make a second version that deliberately wraps. Run both in release too, and see that both behaviours are now the same in either build. **That's the point of this exercise.**

### Implement

Four functions in `src/lib.rs`:

```sh
cargo test -p p1-01-02-scalar-types-and-overflow
```

Read `is_close` carefully: the test checks that argument order doesn't matter. If your version only works one way round, you've missed something.

### Build

Write a `pub fn safe_price_total(unit_rial: i64, quantity: i64) -> i64` that computes a total price in rial and stops at `i64::MAX` rather than overflowing.

Then write down why this function doesn't take an `f64`, even though "price" feels like a decimal. If you can explain that to someone else, you've removed one of the most common backend bugs from your life.

### Challenge (optional)

Run this and explain it:

```rust
println!("{}", 300_u32 as u8);
println!("{}", -1_i32 as u32);
println!("{}", 3.99_f64 as i32);
```

All three compile with no warning at all. For each, say what happened and why `as` stayed silent.

Then see what `u8::try_from(300_u32)` gives you. The difference between those two approaches — "truncate silently" versus "tell me it didn't fit" — is what [Phase 2 — `TryFrom`](../../../phase2-intermediate/08-rust-toolbox/04-tryfrom-fallible-conversions/README.md) finishes.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| unsigned / signed | `u` is non-negative only, `i` allows negatives | picking a type |
| width (8, 32, 64 …) | how many bits, and therefore what range | every numeric annotation |
| `usize` | pointer-sized; the type of lengths and indices | `.len()`, indexing |
| overflow | the result runs past the range | panics in debug, wraps in release |
| `checked_` / `saturating_` / `wrapping_` / `overflowing_` | four explicit answers to overflow | any arithmetic that might overflow |
| `f32` / `f64` | approximate decimals | measurement, statistics — **not money** |
| `as` | a silent conversion that can truncate | only where you know it fits |
| `_` in a literal | readability separator | `1_000_000` |

### What you now know

- Numeric types are defined by sign and width, and `usize` is pointer-sized.
- Overflow panics in debug and wraps in release — so it's not a safety net.
- You have four explicit methods that behave the same in both builds.
- Floats are approximate; you compare with a tolerance, not with `==`.
- Money is stored as an integer.
- `as` truncates silently.

### What comes back later

- **The `Option` you saw in `checked_add`** — [1.6.1 — `Option` and null safety](../../06-absence-and-failure/01-option-and-null-safety/README.md)
- **`char` versus byte, with Persian text** — [1.4.2 — UTF-8, bytes, chars](../../04-text-and-strings/02-utf8-bytes-chars-graphemes/README.md)
- **Safe conversion with `TryFrom`** — [Phase 2 — Fallible conversions](../../../phase2-intermediate/08-rust-toolbox/04-tryfrom-fallible-conversions/README.md)
- **Money, precision, and release builds in production** — [Phase 4 — Performance and profiling](../../../phase4-backend-advanced/08-performance-and-profiling/README.md)

### Can you explain?

- What's the difference between `u32` and `i32`, and when do you take each?
- What is `usize` and why does `.len()` return one?
- What does a `u8` reaching 256 do in debug, and in release?
- Name the four overflow methods and when each is right.
- Why is `0.1 + 0.2 == 0.3` false, and what do you write instead?
- Why don't you keep money in an `f64`?

---

## Going further

- [The Rust Book — Data Types](https://doc.rust-lang.org/book/ch03-02-data-types.html) — the same ground, officially.
- [`std::primitive::u8`](https://doc.rust-lang.org/std/primitive.u8.html) — the full list of those `checked_`/`saturating_`/`wrapping_` methods. Skim it once so you know what's there.
- [What Every Computer Scientist Should Know About Floating-Point Arithmetic](https://docs.oracle.com/cd/E19957-01/806-3568/ncg_goldberg.html) — the classic reference. Heavy going; you don't need all of it, but know it exists.
