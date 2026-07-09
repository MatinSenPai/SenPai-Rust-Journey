# Checkpoint

1. Try changing `longest`'s signature to `fn longest<'a>(a: &'a str, b: &str) -> &'a str`
   (only `a` and the return type share `'a`; `b` gets its own inferred
   lifetime). Does it compile? What does that tell you about what the
   signature is actually promising the caller?
2. `FirstSentence<'a>` needs an explicit lifetime, but its own `as_str`
   method doesn't need one written on the method itself. Explain why, in
   terms of which elision rule applies where.
3. `first_word` takes one reference and needs no explicit lifetime; `longest`
   takes two and does. Write out, in your own words, exactly which of the
   three elision rules explains each case.
4. What compile error do you get if you try to make `FirstSentence` outlive
   the `&str` it borrows from — e.g. constructing one from a `String` that's
   then dropped while the `FirstSentence` is still around? (Try writing
   this out; you don't need to make it compile.)
