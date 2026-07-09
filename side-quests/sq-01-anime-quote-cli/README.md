# Side-quest 1 — Anime Quote CLI

Your first real, standalone program — everything from Phase 1 (structs,
`Vec`, `Option`, string slices, pattern matching) put to work on something
you'll actually enjoy running. No new concepts here; this is deliberately a
consolidation exercise, not a new lesson.

## What it does

A command-line tool over a small built-in collection of anime quotes:

```sh
cargo run -p sq-01-anime-quote-cli                      # prints one random quote
cargo run -p sq-01-anime-quote-cli -- list               # lists every quote
cargo run -p sq-01-anime-quote-cli -- anime "One Piece"   # quotes from that anime
cargo run -p sq-01-anime-quote-cli -- character "Luffy"   # quotes from that character
```

`src/main.rs` (the CLI wiring — argument parsing, printing) is already
written for you. Your job is the library logic in `src/lib.rs`: the actual
searching, filtering, and formatting `main.rs` calls into. This mirrors a
real pattern you'll use constantly: keep your core logic in a testable
library, keep the command-line/HTTP/whatever-front-end layer thin.

## Your task

Implement the four `todo!()` functions in `src/lib.rs`. `all_quotes()` (the
dataset itself) is already provided — feel free to add your own favorite
quotes to it once everything else passes, it won't break any test that
checks specific quotes by anime/character you didn't touch.

## Checkpoint

`cargo test -p sq-01-anime-quote-cli`, then actually run all four CLI
commands above and read the output. Then `CHECKPOINT.md`, then
`solution/SOLUTION.md`.
