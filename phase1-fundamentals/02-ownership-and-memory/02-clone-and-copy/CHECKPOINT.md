# Checkpoint

1. Remove `.clone()` from `double_prices` (call the doubling logic directly
   on `prices` and still try to return the original `prices` too). What
   compile error do you get, and how does it relate to move semantics from
   the previous lesson?
2. `f64` is `Copy`, but `Vec<f64>` is not. In your own words, why doesn't
   "the elements are `Copy`" make the container `Copy` too?
3. `.clone()` on a `Vec` with a million elements has a real runtime cost.
   Can you think of a scenario in a real backend service (Phase 3-4) where
   reaching for `.clone()` too eagerly, instead of a borrow, could become a
   genuine performance problem?
4. Is every `Copy` type also `Clone`? Is every `Clone` type also `Copy`?
