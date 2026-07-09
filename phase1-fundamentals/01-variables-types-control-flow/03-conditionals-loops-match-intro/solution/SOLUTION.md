# Solution

`letter_grade` is a single `if`/`else if`/`else` **expression** — no
`return` anywhere, the whole thing evaluates to a `&'static str` because
every branch produces one. `'static` here just means "a reference valid for
the entire program" — string literals like `"A"` are baked into the binary
itself, so a reference to one is always valid; full lifetime reasoning comes
in Phase 2.

`first_multiple_above` uses `loop` specifically because `break candidate;`
lets the loop hand a value back out as the whole loop's result — `while`
can't do that (a `while` loop always evaluates to `()`, Rust's empty/unit
type), so with `while` you'd need a `mut candidate` declared *before* the
loop and just read it after, which is one extra line and one extra mutable
variable hanging around outside the loop's scope for no reason.

`classify`'s `match` is exhaustive by construction: `0`, `1 | 2`, `3..=9`,
and `_` together cover every possible `i32`. Delete the `_` arm and the
compiler refuses to compile — "non-exhaustive patterns" — because unlike an
`if` chain (where a missing final `else` just means "do nothing, which is
always a valid `()`"), a `match` used as an expression must produce a value
for literally every possible input, and the compiler can prove that
statically instead of trusting you to remember every case.
