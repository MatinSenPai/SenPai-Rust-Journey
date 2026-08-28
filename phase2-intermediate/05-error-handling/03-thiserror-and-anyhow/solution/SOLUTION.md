# Solution

```rust
impl From<std::num::ParseIntError> for ConfigError {
    fn from(source: std::num::ParseIntError) -> Self {
        ConfigError::InvalidNumber {
            field: "max_retries".to_string(),
            source,
        }
    }
}
```

Identical to lesson 1's hand-written version — `thiserror` generates
`Display`/`Error` for you, but a custom `From` impl for `?`-based automatic
conversion is still something *you* write, because only you know which
field a given `ParseIntError` should be attributed to.

```rust
pub fn load_and_parse(input: &str) -> anyhow::Result<Config> {
    use anyhow::Context;
    parse_config(input).context("failed to load application config")
}
```

`.context(...)` is a method `anyhow::Context` adds to any `Result<T, E>`
where `E: std::error::Error + Send + Sync + 'static` (which `ConfigError`
satisfies, thanks to `#[derive(thiserror::Error)]`). It converts
`Result<T, ConfigError>` into `anyhow::Result<T>`, wrapping the original
error rather than discarding it — that's why `err.source()` in the test
still finds the underlying `ConfigError`: `.context(msg)` builds a small
chain, `msg` on top, the original error underneath, and `anyhow::Error`'s
`Display` only shows the top of that chain (`err.to_string() ==
"failed to load application config"`), while `.source()` lets you walk
down into it.

This is the whole reason `thiserror` bothered implementing
`std::error::Error` properly back in lesson 1 (rather than the lesson
just using `String` for errors): `anyhow::Context` — and `?` converting
into `anyhow::Result` in general — only works for types that implement the
real `Error` trait. A `String`-based error couldn't plug into this chain at
all.
