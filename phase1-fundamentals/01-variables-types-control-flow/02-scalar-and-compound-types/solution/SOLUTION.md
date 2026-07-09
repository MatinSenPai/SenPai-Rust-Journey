# Solution

- `a.checked_add(b)` — every integer type ships `checked_add`/`checked_sub`/
  `checked_mul`, each returning `Option<T>`: `None` on overflow instead of
  panicking (debug builds) or silently wrapping (release builds, unless you
  use plain `+`, which panics in debug and wraps in release — deliberately
  inconsistent enough that you should almost always prefer `checked_*`,
  `saturating_*`, or `wrapping_*` when overflow is a real possibility).
- `(point.0 * point.0 + point.1 * point.1).sqrt()` — `.0`/`.1` index into a
  tuple positionally; `f64` ships `.sqrt()` directly (no `math` import
  needed, unlike Python).
- `nums.iter().sum()` — arrays implement `IntoIterator`, and `Iterator::sum`
  works for anything summable. This is a one-line preview of Phase 2's
  iterators module.
- `c.len_utf8()` — `char::len_utf8` returns exactly the byte count that
  encoding this character in a UTF-8 `String` would take. This distinction
  (in-memory `char` size vs. encoded byte length) is exactly why Rust
  `String`s don't support `O(1)` indexing by character position the way
  Python strings do — see `04-strings-and-slices/01-string-vs-str`.
