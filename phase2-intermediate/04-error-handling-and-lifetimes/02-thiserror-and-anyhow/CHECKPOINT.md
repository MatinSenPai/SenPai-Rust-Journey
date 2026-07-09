# Checkpoint

1. Delete the `#[error("...")]` attribute from one `ConfigError` variant
   and run `cargo check`. What happens — is it a compile error or a
   runtime difference? What does that tell you about how essential
   `#[error(...)]` is to `#[derive(thiserror::Error)]`, versus being
   optional decoration?
2. `err.source()` in the last test returns the *original* `ConfigError`
   underneath the `anyhow::Context` wrapping. Why does `.context(...)`
   preserve the original error instead of replacing it — what would be
   lost if `anyhow::Error`'s `Display` were the *only* way to inspect a
   failure?
3. This lesson's rule of thumb is "`thiserror` in `lib.rs`, `anyhow` in
   `main.rs`." `parse_config` returns `Result<Config, ConfigError>`;
   `load_and_parse` returns `anyhow::Result<Config>`. If you were designing
   a real CLI tool around this, which function's signature would you put
   in a reusable library crate, and which would live only in the binary's
   `main.rs`? Why?
4. Try changing `load_and_parse` to use `.with_context(|| ...)` instead of
   `.context(...)`. When would the lazy (`with_context`) version actually
   matter for performance, versus being purely stylistic here?
