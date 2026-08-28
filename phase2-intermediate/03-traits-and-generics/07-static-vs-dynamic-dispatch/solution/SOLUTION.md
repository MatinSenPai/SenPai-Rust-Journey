# Solution — 2.3.7 Static versus dynamic dispatch, and object safety

```rust
pub fn total_summary_length_generic<T: Summarize>(items: &[T]) -> usize {
    items.iter().map(|item| item.summary().len()).sum()
}

pub fn total_summary_length_dyn(items: &[Box<dyn Summarize>]) -> usize {
    items.iter().map(|item| item.summary().len()).sum()
}

pub fn lineup(series: AnimeSeries, volume: MangaVolume) -> Vec<Box<dyn Summarize>> {
    vec![Box::new(series), Box::new(volume)]
}

pub fn fibonacci() -> impl Iterator<Item = u64> {
    Fibonacci {
        current: 0,
        next: 1,
    }
}
```

## `total_summary_length_generic` and `total_summary_length_dyn` — identical bodies, different signatures

```rust
items.iter().map(|item| item.summary().len()).sum()
```

Both functions have the exact same line as their body. That's not a coincidence — it's the whole point of this lesson. Nothing about *computing* the total changes between static and dynamic dispatch; only the *type* of `items` does. `&[T]` commits every element to one concrete type, chosen once by the caller; `&[Box<dyn Summarize>]` lets every element be a different concrete type, decided element by element, with the cost paid at each `.summary()` call instead of at compile time. If you found yourself writing two different bodies here, that's a sign you reached for something more complicated than either signature actually needed.

## `lineup` — the coercion happens at `Box::new`, not before

```rust
vec![Box::new(series), Box::new(volume)]
```

`Box::new(series)` produces a `Box<AnimeSeries>`; `Box::new(volume)` produces a `Box<MangaVolume>` — two genuinely different types. What makes both lines valid elements of the *same* `vec![...]` is the function's own return type, `Vec<Box<dyn Summarize>>`: Rust performs an implicit unsizing coercion from `Box<AnimeSeries>` to `Box<dyn Summarize>` (and the same for `Box<MangaVolume>`) at the point each element needs to match that type, erasing both down to "some `Box` of something implementing `Summarize`, plus a vtable." Neither struct needed to know about the other, and neither needed any explicit cast — the return type annotation was enough to steer the coercion.

## `fibonacci` — the return type promise, the constructor call

```rust
Fibonacci {
    current: 0,
    next: 1,
}
```

The body is exactly what you'd write if the return type were `-> Fibonacci` instead — a plain struct literal, nothing hidden. The only thing `-> impl Iterator<Item = u64>` changes is what the *caller* is allowed to know: they can `.take()`, `.filter()`, `.collect()` on whatever comes back, because it genuinely is an `Iterator<Item = u64>`, but they can never write the name `Fibonacci` themselves, and they can never rely on it specifically being that struct. This is still one, single, compile-time-known concrete type — static dispatch, exactly like every other function in this file, just with its name kept private.

## What this lesson was really about

- **The dispatch mechanism lives entirely in the signature, never in the body.** `total_summary_length_generic` and `total_summary_length_dyn` prove this by sharing one line of logic across two completely different call-site costs.
- **`Box<dyn Trait>` is what makes a mixed `Vec` possible.** `lineup` could not have been written to return `Vec<T>` for any single `T` — the whole reason `Box<dyn Summarize>` exists here is to give two different structs one shared, sized element type.
- **`impl Trait` in return position costs nothing at run time.** `fibonacci`'s body never changes based on how it's spelled in the signature; only what the compiler lets a caller *say* about the return type changes.
