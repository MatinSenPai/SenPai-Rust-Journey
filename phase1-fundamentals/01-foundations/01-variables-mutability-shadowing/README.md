# 1.1.1 — Variables, mutability, shadowing

## At a glance

After this lesson you can:

- Say why `let x = 5` refuses a second assignment, without saying "because Rust is strict".
- Choose between `mut` and shadowing for a real piece of code, and justify the choice.
- Read `E0384` from its own text and fix it yourself.

**Time:** ~35 minutes · **Prerequisites:** [Phase 0 — Hello, Rust](../../../phase0-setup/03-hello-rust/README.md)

---

## Why this matters

This is the first place Rust takes a Python habit away from you, and without an explanation it's just annoying.

In Python every variable is reassignable. You've never thought about it, because you've never had to. In Rust the default is the opposite: **a value bound once stays bound, unless you explicitly allow it to change.**

It looks like a pointless restriction. It isn't. When you see `let total = ...` and not `let mut`, you know without reading the next thirty lines that `total` is that value for the rest of the block. That knowledge lowers the cost of reading code, and in a large codebase that cost is what decides how fast you can move.

There's also a second tool that beginners confuse with `mut` and then spend months not using: **shadowing**. This lesson makes the difference stick.

---

## The concept

### `let` — bound once

```rust
let orders = 7;
println!("orders: {orders}");
```

```text
orders: 7
```

`orders` is a **binding**, not a "variable" in the Python sense. A name attached to a value. Write `orders = 8` afterwards and the program won't compile.

Not because Rust is being difficult. Because `let` without `mut` means "treat this as settled", and the compiler holds you to it.

### `mut` — when the same value genuinely changes over time

```rust
let mut remaining = 7;
remaining -= 3;
println!("remaining: {remaining}");
```

```text
remaining: 4
```

Reach for `mut` when you have **one logical value being updated over time**: a counter, an accumulator, a stock level going down.

The part people take a while to internalise: **`mut` does not change the type.** A `mut` binding holding an integer holds an integer forever. You can change its value, not what kind of thing it is.

### Shadowing — which isn't mutation at all

```rust
let total = 100;
let total = total * 2;
let total = total - 30;
println!("{total}");
```

```text
170
```

This *looks* like reassignment but isn't. Each `let total = ...` creates an entirely **new** binding that happens to reuse the name, hiding — shadowing — the previous one.

The difference from `mut` is practical, not philosophical:

```rust
let total = 170;
let total = total.to_string();
println!("as text: {total}");
```

```text
as text: 170
```

The type went from a number to text. `mut` could not have done that — and you'll see its error below.

(Just observe that `to_string()` for now; text gets its own lesson: [1.1.6 — `Vec` and `String`](../06-vec-and-string-basics/README.md).)

### So which, when?

| Situation | Tool | Why |
|---|---|---|
| a counter going up | `mut` | one logical value, updated over time |
| stock going down | `mut` | same |
| raw input → trimmed → parsed | shadowing | different values, same name, and the type changes |
| a value transformed in three steps | shadowing | you don't want three throwaway names |

Rule of thumb: **if the old value is of no further use after this line, shadow. If it's the same thing being updated, `mut`.**

### Scope — shadowing inside a block

```rust
let level = 1;
{
    let level = 99;
    println!("inside:  {level}");
}
println!("outside: {level}");
```

```text
inside:  99
outside: 1
```

Braces create a **scope**. A shadow made inside one ends when the brace closes, and the outer binding comes back intact.

Take that as a fact for now. When you reach [1.3 — Borrowing](../../03-borrowing-and-references/README.md), scope suddenly matters a great deal: how long a borrow lasts is decided by these same boundaries.

### `const` — a constant computed before the program runs

```rust
const MAX_PER_ORDER: u32 = 50;
```

Three differences from `let`:

1. **Always immutable.** There is no `mut const`.
2. **The type is mandatory.** It isn't inferred; you write it.
3. **It must be computable at compile time.** You can't put the result of a run-time function in one.

The nearest Python analogy is a module-level `MAX_PER_ORDER = 50` you've promised never to reassign — except Rust holds you to the promise.

Convention is `SCREAMING_SNAKE_CASE`. `cargo clippy` will tell you if you forget.

**There's also `static`**, which looks similar with one important difference: a `static` has a fixed address in memory and lives for the whole program, whereas a `const` is substituted inline wherever you use it. Until Phase 2 you want `const` essentially always. `static` reappears when you reach [state shared between threads](../../../phase2-intermediate/07-concurrency-and-async/01-threads-mutex-arc/README.md).

### Type inference — and when to write it yourself

```rust
let orders = 7;              // the compiler says i32
let orders: u32 = 7;         // you said u32
```

Rust works out the type of most bindings. When you write nothing and it's ambiguous, an integer defaults to `i32`.

Write it yourself when the compiler can't work it out (it will tell you), or when the exact type is part of what the code means. Otherwise let it infer — and if your editor has inlay type hints, you'll see what it inferred every time.

---

## Hands on

```sh
cargo run -p p1-01-01-variables-mutability-shadowing --example 01-immutable
cargo run -p p1-01-01-variables-mutability-shadowing --example 02-shadowing
```

```text
orders: 7
remaining: 7
after shipping three: 4
max per order: 50
```

```text
start:      100
doubled:    200
less 30:    170
as text:    170
inside:     99
outside:    1
```

Then try these:

1. In `01-immutable`, remove `mut` from `remaining`. What error do you get?
2. In `02-shadowing`, change `let total = total * 2;` to `total = total * 2;` (drop the `let`). What changes?
3. In `02-shadowing`, add another `println!` after the inner block that prints `level`. Predict the output, then run it.

---

## Errors you will meet

### `E0384` — assigning twice to an immutable binding

```sh
cargo run -p p1-01-01-variables-mutability-shadowing --example 03-reassign --features broken
```

```text
error[E0384]: cannot assign twice to immutable variable `orders`
 --> phase1-fundamentals\01-foundations\01-variables-mutability-shadowing\examples\03-reassign.rs:7:5
  |
6 |     let orders = 7;
  |         ------ first assignment to `orders`
7 |     orders = 8;
  |     ^^^^^^^^^^ cannot assign twice to immutable variable
  |
help: consider making this binding mutable
  |
6 |     let mut orders = 7;
  |         +++
```

**What the compiler is objecting to:** you reassigned an immutable binding.

Note the two labels — the thing you practised in [Phase 0 — Reading compiler errors](../../../phase0-setup/05-reading-compiler-errors/README.md). The `----` on line 6 says *first assignment*, and the `^^^^` on line 7 says the problem is here. The compiler kept track.

**The fix:** either add `mut` (if this really is the same value being updated), or add `let` and shadow (if it's a new value).

**Why that's the fix:** the `help` only offers one of the two, because the compiler can't know what you meant. Here `orders = 8` probably means "the count changed" — so `mut` is right. If you'd meant "now I have the *filtered* orders", `let orders = ...` would be.

### `E0308` — `mut` doesn't change the type

```sh
cargo run -p p1-01-01-variables-mutability-shadowing --example 04-mut-keeps-its-type --features broken
```

```text
error[E0308]: mismatched types
  --> phase1-fundamentals\01-foundations\01-variables-mutability-shadowing\examples\04-mut-keeps-its-type.rs:10:13
   |
 9 |     let mut total = 100;
   |                     --- expected due to this value
10 |     total = total.to_string();
   |             ^^^^^^^^^^^^^^^^^ expected integer, found `String`
   |
help: try removing the method call
   |
10 -     total = total.to_string();
10 +     total = total;
   |
```

**What the compiler is objecting to:** `total` became an integer when it was bound to `100`. `mut` lets its value change, not its type. `String` is a different type.

The `--- expected due to this value` label on line 9 says exactly where the type came from: that `100`.

**The fix:** shadow, don't assign:

```rust
let total = 100;
let total = total.to_string();
```

**Why that's the fix:** the compiler's suggestion (`total = total`) is useless — it doesn't know what you wanted, only how to silence the error. That's a good reminder: **read suggestions, don't accept them blindly.** The right answer is the one the compiler didn't offer.

And this is precisely why shadowing is a separate tool: a fresh binding gets a fresh type.

---

## Exercises

### Warm up

<details>
<summary>Does this compile?<br><code>let x = 5; let x = x + 1; println!("{x}");</code></summary>

Yes, and it prints `6`. That's shadowing, not reassignment — each `let` makes a new binding.

</details>

<details>
<summary>And this?<br><code>let mut x = 5; x = "five";</code></summary>

No. `E0308`. `mut` makes the value changeable, not the type. `x` became an integer when bound to `5` and stays one.

</details>

<details>
<summary>What does this print?<br><code>let n = 1; { let n = 2; } println!("{n}");</code></summary>

`1`. The shadow inside the block ends when the brace closes and the outer binding comes back.

</details>

<details>
<summary>Why must <code>const</code> have its type written but <code>let</code> needn't?</summary>

Because a `const` is substituted at compile time wherever it's used, with no context to infer from. A `let` has one specific value in one specific place that the compiler can work backwards from.

</details>

### Repair

Fix the two broken examples:

```sh
cargo run -p p1-01-01-variables-mutability-shadowing --example 03-reassign --features broken
cargo run -p p1-01-01-variables-mutability-shadowing --example 04-mut-keeps-its-type --features broken
```

For each, **before you type anything**, say which tool is right — `mut` or shadowing — and why. For `03` both work; say which you picked and what your reason was.

### Implement

Four functions in `src/lib.rs`:

```sh
cargo test -p p1-01-01-variables-mutability-shadowing
```

Each deliberately drills one tool: `total_seconds` named bindings, `running_total` a single `mut`, `scaled` shadowing, `full_orders` a `const`. Some could be written other ways — but this time write what the doc comment asks for. It's practice with the tools, not a race to the answer.

### Build

Write a `pub fn shipping_cost(weight_grams: u32) -> u32` that:

- has a base cost of 5000 (make it a `const`),
- adds 200 for every 100 grams,
- returns the result.

Write it once with a `mut` and once with shadowing. Keep both and put them side by side: which do you prefer, and why? There's no right answer; having an opinion is the point.

### Challenge (optional)

Try this:

```rust
let spaces = "   ";
let spaces = spaces.len();
```

It compiles, and `spaces` is now a number. Now try it with `mut` and read what the error says.

Then go one step further: why do you think Rust's designers *allowed* shadowing, when in many languages the same thing is a linter warning? Hint: think about the `raw → trimmed → parsed` pattern, and how many names you'd have had to invent without it.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| binding | attaching a name to a value with `let` | everywhere |
| `mut` | permission to update the same value | counters, accumulators, state |
| shadowing | a new binding with the same name, possibly a new type | chains of transformation |
| scope | the boundary a pair of braces creates | shadows, and later borrows |
| `const` | a compile-time constant with a mandatory type | magic numbers, configuration values |
| `static` | a value with a fixed address, living for the whole program | Phase 2, shared state |
| type inference | the compiler working out the type for you | almost every `let` |

### What you now know

- `let` binds once; `mut` grants permission to update.
- `mut` doesn't change the type; shadowing does.
- Shadowing is a fresh binding, not a reassignment.
- Braces create scopes, and shadows end with them.
- `const` needs its type written and is computed at compile time.
- You can recognise `E0384` and `E0308` on this material.

### What comes back later

- **Numeric types, and what `i32` actually means** — [1.1.2 — Scalar types and overflow](../02-scalar-types-and-overflow/README.md)
- **`String` and the `to_string()` you just saw** — [1.1.6 — `Vec` and `String`](../06-vec-and-string-basics/README.md)
- **Scopes, when they start to matter** — [1.3.3 — Borrow scopes and NLL](../../03-borrowing-and-references/03-borrow-scopes-and-nll/README.md)
- **`static` and state shared between threads** — [Phase 2 — Concurrency](../../../phase2-intermediate/07-concurrency-and-async/01-threads-mutex-arc/README.md)

### Can you explain?

- Why is `let` without `mut` the default, and what does it make easier for you?
- What can shadowing do that `mut` can't, and vice versa?
- Give a real example where shadowing is the right choice.
- `const` differs from `let` in three ways. Which?
- In `E0384`, what does the `----` label point at, and why is that useful?

---

## Going further

- [The Rust Book — Variables and Mutability](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html) — the same ground in the official book.
- [`rustc --explain E0384`](https://doc.rust-lang.org/error_codes/E0384.html) — the page that's also on your own machine.
- [Rust by Example — Variable Bindings](https://doc.rust-lang.org/rust-by-example/variable_bindings.html) — short runnable examples, including scope and shadowing.
