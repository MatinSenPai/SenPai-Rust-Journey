# Checkpoint — Cargo basics

1. What's the difference between `cargo check` and `cargo build`? When would
   you reach for each?
2. Where did the `rand` crate's actual code get downloaded to? (Hint: look
   for a new folder cargo created outside this lesson folder, and check
   `Cargo.lock` at the workspace root after building.)
3. What does `edition.workspace = true` in this lesson's `Cargo.toml` mean,
   and where is the actual value it resolves to defined?
4. `cargo test` ran three tests from `src/lib.rs` but you never called
   `cargo run` in `mod tests`. How does `cargo test` know to find and run
   them?
5. Try `cargo run -- YourName 3` then `cargo run -- YourName abc` — what
   happens in the second case, and why, given how `times` is parsed in
   `main.rs`?
