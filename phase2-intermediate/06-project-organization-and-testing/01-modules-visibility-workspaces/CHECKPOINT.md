# Checkpoint

1. `internal_rating` is `pub(crate)`, not `pub`. Concretely — if a separate
   crate added this one as a `[dependencies]` entry and wrote
   `let band = my_crate::Anime::new("X", 5); band.internal_rating`, what
   would happen, and at what stage (compile time or run time)?
2. `mod catalog { ... }` has no `pub` in front of it, yet the tests module
   (also with no `pub`) can still write `catalog::normalize_rating(...)`.
   Why does that resolve — what rule makes a private item in one module
   reachable from another?
3. `pub use catalog::Anime;` at the crate root doesn't move `Anime` out of
   `catalog` — it's still defined there. What does it actually do, and why
   would a real library want to do this instead of just making `catalog`
   itself `pub` and telling callers to write `my_crate::catalog::Anime`?
4. `public_rating_band` returns a coarse `"low"`/`"medium"`/`"high"` string
   instead of the raw `internal_rating` number. In your own words, what
   future flexibility does the crate author keep by doing this — what could
   they change later without breaking any code that calls
   `public_rating_band()`?
5. This whole repo is one Cargo workspace. Name one concrete build-time
   benefit that gives you, compared to every lesson being its own totally
   separate crate with no shared `Cargo.lock`.
