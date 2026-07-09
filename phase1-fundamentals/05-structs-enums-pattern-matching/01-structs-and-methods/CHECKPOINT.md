# Checkpoint

1. Why does `new` return `Self` instead of `Book`? Are they actually
   different, or just two spellings of the same thing here?
2. `read_chapter` and `mark_favorite` take `&mut self`; `progress_percent`
   and `describe` take `&self`. What would go wrong (in terms of what the
   caller can do afterward) if you made `describe` take `&mut self` instead,
   even though it never modifies anything?
3. All four fields on `Book` are `pub`. What's the trade-off of that versus
   keeping them private and only exposing methods? (You don't need to fix
   this — just reason about it; Phase 2's modules/visibility lesson goes
   deeper.)
4. `new` is called as `Book::new(...)`, not `book_instance.new(...)`. Why —
   what's different about a function with no `self` parameter?
