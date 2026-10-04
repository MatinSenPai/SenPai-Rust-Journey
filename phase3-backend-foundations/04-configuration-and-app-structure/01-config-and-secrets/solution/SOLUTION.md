# Solution — 3.4.1 12-factor config and secrets

## `parse_bool`

```rust
if raw.eq_ignore_ascii_case("true") || raw == "1" {
    Ok(true)
} else if raw.eq_ignore_ascii_case("false") || raw == "0" {
    Ok(false)
} else {
    Err(ConfigError::Invalid { key, value: raw.to_string(), expected: "true, false, 1 or 0" })
}
```

`eq_ignore_ascii_case` covers `TRUE` and `False` without allocating a lowercase copy. Nothing is trimmed on purpose: `" true"` is an error, because a stray space in an environment variable is a typo worth reporting, not hiding.

## `parse_origins`

```rust
raw.split(',')
    .map(str::trim)
    .filter(|piece| !piece.is_empty())
    .map(String::from)
    .collect()
```

`"a, b,,c "` gives `["a", "b", "c"]`, and `""` gives an empty `Vec` because the single empty piece is filtered out.

## `Layer::from_env`

```rust
let port = match env.get("APP_PORT") {
    Some(raw) => Some(raw.parse::<u16>().map_err(|_| ConfigError::Invalid {
        key: "APP_PORT",
        value: raw.clone(),
        expected: "a port number from 0 to 65535",
    })?),
    None => None,
};
```

The other three variables follow the same shape: `APP_DEBUG` goes through `parse_bool`, `APP_ALLOWED_ORIGINS` through `parse_origins`, and `APP_DB_PASSWORD` is wrapped with `SecretString::from` as-is (not trimmed: whitespace can be part of a password). The `?` inside each `match` arm returns on the first bad variable, in table order. `env.get` gives `Some("")` for a present-but-empty variable, so `APP_PORT=` fails to parse (an empty string is not a `u16`) and `APP_ALLOWED_ORIGINS=` becomes `Some(vec![])`, both as the doc comment specifies. `"-1"`, `"70000"` and `"80 "` all fail `u16::from_str`.

## `Layer::over`

```rust
Layer {
    port: self.port.or(lower.port),
    allowed_origins: self.allowed_origins.or(lower.allowed_origins),
    debug: self.debug.or(lower.debug),
    db_password: self.db_password.or(lower.db_password),
}
```

`Option::or` is the whole precedence rule: the first `Some` wins, per field. Using `or` per field rather than "use the whole higher layer if it has anything" is what lets the environment override only the one setting it names and leave the rest to the file.

## `Layer::finish`

```rust
Ok(Config {
    port: self.port.unwrap_or(DEFAULT_PORT),
    allowed_origins: self.allowed_origins.unwrap_or_else(|| vec![DEFAULT_ORIGIN.to_string()]),
    debug: self.debug.unwrap_or(false),
    db_password: self.db_password.ok_or(ConfigError::Missing { key: "db_password" })?,
})
```

`unwrap_or_else` for the origins, because building the `Vec` allocates and should only run when needed. The password is the only `ok_or`: no default, so absence is an error.

## `Config::load`

```rust
let file_layer = match file {
    Some(text) => Layer::from_toml(text)?,
    None => Layer::default(),
};
Layer::from_env(env)?.over(file_layer).finish()
```

The file is parsed first, so a file error is reported before an environment error, as the doc comment promises. An absent file is simply an empty layer, not an error. Reading `config.toml` from disk is deliberately not in here: `main` does the I/O, so `load` stays a pure function that tests can call with strings.

## On the challenge

```rust
if let Some(origin) = origins.iter().find(|o| o.ends_with('/')) {
    return Err(ConfigError::Invalid {
        key: "APP_ALLOWED_ORIGINS",
        value: origin.clone(),
        expected: "an origin without a trailing slash",
    });
}
```

Put it in `finish`: it is the one place every layer, file or environment, has already been merged, so the check runs once on the final list. The cost is that the error says `APP_ALLOWED_ORIGINS` even when the bad value came from the file's `allowed_origins`. If that matters, validate in each layer's constructor and name the real source.
