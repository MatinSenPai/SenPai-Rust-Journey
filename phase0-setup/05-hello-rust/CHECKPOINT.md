# Checkpoint — Hello, Rust

1. What's the one function every Rust *binary* crate must define? What
   happens if this lesson were a *library* crate instead (like `solution/`'s
   internals) — would it still need one?
2. `println!` has a `!`. What does that tell you about it, and what can it
   do that an ordinary function couldn't?
3. What error do you get if you deliberately break the format string in
   `main.rs` (e.g. change `{}` to `{}{}`with only one argument)? At what
   stage does that error show up — compiling or running?
4. `main.rs` calls `p0_05_hello_rust::describe_journey`. Where does the name
   `p0_05_hello_rust` come from, given the crate's `Cargo.toml` says
   `name = "p0-05-hello-rust"` (with hyphens)?
