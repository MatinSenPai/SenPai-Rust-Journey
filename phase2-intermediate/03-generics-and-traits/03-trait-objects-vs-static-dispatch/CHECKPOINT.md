# Checkpoint

1. Why can't `total_summary_length_generic::<T>` be called with a slice
   containing both `AnimeSeries` and `MangaVolume`, even though both
   implement `Summarize`? Point to exactly what about `T` makes this
   impossible.
2. What is a vtable, in your own words? When does Rust build one for a
   `Box<dyn Summarize>` value — compile time or runtime — and what does a
   call to `.summary()` on that value actually do at runtime that a call
   through `total_summary_length_generic` does not need to do?
3. `make_mixed_collection` returns `Vec<Box<dyn Summarize>>`, not
   `Vec<AnimeSeries>` or some other single concrete type. What would happen
   if you tried to write it as `Vec<dyn Summarize>` (no `Box`)? (Hint: think
   about what the compiler needs to know about a value's size to put it
   directly inside a `Vec`'s backing array, versus what it knows about a
   `dyn Summarize` on its own.)
4. Give a concrete example — not from this lesson — of a scenario where you
   would deliberately reach for `dyn Trait` over a generic, and explain what
   about that scenario specifically requires it.
5. The lesson claims generics are usually faster than trait objects. Where,
   specifically, does that speed difference come from — what is a generic
   call doing at runtime that a `dyn Trait` call is not?
