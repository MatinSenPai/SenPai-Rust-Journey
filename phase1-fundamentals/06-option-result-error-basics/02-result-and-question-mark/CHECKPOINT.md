# Checkpoint

1. Rewrite `chained_division`'s body using `match` instead of `?` (still
   two calls to `safe_divide`). How many more lines does it take, and how
   many more `match` arms do you write than actual pieces of logic you
   care about?
2. Try adding `?` inside `fn main() { ... }` (in a scratch file, or mentally
   reason about it) without changing `main`'s signature. What compile error
   would you expect, and what would you need to change about `main`'s
   return type to fix it?
3. `parse_positive` converts a `ParseIntError` into a `String` via
   `.map_err(|e| e.to_string())`. Why is this conversion necessary at all —
   what would go wrong if `parse_positive`'s return type were just
   `Result<u32, String>` but you tried to `?` the raw `.parse::<u32>()`
   call directly without `.map_err`?
4. `Option<T>` and `Result<T, E>` are both enums with two variants you
   pattern-match on. What's the one essential piece of information
   `Result::Err` carries that `Option::None` structurally cannot?
