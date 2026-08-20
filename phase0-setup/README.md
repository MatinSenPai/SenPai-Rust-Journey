# Phase 0 — Setup and orientation

Before you meet a single borrow-checker error, get oriented: what a compiled language actually is, a working toolchain, your first program, and the habit of reading what the compiler tells you.

Seven lessons, mostly reading. Don't rush them anyway — the mental model from lesson 1 pays for itself across the whole journey, and lesson 5 is the one that decides whether your first month with Rust is frustrating or interesting.

| # | Lesson | What it's for |
|---|---|---|
| 1 | [What is a compiled language?](01-what-is-a-compiled-language/README.md) | why Rust catches things Python can't, and what you pay for it |
| 2 | [Installing Rust and toolchains](02-installing-rust-and-toolchains/README.md) | `rustup`, channels, editions, and the Windows linker |
| 3 | [Hello, Rust](03-hello-rust/README.md) | your first program: `fn`, `println!`, and how a function returns |
| 4 | [Cargo basics](04-cargo-basics/README.md) | the tool you'll use every day, and what `Cargo.toml` says |
| 5 | [Reading compiler errors](05-reading-compiler-errors/README.md) | the highest-leverage skill in this phase |
| 6 | [Tooling: clippy, fmt, rust-analyzer](06-tooling-clippy-fmt-rust-analyzer/README.md) | past "does it compile" to "is this good Rust" |
| 7 | [Git and repo workflow](07-git-and-repo-workflow/README.md) | how not to get lost over the coming months |

The order is deliberate: you run a program (3) before you study the tool that runs it (4), and you write code (3) before you learn to read the errors it produces (5). Neither works the other way round.

When all seven are ticked off in [`PROGRESS.md`](../PROGRESS.md), move on to [Phase 1](../phase1-fundamentals/README.md).
