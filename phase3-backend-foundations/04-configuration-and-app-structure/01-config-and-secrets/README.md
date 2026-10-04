# 3.4.1 — 12-factor config and secrets

## At a glance

After this lesson you can:

- Explain the twelve-factor rule "config lives in the environment" and say which settings belong in code, in a file, and in an environment variable.
- Stack three layers (defaults, a TOML file, environment variables) into one typed `Config`, where the highest layer wins field by field.
- Report a missing or mistyped setting as a typed `ConfigError` at startup, with no `unwrap` and no panic.
- Keep a password in a `secrecy::SecretString` so that `{:?}`, a log line or a panic message cannot print it.
- Test configuration without touching the real environment, by passing the environment in as a `HashMap`.

**Time:** ~75 minutes · **Prerequisites:**
[3.2.5 — CORS and frontend integration](../../02-axum-and-rest-api-design/05-cors-and-frontend-integration/README.md),
[3.2.3 — Anime catalog CRUD (in-memory)](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.md)

---

## Why this matters

Every server in this phase so far had its settings written into the source: `127.0.0.1:3002`, `"http://localhost:5173"`. That works on your laptop and nowhere else. The same build has to run on your machine, in staging and in production, with a different port, a different frontend origin and a different database password in each. If those differences live in the source, you rebuild per environment, and the production password sits in git.

The Django version of this is `settings.py`: you write `SECRET_KEY = os.environ["SECRET_KEY"]` and `DEBUG = os.environ.get("DEBUG") == "1"`, and a missing key blows up with a `KeyError` when the module is imported. Django also quietly hides any setting whose name contains `PASSWORD` or `SECRET` from its debug error page, because printing settings is exactly how secrets leak. Rust has no `settings.py`. This lesson builds the small version of it, and the Rust part is that the result is a real struct with real types, validated once at startup.

[3.2.5](../../02-axum-and-rest-api-design/05-cors-and-frontend-integration/README.md) left a promise: the allowed CORS origins were a hard-coded string, and "origins that come from the environment" were deferred to here. By the end, `APP_ALLOWED_ORIGINS` feeds the same `CorsLayer`.

---

## The concept

### Twelve-factor config

The [twelve-factor app](https://12factor.net/config) is a checklist for services that deploy cleanly. Its config rule is short: anything that differs between deploys (ports, URLs, credentials) is stored in **environment variables**, not in the code. The test it gives: could you open-source the repository right now without leaking a credential? If not, config is leaking into code.

Three kinds of setting, three homes:

| Kind | Example | Where it lives |
|---|---|---|
| Never changes between deploys | the route table, the JSON field names | code |
| Changes sometimes, safe to read | default port, log format | a config file, in git |
| Differs per deploy, or secret | the database password, the allowed origins | environment variables |

### Layers: default < file < environment

Real services use all three at once. The precedence runs from least to most specific: a compiled-in **default**, then a **file**, then the **environment**, so an operator can override one setting for one deploy without editing a file. Each layer sets only some of the settings, so a layer is a struct in which every field is an `Option`:

```rust
#[derive(Debug, Default, Clone)]
pub struct Layer {
    pub port: Option<u16>,
    pub allowed_origins: Option<Vec<String>>,
    pub debug: Option<bool>,
    pub db_password: Option<SecretString>,
}
```

`None` means "this layer has no opinion". Stacking two layers is a field-by-field `or`: `self.port.or(lower.port)` is `self`'s value if there is one, and `lower`'s otherwise. That one method, `over`, is the whole precedence mechanism.

```senpai-visual
{"kind":"concept","labels":["defaults in code","config.toml file","APP_ environment variables","stacked field by field","typed Config or a ConfigError"]}
```

After stacking, `finish` fills the remaining `None`s with defaults and turns the result into a `Config` whose fields are plain, non-optional types. The one setting with no default is the password: a service that starts without it should refuse to start.

### Errors, not panics

`std::env::var("APP_PORT")` returns `Result<String, VarError>`, and the lazy reaction is `.unwrap()`. At startup that is the worst place to panic: the operator sees `called Result::unwrap() on an Err value: NotPresent`, which names neither the setting nor the fix ("Errors you will meet" shows it). So every failure is a variant of one enum that says which setting and what was wrong:

```rust
pub enum ConfigError {
    Missing { key: &'static str },
    Invalid { key: &'static str, value: String, expected: &'static str },
    File(String),
}
```

`main` prints the error with `Display` and exits with a non-zero code. That is the whole error strategy: configuration is read once, before anything serves a request, so a bad value should stop the process loudly and immediately.

### The environment is just an argument

Reading `std::env::var` inside the parsing code makes it untestable: the process environment is global, shared between tests that run in parallel, and changing it is not thread-safe. Instead the parsing takes the environment as a parameter:

```rust
pub fn from_env(env: &HashMap<String, String>) -> Result<Layer, ConfigError>
```

A test builds a `HashMap` with exactly the variables it wants. Only `main` touches the real thing, with `std::env::vars().collect()`. This is the same idea as passing a `Router` to `oneshot` instead of binding a socket in [3.2.1](../../02-axum-and-rest-api-design/01-routing-handlers-extractors/README.md): push the global thing to the edge, and everything inside becomes a plain function.

### Secrets that cannot leak

Try the obvious design: `db_password: String` in a struct that derives `Debug`. The first time anyone writes `println!("{config:?}")` or `tracing::info!(?config)` to see what the service started with, the password is in the log. `examples/01-debug-leak.rs` prints the same struct twice:

```text
Leaky { port: 8080, db_password: "hunter2" }
Careful { port: 8080, db_password: SecretBox<str>([REDACTED]) }
on purpose: hunter2
```

`secrecy::SecretString` (crate `secrecy` 0.10, an alias for `SecretBox<str>`) holds the text, implements `Debug` as `[REDACTED]`, and implements neither `Display` nor `Serialize`, so the wrong `{}` is a compile error rather than a leak. To read the value you must call `expose_secret()` from the `ExposeSecret` trait:

```rust
use secrecy::ExposeSecret;
let url = format!("postgres://app:{}@db/anime", config.db_password.expose_secret());
```

Two things follow. The method name is the audit trail: `grep expose_secret` lists every place the secret can possibly leave. And the memory is zeroed when the value is dropped, which is a bonus, not the point. The point is that the safe behaviour is the default and the unsafe one needs a deliberate, greppable call. Note that secrecy does not protect a secret you copy out yourself: `expose_secret().to_string()` into a log line is still a leak.

### The file layer, and typos

The file layer uses `toml` and `serde`'s derive, the same way you parsed JSON. One attribute earns its place: `#[serde(deny_unknown_fields)]` turns a mistyped key into an error instead of a silently ignored line:

```text
Err(File("TOML parse error at line 1, column 1\n  |\n1 | prot = 7000\n  | ^^^^\nunknown field `prot`, expected one of `port`, `allowed_origins`, `debug`, `db_password`\n"))
```

Without it, `prot = 7000` would be accepted, the real `port` would stay at its default, and you would spend an evening wondering why your setting does nothing.

---

## Hands on

Two examples work on the code you were given:

```sh
cargo run -p p3-04-01-config-and-secrets --example 01-debug-leak
cargo run -p p3-04-01-config-and-secrets --example 02-file-layer
```

The first prints the three lines shown above. The second parses three TOML snippets with the given `Layer::from_toml`, one good, one with a typo, one with a wrong type:

```text
Ok(Layer { port: Some(7000), allowed_origins: Some(["https://anime.example.com"]), debug: None, db_password: None })
Err(File("TOML parse error at line 1, column 1\n  |\n1 | prot = 7000\n  | ^^^^\nunknown field `prot`, expected one of `port`, `allowed_origins`, `debug`, `db_password`\n"))
Err(File("TOML parse error at line 1, column 8\n  |\n1 | port = \"seven thousand\"\n  |        ^^^^^^^^^^^^^^^^\ninvalid type: string \"seven thousand\", expected u16\n"))
```

(Until you finish the exercises, `cargo` also prints `unused variable` warnings for the functions you have not written yet. Ignore them.)

The next two need the Implement and Build rungs done, because they call your functions. Here is what they print with the finished solution. `03-load-layers` loads a file and several made-up environments, with no real environment involved:

```sh
cargo run -p p3-04-01-config-and-secrets --example 03-load-layers
```

```text
Ok(Config { port: 7000, allowed_origins: ["http://localhost:5173"], debug: true, db_password: SecretBox<str>([REDACTED]) })
Ok(Config { port: 9000, allowed_origins: ["http://localhost:5173"], debug: true, db_password: SecretBox<str>([REDACTED]) })
Err(Missing { key: "db_password" })
Err(Invalid { key: "APP_PORT", value: "eighty", expected: "a port number from 0 to 65535" })
missing required setting `db_password`
```

Line 1 is the file alone, line 2 adds `APP_PORT=9000` and wins over the file's `7000`, and the `allowed_origins` is the default because nobody set it. `04-serve-from-env` is the real thing: the real environment, a real port, and the CORS layer from 3.2.5 fed from configuration. Start it on port 3140, in one terminal, and leave it running:

```sh
APP_PORT=3140 APP_DB_PASSWORD=hunter2 APP_ALLOWED_ORIGINS=https://anime.example.com \
  cargo run -p p3-04-01-config-and-secrets --example 04-serve-from-env
```

```text
Config { port: 3140, allowed_origins: ["https://anime.example.com"], debug: false, db_password: SecretBox<str>([REDACTED]) }
listening on http://127.0.0.1:3140
```

In a second terminal, send a request from the configured origin, then one from another:

```sh
curl -si http://127.0.0.1:3140/ -H "Origin: https://anime.example.com"
curl -si http://127.0.0.1:3140/ -H "Origin: http://localhost:5173"
```

```text
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
vary: origin, access-control-request-method, access-control-request-headers
access-control-allow-origin: https://anime.example.com
content-length: 3
date: Sun, 04 Oct 2026 11:14:04 GMT

ok
```

The second request gets the same `200` and `ok` but no `access-control-allow-origin` line, exactly the "declining to vouch" behaviour from 3.2.5, now driven by an environment variable. (The `date` changes every run.) Stop the server with Ctrl+C. Now start it with no password, and then with a bad port:

```text
cannot start: missing required setting `db_password`
cannot start: invalid value "eighty" for APP_PORT: expected a port number from 0 to 65535
```

Both exit with code 1 before any socket is bound. Then try these:

1. Create a `config.toml` next to where you run the example with `port = 3142` and `db_password = "from-file"`, and start it with no environment at all. Which port does it use? Now add `APP_PORT=3143`.
2. Start it with `APP_DEBUG=yes`. What does the error say the accepted values are?

---

## Errors you will meet

### A run-time panic: `unwrap` on a missing variable

```text
thread 'main' (42364) panicked at phase3-backend-foundations\04-configuration-and-app-structure\01-config-and-secrets\examples\05-unwrap-env-broken.rs:10:47:
called `Result::unwrap()` on an `Err` value: NotPresent
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(The number in parentheses is the thread's id and changes every run.)

**What's actually wrong:** `std::env::var("APP_PORT").unwrap()` assumes the variable is set. It is not, so the program panics, and the message names neither `APP_PORT` nor what to do about it: `NotPresent` is the `VarError` variant, and the line number points at your code, not at the setting. A mistyped value fails the same way one `unwrap` later, as a `ParseIntError`.

**The fix:** return a typed error that carries the setting's name:

```rust
let raw = std::env::var("APP_PORT").map_err(|_| ConfigError::Missing { key: "APP_PORT" })?;
```

**Why this is the fix:** the operator reading the log needs the key and the problem, and `Display` on `ConfigError` gives both in one line. In the lesson's code the parsing lives in `Layer::from_env`, which gets the first bad variable's name and value into the error.

### E0277: printing a secret like a `String`

```text
error[E0277]: `SecretBox<str>` doesn't implement `std::fmt::Display`
  --> phase3-backend-foundations\04-configuration-and-app-structure\01-config-and-secrets\examples\06-print-secret-broken.rs:11:45
   |
11 |     println!("connecting with password {}", password);
   |                                        --   ^^^^^^^^ `SecretBox<str>` cannot be formatted with the default formatter
   |                                        |
   |                                        required by this formatting parameter
   |
   = help: the trait `std::fmt::Display` is not implemented for `SecretBox<str>`
   = note: in format strings you may be able to use `{:?}` (or {:#?} for pretty-print) instead

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-04-01-config-and-secrets` (example "06-print-secret-broken") due to 1 previous error
```

**What the compiler is objecting to:** `{}` needs `Display`, and `SecretString` deliberately does not implement it. The `help` line suggests `{:?}`, and that one is safe here: it prints `[REDACTED]`.

**The fix:** most of the time, do not print it at all. If the program really needs the text, to build a connection string, say so in the code:

```rust
println!("connecting with password {}", password.expose_secret());
```

**Why this is the fix:** the compile error is the feature. A `String` lets the leak happen by accident in any `{}`; `SecretString` makes the leak a call you wrote on purpose and can find with `grep`.

### No error at all: an empty variable that locks everyone out

```text
Config { port: 3141, allowed_origins: [], debug: false, db_password: SecretBox<str>([REDACTED]) }
listening on http://127.0.0.1:3141
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
vary: origin, access-control-request-method, access-control-request-headers
content-length: 3
date: Sun, 04 Oct 2026 11:14:54 GMT

ok
```

**What's actually wrong:** the server was started with `APP_ALLOWED_ORIGINS=` (set, but empty). A variable that is present but empty is still present, so it overrides the default and the list of allowed origins becomes empty: no frontend is vouched for, the status is `200`, and the only symptom is a browser console full of CORS errors. This is exactly the silent failure of 3.2.5's origin that never matches, arriving through configuration.

**The fix:** decide what an empty value means and say it in the type. Either treat an empty list as `Missing`, or as in this lesson accept it as "no origins" and print the loaded `Config` at startup, as `main` does here, so the empty list is visible in the first log line.

**Why this is the fix:** the compiler cannot tell "intentionally none" from "forgot to fill in", but a startup line that shows the final values can. Printing the config at startup is only safe because the password is a `SecretString`.

---

## Exercises

### Warm up

<details>
<summary>The twelve-factor rule says config belongs in environment variables. Why not a file in the repository for the database password?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

Because a file in the repository is in git, and git history is forever and is read by everyone with access. The password differs per deploy and is secret, so it must come from outside the code at the moment the process starts. A file may hold non-secret defaults, but the deploy-specific and secret values go in the environment.

</details>

<details>
<summary>A struct that derives <code>Debug</code> has a <code>db_password: String</code>. Where might the password end up without anyone writing <code>println!</code> on it?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

Anywhere the whole struct is formatted with `{:?}`: a `tracing::info!(?config)` line, an `unwrap()` on a `Result` that contains it, a panic message, an error report that includes the state of the program. Each of them is a log or a crash report that many people can read.

</details>

<details>
<summary>The file says <code>port = 7000</code> and the environment has <code>APP_PORT=9000</code>. Which one wins, and which method decides?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`9000`. The environment layer is the higher-priority one, and `env_layer.over(file_layer)` takes `self`'s value whenever it is `Some`. A field the environment leaves unset, `None`, falls back to the file's value, and then to the default.

</details>

<details>
<summary>Why does <code>from_env</code> take a <code>&amp;HashMap</code> instead of calling <code>std::env::var</code> itself?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

Because the real environment is global, shared by every test thread, and changing it is not thread-safe. With a map as a parameter each test builds its own environment and nothing touches the process. Only `main` reads the real one, once.

</details>

### Repair

Fix both broken examples:

1. `examples/05-unwrap-env-broken.rs` should not panic when `APP_PORT` is missing or is not a number: it prints one readable line that names `APP_PORT` and exits with code 1.
2. `examples/06-print-secret-broken.rs` should compile and print the password as `[REDACTED]`, without calling `expose_secret`.

### Implement

Two small functions in `src/lib.rs`, each fully specified by its doc comment:

```sh
cargo test -p p3-04-01-config-and-secrets
```

- `parse_bool`: the accepted spellings of a boolean setting, and the error for anything else.
- `parse_origins`: one comma-separated string into a clean list of origins.

The `todo!()` tests in the same file fail until these two are done; the three tests in `tests/` pass from the start, because they only use the code you were given.

### Build

The pipeline, in four pieces. Each has a doc comment that says exactly what it does:

- `Layer::from_env`: the four `APP_` variables into a `Layer`, with the typed errors.
- `Layer::over`: stacking two layers.
- `Layer::finish`: defaults, and the one required setting.
- `Config::load`: the whole pipeline in one call. When all four pass, run `03-load-layers` and `04-serve-from-env`.

### Challenge (optional)

Two origins that differ only by a trailing slash never match, as you saw in 3.2.5. Make `finish` reject an `allowed_origins` entry that ends with `/`, with `ConfigError::Invalid { key: "APP_ALLOWED_ORIGINS", .. }` naming it. Write the test in a new file under `tests/`. Then consider: is `finish` the right place, or should `from_env` and `from_toml` each do it?

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Twelve-factor config | per-deploy settings live in the environment, not in code | every deployed service |
| Layered configuration | default < file < environment, merged field by field | `Layer::over` |
| `ConfigError` | a typed failure that names the setting and the problem | startup, instead of `unwrap` |
| `SecretString` | a string whose `Debug` is `[REDACTED]` and that has no `Display` | passwords, tokens, API keys |
| `expose_secret()` | the one deliberate call that reads a secret | building a connection string |
| `deny_unknown_fields` | a serde attribute that turns a typo into an error | any config file |

### What you now know

- Config that differs per deploy lives in the environment; defaults and file values sit underneath it, and the highest layer wins field by field.
- A missing or mistyped setting is a typed error found at startup, never a panic with `NotPresent` in it.
- The environment is a parameter, so tests build their own and never touch the process.
- `SecretString` makes the safe thing the default: `{:?}` redacts, `{}` does not compile, and reading is a greppable `expose_secret()`.
- An empty variable is still a variable, and `deny_unknown_fields` is what stops a typo in a file from hiding.

### What comes back later

- **Putting the finished `Config` where every handler can reach it** — [3.4.2 — Application state and dependency wiring](../02-app-state-and-dependency-wiring/README.md)
- **A process that stops cleanly, and the `/health` endpoint a deployment polls** — [3.4.3 — Graceful shutdown, health and readiness](../03-graceful-shutdown-health-readiness/README.md)
- **The database URL and pool size are configuration too** — [Module 5 — PostgreSQL & `sqlx`](../../05-postgres-and-sqlx/README.md)
- **A password you must never store at all, only a hash of** — [3.7.1 — Password hashing with `argon2`](../../07-auth-and-security/01-password-hashing-argon2/README.md)

Back to the module: [Module 4 index](../README.md).

### Can you explain?

- Which three kinds of setting exist, and where does each one live?
- Why is `over` a field-by-field `or` and not "use the whole higher layer if it has anything"?
- Why is `unwrap` on `std::env::var` the wrong tool at startup, even though it "works"?
- What stops `{:?}` on a `Config` from printing the password, and what stops `{}` from compiling?
- What does `deny_unknown_fields` protect you from?

---

## Going further

- [The Twelve-Factor App — III. Config](https://12factor.net/config): the original rule and its reasoning, in a page.
- [`secrecy` 0.10 on docs.rs](https://docs.rs/secrecy/0.10.3/secrecy/): `SecretBox`, `SecretString` and the `ExposeSecret` trait.
- [`toml` on docs.rs](https://docs.rs/toml/0.8/toml/): the parser used for the file layer, and its error type.
- [Module 4 index](../README.md): the three lessons of this module in order.
