# Checkpoint

1. Rewrite `describe_rating` using a full `match` instead of `if let`/`else`
   — how many lines longer is it, and does it feel like it's pulling its
   weight here, or does `if let`/`else` communicate the intent more
   directly?
2. `drain_positive` pops from the *end* of the vec, so the result comes out
   reversed relative to `nums`'s original order. If you needed the result in
   the original order instead, what would you change (you don't need to
   write it — name the approach)?
3. `sum_stack` takes `mut stack: Vec<i32>` by value (not `&mut Vec<i32>`).
   Given the previous ownership lessons, what happens to the caller's
   original vec after calling `sum_stack(my_vec)` — is it still usable, and
   does it still have its elements?
4. What does `while let Some(top) = stack.pop()` do differently from a
   `for top in stack` loop, given that both eventually visit "every element"?
   (Hint: think about order, and about what `stack` looks like *during* the
   loop, not just after.)
