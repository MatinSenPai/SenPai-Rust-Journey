# 04 — Tooling: clippy, fmt, rust-analyzer

Unlike every other lesson so far, `src/lib.rs` in this one **already
compiles and already passes its tests**. Your job isn't to make it work —
it's to make it *idiomatic*, using two tools that are part of daily Rust
work at basically every job:

- **`cargo clippy`** — a linter with hundreds of checks for things the
  compiler considers perfectly valid but experienced Rust developers
  consider bad style, inefficient, or a likely bug. Closest Python analogy:
  `pylint`/`ruff`, except clippy is maintained by the Rust project itself and
  is a much stronger convention in the ecosystem — most real Rust projects
  run it in CI (this repo's `.github/workflows/ci.yml` does).
- **`cargo fmt`** — auto-formatting, closest analogy `black`. There's
  effectively one standard Rust style; you don't debate tabs vs. spaces on a
  real Rust team, you just run `cargo fmt`.
- **rust-analyzer** — the "language server" your editor talks to for
  inline type hints, autocomplete, jump-to-definition, and (usefully for
  this lesson) inline squiggly warnings *as you type*, often before you'd
  even run clippy manually.

## Your task

1. Run `cargo test` — confirm everything passes already. This lesson is
   purely about style, not correctness.
2. Run `cargo clippy`. Read every warning's message — clippy explains *why*
   it's flagging each thing, and often suggests the exact fix. Don't just
   apply the suggestion blindly; make sure you understand the reasoning
   (that's what `CHECKPOINT.md` will check).
3. Fix every function in `src/lib.rs` until `cargo clippy` reports zero
   warnings, **without changing what any function returns** — `cargo test`
   must stay green the whole time.
4. Deliberately mangle the formatting of one function (extra spaces, wrong
   indentation, whatever) and run `cargo fmt` to watch it get fixed
   automatically. Then run `cargo fmt -- --check` (what CI runs) to see how
   it reports "needs formatting" without changing anything.

## Checkpoint

Once `cargo clippy` is clean and `cargo fmt -- --check` passes, do
`CHECKPOINT.md`, then compare your reasoning with `solution/SOLUTION.md`
(there's more than one acceptable way to silence some of these).
