# 03 — Cargo basics

`cargo` is Rust's build tool *and* package manager *and* test runner *and*
formatter/linter front-end, all in one CLI. If you've used `pip` + `venv` +
a `Makefile`, cargo replaces all three.

## Anatomy of `Cargo.toml`

Open this lesson's `Cargo.toml`:

```toml
[package]
name = "p0-03-cargo-basics"
version = "0.1.0"
edition.workspace = true

[dependencies]
rand.workspace = true
```

- `[package]` — metadata about *this* crate: its name (what other crates
  would depend on it as), its version (see semver below), which language
  edition it targets.
- `[dependencies]` — other crates this one needs, downloaded from
  [crates.io](https://crates.io) (Rust's equivalent of PyPI) the first time
  you build. `rand.workspace = true` means "use whatever version the repo's
  root `Cargo.toml` pins for `rand` under `[workspace.dependencies]`" —
  this whole repo is one cargo **workspace**, so every lesson that needs
  `rand` (or `tokio`, `serde`, etc.) shares one pinned version instead of
  each picking its own. See `docs/conventions.md` if you want the full
  reasoning.

## Semantic versioning (semver)

`version = "0.1.0"` follows `MAJOR.MINOR.PATCH`. Once a crate hits `1.0.0`,
the promise is: PATCH bumps never break your code, MINOR bumps add features
without breaking anything, MAJOR bumps may break things. Before `1.0.0` (like
this lesson's `0.1.0`), even MINOR bumps can break things — that's the
signal a crate gives you that it's still stabilizing. This is the same
convention `pip` packages *often* follow, but cargo/crates.io enforce and
rely on it much more heavily — it's baked into how dependency resolution
works.

## The commands you'll use constantly

Run these from inside this lesson's folder (`phase0-setup/03-cargo-basics/`):

| Command | What it does |
|---|---|
| `cargo check` | Fastest way to ask "does this compile?" — skips codegen, just type-checks. Use this constantly while writing code. |
| `cargo build` | Actually compiles to a binary, under `target/debug/`. |
| `cargo run` | Builds (if needed) and immediately runs the binary. |
| `cargo test` | Compiles and runs every `#[test]` function. |
| `cargo add <crate>` | Adds a dependency to `Cargo.toml` (this is how `rand` got there originally — try `cargo add anyhow` in a throwaway crate sometime to see it in action). |
| `cargo fmt` | Auto-formats your code to the standard style. |
| `cargo clippy` | Lints for non-idiomatic patterns (next lesson goes deep on this). |

## Your task

Open `src/lib.rs`. Implement `format_greeting`, which builds a greeting
string, repeated `times` times, one per line. `src/main.rs` reads a name from
the command line and calls it — don't worry about understanding every detail
of `main.rs` yet (command-line arg parsing, `&str` vs `String`) — Phase 1
covers all of that from first principles. For this lesson, focus on the
cargo *workflow*: `cargo check` → `cargo test` → `cargo run -- YourName`.

```sh
cargo test
cargo run -- Matin
cargo run -- Matin 3
```

## Next

Once `cargo test` is green, do the recall questions, then compare with
`solution/SOLUTION.md`.
