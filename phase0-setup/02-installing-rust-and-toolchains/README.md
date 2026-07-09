# 02 — Installing Rust and toolchains

This lesson is the *why* behind the install; the exact commands live in
[`docs/setup-guide.md`](../../docs/setup-guide.md) so they stay in one place
you can re-check later. Do the install now, then come back and finish
reading this page.

## `rustup`: not just an installer

You're installing `rustup`, not "Rust" directly. `rustup` is a toolchain
**manager** — closest Python analogy is `pyenv` (which lets you install and
switch between Python 3.9/3.11/3.13), except `rustup` is the official,
universally-used way to manage Rust versions, not a third-party convenience.

A **toolchain** is the bundle of `rustc` (the compiler), `cargo` (build
tool/package manager), and a few other pieces, all versioned together.

## Channels: stable, beta, nightly

Rust ships three channels:
- **stable** — a new stable release every 6 weeks. This is what you use for
  everything in this repo, and what you'd use at almost any job.
- **beta** — the next stable release, in testing.
- **nightly** — built daily from the latest code, includes experimental
  features gated behind `#![feature(...)]` flags. Some tools (certain clippy
  lints, some macros libraries) recommend nightly for their best experience,
  but you won't need it for this curriculum.

This repo's `rust-toolchain.toml` (at the repo root) pins `channel = "stable"`
— when you run any `cargo`/`rustc` command *inside this repo*, `rustup`
automatically uses stable, regardless of what your global default is
elsewhere on your machine. This is the same idea as a `.python-version` file
for `pyenv`, or a `Dockerfile`'s base image tag — pin the version, don't rely
on "whatever's installed."

## Editions

You'll see `edition = "2021"` in every `Cargo.toml` in this repo. A Rust
**edition** is a way to introduce small, deliberately-opt-in breaking changes
to the language every few years (2015, 2018, 2021, 2024 editions so far)
without breaking every existing crate on crates.io — old editions keep
compiling forever, `cargo` just needs to know which one a given crate
targets. This is different from Python 2 vs. 3 (a genuine, painful breaking
migration) — editions are designed specifically so upgrading is close to
painless, and different crates in the same build can even use different
editions simultaneously.

## What "installed" should look like when you're done

After following `docs/setup-guide.md`, both of these should print a version
number (not "command not found"):

```sh
rustc --version
cargo --version
```

And your editor should show inline squiggles/type-hints from
**rust-analyzer** when you open a `.rs` file — if it doesn't, re-check the
editor-setup section of the setup guide.

## Checkpoint

Once installed and confirmed, answer `CHECKPOINT.md`, then move to
lesson 03.
