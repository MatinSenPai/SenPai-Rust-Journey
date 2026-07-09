# Checkpoint

1. Suppose `ConfigError` were still just `String`, like in Phase 1. Write,
   in your own words (no code needed), what a caller would have to do to
   tell "a field was missing" apart from "a field wasn't a valid number" if
   all they have is a `String`. Why is that fragile?

2. `parse_config` uses a bare `?` after `.parse()` for `max_retries`, but
   needs `.map_err(...)` for `timeout_secs`. Both failures produce the exact
   same `ConfigError::InvalidNumber` variant. Explain precisely why one call
   site can use `?` alone and the other can't.

3. `impl std::error::Error for ConfigError {}` has an empty body. What are
   we relying on already being true about `ConfigError` for that empty
   `impl` to even compile (hint: re-read what `Error` requires)?

4. `#[derive(Debug)]` sits right above `pub enum ConfigError`, and you also
   hand-wrote `impl Display for ConfigError`. Why do you need *both* —
   what's one concrete thing each trait is used for in this lesson's tests
   that the other trait couldn't give you?

5. If you later wanted `ConfigError` to wrap a *different* library's error
   type (say, one for reading the config from a file instead of a `&str`),
   what would you add to `ConfigError`, and what new `impl` would let `?`
   convert that library's error automatically, the same way it already does
   for `ParseIntError`?
