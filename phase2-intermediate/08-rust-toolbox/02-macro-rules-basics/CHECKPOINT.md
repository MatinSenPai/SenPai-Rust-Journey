# Checkpoint

1. In your own words: what's the difference between the matcher and the
   transcriber of a `macro_rules!` arm, and at what point in compilation
   does the swap between them happen?
2. `string_map!`'s matcher uses `$key:expr => $value:expr`. Why does this
   *have* to be a macro — what happens if you try to design a plain
   function or even a generic function that accepts `"k" => "v"` syntax?
3. `max_of!` has two arms: `($only:expr)` and `($first:expr,
   $($rest:expr),+)`. Trace the expansion of `max_of!(3, 9, 7)` by hand,
   one recursion step at a time. What goes wrong if you delete the
   single-expression base-case arm?
4. The test `timed_evaluates_the_expression_exactly_once` passes a block
   that increments a counter. Which buggy-but-plausible transcriber is
   this test designed to catch, and why is `let result = $work;` the fix?
5. `timed!`'s transcriber declares `let start = ...` internally. A caller
   writes `let start = 5; let (r, d) = timed!(start + 1);` — does the
   macro's `start` shadow the caller's? What is this property called?
6. A teammate proposes a macro `get_or_return!(map, key)` that expands to
   an early `return` from the *caller's* function. It works. Give two
   reasons from this lesson's "when not to write a macro" guidance to
   push back in review.
