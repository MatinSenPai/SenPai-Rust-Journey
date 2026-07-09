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

```rust
let max_retries_raw = fields
    .get("max_retries")
    .ok_or_else(|| ConfigError::MissingField("max_retries".to_string()))?;
let max_retries: u32 = max_retries_raw.parse()?;
```

The interesting line is `max_retries_raw.parse()?`. `.parse::<u32>()` returns
`Result<u32, ParseIntError>` — a different error type than `parse_config`'s
declared `Result<Config, ConfigError>`. Normally `?` would refuse to compile
here (mismatched error types), but because `impl From<ParseIntError> for
ConfigError` exists, the compiler inserts a call to it automatically: `?`
expands to roughly "if `Err(e)`, `return Err(ConfigError::from(e))`." That's
the entire mechanism — no macro magic, just a trait lookup the compiler
performs for you at every `?`.

Compare that to `timeout_secs`:

```rust
let timeout_secs: u32 = timeout_secs_raw
    .parse()
    .map_err(|source| ConfigError::InvalidNumber {
        field: "timeout_secs".to_string(),
        source,
    })?;
```

Same underlying failure (`ParseIntError`), same target variant
(`InvalidNumber`) — but this call site can't lean on the `From` impl, because
that impl hardcodes `field: "max_retries"`. `From::from` only ever receives
the `ParseIntError` itself; it has no way to know which `.parse()` call
produced it. So the only way to get the *correct* field name into the error
is to supply it explicitly at the call site, which means an explicit
`.map_err` instead of a bare `?`. This is checkpoint question 2: the two
call sites produce the same enum variant, but only one of them can rely on
`From` doing the labeling for it.

If this were a bigger config with many numeric fields, hardcoding one field
name into a single blanket `From` impl would stop being a good tradeoff —
you'd drop the `From` impl entirely and use `.map_err(...)` everywhere for
consistency. `From` earns its keep specifically when there's one dominant,
unambiguous conversion; the moment context-dependent information (like a
field name) is required, an explicit conversion at the call site is more
honest than a `From` impl silently guessing.

On checkpoint question 3: `impl std::error::Error for ConfigError {}`
compiles with an empty body only because `Error`'s supertrait bounds
(`Debug + Display`) are already satisfied — `#[derive(Debug)]` gives us
`Debug`, and the hand-written `impl Display` above gives us `Display`. Try
deleting the `#[derive(Debug)]` line and the `impl Error` line stops
compiling, with an error pointing at the missing `Debug` bound — a good way
to see the requirement made concrete instead of just reading about it.
