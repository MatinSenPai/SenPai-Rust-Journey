# Checkpoint

1. `word_frequency` uses `*counts.entry(word).or_insert(0) += 1`. Why is
   there a `*` in front of `counts.entry(...)`? What type does
   `.or_insert(0)` actually return, and why does incrementing it require
   dereferencing first?
2. Rewrite the body of `word_frequency` *without* the entry API — using an
   `if counts.contains_key(&word) { ... } else { ... }` check instead. How
   many `HashMap` lookups does your version do per word, versus the entry
   API's version?
3. If you ran `word_frequency("a b c").iter().collect::<Vec<_>>()` on two
   different runs of your program, is it guaranteed to print the three
   pairs in the same order both times? What language feature/property is
   responsible for your answer?
4. `top_n` sorts by count descending, then alphabetically ascending for
   ties. What would break — concretely, in a test — if you only sorted by
   count and left ties in whatever order `.iter()` happened to produce?
5. `dedupe_preserve_order` takes `Vec<i32>` (not `&Vec<i32>`) and returns a
   new `Vec<i32>`. Could you have implemented this by mutating the input
   `Vec` in place instead (like `.sort()` does)? Why is "first-seen order"
   awkward to preserve with an in-place approach?
