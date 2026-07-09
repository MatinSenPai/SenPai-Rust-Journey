# 04.2 — `thiserror` and `anyhow`

## `thiserror`: the boilerplate from lesson 1, generated for you

Lesson 1's `ConfigError` needed a hand-written `impl Display` (a `match`
with one arm per variant) and an empty `impl std::error::Error for
ConfigError {}`. That pattern — enum, `Display`, `Error` — is so common in
real Rust codebases that the `thiserror` crate exists purely to generate it
from attributes:

```rust
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("missing required field: {0}")]
    MissingField(String),

    #[error("invalid number for field '{field}': {source}")]
    InvalidNumber {
        field: String,
        #[source]
        source: std::num::ParseIntError,
    },
}
```

`#[derive(thiserror::Error)]` generates the exact `impl Display` and `impl
std::error::Error` you wrote by hand last lesson:
- `#[error("...")]` on a variant becomes that variant's `Display` message.
  `{0}` refers to a tuple variant's first field; `{field}` / `{source}`
  refer to a struct variant's named fields — same interpolation syntax as
  `format!`, just written inside the attribute instead of a `write!` call.
- `#[source]` on a field marks it as the *underlying cause*, which wires up
  `Error::source()` (the method you didn't need to override manually in
  lesson 1, because `ConfigError` didn't chain to another error in a way
  that needed exposing).
- Add `#[from]` on a field of that type and thiserror *also* generates the
  `From` impl you wrote by hand for the `max_retries` case — one attribute
  instead of a whole `impl From<...> for ...` block.

Same behavior as lesson 1, dramatically less code to maintain. `thiserror`
doesn't change what a Rust error *is* — it's still `Display` + `Error`, the
same two traits — it just writes the boilerplate for you.

## `anyhow`: for when callers *don't* need to match on the error kind

`thiserror` is for **library code**: code whose caller might reasonably
want to say `match err { ConfigError::MissingField(f) => ..., ... }` and
handle each failure kind differently. But plenty of code — especially at
the very top of a program — has no such caller. `main()`, a CLI's top-level
logic, an HTTP handler that just wants to return a 500 and log the details:
these places don't care *which* of forty possible error types went wrong,
they care that *something* did, and want to propagate it up to one place
that prints/logs it and stops.

That's `anyhow::Result<T>` (shorthand for `Result<T, anyhow::Error>`).
`anyhow::Error` can wrap **any** type implementing `std::error::Error` —
which is exactly why lesson 1 and this lesson both bothered implementing
that trait properly instead of just returning `String`. `?` inside a
function returning `anyhow::Result<T>` will happily convert *any* error
type into `anyhow::Error` automatically, no per-type `From` impl needed.

**Rule of thumb professional Rust codebases follow: `thiserror` in your
`lib.rs`, `anyhow` in your `main.rs`.** Libraries expose precise, matchable
error enums so callers *can* handle specific failures if they want to.
Applications (the outermost layer that actually runs, with no further
caller of its own) collapse everything into `anyhow::Error` because there's
no one left to `match` on it.

## `anyhow::Context` — breadcrumbs as an error propagates

```rust
use anyhow::Context;

pub fn load_and_parse(input: &str) -> anyhow::Result<Config> {
    parse_config(input).context("failed to load application config")
}
```

`.context("...")` wraps an error with an extra human-readable message
*without* discarding the original — printing an `anyhow::Error` shows the
context message and, in `{:#}`/`{:?}` form, the full chain underneath it.
`.with_context(|| format!("..."))` is the lazy version, useful when
building the message costs something (e.g. formatting a value) that you'd
rather not pay unless the error path is actually hit. Once real backend
code (Phase 3+) has call stacks five or six functions deep, `.context(...)`
at each layer is what turns "invalid digit found in string" into "failed to
load application config: invalid number for field 'max_retries': invalid
digit found in string" — the difference between a stack trace and a story.

## Your task

Rewrite `ConfigError` using `#[derive(thiserror::Error)]`, keep
`parse_config` behaviorally identical to lesson 1, and implement
`load_and_parse` using `anyhow::Context`.

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`.
