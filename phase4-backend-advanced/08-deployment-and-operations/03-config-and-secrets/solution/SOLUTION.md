# Solution

```rust
pub fn parse_env_file(contents: &str) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue; // malformed: no '=' at all
        };
        let key = key.trim();
        if key.is_empty() {
            continue; // malformed: `=value` with no name
        }
        vars.insert(key.to_string(), value.trim().to_string());
    }
    vars
}
```

The promised ~20 lines. Two stdlib pieces do all the work:
`str::split_once('=')` splits on the **first** `=` and hands back both
halves — which is the whole correctness story for values like
`postgres://...?sslmode=disable` (split on the last `=`, or `.split('=')`
into pieces, and that value silently loses its tail). And `let ... else`
makes "malformed → skip" one visible line instead of a nested `match`.
Note what "ignore malformed lines" buys: a `.env` parser that panics on a
stray line is a parser someone will delete in a hurry mid-incident.

```rust
impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}
```

One line, and it's load-bearing: `#[derive(Debug)]` on any struct
*delegates to each field's own `Debug` impl*. So `Config` can derive
`Debug` freely — when the derive reaches the `database_url` field, it
calls *this* impl, not `String`'s, and the whole dump stays loggable.
That's the deepest idea in the lesson: you changed the behavior of every
present *and future* struct that contains a `SecretString`, by
implementing one trait in one place. In Django terms it's the difference
between "remember to exclude the password from every `__repr__`" and
making the password *type itself* refuse to print. (The `secrecy` crate
is this same newtype, productionized — zeroize-on-drop and all.)

`expose()` rather than `Deref`/`AsRef<str>`: the friction is the feature.
Every read of secret material is an explicit, searchable call site.

```rust
fn required(merged: &mut HashMap<String, String>, key: &str) -> Result<String, ConfigError> {
    merged
        .remove(key)
        .ok_or_else(|| ConfigError::Missing(key.to_string()))
}

pub fn resolve(
    defaults: HashMap<String, String>,
    file_vars: HashMap<String, String>,
    env_vars: HashMap<String, String>,
) -> Result<Config, ConfigError> {
    let mut merged = defaults;
    merged.extend(file_vars);
    merged.extend(env_vars);

    Ok(Config {
        database_url: SecretString::new(required(&mut merged, "DATABASE_URL")?),
        bind_addr: required(&mut merged, "BIND_ADDR")?,
        log_level: required(&mut merged, "LOG_LEVEL")?,
    })
}
```

The entire precedence policy is two `extend` calls: `HashMap::extend`
overwrites existing keys, so extending defaults with the file layer, then
with the environment layer, *is* "defaults < file < env" — no
per-key `if` chains. `remove` (not `get` + clone) because the merged map
is scaffolding we own and are about to drop; and `ok_or_else` (not
`ok_or`) so the `String` allocation for the error only happens on the
failure path. The `?` after each `required` is the fail-fast: the first
missing key aborts the whole constructor with its name attached.

## On the recall questions

**Q1 (env over file):** The file layer is a *developer's* convenience;
the environment is the *deployment* speaking. If the file won, any `.env`
accidentally baked into a container image (a missing `.dockerignore`
entry is all it takes) would silently override what the orchestrator set
— prod pointing at a laptop's database URL is the canonical incident.

**Q2 (fail at startup):** A lazy read moves the failure from deploy time
(watched, easy to roll back, obviously caused by the deploy) to first-use
time — potentially hours later, in a request handler, with a stack trace
about connection pools. Startup validation turns a 3am mystery into a
refused deploy. The error must carry the *variable's name*; "missing
required config variable: DATABASE_URL" is an instruction, "invalid
configuration" is a scavenger hunt.

**Q3 (why the derive is safe):** Derived `Debug` calls `Debug::fmt` on
each field in turn. For `database_url` the receiver type is
`SecretString`, so our redacting impl runs. Were the field a plain
`String`, `String`'s impl would run and print the URL — credentials
included — with no warning from the compiler, because *both versions are
perfectly valid programs*. The safety lives entirely in the type choice.

**Q4 (remaining leaks):** Anywhere the code calls `.expose()` and then
puts the `&str` somewhere loggable: interpolating the URL into a
connection *error message* ("failed to connect to postgres://app:pw@...")
is the classic, and serializing config (`Serialize` isn't `Debug` — a
derive would happily emit the real value) is the sneaky one. That's
exactly why the accessor is a loud verb: auditing leaks reduces to
reading the output of `rg 'expose\(\)'`.

**Q5 (first `=`):** Any Postgres URL with query parameters —
`postgres://app:pw@localhost/dev?sslmode=disable` — or a base64 value
with `=` padding. Split on the last `=` and `sslmode=disable` becomes the
value's tail chopped onto the key side; split on all of them and the
value shatters entirely.

**Q6 (who gets a default):** Rule one: secrets never get defaults — a
default secret is a credential committed to source control with extra
steps. Rule two: only values with a truthful universal answer get one —
`LOG_LEVEL=info` is a reasonable claim about any deployment,
`DATABASE_URL=anything` is a guess about infrastructure the binary can't
know. `DATABASE_URL` violates both at once, which is why its absence is
an *error* rather than a fallback.
