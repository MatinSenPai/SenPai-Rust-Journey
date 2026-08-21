# Solution — 1.1.1 Variables, mutability, shadowing

```rust
pub fn total_seconds(hours: u32, minutes: u32, seconds: u32) -> u32 {
    let from_hours = hours * 3600;
    let from_minutes = minutes * 60;
    from_hours + from_minutes + seconds
}

pub fn running_total(a: u32, b: u32, c: u32) -> u32 {
    let mut total = a;
    total += b;
    total += c;
    total
}

pub fn scaled(raw: u32) -> u32 {
    let raw = raw * 2;
    let raw = raw + 10;
    let raw = raw / 2;
    raw
}

pub fn full_orders(stock: u32) -> u32 {
    stock / MAX_PER_ORDER
}

pub const MAX_PER_ORDER: u32 = 50;
```

## `total_seconds` — why named bindings beat one expression

`hours * 3600 + minutes * 60 + seconds` is correct and fits on one line. So why ask for three bindings?

Because a name is the cheapest documentation there is. `from_hours` says what the number *means*; `hours * 3600` makes the reader do the multiplication in their head to find out. On a three-term sum that's a wash. On a real calculation — tax, pagination offsets, retry backoff — it's the difference between code you can review and code you have to decode.

This isn't a Rust rule. It's a habit the exercise is drilling, and Rust makes it free: bindings like these cost nothing at run time.

## `running_total` — the textbook case for `mut`

```rust
let mut total = a;
total += b;
total += c;
```

One logical value, updated over time. That's the entire definition of when `mut` is right.

You could have written `a + b + c`, and in real code you would. The exercise forces the shape so the shape is familiar when you need it — which is the moment you're accumulating inside a loop, in [1.1.5](../../05-control-flow/README.md).

Writing `let mut total = 0; total += a; ...` is equally fine.

## `scaled` — why not just chain it

```rust
let raw = raw * 2;
let raw = raw + 10;
let raw = raw / 2;
raw
```

`(raw * 2 + 10) / 2` is shorter and correct. Again, the exercise wants the shape.

But notice what shadowing bought: **three steps, one name, no throwaway identifiers.** Without it you would have written `doubled`, `plus_ten`, `halved` — three names never used again, each of which the reader must hold in their head.

That's the real case for shadowing, and it gets much stronger when the type changes at each step, as in the `raw → trimmed → parsed` pipeline you write for real in [1.6](../../06-absence-and-failure/README.md).

## `full_orders` — integer division already truncates

```rust
stock / MAX_PER_ORDER
```

If you wrote something with a remainder check, it works but wasn't needed: whole-number division in Rust drops the remainder. `49 / 50` is `0`.

Two things worth banking:

- **`120 / 50` is `2`, not `2.4`.** Both operands are integers, so the result is an integer. Rust never quietly turns your integers into floats.
- **Division by zero panics.** Not here, since `MAX_PER_ORDER` is a constant 50 — but it's exactly the kind of thing [1.1.2](../../01-foundations/02-scalar-types-and-overflow/README.md) covers, along with what happens when arithmetic overflows.

## The `const` at the bottom

```rust
pub const MAX_PER_ORDER: u32 = 50;
```

Writing `stock / 50` would have passed the tests. The constant earns its place for two reasons: the number gets a name that says what it is, and it lives in one place, so changing the policy is a one-line edit rather than a search.

That's the whole argument against magic numbers, and it's the same in any language. Rust just gives you a compile-time constant with no run-time cost to do it with.
