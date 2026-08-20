# 02.1 — Closures and the `Fn` traits

## What a closure is

A closure is an anonymous, inline function that can **capture variables
from the scope it's defined in**:

```rust
let threshold = 10;
let is_long = |title: &str| title.len() > threshold;
```

`is_long` didn't need `threshold` passed in as a parameter — it just reaches
out and grabs it from the surrounding scope. If you've written a Python
closure before, this is exactly the same idea:

```python
threshold = 10
is_long = lambda title: len(title) > threshold
```

The difference isn't *whether* closures can capture — both languages allow
it. The difference is that Rust's borrow checker gets involved in *how* the
capture happens (by reference, by mutable reference, or by taking
ownership), and it decides that automatically based on what the closure's
body actually does with the captured variable. That decision is exactly
what the three `Fn` traits describe.

## `Fn`, `FnMut`, `FnOnce` — not something you choose, something Rust infers

Every closure automatically implements one or more of three traits,
depending on how its body uses its captures:

- **`Fn`** — only reads captured variables (borrows them as `&T`). Can be
  called any number of times. `is_long` above is `Fn`: it only reads
  `threshold`.
- **`FnMut`** — mutates a captured variable (borrows it as `&mut T`). Can
  still be called repeatedly, but each call may change what it captured.
  Example: a closure that increments a counter it captured.
- **`FnOnce`** — consumes (moves) a captured variable. Can only be called
  **once**, because after the first call the moved value is gone. Example:
  a closure that returns an owned `String` it captured by moving it out.

They form a hierarchy: every `Fn` closure is also a valid `FnMut` (reading
is a special case of "doesn't need exclusive access"), and every `FnMut`
closure is also a valid `FnOnce` (mutating repeatedly is a special case of
"can be called at least once"). So `FnOnce` is the *weakest* guarantee and
`Fn` the *strongest* — when you write a generic bound, ask for the weakest
one your function actually needs, so callers have maximum flexibility.

You never write `impl Fn for MyClosure` yourself — the compiler looks at
the closure body and infers the tightest trait that applies.

## Closures as parameters and return values

```rust
fn apply<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(x)
}

fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}
```

`make_adder` returns `impl Fn(i32) -> i32` — "some concrete type that
implements `Fn(i32) -> i32`, I'm not telling you which one." Every closure
has its own unique, compiler-generated type, so `impl Trait` in return
position is how you return "a closure" at all without writing that type out
by hand. The `move` keyword forces the closure to take ownership of `n`
(instead of borrowing it) — necessary here because `n` is a local variable
that would otherwise be dropped when `make_adder` returns, leaving the
closure holding a dangling reference.

The catch: `impl Fn(i32) -> i32` promises **one single concrete type** —
you can't return `move |x| x + n` from one `if` branch and a *different*
closure from the `else` branch, even though both satisfy `Fn(i32) -> i32`.
When you need that, erase the concrete type with `Box<dyn Fn(i32) -> i32>`
instead, which stores the closure on the heap behind a trait object. You'll
see `Box<dyn Fn>`/`Box<dyn FnMut>` again in Phase 2 module 05
(smart pointers) — for now just recognize it as "the escape hatch when
`impl Trait` can't work because the concrete type varies."

## Your task

Implement the four functions in `src/lib.rs`.

## Next

`solution/SOLUTION.md` — but only after a real attempt.
