# 05 — Hello, Rust

Your first real program. Small on purpose — the goal is to see `cargo new`
→ edit → `cargo run` end-to-end once before Phase 1 starts covering syntax
in depth.

## `fn main`

Every Rust **binary** crate needs exactly one `fn main() { ... }` — this is
the entry point, same role as `if __name__ == "__main__":` in Python, except
mandatory and unambiguous: there's no equivalent of "this file might be
imported instead of run."

## `println!` is a macro, not a function

Notice the `!`. `println!("Hello, {name}!")` is a **macro** — it runs at
compile time and expands into the actual printing code, which is how it can
do something a normal function can't: check your format string against your
arguments *at compile time* and refuse to compile if they don't match (try
`println!("{name}")` without a `name` in scope — the compiler error appears
before you ever run anything). You'll write your own simple macros much
later (Phase 2 stretch material); for now just recognize `!` as "this is a
macro" whenever you see it (`println!`, `vec!`, `format!`, `todo!` — you've
already used that last one in every lesson so far).

## Your task

`src/lib.rs` has one function to implement: `describe_journey`. `src/main.rs`
already calls it — don't edit `main.rs`, just make the library function
correct.

```sh
cargo test -p p0-05-hello-rust
cargo run -p p0-05-hello-rust
```

## Checkpoint

Once both commands work, do `CHECKPOINT.md`, then compare with
`solution/SOLUTION.md`.
