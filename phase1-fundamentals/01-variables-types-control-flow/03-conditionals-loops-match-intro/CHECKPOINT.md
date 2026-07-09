# Checkpoint

1. `letter_grade` returns `&'static str`. For now, just describe in your own
   words what you think `'static` might mean (you're not expected to fully
   understand lifetimes yet — Phase 2 covers them properly. What's your
   working guess?).
2. Why does `first_multiple_above` need `loop` + `break value` rather than
   `while`? Could you rewrite it with `while` instead — would it be shorter
   or longer?
3. `match` in `classify` is exhaustive — try deleting the `_ => "large"`
   arm and running `cargo check`. What does the compiler say, and why does
   it care so much compared to, say, an `if`/`else if` chain with no final
   `else`?
