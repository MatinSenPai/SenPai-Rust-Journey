# Checkpoint

1. Uncomment `conflicting_borrows_demo`, run `cargo check -p
   p1-03-02-borrow-checker-rules`. Paste the error code and, in your own
   words, explain what the compiler is protecting against — not just what
   line to delete to fix it.
2. In `first_word_len_then_clear`, what would happen if you tried to keep
   holding onto the `&str` slice from the first word (instead of converting
   it to a `usize` length immediately) and *then* called `s.clear()`?
   You don't have to write it — reason through what error you'd expect.
3. `describe_and_grow` scopes its read-only work inside `{ ... }`. What is
   it about a value's scope ending that makes the compiler consider a
   borrow "over"? (This is related to something called "non-lexical
   lifetimes" if you want to look up the term — but try answering in plain
   language first.)
4. Rust's dangling-reference prevention is a compile-time guarantee. Name
   one category of runtime bug in a language like C or C++ that this
   specific guarantee eliminates *entirely*, not just "usually catches."
