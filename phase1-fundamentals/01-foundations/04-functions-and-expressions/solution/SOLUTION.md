# Solution — 1.1.4 Functions and expressions

```rust
pub fn hypotenuse(a: f64, b: f64) -> f64 {
    (a * a + b * b).sqrt()
}

pub fn box_count(items: u32, per_box: u32) -> u32 {
    items.div_ceil(per_box)
}

pub fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

pub fn total_price(unit_price_rial: u64, quantity: u64, discount_percent: u64) -> u64 {
    let subtotal = unit_price_rial * quantity;
    let discount = subtotal * discount_percent / 100;
    subtotal - discount
}

pub fn diagonal_inches(width_px: f64, height_px: f64, dpi: f64) -> f64 {
    hypotenuse(width_px, height_px) / dpi
}
```

Every one of those bodies ends without a semicolon. That is not a style choice — the last expression *is* the return value, and a semicolon would turn it into a statement and make the function return `()`. If you added `return` and a semicolon, the code is correct and it reads like a translation from another language; Rust programmers will read the tail form faster.

## `hypotenuse` — and the version that is better than yours

```rust
(a * a + b * b).sqrt()
```

That is the expected answer and it is fine. But `f64` already has this:

```rust
a.hypot(b)
```

Same answer, and better in a way you cannot see from the test: `hypot` does not overflow when `a` and `b` are large. `a * a` for an `a` near `1e200` becomes infinity and the answer is lost; `hypot` scales the numbers first and gets it right.

You were not expected to know that. **You were expected to eventually go looking**, and that is the actual habit this exercise is training: when a piece of arithmetic feels standard, the standard library has usually already named it.

## `box_count` — the idiom, and the method that replaced it

The classic way to divide and round up, in any language with integer division:

```rust
(items + per_box - 1) / per_box
```

It works, and for decades it was the answer. Read it once and you can see why: adding one less than the divisor pushes any non-zero remainder over the next boundary.

Rust has a name for it:

```rust
items.div_ceil(per_box)
```

Take the named one. It says what it means, it cannot be mistyped into something that silently almost works, and — the part people miss — **the idiom can overflow.** If `items` is near `u32::MAX`, then `items + per_box - 1` runs past the top and panics in debug, wraps in release, exactly as [1.1.2](../../02-scalar-types-and-overflow/README.md) warned. `div_ceil` has no such problem.

That is the same lesson as `hypot`, twice in one exercise: **the named method is not just tidier, it is usually more correct.**

## `is_leap_year` — a decision with no `if` in it

```rust
(year % 4 == 0 && year % 100 != 0) || year % 400 == 0
```

The point of this one is that you wrote a three-branch rule as a single expression and never reached for a condition.

`==`, `!=`, `&&` and `||` all *produce values*. `year % 4 == 0` is a `bool` in the same way `2 + 2` is an `i32`. Once you see that, a whole class of code you would have written as five lines of `if` collapses into one line that reads like the rule it implements.

The brackets are load-bearing. `&&` binds tighter than `||`, so they are not strictly required — but the rule has a shape ("this, unless that, unless the other") and the brackets show it. Write for the reader.

**`&&` and `||` short-circuit.** For `year = 2023`, `year % 4 == 0` is false, so `year % 100 != 0` is never evaluated at all. It does not matter here. It matters enormously when the right-hand side is a function call that would panic, or a database query — and it is the reason you can write a check and its use in one line.

## `total_price` — two `let`s and then an expression

```rust
let subtotal = unit_price_rial * quantity;
let discount = subtotal * discount_percent / 100;
subtotal - discount
```

The one-liner is possible and worse:

```rust
unit_price_rial * quantity - unit_price_rial * quantity * discount_percent / 100
```

Naming intermediate steps is free. The compiler produces the same machine code either way, and the reader gets the names. That is what `let` is for inside a function body.

**Why `* discount_percent / 100` and not `* (discount_percent / 100)`:** the second is integer division and `33 / 100` is `0`, so every discount would be nothing. Multiply first, divide last. It is the same trap as `celsius * 9 / 5` from [1.1.2](../../02-scalar-types-and-overflow/README.md), and it stays a trap for as long as you write integer arithmetic.

**Why the numbers are `u64`:** money. A price in rial gets large quickly, and `subtotal * discount_percent` is larger still — an intermediate value up to a hundred times the subtotal. That is exactly where a `u32` would overflow. Production code would take this further and use `checked_mul`, so that an absurd quantity becomes an error rather than a panic.

## `diagonal_inches` — the one-line body that matters most

```rust
hypotenuse(width_px, height_px) / dpi
```

Nothing clever, and that is the point. This function knows about screens and inches; it knows nothing about squares and square roots. If `hypotenuse` gets replaced with `a.hypot(b)` tomorrow, this function does not change.

Calling your own functions is not a performance concern. Rust inlines small functions aggressively, so `diagonal_inches` almost certainly compiles to the same instructions as if you had written the arithmetic out. **You do not trade speed for clarity here.** Write the small function.

## What this lesson was really about

- **Almost everything in Rust is an expression**, so almost everything is worth something. Blocks, comparisons, `&&` chains, and — from the next lesson — `if` and `match` too.
- **The semicolon is a real operator.** It turns something worth a value into something worth `()`. Nearly every "expected `u32`, found `()`" you will ever see is one stray semicolon.
- **`return` exists for leaving early**, and that is nearly all it is for. The value at the end of a body needs no keyword.
- **The named method is usually the more correct one.** `hypot`, `div_ceil`, `checked_mul` — each of them is someone else having already thought about the edge case you have not reached yet.
