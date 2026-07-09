# Checkpoint

1. `Summarize::summary`'s default body calls `self.title()`, even though at
   the point the trait is *defined*, no concrete type exists yet. Why is
   this allowed? What exactly is the compiler relying on to guarantee the
   call is safe?
2. `MangaVolume` implements `Summarize` but never writes a `summary` method
   at all. Where does `vol.summary()` actually execute from? Is that code
   duplicated into `MangaVolume`'s `impl` block, or shared?
3. If you added a third struct, `LightNovel`, and implemented `Summarize`
   for it but forgot `title`, what would happen — a runtime panic the first
   time `.summary()` is called, or something else? When, exactly, would you
   find out?
4. Compare this to a Python `ABC` with `@abstractmethod title` and a mixin
   `summary` method. Both catch a missing `title` eventually — what's
   different about *when* each language catches it, and why might that
   matter for a program that's already running in production?
5. `print_all_summaries<T: Summarize>` takes `&[T]` — one slice, all
   elements the same concrete type `T`. Could you pass it a slice containing
   *both* `AnimeSeries` and `MangaVolume` values mixed together? Why or why
   not? (Next lesson answers this properly — just reason about it here.)
