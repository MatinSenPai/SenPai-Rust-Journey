# 03.1 — Generic functions and structs

## The problem generics solve

Say you want a function that finds the largest number in a slice of `i32`,
and later you also want one for `f64`, and later still one for `String`. In
Python you'd write **one** function and just call it on whatever — Python
doesn't check argument types until the code actually runs, so a single
`def largest(items): ...` using `>` internally works on ints, floats,
strings, anything that supports `>`, without you doing anything special.

Rust checks types at compile time, so a plain, non-generic Rust function
signature locks in one concrete type: `fn largest(items: &[i32]) -> i32` only
ever works on `i32`. Writing a near-identical copy for `f64` and `String`
would work, but it's exactly the kind of repetition programming languages
exist to eliminate. **Generics** are Rust's answer: write the function once,
parameterized over a placeholder type, and let the compiler generate the
concrete versions for you.

```rust
fn largest<T: PartialOrd + Copy>(items: &[T]) -> Option<T> {
    let mut max = *items.first()?;
    for &item in items {
        if item > max {
            max = item;
        }
    }
    Some(max)
}
```

`<T: PartialOrd + Copy>` reads as "for some type `T`, as long as `T`
implements `PartialOrd` (supports `<`, `>`, etc.) and `Copy` (can be
duplicated with a bitwise copy instead of a move)." `T` itself is just a
placeholder name — by convention a single capital letter, standing in for
"whatever concrete type the caller uses."

## Why the trait bound is required

Delete `PartialOrd` from the bound above and try to compile: Rust rejects
`item > max` with an error like "binary operation `>` cannot be applied to
type `T`." This surprises people coming from Python, where you just write
`a > b` and find out *at runtime* whether the objects involved support it
(a `TypeError` if not, possibly deep inside a call stack, possibly only on
the one input that hits that code path in production).

Rust instead asks: for an *arbitrary, unknown* `T`, what can you possibly
assume about it? Nothing — unless you say so. `T: PartialOrd` is you telling
the compiler "any `T` this function gets called with is guaranteed to
support ordering comparisons," which lets the compiler both allow `>` inside
the function body *and* reject, at the call site, any attempt to call
`largest` with a type that doesn't support it. The bug moves from "something
a user might trigger in production" to "something that fails `cargo build`
on your machine, with a message that tells you exactly what's missing."

## Generic structs

Structs take type parameters too:

```rust
pub struct Stack<T> {
    items: Vec<T>,
}
```

`Stack<i32>` and `Stack<String>` are both valid uses of the *same* struct
definition. Inside `impl<T> Stack<T> { ... }`, `T` again stands for whatever
concrete type a particular `Stack` was built with.

## Monomorphization

The glossary already defines this (see "Generic"), so here's the concrete
version: when you write `Stack::<i32>::new()` and `Stack::<String>::new()`
in the same program, the compiler generates two entirely separate compiled
types — one with all `T`s replaced by `i32`, one with all `T`s replaced by
`String` — as if you'd hand-written both. There is no shared "generic"
machine code at runtime, no tag checked on every method call to decide
"which version am I." This is *unlike* Python, where one function body
really does exist once at runtime and re-checks, on every single call, what
type it was actually handed. Rust's generics cost you compile time (more
code gets generated, so builds are a bit slower) in exchange for zero
runtime overhead and errors caught before the program ever runs.

## Your task

Implement `largest` and the generic `Stack<T>` in `src/lib.rs`.

## Next

`solution/SOLUTION.md` — but only after a real attempt.
