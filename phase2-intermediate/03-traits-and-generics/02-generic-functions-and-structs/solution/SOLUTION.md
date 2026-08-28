# Solution — 2.3.2 Generic functions and structs, bounds, `where`

```rust
pub fn smallest<T: PartialOrd>(list: &[T]) -> &T {
    let mut smallest = &list[0];
    for item in list {
        if item < smallest {
            smallest = item;
        }
    }
    smallest
}

pub struct Pair<T> {
    first: T,
    second: T,
}

impl<T> Pair<T> {
    pub fn new(first: T, second: T) -> Self {
        Pair { first, second }
    }

    pub fn first(&self) -> &T {
        &self.first
    }

    pub fn second(&self) -> &T {
        &self.second
    }

    pub fn larger(&self) -> &T
    where
        T: PartialOrd,
    {
        if self.second > self.first {
            &self.second
        } else {
            &self.first
        }
    }
}

pub fn matches_count<T, F: Fn(&T) -> bool>(items: &[T], predicate: F) -> usize {
    let mut count = 0;
    for item in items {
        if predicate(item) {
            count += 1;
        }
    }
    count
}

pub fn label_and_duplicate<T: std::fmt::Display + Clone>(item: T) -> (String, T) {
    let label = format!("label: {item}");
    (label, item.clone())
}
```

All four signatures are exactly what "The concept" already showed you — this time with the bodies filled in.

## `smallest` — the same shape as `largest`, flipped

```rust
let mut smallest = &list[0];
for item in list {
    if item < smallest {
        smallest = item;
    }
}
smallest
```

The body is `largest` from "The concept" and `examples/01-largest.rs`, word for word, with `<` in place of `>`. The `T: PartialOrd` bound is required for the same reason: without it, the compiler has no idea what `item < smallest` even means, and refuses to compile.

Like `largest`, this panics on the first element access (`&list[0]`) if `list` is empty — exactly what the doc comment said it would do.

## `Pair<T>` — an unbounded struct, plus one method with a bound of its own

The `impl<T> Pair<T>` block carries no bound at all — `new`, `first`, and `second` work for *any* `T`, even one that implements nothing whatsoever. `larger` needs one extra promise, though: it has to compare `self.second` and `self.first` with `>`. Rather than put that promise on the whole `impl` block — which would have forced `new`, `first`, and `second` to demand `T: PartialOrd` too, for no reason — it sits on `larger` alone, with a `where` clause after the signature:

```rust
pub fn larger(&self) -> &T
where
    T: PartialOrd,
{
    if self.second > self.first {
        &self.second
    } else {
        &self.first
    }
}
```

`pair_works_with_non_numeric_types` tests exactly this: it builds a `Pair<String>` and calls `.larger()` — which works because `String` itself implements `PartialOrd`, not because `Pair` demanded it up front.

The tie rule is deliberate too: `if self.second > self.first` returns `second` only when it is *strictly* greater; every other case — a tie, or `first` being bigger — returns `first`. `pair_larger_breaks_a_tie_toward_first` confirms it.

## `matches_count` — the bound is on the closure, not on `T`

```rust
pub fn matches_count<T, F: Fn(&T) -> bool>(items: &[T], predicate: F) -> usize {
    let mut count = 0;
    for item in items {
        if predicate(item) {
            count += 1;
        }
    }
    count
}
```

`T` carries no bound at all this time — the function never compares two `T`s against each other, it only hands each one to `predicate` and counts the answer. The bound that is actually needed sits on the second parameter: `F: Fn(&T) -> bool`. Written inline, not with `where`, because it is only one bound on one parameter — exactly the line "The concept" drew for when `where` is still optional.

## `label_and_duplicate` — two bounds on one parameter

```rust
pub fn label_and_duplicate<T: std::fmt::Display + Clone>(item: T) -> (String, T) {
    let label = format!("label: {item}");
    (label, item.clone())
}
```

`Display` is needed for `format!("label: {item}")`, `Clone` for `item.clone()` — two entirely independent promises, both about the same `T`. Drop either one and you get exactly the `E0599` from "Errors you will meet".

## What this lesson was really about

- **A bound is exactly the promise a function needs to compile — no more, no fewer.** `matches_count` never asked for `T: PartialOrd`, because it never compared two `T`s.
- **A bound on the method itself is more precise than a bound on the whole `impl` block.** `Pair<T>` kept its other three methods available for every `T`, because only `larger` carried a bound.
- **`where` is a notational choice, not a new capability** — `<T: Trait>` and `where T: Trait` tell the compiler exactly the same contract.
- **Two bounds on one parameter join with `+`** — `T: Display + Clone` means both promises are required at once.
- **None of these four functions locked onto one concrete type.** As the tests show, `i32`, `&str`, `String`, and even types you define yourself all pass through the same one definition — each, separately, at compile time.
