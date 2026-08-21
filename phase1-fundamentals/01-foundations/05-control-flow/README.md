# 1.1.5 — Control flow

## At a glance

After this lesson you can:

- Write a decision as a *value* rather than as a sequence of statements.
- Say why `if x` doesn't compile when `x` is a number, and why that's good news.
- Choose between `for`, `while` and `loop`, and say why.
- Carry a value out of a loop with `break`, and break out of an outer loop with a label.

**Time:** ~45 minutes · **Prerequisites:** [1.1.4 — Functions and expressions](../04-functions-and-expressions/README.md)

---

## Why this matters

You've written `if` and `for` in every language you know. Two things are different here and both matter.

**First: `if` is an expression.** The last lesson said almost everything in Rust is worth something; here you see it applied to decisions. Python had to invent a second syntax for this:

```python
grade = "A" if score >= 90 else "B"     # special syntax
if score >= 90:                          # and this, which is worth nothing
    grade = "A"
```

Rust has one `if` and it produces a value. Which means the pattern you write constantly in Python and Java — "declare a variable, then reassign it inside the branches" — is unnecessary here, and your variable doesn't have to be `mut`.

**Second: a condition must be a `bool`.** In Python, `if items:` means "if it isn't empty". In C, `if (n)` means "if it isn't zero". In Rust neither compiles.

That's irritating at first. Then you remember one of C's oldest bugs:

```c
if (x = 5) { ... }   // assignment, not comparison — and always true
```

That bug is unavailable to you. `x = 5` is worth `()` and `()` isn't a `bool`, so the compiler stops you. **A whole class of mistake, deleted by one simple rule.**

---

## The concept

### `if`, `else if`, `else`

```rust
if score >= 90 {
    println!("grade:     A");
} else if score >= 70 {
    println!("grade:     B");
} else {
    println!("grade:     C");
}
```

```text
grade:     B
```

Two surface differences from what you're used to:

- **No brackets round the condition.** You may write them, and clippy will tell you not to.
- **Braces are mandatory**, even for one line. That `if (x) foo();` where somebody later adds a second line underneath and everything quietly breaks — impossible here.

### `if` as an expression

Here's where it gets interesting:

```rust
let grade = if score >= 90 {
    'A'
} else if score >= 70 {
    'B'
} else {
    'C'
};
```

```text
grade:     B
```

Notice the semicolon after that final `}`: the whole `if` is on the right of a `let`, so this line is a `let` statement like any other.

Three rules that come together:

**1. Every branch must be the same type.** A binding has exactly one type. `if ... { 'B' } else { "failed" }` doesn't compile, and its error is in the next section.

**2. Used as a value, `else` is compulsory.** Without it there's no value when the condition is false — and Rust says `()` in that case, which won't match the other branch's type. That error is below too.

**3. The condition must be a `bool`.**

```rust
let capped = if score > 100 { 100 } else { score };
let message = if stock == 0 { "sold out" } else { "in stock" };
```

```text
capped:    73
message:   sold out
```

Recognise the pattern this replaces, because it's probably your habit:

```rust
// what you bring from your last language
let mut grade = 'C';
if score >= 90 {
    grade = 'A';
}
```

It works. But you forced `grade` to be `mut`, you invented a placeholder initial value, and if you miss a branch the compiler says nothing because the placeholder covers it. **The expression version has none of those three problems.**

### Three loops

Rust has three looping constructs and each has a job.

#### `for` — when the count is known up front

```rust
for n in 0..5 {
    print!("{n} ");
}
```

```text
0..5:      0 1 2 3 4 
```

`0..5` is a **range**: it starts at zero and stops *before* five. If you want the end included, use `..=`:

```text
0..=5:     0 1 2 3 4 5 
```

| Form | Covers | When |
|---|---|---|
| `a..b` | `a` to `b - 1` | indices — `0..len` visits every position exactly once |
| `a..=b` | `a` to `b` | counting things — "one to five" means five numbers |

If you find yourself writing `0..len - 1` or `1..n + 1`, you've picked the wrong form.

And you can loop over the array itself, with no index at all:

```rust
let readings = [12, 7, 19, 3, 14];
let mut total = 0;
for reading in readings {
    total += reading;
}
```

```text
total:     55
```

**There is no three-part C-style `for` here.** No `for (i = 0; i < n; i++)`. It feels like a loss for about a day, and then you realise those three parts were exactly where off-by-one errors were born. `for x in collection` has no index to get wrong.

#### `while` — when a condition decides

```rust
let mut remaining = 100;
let mut halvings = 0;
while remaining > 1 {
    remaining /= 2;
    halvings += 1;
}
```

```text
halvings:  6
```

Nobody knows in advance that it takes six. That's why it's `while` and not `for`.

#### `loop` — when you break out from the middle

```rust
let mut attempt = 0;
let outcome = loop {
    attempt += 1;
    if attempt * attempt > 50 {
        break attempt;
    }
};
```

```text
outcome:   8
```

`loop` runs forever until something breaks out. The fair question is why it exists when `while true` does. Two reasons:

- **It states intent.** `loop` says "this is deliberately unbounded", not "I had a condition and it simplified".
- **The compiler knows it's unbounded.** So it knows execution doesn't fall out the bottom, and it can reason more strictly about initialisation and return types.

And `break attempt` is the real point: **`break` can carry a value out**, which makes `loop` an expression like everything else in this language. (`while` and `for` can't; only `loop`.)

### `continue` and labels

`continue` skips the rest of this turn:

```rust
for n in 0..10 {
    if n % 2 == 0 {
        continue;
    }
    print!("{n} ");
}
```

```text
odds:      1 3 5 7 9 
```

And when loops nest, `break` leaves the nearest one — which usually isn't what you wanted. A label fixes that:

```rust
'search: for row_index in 0..grid.len() {
    for column_index in 0..grid[row_index].len() {
        if grid[row_index][column_index] > 6 {
            found = (row_index, column_index);
            break 'search;
        }
    }
}
```

```text
found at:  (0, 2)
```

That `'search` is a **loop label**. The leading `'` is the same mark you've seen on `'static` and will see again on lifetimes. If you know C: this is not `goto`. It can only leave enclosing loops; it can't jump to an arbitrary point.

### Early `return` — it finally has a job

The last lesson said `return` is for leaving early and that you'd have to wait. Now:

```rust
fn split_evenly(total: u32, parts: u32) -> u32 {
    if parts == 0 {
        return 0;
    }
    total / parts
}
```

That shape is called a **guard clause**: deal with the impossible case first and leave, so the rest of the function doesn't have to think about it. The alternative is the real work sinking inside an `else`, with every new guard adding another level of indentation.

> Returning `0` for "can't divide" is a poor answer and you can probably already feel it: `0` is also a perfectly valid real answer. [1.6.1](../../06-absence-and-failure/01-option-and-null-safety/README.md) fixes this. For now, just notice that it's uncomfortable.

And its second use, leaving a loop the moment the answer is settled:

```rust
fn is_prime(n: u32) -> bool {
    if n < 2 {
        return false;
    }
    let mut divisor = 2;
    while divisor * divisor <= n {
        if n % divisor == 0 {
            return false;
        }
        divisor += 1;
    }
    true
}
```

```text
prime 97:  true
prime 91:  false
```

### So which, when?

| You want | Take |
|---|---|
| a decision that produces a value | `if`/`else` as an expression |
| every element of a collection | `for x in collection` |
| every number in a range | `for n in a..b` |
| as long as a condition holds | `while` |
| unbounded, with an exit in the middle | `loop` |
| a value out of a loop | `loop` + `break value` |
| out of the outer loop | `break 'label` |

---

## Hands on

```sh
cargo run -p p1-01-05-control-flow --example 01-if-as-an-expression
cargo run -p p1-01-05-control-flow --example 02-loops
cargo run -p p1-01-05-control-flow --example 03-early-return
```

Then the two broken ones:

```sh
cargo run -p p1-01-05-control-flow --example 04-truthiness --features broken
cargo run -p p1-01-05-control-flow --example 05-branch-types --features broken
```

Then try:

1. In `02-loops`, swap `0..5` for `0..=5` and back. Which is right for indexing a five-element array?
2. In `02-loops`, change `break 'search` to a plain `break`. What prints now, and why?
3. In `03-early-return`, delete the `parts == 0` guard and call `split_evenly(21, 0)`. What happens? Is it a compile error or a panic?

---

## Errors you will meet

### `E0308` — the condition isn't a `bool`

```text
error[E0308]: mismatched types
  --> examples\04-truthiness.rs:10:8
   |
10 |     if remaining {
   |        ^^^^^^^^^ expected `bool`, found integer
```

**What the compiler is objecting to:** `remaining` is a number and `if` wants a `bool`. Rust converts nothing to `bool` automatically.

**The fix:** write what you're actually asking: `if remaining > 0` or `if remaining != 0`.

**Why that's the fix:** "truthiness" really means the language guessing on your behalf that "zero means no". For a number that might be obvious; for an empty string, an empty list, or `-1` it isn't obvious at all. Rust declines to guess, and you type one more character that says exactly what you asked. That's what makes `if (x = 5)` impossible.

### `E0308` — the branches disagree

```text
error[E0308]: `if` and `else` have incompatible types
 --> examples\05-branch-types.rs:9:47
  |
9 |     let grade = if score >= 70 { 'B' } else { "failed" };
  |                                  ---          ^^^^^^^^ expected `char`, found `&str`
  |                                  |
  |                                  expected because of this
```

**What the compiler is objecting to:** `grade` has one type. The first branch produces a `char` and the second produces something else.

**The fix:** make both branches the same type.

**Why that's the fix:** look at `expected because of this` — the compiler took the type from the **first** branch and measured the rest against it. Which means if you swapped the branches round you'd get the same error with the types the other way about. The second branch isn't really at fault; the two just don't agree.

### `E0317` — an `if` with no `else`, used as a value

```text
error[E0317]: `if` may be missing an `else` clause
 --> missing-else.rs:3:18
  |
3 |     let capped = if score > 100 { 100 };
  |                  ^^^^^^^^^^^^^^^^^---^^
  |                  |                |
  |                  |                found here
  |                  expected integer, found `()`
  |
  = note: `if` expressions without `else` evaluate to `()`
  = help: consider adding an `else` block that evaluates to the expected type
```

(No example file for this one — it's a `let`, not a whole function.)

**What the compiler is objecting to:** when the condition is false that `if` produces no value, and "no value" in Rust is `()`. So the branches are `i32` and `()`, which don't agree.

**The fix:** add an `else` that produces a value of the same type.

**Why that's the fix:** the trailing note states the rule outright: **an `if` with no `else` is worth `()`.** That's why it's perfectly legal as a statement (that `if`'s job was to do something, not to produce a value) and illegal as a value. One rule, two behaviours.

---

## Exercises

### Warm up

<details>
<summary>Why doesn't <code>if remaining { }</code> compile when <code>remaining</code> is a number?</summary>

Because a condition must be a `bool` and Rust has no automatic conversion to one. "Zero means false" is a guess Rust won't make on your behalf.

</details>

<details>
<summary>How many numbers are in <code>0..5</code>? And <code>0..=5</code>?</summary>

Five (0 to 4) and six (0 to 5). For indexing a five-element array, `0..5` is the right one.

</details>

<details>
<summary>Why is <code>let x = if c { 1 };</code> an error?</summary>

Because when `c` is false that `if` produces nothing, meaning `()`. The branches are `i32` and `()` and they don't agree. Used as a value, `else` is compulsory.

</details>

<details>
<summary>What does <code>loop</code> have that <code>while true</code> doesn't?</summary>

Two things: it states intent, and the compiler knows execution doesn't fall out the bottom. And only `loop` can hand a value out with `break value`.

</details>

<details>
<summary>In nested loops, which one does <code>break</code> leave?</summary>

The nearest one. To leave the outer one, label it and write `break 'label`.

</details>

<details>
<summary>What's a guard clause and what does it improve?</summary>

An early `return` at the top of a function that disposes of the exceptional case. The rest of the function no longer has to think about it, and the real work doesn't sink inside an `else`.

</details>

### Repair

Fix `examples/04-truthiness.rs` so it compiles — without changing `remaining`'s type.

Then fix `examples/05-branch-types.rs` two different ways: once by making both branches match the first, once the other way round. Which is the better code, and why?

### Implement

Five functions in `src/lib.rs`, one for each shape:

```sh
cargo test -p p1-01-05-control-flow
```

`index_of_first_negative` has a deliberately bad signature. Notice what bothers you while writing it — [1.6.1](../../06-absence-and-failure/01-option-and-null-safety/README.md) solves exactly that.

### Build

Write a `pub fn nth_prime(n: u32) -> u32` returning the `n`th prime number, starting from `nth_prime(1) == 2`.

You'll want two loops: one hunting for the next prime, one checking primality. Take `is_prime` from `examples/03-early-return.rs`.

Then write a sentence on why the outer loop is a `loop` and not a `for`.

### Challenge (optional)

**Part one.** Write this with a `for` and a range, then again with a `while`:

```
Print the numbers 1 to 20, skipping multiples of 3.
```

Then say which version is easier to break, and why.

**Part two.** Run this and explain it:

```rust
let result = 'outer: loop {
    let mut n = 0;
    loop {
        n += 1;
        if n > 5 {
            break 'outer n * 100;
        }
    }
};
println!("{result}");
```

That `break 'outer n * 100` does three things at once. Name all three.

**Part three.** Without running it, say what this prints and why:

```rust
for n in (0..10).step_by(3) {
    print!("{n} ");
}
```

Then run it. If you got it wrong, look `step_by` up in the documentation and read it.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `if` as an expression | a decision that produces a value | `let x = if ...` |
| branch type agreement | every branch produces one type | any `if` used as a value |
| no truthiness | the condition must be a `bool` | every condition |
| range `a..b` | `a` up to `b - 1` | indices |
| range `a..=b` | `a` up to `b` | counting |
| `for` | count known up front | collections, ranges |
| `while` | as long as a condition holds | unknown count |
| `loop` | unbounded until `break` | exit from the middle |
| `break value` | carries a value out | `loop` only |
| `continue` | skip the rest of this turn | filtering |
| loop label `'name` | which loop `break` leaves | nested loops |
| guard clause | early `return` for the exceptional case | top of a function |

### What you now know

- `if` is an expression, so a decision is a value and your variable needn't be `mut`.
- Every branch must be the same type, and used as a value the `else` is compulsory.
- The condition must be a `bool`; there is no truthiness.
- `a..b` excludes the end and `a..=b` includes it.
- `for` when the count is known, `while` when a condition decides, `loop` when you exit from the middle.
- `break` can carry a value out, and with a label it can leave an outer loop.
- Early `return` is for guard clauses and for stopping the moment the answer is settled.

### What comes back later

- **`match`, which is a checked multi-way `if`** — [1.5.4 — `match` in depth](../../05-your-own-types/04-match-in-depth/README.md)
- **`if let` and `while let`** — [1.5.5](../../05-your-own-types/05-if-let-while-let-let-else/README.md)
- **The right answer to "not found"** — [1.6.1 — `Option`](../../06-absence-and-failure/01-option-and-null-safety/README.md)
- **Loops that become iterator chains** — [Phase 2 — Iterators and closures](../../../phase2-intermediate/02-iterators-and-closures/README.md)
- **Looping over something you don't own** — [1.3.1 — References](../../03-borrowing-and-references/01-shared-and-mutable-refs/README.md)

### Can you explain?

- Why is `let grade = if ... else ...` better than declaring a `mut` and reassigning it in branches?
- Why must a condition be a `bool`, and which bug does that delete?
- What's the difference between `0..n` and `0..=n`, and when is each right?
- Name the three loops and say when each is right.
- How can `break` return a value, and which loop can do it?
- What's a guard clause and why does it improve the code?

---

## Going further

- [The Rust Book — Control Flow](https://doc.rust-lang.org/book/ch03-05-control-flow.html) — the same ground, officially.
- [`std::ops::Range`](https://doc.rust-lang.org/std/ops/struct.Range.html) — ranges are a type of their own, with methods. Look at `step_by` and `rev`.
- [The Rust Reference — Loop expressions](https://doc.rust-lang.org/reference/expressions/loop-expr.html) — the exact rules for `break` and labels.
