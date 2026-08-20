# Setup Guide

Do this once before starting `phase0-setup/04-cargo-basics`. Phase 0's first
two lessons explain the *why*; this page is the *how*, kept separate so it
stays easy to re-check later.

## 1. Install Rust via rustup

Rust is installed and managed through `rustup`, a version manager (like
`pyenv`/`nvm`, but official and the standard way everyone installs Rust):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Restart your shell, then confirm:

```sh
rustc --version
cargo --version
```

This repo pins its toolchain in `rust-toolchain.toml` (channel = stable) —
`rustup` will automatically install and use that version whenever you run
`cargo`/`rustc` inside this repo, even if your global default differs.

## 2. Editor setup

Install the **rust-analyzer** extension for your editor (VS Code: search
"rust-analyzer" in the marketplace). It gives you inline type hints, jump-to-
definition, and — most importantly for learning — inline compiler errors as
you type, instead of only after running `cargo build`.

Also install these two components (already listed in `rust-toolchain.toml`,
but worth knowing what they are):
- **`rustfmt`** — the standard formatter. Run `cargo fmt` in any crate.
- **`clippy`** — a linter that catches non-idiomatic patterns the compiler
  itself won't complain about. Run `cargo clippy` in any crate. Treat every
  clippy warning as a mini-lesson, not noise to silence.

## 3. First commands to run

From the repo root:

```sh
cargo build --workspace   # compiles every lesson crate that exists so far
cargo test -p p0-04-cargo-basics   # test a single lesson by its package name
```

You do not need PostgreSQL, Redis, Docker, or any other backend
infrastructure until Phase 3 — each of those lessons' `README.md` tells you
exactly what to install right when you need it, not before.

## 4. If something won't compile and you don't know why

This will happen constantly and is normal — the compiler is unusually verbose
on purpose. Read the full error message top to bottom (Rust error messages
usually tell you exactly what's wrong and often suggest the fix), then check
`docs/glossary.md` for any unfamiliar term before asking anyone else.
