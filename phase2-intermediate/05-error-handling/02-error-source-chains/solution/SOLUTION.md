# Solution — 2.5.2 Source chains and `Box<dyn Error>`

```rust
impl From<std::io::Error> for ConfigError {
    fn from(source: std::io::Error) -> Self {
        ConfigError::Io(source)
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),
            ConfigError::MissingField(_) => None,
            ConfigError::InvalidNumber { source, .. } => Some(source),
        }
    }
}

pub fn load_config(path: &str) -> Result<Config, ConfigError> {
    let contents = std::fs::read_to_string(path)?;
    parse_config_str(&contents)
}

pub fn error_chain(err: &dyn Error) -> Vec<String> {
    let mut chain = vec![err.to_string()];
    let mut cause = err.source();
    while let Some(source) = cause {
        chain.push(source.to_string());
        cause = source.source();
    }
    chain
}

pub fn effective_max_retries(
    path: &str,
    override_max_retries: Option<&str>,
) -> Result<u32, Box<dyn Error>> {
    match override_max_retries {
        Some(text) => Ok(text.parse::<u32>()?),
        None => Ok(load_config(path)?.max_retries),
    }
}
```

## `From<std::io::Error>` — the one-line wrapper that makes `?` work

```rust
fn from(source: std::io::Error) -> Self {
    ConfigError::Io(source)
}
```

Nothing clever — just move `source` into the variant built for it. The reason this exists at all is `load_config`: without this `impl`, `std::fs::read_to_string(path)?` would refuse to compile, because `?` has no way to turn an `io::Error` into a `ConfigError` on its own. This is the exact same mechanism 2.5.1 used for `ParseIntError`, now for the failure mode a real file introduces.

## `source()` — one line per variant, matching what each one actually wraps

```rust
fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
        ConfigError::Io(e) => Some(e),
        ConfigError::MissingField(_) => None,
        ConfigError::InvalidNumber { source, .. } => Some(source),
    }
}
```

`Io` and `InvalidNumber` each hold another error as a field, so their arms hand back a reference to it: `Some(e)`, `Some(source)`. `MissingField` holds only a `String` — there's no error underneath it to point at — so its arm is `None`. If you wrote `Some(...)` for `MissingField` too, the compiler would refuse it outright: there's no `&(dyn Error + 'static)` to borrow from a plain `String` field. Getting this wrong the other way — writing `None` for `Io` — would compile fine and just quietly throw away information a caller could have used; nothing catches that mistake for you, which is exactly why matching each variant to its real shape matters.

## `load_config` — two conversions through one `?` each

```rust
pub fn load_config(path: &str) -> Result<Config, ConfigError> {
    let contents = std::fs::read_to_string(path)?;
    parse_config_str(&contents)
}
```

The first `?` is doing real work: converting an `io::Error` into a `ConfigError` via the `From` impl above. The second line needs no `?` and no conversion at all — `parse_config_str` already returns `Result<Config, ConfigError>`, the same type this function returns, so its `Result` is simply handed back as-is.

## `error_chain` — the loop from "Walking the chain," now returning instead of printing

```rust
pub fn error_chain(err: &dyn Error) -> Vec<String> {
    let mut chain = vec![err.to_string()];
    let mut cause = err.source();
    while let Some(source) = cause {
        chain.push(source.to_string());
        cause = source.source();
    }
    chain
}
```

`chain` starts with exactly one entry: `err`'s own message. Then the loop is the same shape the lesson ran with `println!` — call `.source()`, and as long as it's `Some`, push that link's message and ask it for *its* `.source()` in turn. The loop condition itself is the termination proof: it stops the moment a `.source()` call returns `None`, which is guaranteed to happen eventually because nothing in this lesson's `ConfigError` can point at itself.

## `effective_max_retries` — the override short-circuits before `path` is ever touched

```rust
pub fn effective_max_retries(
    path: &str,
    override_max_retries: Option<&str>,
) -> Result<u32, Box<dyn Error>> {
    match override_max_retries {
        Some(text) => Ok(text.parse::<u32>()?),
        None => Ok(load_config(path)?.max_retries),
    }
}
```

The `match` decides everything up front. On the `Some(text)` arm, `path` never appears at all — `text.parse::<u32>()?` either succeeds or propagates a `ParseIntError`, and that's the whole story. On the `None` arm, `load_config(path)?` either succeeds or propagates a `ConfigError`. Two completely different concrete error types come out of the same function, and both convert into `Box<dyn Error>` through their own bare `?` — the return type is the only place that decision is made; neither branch's code even mentions `Box`.

## What this lesson was really about

- **`source()`'s signature is not a suggestion.** `Option<&(dyn Error + 'static)>` is exactly what the trait declared, and an `impl` that elides the lifetime differently — tying it to `&self` instead of `'static` — is a different signature the compiler rejects outright.
- **A chain is just repeated `.source()` calls, nothing more.** `error_chain` is the same five-line loop from "The concept," proven correct by a test that actually walks two real links.
- **`Box<dyn Error>` earns its keep exactly where two unrelated error types meet in one function.** `effective_max_retries` never had to unify `ConfigError` and `ParseIntError` into a shared enum — the return type did that job by itself.
- **The convenience and the cost are the same fact, seen from two sides.** Type erasure is what let two unrelated errors share a return type; it's also exactly why nothing downstream can `match` on the result without `downcast_ref` first.
