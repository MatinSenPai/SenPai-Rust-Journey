# Checkpoint — Tooling

1. Pick two of the clippy warnings you fixed. For each, explain in your own
   words *why* clippy considers the original version worse — not just what
   the fix was.
2. `is_empty_name` originally took `&String`. Why does clippy prefer `&str`
   there? (This is a preview of Phase 1's `String` vs `&str` lesson — you
   don't need the full answer yet, just your best guess from the warning
   text.)
3. What's the difference between what `cargo clippy --fix` does automatically
   and what you had to decide yourself?
4. Run `cargo fmt -- --check` on a clean version of this file (after fixing
   everything) — does it report anything? What does that tell you about how
   `rustfmt` and `clippy` divide responsibilities (style/whitespace vs.
   idiomatic-code warnings)?
