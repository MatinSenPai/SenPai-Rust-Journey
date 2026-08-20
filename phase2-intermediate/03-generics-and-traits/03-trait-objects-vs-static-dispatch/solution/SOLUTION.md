# Solution

```rust
pub fn make_mixed_collection() -> Vec<Box<dyn Summarize>> {
    vec![
        Box::new(AnimeSeries { title: "Trigun".to_string(), episodes: 26 }),
        Box::new(MangaVolume { title: "Blame!".to_string(), chapters: 10 }),
    ]
}
```

This is the line that only `dyn Trait` can express. `Box::new(AnimeSeries
{ .. })` produces a `Box<AnimeSeries>`, and `Box::new(MangaVolume { .. })`
produces a `Box<MangaVolume>` — two genuinely different types. The `Vec`'s
element type annotation, `Vec<Box<dyn Summarize>>`, is what makes both
lines type-check as elements of the *same* vec: Rust performs an implicit
"unsizing coercion" from `Box<AnimeSeries>` to `Box<dyn Summarize>` (and
likewise for `Box<MangaVolume>`) at the point each element is inserted,
erasing each one down to "some `Box` of something implementing
`Summarize`, plus a vtable telling us which." That erasure is exactly why
this couldn't be written as `Vec<T>` for any single generic `T` — a
generic `T` is chosen once, for the whole `Vec`, and here we need two.

On recall question 3: `Vec<dyn Summarize>` (no `Box`) doesn't compile.
A `Vec`'s backing array stores its elements inline, back-to-back, and needs
to know each element's exact size up front to compute how far apart they
are in memory. `dyn Summarize` on its own has no fixed size — `AnimeSeries`
and `MangaVolume` are different sizes, and "some type implementing
`Summarize`" could be anything. `Box<dyn Summarize>` fixes this: a `Box` is
always the same size (one pointer, to a heap allocation), regardless of
what it's boxing, so `Vec<Box<dyn Summarize>>` has a perfectly ordinary,
fixed-size element type from the `Vec`'s point of view — it just doesn't
know what's on the other end of each pointer. This is the same reason
`dyn Trait` almost always shows up behind a pointer (`Box<dyn T>`, `&dyn T`,
`Rc<dyn T>`) rather than bare.

On recall question 5: `total_summary_length_generic::<AnimeSeries>`
compiles down to a version of the function where `i.summary()` is a direct,
statically-known call to `AnimeSeries::summary` — the compiler can inline
it right into the loop, no indirection at all. `total_summary_length_dyn`
instead calls through a `Box<dyn Summarize>`, so `i.summary()` means: read
the vtable pointer stored alongside the boxed data, look up the `summary`
function pointer in that vtable, then jump to it. That's the "one pointer
hop per call" cost mentioned in the README — small, but real, and it also
blocks the compiler from inlining across the call the way it can with a
monomorphized generic.
