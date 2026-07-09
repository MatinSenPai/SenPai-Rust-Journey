# Solution — Tooling

Each fix, and the reasoning clippy was pointing at:

- **`is_empty_name(name: &String)` → `&str`**: a function that only reads a
  string never needs to demand a `String` specifically — `&str` accepts
  both a borrowed `&String` (Rust auto-derefs `&String` to `&str` at the
  call site) *and* a string literal, so `&str` is strictly more flexible for
  a parameter that doesn't need ownership. `String` vs `&str` gets a full
  lesson in Phase 1 — this is a preview.
- **`name.len() == 0` → `name.is_empty()`**: identical result, but
  `is_empty()` doesn't require the reader to know that "length zero" means
  "empty" — it says what you mean directly, and for some collections
  `is_empty()` can be faster than computing a full `len()`.
- **`return x * 2;` → `x * 2`**: in Rust, the last expression in a block
  (no semicolon) *is* the return value — this is true of `fn` bodies, `if`
  blocks, `match` arms, everywhere. `return` is reserved for early exits out
  of the middle of a function. Using it for the final line just adds noise.
- **`if flag == true { true } else { false }` → `flag`**: both
  `needless_bool` and `bool_comparison` are pointing at the same root issue
  from two angles — comparing a `bool` to `true`/`false` explicitly is
  always redundant (`flag == true` *is* `flag`; `flag == false` is `!flag`),
  and branching just to return a literal `true`/`false` is always redundant
  too (the condition already *is* the answer).
- **`char_count(&s.to_string())` → `char_count(s)`**: `s` is already
  `&str`. Allocating a new owned `String` with `.to_string()` just to
  immediately re-borrow it as `&str` for the call does real, unnecessary
  work at runtime — a heap allocation and a copy of every byte, for nothing.
- **`for i in 0..nums.len() { total += nums[i]; }` → `for &n in nums`**:
  the classic "manual indexing" habit from other languages. Rust slices are
  directly iterable; `for &n in nums` gets you each value with no bounds
  checks to reason about and no off-by-one risk. (`nums[i]` is also a
  runtime bounds-checked access — not unsafe, just needless overhead and
  noise here.)
- **`.map(|x| double_it(x))` → `.map(double_it)`**: a closure that does
  nothing but call another function with the same argument, unchanged, is
  just a longer way to spell the function itself. Pass the function
  directly.
- **`match opt { Some(x) => x, None => 0 }` → `opt.unwrap_or_default()`**:
  `Option<T>` ships this exact "give me the value or a default" operation
  as a method, for any `T` that implements `Default` (for `i32`, the
  default is `0`). Reaching for the standard-library method instead of
  hand-rolling the same `match` is both shorter and instantly recognizable
  to any other Rust reader.
