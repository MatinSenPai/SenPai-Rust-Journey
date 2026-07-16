# 07.2 — Config and secrets

## The disease: `std::env::var`, scattered

Every service in this repo so far that needed a database URL or a port
read it at the point of use: a `std::env::var("DATABASE_URL")` in `main`,
an `.unwrap_or` default somewhere else, maybe a second read of the same
variable in another module with a *different* default. Django codebases
grow the same disease — `os.environ.get(...)` sprinkled through
`settings.py` and beyond, each call site with its own opinion about
defaults, each missing variable discovered at whatever random moment that
line happens to run.

The cure is boring and industrial: **read everything once, at startup,
into one typed struct, validating as you go — then pass the struct
around.** After startup, nothing else in the codebase touches the
environment. This lesson builds that loader by hand, stdlib only.

## 12-factor config

The [12-factor](https://12factor.net/config) rule: *config is everything
likely to vary between deploys* — database URLs, bind addresses,
credentials, log levels — and it lives in the **environment**, never in
code. Lesson 07.1 is the reason why: the Docker image you tagged by
commit SHA is one immutable artifact, promoted unchanged from staging to
prod. The *only* thing that differs between those deployments is the
environment handed to the container. Bake a URL into the binary and
"promote the exact artifact we tested" stops being true.

The litmus test is blunt: could you open-source the repo right now
without leaking a credential? If not, config is living in code.

## Precedence: defaults < `.env` file < real environment

Three layers, and the order is the design:

| Layer | Who sets it | Wins over |
|---|---|---|
| hardcoded defaults | the code, for safe non-secrets (`LOG_LEVEL=info`) | nothing |
| `.env` file | you, per-checkout, gitignored — dev convenience | defaults |
| real environment | the deployment (compose `environment:`, k8s, CI) | everything |

The rule behind the order: **the closer a value lives to the actual
deployment, the more it should win.** Defaults know nothing about where
you're running; a `.env` file knows it's *your* checkout; the real
environment is the deployment itself speaking. Reverse any of it and prod
breaks — if the file beat the environment, a stray `.env` inside a
container image would silently override what the orchestrator set.
(Django parallel: `settings.py` defaults, `django-environ` reading a
local `.env`, real env vars in prod — same pyramid, same order.)

## Fail fast, and NAME the variable

There is no honest default for `DATABASE_URL` — any hardcoded value is a
lie about where the database is, and defaulted *secrets* are worse (a
password in source with a version number). So required keys with no value
must **fail at startup**, not at first use. The alternative is the 3am
special: the service boots fine, health checks pass, traffic arrives, and
the *first request that touches the database* blows up — hours after the
deploy that caused it, in a request handler's stack trace that mentions
connection pools, not configuration.

And the error must *name the variable*. `ConfigError::Missing("DATABASE_URL")`
rendering as `missing required config variable: DATABASE_URL` turns the
crash log into the fix. Compare Python's bare `KeyError: 'DATABASE_URL'`
(decent) or — the usual case with `os.environ.get` — a `NoneType` error
three modules away (useless). Django's `ImproperlyConfigured` at import
time is this exact instinct; you're building the Rust version, as a
`Result` instead of an exception.

## Never log secrets

The leak is rarely dramatic. It's `#[derive(Debug)]` on your config
struct plus one `tracing::info!(?config, "starting up")` — and the
database password is now in the logs, the log aggregator, and every
retention backup thereof, forever.

The fix is a newtype: `SecretString` wraps a `String` and implements
`Debug` as the literal text `[REDACTED]`. Because `Debug` derives
delegate to each field's own impl, a `Config` that *contains* a
`SecretString` stays safe to dump wholesale — the safe thing becomes the
default. Reading the real value requires calling `.expose()`, which is
deliberately loud and grep-able: `rg 'expose\(\)'` lists every place
secret material leaves the wrapper.

## Build vs. buy

Real projects mostly reach for crates: `dotenvy` to load a `.env` file
into the process environment, or layered loaders like `figment`/`config`.
Use them at work — they handle quoting, escapes, and multi-line values.
But a dotenv parser is ~20 lines of stdlib, and building the whole loader
once demystifies exactly what those crates do: read, layer, validate.
Nothing magic. This lesson is stdlib-only on purpose.

## Your task

Three `todo!()`s in `src/lib.rs`, tested by the inline `#[cfg(test)]`
module (kata-style — this is pure logic, no I/O to integration-test):

- **(a) `parse_env_file`** — `&str` in, `HashMap<String, String>` out.
  Skip blanks and `#` comments, split each line on the *first* `=`, trim
  both sides, silently ignore malformed lines.
- **(b) `SecretString`'s `Debug` impl** — print `[REDACTED]`, never the
  contents. One test literally asserts the debug output of a config
  containing `hunter2` doesn't contain `hunter2`.
- **(c) `Config::resolve(defaults, file_vars, env_vars)`** — merge the
  three layers with the precedence above, then fail fast with
  `ConfigError::Missing(name)` on the first required key with no value
  from any layer.

Note what `resolve` *doesn't* do: touch `std::env` or the filesystem. It
takes three plain `HashMap`s, which is exactly what makes every
precedence rule unit-testable — the caller (a real `main`) would build
them from `defaults()`, `parse_env_file(&fs::read_to_string(".env")?)`,
and `std::env::vars()`. Pure logic, I/O at the edges — the same split as
every lesson since Phase 1.

## Checkpoint

`cargo test -p p4-07-02-config-and-secrets`, then `CHECKPOINT.md`, then
`solution/SOLUTION.md`.
