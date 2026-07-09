# Checkpoint

1. `sum_in_threads` never uses `Arc` or `Mutex` at all. Why doesn't it need
   them, given that it's using multiple threads just like
   `count_matching_in_threads`?
2. In `count_matching_in_threads`, what would happen (concretely — a
   compile error, or a runtime bug) if you shared the counter as a plain
   `Mutex<i32>` across threads without wrapping it in `Arc` first? Think
   about ownership: can two threads each *own* the same `Mutex<i32>`
   without `Arc`?
3. `predicate: fn(i32) -> bool` (a plain function pointer) rather than
   `impl Fn(i32) -> bool`. Try changing a call site to pass a capturing
   closure (e.g. one that captures a `threshold` variable from outside) —
   does it still compile? What does that tell you about the difference
   between a function pointer and a closure?
4. `.lock().unwrap()` appears in the solution. When would `.lock()` return
   `Err` instead of `Ok`? (Hint: it's related to what happens if a thread
   panics while holding the lock — look up "lock poisoning" if you want
   the precise term.)
5. Try writing `count_matching_in_threads`'s final line as a bare tail
   expression, `*counter.lock().unwrap()`, instead of binding it to a
   `let final_count = ...; final_count` first. Run `cargo check` — what
   error do you get, and what does it say is being dropped too early? This
   is a real, sharp edge of `MutexGuard` combined with tail-expression
   drop order, not a hypothetical.

