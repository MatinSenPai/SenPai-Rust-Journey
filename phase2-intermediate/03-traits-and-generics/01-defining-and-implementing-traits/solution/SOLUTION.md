# Solution

```rust
fn summary(&self) -> String {
    format!("{} (no summary available)", self.title())
}
```

This default body is the interesting line in the whole lesson. It's written
once, inside the trait definition, before any concrete type (`AnimeSeries`,
`MangaVolume`, or anything else) exists. It compiles because the trait
signature (`fn title(&self) -> String;`) is a *promise*: whatever `self`
turns out to be at the call site, the compiler already knows — from the
`impl Summarize for X` block that got resolved — that `X` has a `title`
method matching that exact signature. The default method doesn't need to
know `X` concretely to call `self.title()` safely; it only needs to know
`X: Summarize`, which is guaranteed by construction (you can't call
`.summary()` on something that doesn't implement `Summarize` at all). This
answers recall question 1.

On recall question 2: `vol.summary()` runs the *same* default body
defined in the trait — it is not copied or regenerated per-implementor.
`MangaVolume`'s `impl Summarize for MangaVolume` block only contains
`title`; when Rust resolves `vol.summary()`, it finds no override in that
`impl` block and falls back to the trait's default, substituting
`MangaVolume` in for `Self` at compile time. Nothing is duplicated in source
form — you'd only see two separate monomorphized copies at the machine-code
level if `summary` were called from a generic function like
`print_all_summaries::<MangaVolume>`, for the same reason `Stack<i32>` and
`Stack<String>` got separate compiled versions in lesson 01.

On recall question 3: a missing `title` on a hypothetical `LightNovel`
is caught by `cargo build`/`cargo check` — a compile error naming the exact
missing method (something like "not all trait items implemented, missing:
`title`") — before any test or binary ever runs. Contrast with the
Python `ABC` case: Python raises `TypeError: Can't instantiate abstract
class LightNovel with abstract method title` only the moment you try to
construct a `LightNovel()` instance, which could be deep into a running
program (or, with a duck-typed `Protocol` instead of an `ABC`, potentially
never caught at all until the exact call site that invokes the missing
method executes). Rust's version of this bug simply cannot ship.
