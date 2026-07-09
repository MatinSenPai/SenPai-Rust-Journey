# Checkpoint

1. In `count_vowels`, try changing the signature to take `s: String`
   instead of `s: &str` but keep the test the same (which calls
   `count_vowels(text)` and then still uses `text` afterward). What compile
   error appears, and how does it connect back to the move-semantics lesson?
2. Try calling `append_exclamation(&mut s)` twice in a row on the same `s`
   in one test — does that compile? Why is that different from trying to
   hold two `&mut` references to `s` *at the same time*?
3. `swap_and_report` takes two separate `&mut i32` parameters
   simultaneously. Why is that allowed, when holding two `&mut` references
   to the *same* `i32` at once wouldn't be?
4. What's the runtime cost of passing `&s` instead of `s` (by value) into a
   function, for a very large `String`? What's the runtime cost of a
   reference itself (roughly — what do you think a reference actually *is*,
   under the hood)?
