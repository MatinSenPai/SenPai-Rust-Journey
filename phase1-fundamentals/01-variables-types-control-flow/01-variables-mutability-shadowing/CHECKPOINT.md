# Checkpoint

1. What compile error do you get if you remove `mut` from `increment_n_times`'s
   counter but still try to reassign it? Paste the key line of the error.
2. Shadowing lets you change a variable's type; `mut` doesn't. Try changing
   `parse_and_double` to use `mut input` instead of shadowing, keeping the
   same logic — what compile error appears, and why?
3. In your own words: when would you reach for `mut`, and when for
   shadowing, in real code?
4. What's different about `const` compared to a `let` binding at the top of
   `main`, besides capitalization convention?
