# 03.3 — Trait objects vs. static dispatch

Lesson 02 ended with a question: can `print_all_summaries<T: Summarize>`
take a slice containing *both* `AnimeSeries` and `MangaVolume`? The answer
is no — and understanding exactly *why not*, and what the fix costs you, is
this lesson's whole point.

This crate is standalone (it can't import lesson 02's crate), so it
redefines a minimal version: `pub trait Summarize { fn summary(&self) ->
String; }`.

## Static dispatch: generics

```rust
pub fn total_summary_length_generic<T: Summarize>(items: &[T]) -> usize {
    items.iter().map(|i| i.summary().len()).sum()
}
```

Just like `largest<T>` and `Stack<T>` in lesson 01, this gets
**monomorphized**: call it with `&[AnimeSeries]` and the compiler generates
one version of this function with `T` fixed to `AnimeSeries`; call it
elsewhere with `&[MangaVolume]` and you get a second, separate version. Each
*call site* commits to exactly one concrete type for `T`. That's why a
`&[T]` can never hold a genuine mix — `T` is one type, chosen once, for the
whole slice. This is called **static dispatch**: the compiler knows, at
compile time, exactly which `summary` implementation each call is
targeting, and can often inline the call directly (no function-call
overhead at all).

## Dynamic dispatch: trait objects

```rust
pub fn total_summary_length_dyn(items: &[Box<dyn Summarize>]) -> usize {
    items.iter().map(|i| i.summary().len()).sum()
}
```

`Box<dyn Summarize>` is a **trait object**: a heap-allocated value (`Box`)
of *some* type that implements `Summarize`, with the concrete type erased.
All the compiler retains is "this implements `Summarize`" — not which
struct it actually is. Alongside the boxed data, Rust stores a **vtable**
(virtual method table): a small runtime lookup table of function pointers,
one per trait method, pointing at whichever concrete type's implementation
that particular value actually has. Calling `.summary()` on a `dyn
Summarize` means: at runtime, look up the right function pointer in the
vtable, then jump to it — one extra indirection ("pointer hop") compared to
a direct, statically-known call.

That erasure is exactly what buys you heterogeneity:

```rust
pub fn make_mixed_collection() -> Vec<Box<dyn Summarize>> {
    vec![
        Box::new(AnimeSeries { /* ... */ }),
        Box::new(MangaVolume { /* ... */ }),
    ]
}
```

A `Vec<Box<dyn Summarize>>` can hold a genuine mix of different concrete
struct types in one collection, because every element's *static* type
(as far as the `Vec` is concerned) is the same: `Box<dyn Summarize>`. No
`Vec<T>` with a single, fixed generic `T` could ever do this.

## Which one to reach for

**Default to generics.** They're faster (no vtable indirection, and the
compiler can often inline and optimize across the call), and type errors
show up at compile time on the specific call site, rather than needing to
be tested at runtime. Reach for `dyn Trait` specifically when:

- You need a genuinely **heterogeneous collection** — a `Vec` of different
  concrete types that all share a trait, as above.
- You want to avoid **code-size bloat** from monomorphizing the same
  generic function body over many different concrete types (each one is a
  full separate copy in the compiled binary).

You'll see this exact trade-off again in Phase 3: `axum`'s router and
plugin-style handler registries lean on `dyn Trait` (or trait-object-like
patterns) precisely because a router needs to hold many *different*
handler types in one collection.

## Your task

Define a minimal `Summarize` trait, `AnimeSeries` and `MangaVolume` structs
implementing it, and the three functions above in `src/lib.rs`.

## Next

`solution/SOLUTION.md` — but only after a real attempt.
