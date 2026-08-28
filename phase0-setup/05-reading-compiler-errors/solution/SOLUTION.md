# Solution — 05 Reading compiler errors

```rust
pub fn total_items(orders: u32, per_order: u32) -> u32 {
    orders * per_order
}

pub fn boxes_needed(items: u32, per_box: u32) -> u32 {
    items.div_ceil(per_box)
}

pub fn title_len(title: &str) -> usize {
    title.len()
}
```

## `total_items`

There's nothing to say about it, and that's the point: a multiplication should be one line. If yours is longer, you probably wrote a `return` or left a stray semicolon — the two things lesson 03 warned you about.

## `boxes_needed` — and why `div_ceil` is the best answer

You probably wrote one of these:

```rust
// the manual version
if items % per_box == 0 {
    items / per_box
} else {
    items / per_box + 1
}

// the classic trick
(items + per_box - 1) / per_box

// the standard-library way
items.div_ceil(per_box)
```

**All three are correct for the test's inputs.** They are not equivalent:

- The manual version is correct and readable, just long. If that's what you wrote, nothing is wrong with it.
- The `(items + per_box - 1)` trick is what you'd see in C or Java, and it **has a trap**: if `items` is near the maximum of `u32`, that addition overflows. In debug builds the program panics; in release builds it quietly gives the wrong number. That's exactly the class of bug [Phase 1 — Scalar types and overflow](../../../phase1-fundamentals/01-foundations/02-scalar-types-and-overflow/README.md) is about.
- `div_ceil` comes from the standard library itself, can't overflow, and its name says what it does.

**The general lesson:** before you hand-write integer arithmetic, check whether `u32` already has a method for it. It usually does, and it usually handles the edge cases better than you would.

## `title_len` — bytes, not characters

`title.len()` gives you the number of **bytes**, not characters. The test shows that deliberately:

```rust
assert_eq!(title_len("Frieren"), 7);   // ASCII: bytes and characters agree
assert_eq!(title_len("سلام"), 8);      // Persian: four characters, eight bytes
```

If you expected `4`, that was a perfectly reasonable expectation — and it's exactly where Rust differs from Python. In Python `len("سلام")` is 4, because Python strings are sequences of characters. In Rust strings are UTF-8 and `len()` counts bytes, because that's the only thing computable in constant time.

If you wanted characters:

```rust
title.chars().count()   // 4 for "سلام"
```

which walks the whole string — linear time, not constant. Rust doesn't hide that cost from you, which is why the two methods have different names.

This has a lesson of its own: [Phase 1 — UTF-8, bytes and characters](../../../phase1-fundamentals/04-text-and-strings/02-utf8-bytes-chars-graphemes/README.md). As someone working with Persian text, it's one of the most important lessons in Phase 1 for you specifically.

## The four broken examples

| File | The mistake | The fix |
|---|---|---|
| `02-unknown-name` | `total_itens` typo | change it to `total_items` |
| `03-wrong-type` | `"7"` instead of `7` | drop the quotes |
| `04-wrong-arity` | one argument instead of two | `total_items(7, 3)` |
| `05-no-such-method` | `lenght` typo | change it to `len` |

In all four cases the compiler had already suggested the fix. The point of the exercise wasn't that they're hard — it was to say what's wrong **before running anything**, from the error text alone.

If you managed that for all four, you have the skill this lesson is actually about.
