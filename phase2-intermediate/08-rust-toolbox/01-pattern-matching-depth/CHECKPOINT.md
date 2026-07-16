# Checkpoint

1. Match guards look like they should help exhaustiveness — `n if n >= 0`
   plus `n if n < 0` clearly covers every integer. Why does the compiler
   still reject that match, and what does that tell you about how guards
   participate in exhaustiveness checking?
2. In `describe_status`, the arm `n @ 400..=499` uses an `@` binding. What
   are your two options if `@` didn't exist, and what does each one lose?
3. `Ok(n) | Err(n) => ...` is a legal or-pattern only under a specific
   condition. What is it, and why does the compiler need it? (Think about
   what the arm's body is allowed to do with `n`.)
4. In `method_of`, you matched a `&LogEvent` and the binding `method` came
   out as `&String` even though you never wrote `ref` or `&` in the
   pattern. What is this behavior called, and why does binding by
   reference (rather than by move) have to be the default here?
5. `summarize_samples` compiles with exactly three arms: `[]`, `[only]`,
   and `[first, .., last]`. Explain how the compiler convinces itself
   that's exhaustive. What error would you get if you deleted the `[only]`
   arm, and would an equivalent `if/else if` chain on `.len()` have caught
   the same gap?
