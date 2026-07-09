# Solution

`keep_and_modify` relies entirely on `i32` being `Copy`: `let mut m = n;`
doesn't move `n`, it duplicates it — both `n` and `m` are fully independent
afterward, so returning `(n, m)` needs no cloning, no borrowing, nothing
special.

`duplicate_and_uppercase` and `double_prices` both need an explicit
`.clone()` because `String` and `Vec<f64>` aren't `Copy` — even though
`f64` (what the `Vec` holds) is. `Copy`-ness is a property of a *specific
type*, not something that automatically propagates into any container built
around it: `Vec<T>` owns a heap allocation and a length/capacity, and
duplicating that unconditionally on every assignment would be a hidden,
unbounded-cost operation — exactly what Rust refuses to do implicitly,
regardless of what `T` is.

Without `.clone()`, `double_prices(prices)` would move `prices` into the
function body (e.g. into the `for price in prices` loop, which — same as
the previous lesson's `total_length` — consumes the vec via `IntoIterator`
by value). There'd be nothing left to return as `original`. That's the same
`E0382`-flavored error from the move-semantics lesson, just with a `Vec`
instead of a bare `String`.
