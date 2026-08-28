# Solution — 2.6.4 `Weak` and reference cycles

```rust
pub fn is_reachable<T>(weak: &Weak<T>) -> bool {
    weak.upgrade().is_some()
}

pub fn describe<T: std::fmt::Debug>(weak: &Weak<T>) -> String {
    match weak.upgrade() {
        Some(value) => format!("alive: {value:?}"),
        None => "gone".to_string(),
    }
}

pub fn parent_name(folder: &Folder) -> Option<String> {
    folder.parent.upgrade().map(|parent| parent.name.clone())
}

pub fn depth(folder: &Rc<Folder>) -> usize {
    let mut steps = 0;
    let mut current = Rc::clone(folder);
    while let Some(parent) = current.parent.upgrade() {
        steps += 1;
        current = parent;
    }
    steps
}
```

All four functions are written with nothing but `.upgrade()` and what you already knew — no `RefCell`, no building a tree. The tree already existed; your job was only to read it.

## `is_reachable` — the definition of `Weak`, almost word for word

```rust
weak.upgrade().is_some()
```

This is close to the literal definition of `Weak`: a `Weak<T>` is "alive" exactly when `.upgrade()` gives back `Some`. No `match` needed — `Option::is_some()` asks precisely that question.

## `describe` — two cases, exactly those two strings

```rust
match weak.upgrade() {
    Some(value) => format!("alive: {value:?}"),
    None => "gone".to_string(),
}
```

Here you actually need the value inside `Some` (to print it), so `match` instead of `is_some()`. The `T: std::fmt::Debug` bound on the signature is there for exactly this reason — without it the compiler has no idea how to write `{value:?}`.

## `parent_name` — `.upgrade()` plus `.map()`

```rust
folder.parent.upgrade().map(|parent| parent.name.clone())
```

`folder.parent.upgrade()` gives an `Option<Rc<Folder>>`. `.map()` — the same combinator from
[1.6.2](../../../../phase1-fundamentals/06-absence-and-failure/02-option-combinators/README.md) —
runs on the value inside when it's `Some` and passes `None` straight through untouched; it's the
same `match` above, written shorter. `parent.name.clone()` returns an owned `String`, not a `&String`
tied to the lifetime of that temporary `Rc`.

## `depth` — climb, one step at a time

```rust
let mut steps = 0;
let mut current = Rc::clone(folder);
while let Some(parent) = current.parent.upgrade() {
    steps += 1;
    current = parent;
}
steps
```

`current` starts out as `folder` itself (a cheap `Rc::clone`, not a deep copy). Each pass through the
loop tries to climb one step: if `.upgrade()` gives something back, `steps` goes up by one and
`current` becomes that parent; otherwise (no parent, or one that's no longer alive) `while let` ends
the loop on its own. For the root — whose `parent` is an empty `Weak::new()` — `.upgrade()` returns
`None` on the very first try, so `steps` stays at its starting `0`.

## What this lesson was actually about

- **`Weak<T>` has exactly one way to reach the value: `.upgrade()`.** And because it can never
  promise the value is still there, that one way returns an `Option<Rc<T>>`, not a bare `Rc<T>`.
- **Holding a `Weak` never changes the strong count** — exactly what let every one of these
  functions skip `RefCell` entirely: none of them wanted ownership of anything, only a look.
- **The sample tree was built with `Rc::new_cyclic`** — the same constructor from `README.md`, built
  for exactly this shape: every child has a working `Weak<Folder>` back to its parent from the moment
  it exists.
- **`.map()` on an `Option<Rc<T>>` behaves exactly like `.map()` on any other `Option`** — nothing
  about `Weak` is special here; once `.upgrade()` has run, what's left is an ordinary `Option`.
