# 06.2 — Unit, integration, and doc tests

You've written dozens of `#[cfg(test)] mod tests` blocks since Phase 0.
This lesson names the pattern, contrasts it with two others, and is
deliberately structured so you can *feel* the difference, not just read
about it.

## Unit tests: inside the crate, can see private items

Every `#[cfg(test)] mod tests { ... }` block you've written so far is a
**unit test** — compiled as part of the same crate it tests (the
`#[cfg(test)]` attribute just means "only include this when building for
`cargo test`, not a normal build"). Because it's the *same* crate, unit
tests can reach private (non-`pub`) items — exactly like how last lesson's
`pub(crate)` field was visible to code anywhere in the crate. This lesson's
`src/lib.rs` has a genuinely private helper function, and its unit test
calls it directly.

## Integration tests: a separate crate, only the public API

Anything under a top-level `tests/` directory is compiled as its **own
separate crate**, which depends on your library the exact same way an
external user of your crate would — `use your_crate::whatever;`, nothing
more. This is why integration tests can *only* reach `pub` items: they
have no special access, unlike unit tests living inside the crate itself.
This is the right place to test your crate's public *contract* — "does
this behave correctly from the outside" — as opposed to unit tests, which
can dig into internals.

`tests/conversion_test.rs` in this lesson tests the same public function
`src/lib.rs`'s unit tests do, but deliberately **cannot** call the private
helper — try it, and see the exact compiler error yourself (recall questions).

## Doc tests: examples in documentation that must stay correct forever

```rust
/// Converts Celsius to Fahrenheit.
///
/// # Examples
///
/// ```
/// # use p2_06_02_unit_integration_doc_tests::celsius_to_fahrenheit;
/// assert_eq!(celsius_to_fahrenheit(0.0), 32.0);
/// ```
pub fn celsius_to_fahrenheit(c: f64) -> f64 {
    // ...
}
```

Any ` ```rust ` (or bare ` ``` `, Rust is the default) fenced code block
inside a `///` doc comment is compiled and **run** as a test by `cargo
test`. This is a genuinely different guarantee than a comment: a
hand-written example in a comment can silently go stale the moment the
function's behavior changes; a doc test *fails the build* the moment it
does. Lines starting with `# ` inside the block (like the `use` above) are
compiled and run but hidden from the *rendered* documentation — used for
setup noise (imports, boilerplate) that a reader doesn't need to see to
understand the example.

## Your task

Implement both functions in `src/lib.rs`. The doc test above is already
correct and complete — once `celsius_to_fahrenheit` is implemented, it
passes automatically as part of `cargo test`.

## Next

`cargo test -p p2-06-02-unit-integration-doc-tests` (all three kinds run
together, labeled separately in the output — look for "Doc-tests" in
particular). Then `solution/SOLUTION.md` — but only after a real attempt.
