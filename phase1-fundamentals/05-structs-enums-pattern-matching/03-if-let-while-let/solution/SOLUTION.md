# Solution

The whole lesson is really about recognizing when `if let`/`while let` say
what you mean more directly than a full `match` would. `describe_rating`'s
`match` equivalent is only one arm longer, but it forces you to write
`None => "not rated yet".to_string(),` explicitly rather than reading
naturally as "if there's a rating, format it; otherwise, this default."
Both are fine Rust — this is a readability judgment call, not a
correctness one.

`sum_stack` and `drain_positive` both take `Vec<i32>` **by value** (moved
in, per the ownership module), and both mark the parameter `mut` because
`.pop()` requires mutable access to remove elements. After calling
`sum_stack(my_vec)`, `my_vec` is gone — moved into the function, drained by
the `while let` loop, and dropped (now empty) at the end of the function
body. This is different from a `for top in stack` loop: `for` would also
consume the vec (via `IntoIterator` by value, same as earlier lessons), but
it visits elements front-to-back in original order and doesn't require
`mut` at all (no in-place removal happening) — `while let Some(_) =
stack.pop()` specifically visits back-to-front, one real removal per
iteration, and needs `mut` because `.pop()` is a mutating method.
