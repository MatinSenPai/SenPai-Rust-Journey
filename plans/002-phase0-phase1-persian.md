# Plan 002: Translate onboarding and Rust fundamentals for Matin

> Drift check: `git diff --stat 80ec4d1..HEAD -- phase0-setup phase1-fundamentals docs`.

Create `.fa.md` companions without modifying canonical English Markdown. Use a
friendly, exact voice addressed to Matin. Introduce Persian term plus English
term on first use. Rewrite ownership, move, Clone/Copy, Drop, borrowing,
strings, enums, Option and Result with Iranian everyday examples followed by
backend/Python comparisons. Preserve identifiers, commands and compiler errors.

Ownership lessons must explicitly cover where each analogy stops being exact,
show E0382/borrow-checker output, and include accessible animated concept
figures. UTF-8 lessons must distinguish bytes from Unicode scalar values and
must not replace ASCII fixtures when that changes the concept under test.

Verify every Phase 0–1 canonical Markdown file has a Persian companion, all
relative links resolve, all visual specs parse, and all starter crates compile.
