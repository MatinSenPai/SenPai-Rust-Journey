# Solution — 2.2.1 Closures, `Fn`/`FnMut`/`FnOnce`, and `move`

```rust
pub fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(f(x))
}

pub fn make_multiplier(factor: i32) -> impl Fn(i32) -> i32 {
    move |value| value * factor
}

pub fn count_matching<F: Fn(&str) -> bool>(items: &[String], predicate: F) -> usize {
    let mut count = 0;
    for item in items {
        if predicate(item) {
            count += 1;
        }
    }
    count
}

pub fn call_n_times<F: FnMut()>(mut f: F, n: u32) {
    for _ in 0..n {
        f();
    }
}

pub fn run_once<F: FnOnce() -> String>(f: F) -> String {
    f()
}
```

None of these five needed anything beyond what "The concept" already
showed — the only real work was deciding, yourself, which bound to reach
for.

## `apply_twice` — why `Fn` is enough

```rust
pub fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(f(x))
}
```

`f` gets called twice, but it is never mutated and nothing is ever moved
out of it — it is just invoked, `f(...)`. So the strictest, weakest-in-power
trait of the three is already enough: `Fn`. Writing `FnMut` or `FnOnce`
here would still compile and still pass the tests — but it would lie to the
caller: "I might need to mutate your closure" or "I only call it once,"
neither of which is true. Pick the smallest trait that actually does the
job, so the caller keeps maximum flexibility.

## `make_multiplier` — why `move` is mandatory here

```rust
pub fn make_multiplier(factor: i32) -> impl Fn(i32) -> i32 {
    move |value| value * factor
}
```

Without `move`, `|value| value * factor` would try to borrow `factor` from
`make_multiplier`'s own stack frame. That frame is gone the instant
`make_multiplier` returns — the closure would be left holding a reference
to memory that no longer exists, exactly the `E0597` you saw in "Errors you
will meet", just on a parameter instead of a block-local variable. `move`
fixes it: it carries `factor` fully into the closure, and the closure owns
its own copy from then on, entirely independent of `make_multiplier`'s
frame.

## `count_matching` — another `Fn`, this time with a plain loop

```rust
pub fn count_matching<F: Fn(&str) -> bool>(items: &[String], predicate: F) -> usize {
    let mut count = 0;
    for item in items {
        if predicate(item) {
            count += 1;
        }
    }
    count
}
```

`predicate`, like `f` in `apply_twice`, is only ever read — called once per
item, its `true`/`false` answer read, nothing mutated or moved out. `Fn` is
enough again. Notice this solution didn't reach for an iterator adapter
either — just the same plain Phase 1 loop, since adapters
(`.iter().filter(...)`) are the next lesson's subject.

## `call_n_times` — why `FnMut` is genuinely required here

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
`FnMut::call_mut(&mut f, ())` — each call reborrows `f` mutably for just
that one invocation, which is exactly what lets you call it inside a loop
instead of being limited to once — the same limit that separates `FnMut`
from `FnOnce`.

Had this written `F: Fn()` instead, a perfectly reasonable closure like
`|| log.push("tick")` would no longer be accepted — it mutates `log`, so it
is only `FnMut`, not `Fn`.

## `run_once` — why `FnOnce` is enough here (and only here)

```rust
pub fn run_once<F: FnOnce() -> String>(f: F) -> String {
    f()
}
```

`f` is called exactly once — this single line, no loop. Because the
function's whole promise is "I call this once," the weakest of all three
traits is already enough: `FnOnce`. That weakness is exactly what lets
closures like `move || owned` (which moves `owned` out of its own captures
and returns it) through the door at all — a closure that a `Fn` or `FnMut`
bound would have rejected outright.

The hierarchy only runs one way, so any `Fn` closure sails through this
function too — the test `run_once_also_accepts_a_plain_fn_closure` confirms
exactly that: `|| "static".to_string()` captures nothing, so it is `Fn`,
and `Fn` is already a valid `FnOnce`.
