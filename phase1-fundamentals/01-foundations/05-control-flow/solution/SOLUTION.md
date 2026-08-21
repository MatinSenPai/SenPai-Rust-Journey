# Solution — 1.1.5 Control flow

```rust
pub fn grade(score: u32) -> char {
    if score >= 90 {
        'A'
    } else if score >= 80 {
        'B'
    } else if score >= 70 {
        'C'
    } else if score >= 60 {
        'D'
    } else {
        'F'
    }
}

pub fn sum_to(n: u32) -> u32 {
    let mut total = 0;
    for value in 1..=n {
        total += value;
    }
    total
}

pub fn count_digits(n: u32) -> u32 {
    let mut remaining = n;
    let mut digits = 1;
    while remaining >= 10 {
        remaining /= 10;
        digits += 1;
    }
    digits
}

pub fn index_of_first_negative(readings: [i32; 6]) -> usize {
    for position in 0..readings.len() {
        if readings[position] < 0 {
            return position;
        }
    }
    readings.len()
}

pub fn collatz_steps(start: u32) -> u32 {
    let mut current = start;
    let mut steps = 0;
    loop {
        if current == 1 {
            break steps;
        }
        current = if current % 2 == 0 {
            current / 2
        } else {
            current * 3 + 1
        };
        steps += 1;
    }
}
```

## `grade` — the whole chain is one expression

Notice what isn't there: no `return`, no semicolons on the branches, no `let result = ...` accumulator. The `if`/`else if`/`else` chain is a single expression, it's the last thing in the body, and so it *is* the return value.

The version you may have written first is fine too:

```rust
if score >= 90 {
    return 'A';
}
if score >= 80 {
    return 'B';
}
// ...
```

It works and it reads acceptably. The chained form is better for one specific reason: **the compiler checks that every path produces a `char`.** In the `return` version, forgetting the last case gives you "expected `char`, found `()`" pointing at the function; in the chained version the missing `else` is flagged as `E0317` right where the mistake is. Structure the code so that the compiler can help you.

**Order matters and is the trap.** `score >= 60` is true for 95 as well as for 65. The chain works because each arm is only reached when every arm above it has already failed. Write the bands in the other order and everything is a `'D'`.

## `sum_to` — `1..=n` and the reason `..=` exists

```rust
for value in 1..=n {
```

`1..n` stops one short: `sum_to(5)` would give 10 rather than 15. `..=` includes the end.

The rule is worth stating once, plainly, because it decides which you reach for: **`a..b` is the natural form for indices** (`0..len` visits every valid position exactly once), and **`a..=b` is the natural form for counting things** ("one to five" means five numbers). If you find yourself writing `0..len - 1` or `1..n + 1`, you have picked the wrong one.

`sum_to(0)` works without a special case, and that's worth a moment: `1..=0` is an empty range and the loop body never runs, so `total` stays 0. Ranges that don't make sense are empty, not an error.

There is of course a closed form:

```rust
n * (n + 1) / 2
```

Same answer with no loop at all. It's the better code and it was not the exercise. Worth knowing that Gauss got there first.

## `count_digits` — why it starts at 1

```rust
let mut digits = 1;
while remaining >= 10 {
```

The obvious version starts at 0 and loops `while remaining > 0`. It's correct for everything except `0` itself, which has one digit and would come out as none. Starting at 1 and asking "can I strip another?" handles zero without a special case.

This is a small instance of something worth naming: **the boundary case usually points at the loop condition, not at a missing `if`.** When a loop is wrong for exactly one input, the fix is often to reshape the condition rather than to bolt a guard on the front.

`while` is right here and `for` is not, because nothing knows how many turns there will be until the work is done. That's the whole rule: `for` when the count is known up front, `while` when the condition decides.

## `index_of_first_negative` — the answer that is deliberately bad

```rust
for position in 0..readings.len() {
    if readings[position] < 0 {
        return position;
    }
}
readings.len()
```

Two things are being taught here at once.

**The good part** is the early `return`. The moment the answer is settled, the function stops. No flag variable, no `break` plus a check afterwards — the work is done, so leave.

**The bad part is the return value**, and you were meant to feel it. `6` means "not found", but `6` is also just a number: nothing stops a caller doing `readings[index_of_first_negative(r)]` and getting a panic. The type says `usize`, and `usize` cannot express "there wasn't one".

This is the C convention, it's the source of an enormous number of real bugs, and it's exactly what [1.6.1 — `Option`](../../../06-absence-and-failure/01-option-and-null-safety/README.md) exists to remove. When you get there, the signature becomes `-> Option<usize>` and the caller *cannot* use the answer without first dealing with the possibility that there isn't one. Remember this function when you arrive.

## `collatz_steps` — `loop`, and `break` with a value

```rust
let mut current = start;
let mut steps = 0;
loop {
    if current == 1 {
        break steps;
    }
    current = if current % 2 == 0 { current / 2 } else { current * 3 + 1 };
    steps += 1;
}
```

Two things in there earn their place.

**`break steps` carries a value out of the loop**, so the whole `loop` is an expression and *is* the function's return value. That's the same idea as the tail expression from [1.1.4](../../04-functions-and-expressions/README.md), applied to a loop.

**`current = if ... { } else { }`** is the `if`-as-expression from this lesson doing real work. The alternative is:

```rust
if current % 2 == 0 {
    current /= 2;
} else {
    current = current * 3 + 1;
}
```

Both are correct. The expression version says "current becomes one of these two things" once, instead of saying "assign to current" twice — and if you rename `current`, there's one place to change rather than two.

**Why `loop` and not `while current != 1`?** The `while` version is genuinely fine here and slightly shorter. `loop` was chosen so that `break` with a value has somewhere to show itself. In real code, prefer `while` when the condition can be stated up front, and keep `loop` for when the exit is in the middle or when you need to return something from it.

## What this lesson was really about

- **`if` is an expression**, so a decision is a value. Once that lands, a lot of code you would have written with a mutable accumulator collapses.
- **Every branch must agree on a type**, and that constraint is a feature: it's how the compiler knows you covered every case.
- **There is no truthiness.** A condition is a `bool`. `if x` where `x` is a number does not compile, and every "if (x = 5)" bug in C's history is unavailable to you.
- **Three loops, three jobs:** `for` when the count is known, `while` when a condition decides, `loop` when you break out from the middle.
- **`return` is for leaving early**, and a loop is where it earns its keep.
