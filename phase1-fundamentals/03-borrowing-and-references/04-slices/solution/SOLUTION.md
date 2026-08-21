# Solution

`&s[..n]` and `&nums[1..nums.len() - 1]` are both **range indexing** —
`..n` means "from the start up to (excluding) `n`", and a plain `..` (seen
in the README) means "the whole thing." Both produce a slice: a fat
pointer (pointer + length) into existing memory, never a copy.

`middle(&[1, 2])` computes the range `1..(2 - 1)` = `1..1`, which is a
valid, simply *empty* range (start equals end) — not out of bounds. Rust
only panics on an actually invalid range (start > end, or end beyond the
collection's length), never merely on an empty one.

`sum_slice(nums: &[i32])` accepting `&v`, `&v[1..3]`, and array literals
all in the same test is the entire point of slices as a parameter type:
`&[i32]` doesn't care whether the data's original owner was a fixed-size
array, a growable `Vec`, or another slice — it only needs "a contiguous run
of `i32`s I can read." This is why `&[T]` (and `&str`, its text-specific
cousin) are the idiomatic choice for read-only function parameters over
`&Vec<T>` — accepting the more general type accepts strictly more callers
for free.
