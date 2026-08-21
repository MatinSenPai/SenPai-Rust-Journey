# Solution — 1.1.3 Compound types and destructuring

```rust
pub fn split_duration(total_seconds: u32) -> (u32, u32, u32) {
    let hours = total_seconds / 3_600;
    let minutes = total_seconds % 3_600 / 60;
    let seconds = total_seconds % 60;
    (hours, minutes, seconds)
}

pub fn seconds_from(parts: (u32, u32, u32)) -> u32 {
    let (hours, minutes, seconds) = parts;
    hours * 3_600 + minutes * 60 + seconds
}

pub fn celsius_of(sample: (u32, f64, f64)) -> f64 {
    let (_, celsius, _) = sample;
    celsius
}

pub fn endpoints(readings: [i32; 5]) -> (i32, i32) {
    (readings[0], readings[readings.len() - 1])
}

pub fn midpoint(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    let ((ax, ay), (bx, by)) = (a, b);
    ((ax + bx) / 2.0, (ay + by) / 2.0)
}
```

## `split_duration` — the middle line is the interesting one

```rust
let minutes = total_seconds % 3_600 / 60;
```

`hours` and `seconds` are the easy ones. `minutes` is where people go wrong, and the wrong answer is `total_seconds / 60`, which counts *all* the minutes rather than the ones left after the hours are taken out. For 9,045 seconds that gives 150 instead of 30.

The order matters and there is no cleverness in it: `% 3_600` throws away the whole hours, and only then does `/ 60` count the minutes in what remains. If you find that hard to read, this says the same thing:

```rust
let left_over = total_seconds % 3_600;
let minutes = left_over / 60;
```

Rust reads `%` and `/` left to right at the same precedence, so the one-liner and the two-liner are the same code. Take whichever you can read in six months.

**The round-trip test is the real check.** `seconds_from(split_duration(n)) == n` has to hold for every `n`, and it is a much stronger statement than any three examples. This idea — testing a *property* rather than a list of cases — has a name, property-based testing, and it comes back in [Phase 2 — unit, integration and doc tests](../../../../phase2-intermediate/06-project-organization-and-testing/02-unit-integration-doc-tests/README.md).

## `seconds_from` — why destructure instead of `parts.0`

Both work:

```rust
parts.0 * 3_600 + parts.1 * 60 + parts.2   // works
```

```rust
let (hours, minutes, seconds) = parts;     // better
hours * 3_600 + minutes * 60 + seconds
```

The second one is better for a reason worth naming: **`parts.1` tells you nothing, and `minutes` tells you everything.** In a three-field tuple you can just about hold the positions in your head. In a codebase, six months later, `parts.1` is a small puzzle every time you read it.

That is also the honest argument against tuples in general, and the reason [1.5.1 — structs](../../../05-your-own-types/01-structs-and-methods/README.md) exists. A struct gives the fields names permanently, so you never have to remember that position 1 was the minutes.

## `celsius_of` — `_` is not a variable

```rust
let (_, celsius, _) = sample;
```

`_` is not a name and it does not bind anything. It means "there is a field here, I have counted it, and I want nothing from it".

That distinction matters more than it looks. **The pattern still has to match the whole shape.** You cannot write `let (celsius) = sample;` and hope Rust picks the middle one — it has no way to know which you meant, and the arity check (E0308) stops you. Writing the two `_`s is you saying "three fields, and I know it".

`sample.1` also works here and is shorter. Same trade as above: it is fine in a two-line function and it stops being fine the moment anybody else reads it.

## `endpoints` — `readings.len() - 1` versus `readings[4]`

```rust
(readings[0], readings[readings.len() - 1])
```

`readings[4]` is also correct, and for a `[i32; 5]` it is correct forever, because the length is part of the type — it cannot change without the signature changing.

The `len() - 1` version is still the better habit, because it is the version that survives. The moment this becomes a `Vec` in [1.1.6](../../06-vec-and-string-basics/README.md), or a slice in [1.3.4](../../../03-borrowing-and-references/04-slices/README.md), the length is no longer known in advance and `[4]` becomes a panic waiting for a short input.

One trap, worth seeing now rather than at 2am: **`len() - 1` on an empty collection panics**, because `len()` is a `usize` and `0 - 1` overflows below zero rather than going negative. On a `[i32; 5]` that cannot happen. On a `Vec` it very much can, and the fix is `.last()`, which hands back an `Option`.

## `midpoint` — one pattern for two arguments

```rust
let ((ax, ay), (bx, by)) = (a, b);
```

That line builds a tuple of the two points and takes it apart in the same breath — four names out of two arguments, in one line. The plainer version is the same thing spelled twice:

```rust
let (ax, ay) = a;
let (bx, by) = b;
```

Both are fine. The point of showing the first is that **patterns nest as deep as the value does**, and it is the same machinery either way.

### Why the test values look the way they do

Every number in that test is a half, a quarter, or a whole. That is not an accident and it is a callback to [1.1.2](../../02-scalar-types-and-overflow/README.md): those values are exactly representable in binary, so `assert_eq!` on `f64` is safe here.

Had the test used `(0.1, 0.2)` it might have failed for reasons that have nothing to do with your code. When you write float tests of your own, either pick exact values on purpose, or compare with a tolerance. Never assume the values are innocent.

## What this lesson was really about

Not tuples and arrays. It was two ideas:

- **A pattern is a shape, and Rust checks it.** Writing the shape wrong is a compile error, not a run-time surprise. You will use this constantly — in [`match`](../../../05-your-own-types/04-match-in-depth/README.md), in `if let`, in every function that takes a compound value apart.
- **Positional access buys brevity and pays for it in readability.** Tuples are right for two or three values that travel together briefly. Past that, they want names, and names mean a struct.
