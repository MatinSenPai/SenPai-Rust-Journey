# Solution

```rust
pub fn make_multiplier(factor: i32) -> impl Fn(i32) -> i32 {
    move |x| x * factor
}
```

The interesting word here is `move`. Without it, `|x| x * factor` would
try to *borrow* `factor` from `make_multiplier`'s stack frame. But that
stack frame is gone the moment `make_multiplier` returns — the closure
would be holding a reference to memory that no longer exists, which Rust's
borrow checker rejects outright (this is exactly the kind of bug Rust
makes impossible at compile time that would be a real, silent
use-after-free in a language like C). `move` instead copies `factor`
(it's an `i32`, a `Copy` type — see Phase 1 module 02.2) *into* the
closure itself, so the closure owns its own private copy that lives as
long as the closure does, independent of `make_multiplier`'s frame.

```rust
pub fn call_n_times<F: FnMut()>(mut f: F, n: u32) {
    for _ in 0..n {
        f();
    }
}
```

Two things worth noticing. First, the parameter itself needs `mut f: F` —
calling an `FnMut` closure requires exclusive (`&mut`) access to it, and
Rust won't let you take `&mut` of a binding that isn't itself declared
`mut`, even though `f`'s *type* already encodes "this needs to mutate
things." Second, `f()` inside the loop is really sugar for
`FnMut::call_mut(&mut f, ())` — every call reborrows `f` mutably for just
that one invocation, which is why you can call it in a loop instead of
being limited to once (that limit is exactly what distinguishes `FnMut`
from `FnOnce`).

On recall question 1: a closure body like `|x| { count += 1; x + 1 }`
mutates a captured variable (`count`), so it implements `FnMut` (and, by
the hierarchy, `FnOnce`) — but **not** `Fn`, since `Fn` requires the
closure only ever borrow its captures immutably. Passing it to
`apply_twice`, which is bounded `F: Fn(i32) -> i32`, would fail to compile
with an error along the lines of "expected a closure that implements the
`Fn` trait, but this closure only implements `FnMut`." The bound you write
on a generic parameter is a *promise* about what the caller is allowed to
pass — `apply_twice` promised "I'll only ever need to read your closure's
captures," so the compiler holds every caller to that promise.
