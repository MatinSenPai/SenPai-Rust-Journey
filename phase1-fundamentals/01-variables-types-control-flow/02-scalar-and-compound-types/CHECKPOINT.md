# Checkpoint

1. Why does `checked_add` return `Option<u8>` instead of just `u8`? What
   would plain `a + b` do instead if `a = 255, b = 1`?
2. `[i32; 3]` and `[i32; 4]` are different *types*, not just different
   values of the same type — what real consequence does that have for a
   function signature like `fn sum_three(nums: [i32; 3]) -> i32`?
3. `char` is always 4 bytes in memory, but `'a'.len_utf8()` is `1` and
   `'🦀'.len_utf8()` is `4`. What's the difference between "how much memory
   a `char` value takes" and "how many bytes it takes when UTF-8 encoded",
   and why might that distinction matter once you're reading/writing text
   over a network (Phase 3)?
