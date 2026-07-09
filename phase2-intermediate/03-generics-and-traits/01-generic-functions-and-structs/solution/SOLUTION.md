# Solution

```rust
pub fn largest<T: PartialOrd + Copy>(items: &[T]) -> Option<T> {
    let mut max = *items.first()?;
    for &item in items {
        if item > max {
            max = item;
        }
    }
    Some(max)
}
```

`items.first()` returns `Option<&T>` — a reference to the first element, if
any. The `?` operator propagates `None` immediately if the slice is empty
(returning `None` from `largest` right there), otherwise it unwraps to `&T`.
The leading `*` dereferences that to `T`, which is exactly where `Copy`
earns its keep: dereferencing a borrowed value to get an *owned* copy of it
is only allowed when the compiler knows how to duplicate that value cheaply
(a bitwise copy — no heap allocation, no custom cleanup logic to worry
about running twice). Without `Copy`, `*items.first()?` would try to move
`T` out of a slot inside `items`, which Rust forbids — you don't own that
slot, only a shared reference to it, and moving out from behind a shared
reference would leave a hole in `items` that nothing owns. That answers
checkpoint question 1.

The loop itself, `for &item in items`, uses the same `Copy`-powered pattern:
`items` is `&[T]`, so iterating it yields `&T` each time, and the `&item`
pattern immediately copies each reference into an owned `item` so the body
can compare with plain `>` instead of juggling `*a > *b`.

On checkpoint question 2: two. `cargo build` sees `Stack<i32>` and
`Stack<String>` actually used in the test module, so monomorphization
generates two independent copies of every `Stack<T>` method — one with every
`T` replaced by `i32`, one with every `T` replaced by `String` — as if you'd
hand-written `StackI32` and `StackString` yourself. If a third concrete type
were used elsewhere, a third copy would appear. Nothing about this is
visible at the source level; it's purely a compile-time expansion, which is
exactly why there's no runtime type-check cost when you call `.push()` or
`.pop()` — by the time the program runs, `Stack<i32>::push` is just an
ordinary, fully concrete function.
