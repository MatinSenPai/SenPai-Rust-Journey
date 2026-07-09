# Checkpoint

1. `greet` takes `&str`, and the test calls it with both a literal
   (`"Matin"`) and `&owned` where `owned: String`. What's actually happening
   when a `&String` is passed somewhere expecting `&str`? (Look up the term
   "deref coercion" if you want the precise name for it.)
2. `longest_word` returns `&str`, borrowed from its `text` parameter. Try
   calling it, storing the result in a variable, and then dropping/going out
   of scope of the original `text` `String` before you use the result — what
   compile error would you expect, and why? (You don't need to write this
   out — reason through it.)
3. Why can't you write `let c = some_string[3];` to get the 4th character of
   a `String`? What would you write instead to get, say, the first 3 bytes,
   versus the first 3 characters?
4. When would you choose to return `&str` from a function versus `String`?
   Give one example of each from what you've built so far in this repo.
