# 08.3 — Cargo features

Open the `Cargo.toml` of almost any real crate and you'll find a
`[features]` table: `tokio` has ~30 features, `serde` has `derive`, and
this repo's own workspace manifest enables feature lists on half its
dependencies. Features are **compile-time, opt-in switches** for a crate:
extra functions, extra trait impls, extra dependencies that only exist in
the binary if the consumer asked for them.

The closest Python has is `pip install requests[socks]` — an "extra" that
pulls optional dependencies. But a Python extra only changes what gets
*installed*; the code paths are still selected at runtime (`try: import
socks`). A Cargo feature changes what gets **compiled**: code behind a
disabled feature isn't slow-pathed or stubbed, it *does not exist* in the
output. There's no Django-settings equivalent either — `INSTALLED_APPS`
toggles apps at process startup, features toggle them before the compiler
runs.

## Declaring features and optional dependencies

```toml
[features]
default = []
json-export = ["dep:serde", "dep:serde_json"]

[dependencies]
serde = { workspace = true, optional = true }
serde_json = { workspace = true, optional = true }
```

Three pieces. `optional = true` means "don't compile this dependency
unless something turns it on." `dep:serde` in a feature's list means
"enabling `json-export` turns the `serde` dependency on." And `default`
is the feature set consumers get when they say nothing — ours is empty,
so the base crate stays dependency-free. (A feature can also enable
*other features*, of this crate or of a dependency: `derive =
["serde/derive"]`-style. Ours only needs `dep:`.)

## Gating code: `#[cfg]` and `#[cfg_attr]`

```rust
#[cfg_attr(feature = "json-export", derive(serde::Serialize))]
pub struct Report { /* ... */ }

#[cfg(feature = "json-export")]
pub fn to_json(report: &Report) -> String { /* ... */ }
```

`#[cfg(...)]` removes the item entirely when the feature is off — the
function, its body, its use of `serde_json`, all gone before type
checking. `#[cfg_attr(cond, attr)]` is the conditional version of *an
attribute*: "apply `derive(serde::Serialize)` only when the feature is
on." That's what lets `Report` itself exist in both worlds — same struct,
with or without the serde impl. Tests gate the same way: this lesson's
JSON tests sit in a `#[cfg(all(test, feature = "json-export"))]` module,
so the default `cargo test` doesn't even compile them.

## Features must be additive (unification)

If crate A depends on `you` with `json-export` and crate B depends on
`you` without it, Cargo compiles `you` **once**, with the **union** of
requested features — A and B share the build. This is feature
*unification*, and it forces a design rule: enabling a feature may only
**add** API surface, never remove or change behavior. A feature like
`no-std-panic-handler` that *removes* something breaks whichever
dependent didn't ask for it. Corollary: your crate must build, pass
tests, and be clippy-clean in *every* feature combination you claim to
support — which is exactly what this lesson's verification asks of you.

## The commands you'll actually type

```bash
cargo test -p p2-08-03-cargo-features                          # feature off
cargo test -p p2-08-03-cargo-features --features json-export   # feature on
cargo clippy -p p2-08-03-cargo-features --all-targets -- -D warnings
cargo clippy -p p2-08-03-cargo-features --all-targets --features json-export -- -D warnings
```

(`--all-features` and `--no-default-features` exist too; with our empty
default they'd be equivalent to the two lines above.)

## Your task

A tiny stats crate. In `src/lib.rs`:

- `build_report` — always available: label, count, mean, min, max over a
  slice of samples; `None` when the slice is empty. (`Option` + `?` on
  `Iterator::min`/`max` makes this pleasantly short.)
- `to_json` — exists only under `json-export`: serialize a `Report` with
  `serde_json`. The derive is already wired up via `cfg_attr`; look at it
  before you write anything.

Run the test suite **both ways** — the whole point of the lesson is that
both worlds must be green.

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`.
