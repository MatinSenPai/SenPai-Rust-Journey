# Solution — 1.5.2 Tuple structs and the newtype pattern

```rust
impl Rial {
    pub fn new(amount: i64) -> Rial {
        Rial(amount)
    }

    pub fn amount(self) -> i64 {
        self.0
    }
}

impl Percent {
    pub fn new(value: u8) -> Percent {
        if value > 100 {
            Percent(100)
        } else {
            Percent(value)
        }
    }

    pub fn value(self) -> u8 {
        self.0
    }

    pub fn of(self, amount: Rial) -> Rial {
        Rial(amount.0 * self.0 as i64 / 100)
    }
}

pub fn total(amounts: &[Rial]) -> Rial {
    let mut sum = 0;
    for amount in amounts {
        sum += amount.0;
    }
    Rial(sum)
}

pub fn transfer(from: AccountId, to: AccountId, amount: Rial) -> String {
    format!("{} -> {}: {} rial", from.0, to.0, amount.amount())
}
```

Almost every line here is one expression long. That is the point: a newtype is cheap to write, and what you get for it is a compiler that will not let `transfer` be called with the arguments in the wrong order.

## `Rial::new` and `Rial::amount` — the door and the window

```rust
Rial(amount)
```

`Rial` is the type and `Rial(...)` is the function that builds it, so the constructor is a single call. There is no validation because a `Rial` has none to do: a negative amount is a refund, and a zero amount is a zero amount.

That is worth stating explicitly, because it is a design decision and not an oversight. `Percent` validates; `Rial` does not. A newtype does two separable jobs — *naming* and *guarding* — and this one is here only for the naming.

`amount(self)` rather than `amount(&self)`: `Rial` is `Copy`, so `self` copies eight bytes and a reference would be eight bytes to follow. For a wrapper around a scalar, take `self`. For a wrapper around a `String`, take `&self` — the rule is [1.2.3](../../../02-ownership-and-memory/03-clone-and-copy/README.md)'s, unchanged.

The field is private, so `Rial::new` is the only way to build one *from another module*. Inside this file `Rial(sum)` is fine, which is why `total` can write it. Privacy in Rust is per module, not per type.

## `Percent::new` — where the guarantee is made

```rust
if value > 100 {
    Percent(100)
} else {
    Percent(value)
}
```

An `if` used as an expression, from [1.1.4](../../../01-foundations/04-functions-and-expressions/README.md): both arms produce a `Percent` and the `if` is the function's tail.

The important part isn't the clamping, it is what the clamping plus the private field buy you together: **anywhere downstream, a `Percent` is known to be at most 100.** `Percent::of` doesn't check. Nothing checks. The check happened once, at the only door.

That is what "validation boundary" means, and it is the reason `240` never becomes a `Percent`.

It is still the wrong behaviour for a library. Clamping turns bad input into plausible-looking output — 240 silently becomes 100 and the caller is never told their input was nonsense. The honest version returns a `Result`, and you write it in [1.6.3](../../../06-absence-and-failure/03-result-and-question-mark/README.md). Today's version is the best a lesson without `Result` can do, and knowing why it isn't enough is more valuable than the code.

## `Percent::of` — the one with a trap

```rust
Rial(amount.0 * self.0 as i64 / 100)
```

Multiply first, then divide. Writing `amount.0 / 100 * self.0` also compiles, also type-checks, and is wrong:

| amount | percent | multiply first | divide first |
|---|---|---|---|
| 1050 | 10 | **105** | 100 |
| 1050 | 3 | **31** | 30 |
| 99 | 50 | **49** | 0 |

Integer division throws away the remainder, so dividing first throws away everything below 100 rial before the percentage is even applied. This is the same hazard as [1.1.2](../../../01-foundations/02-scalar-types-and-overflow/README.md)'s "money is an integer": integers are exact, but only if you order the operations so the truncation happens last.

`self.0 as i64` is needed because the inner values are a `u8` and an `i64` and Rust does not mix them silently. The cast is safe in the widening direction — every `u8` fits in an `i64`.

And note what the newtype could not save you from. `amount.0 * self.0 as i64` is an ordinary `i64` multiplication, and for a big enough amount it overflows exactly as [1.1.2](../../../01-foundations/02-scalar-types-and-overflow/README.md) described: a panic in debug, a wrapped value in release. A newtype checks *meanings*, not *ranges*. `checked_mul` is the tool for the range half, and putting it here would need a `Result` to report the failure.

## `total` — a plain loop, and why `Rial(sum)` is at the end

```rust
let mut sum = 0;
for amount in amounts {
    sum += amount.0;
}
Rial(sum)
```

`amounts` is a `&[Rial]` — a slice, from [1.3.4](../../../03-borrowing-and-references/04-slices/README.md) — so the loop hands out `&Rial`, and `amount.0` reads through the reference without moving anything.

Sum as a bare `i64`, wrap once at the end. The alternative — keeping a running `Rial` and unwrapping it every turn — writes `.0` once per element instead of once per function, for the same answer. **The wrapper should come off as late as possible and go back on as early as possible.** The narrow band in between is where mistakes can happen, and it should be as narrow as you can make it.

The empty case needs no special handling: the loop doesn't run, `sum` stays `0`, and `Rial(0)` comes out. That is the same guard-free shape as the empty-slice cases in [1.3.4](../../../03-borrowing-and-references/04-slices/README.md).

## `transfer` — the signature this lesson exists for

```rust
format!("{} -> {}: {} rial", from.0, to.0, amount.amount())
```

The body is one `format!` from [1.4.3](../../../04-text-and-strings/03-building-and-transforming-strings/README.md). The signature is the lesson:

```rust
pub fn transfer(from: AccountId, to: AccountId, amount: Rial) -> String
```

Compare it with the version this lesson opened on:

```rust
fn transfer(from: u64, to: u64, rial: u64) -> String
```

Both take three numbers. In the second, six orderings compile and one is correct. In the first, the amount cannot land in an account slot and an account number cannot land in the amount slot, so the six become two.

Two, not one — `AccountId` and `AccountId` are still the same type, so `from` and `to` can still be swapped. That is worth sitting with, because it is the honest limit of the pattern: **a newtype separates different things, not different roles.** If swapping sender and receiver is a risk you need closed, the answer is a struct with named fields, from [1.5.1](../../../05-your-own-types/01-structs-and-methods/README.md):

```rust
pub struct Transfer {
    pub from: AccountId,
    pub to: AccountId,
    pub amount: Rial,
}
```

Now a call site has to write the field names, and there is nothing left to get in the wrong order. That is the general shape of the trade: tuple structs for one value with a meaning, named fields for several values whose roles differ.

## What this lesson was really about

- **A tuple struct is a struct whose fields are numbered instead of named**, and `struct Meters(f64);` is a complete type definition, semicolon included.
- **Two newtypes around the same primitive are unrelated types.** `E0308` saying `expected 'Meters', found 'Feet'` is the whole argument for the pattern.
- **A private field plus a constructor is a value that cannot be built wrong** — checked once, at the door, and trusted everywhere after.
- **The run-time cost is zero.** `size_of::<Rial>()` is `size_of::<i64>()`. The cost is the conversions you write, and `E0369` on `price + fee` is one of them presenting its bill.
- **Every `.0` is a moment with the protection off.** Unwrap late, re-wrap early, and keep the gap small.
