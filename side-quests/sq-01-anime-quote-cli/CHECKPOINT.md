# Checkpoint

1. Run all four CLI commands from the README. Then try `cargo run -p
   sq-01-anime-quote-cli -- anime "naruto"` (lowercase) — does it still
   find the Naruto quote? Why?
2. `find_by_anime` needed an explicit `<'a>` lifetime, but earlier lessons'
   single-reference functions (like `longest_word` in
   `04-strings-and-slices`) didn't. In your own words, why does having
   *two* reference parameters change what the compiler can infer on its
   own?
3. `main.rs` uses `let Some(query) = args.get(1) else { ...; return; };`.
   Rewrite the `"anime"` branch using `if let` instead (nesting the rest of
   that branch's logic inside it) — which reads more clearly to you, and
   why might "bail out early" style be preferred as a branch grows longer?
4. Add three of your own favorite quotes to `all_quotes()`. Do any existing
   tests break? Should they? What does that tell you about which parts of
   this codebase the tests are actually pinned to?
