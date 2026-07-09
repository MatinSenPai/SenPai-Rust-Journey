# Checkpoint

1. What would `first_n_bytes("🦀ust", 1)` do at runtime? (Reason about it —
   you don't have to run it, though you're welcome to try
   `cargo run`-style experimentation in a scratch `fn main` if you want to
   see the panic message yourself.)
2. `middle(&[1, 2])` returns an empty slice rather than panicking. Walk
   through the range `1..nums.len() - 1` for a 2-element array and explain
   why that's a valid (if empty) range rather than an invalid one.
3. `sum_slice` was called with `&v`, `&v[1..3]`, and array literals in the
   tests, all compiling against the same `&[i32]` parameter. What does that
   tell you about why "accept a slice" is a more flexible function
   signature than "accept a `Vec`" or "accept an array of a specific size"?
4. A slice is often described as "a pointer + a length." What do you think
   happens in memory when you write `&nums[1..4]` — is any data copied?
