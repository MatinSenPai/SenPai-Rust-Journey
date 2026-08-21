# 04 — Error handling and lifetimes

Phase 1 taught you to reach for `Option` and `Result` instead of null and exceptions. This module takes those tools from "I can read them" to "I can ship them": writing your own error types so that callers can match on what went wrong, using `thiserror` and `anyhow` — which you will find in very nearly every real Rust codebase, because they remove most of the boilerplate — and then the basics of lifetimes, the rules that guarantee a reference never points at data that has been freed.

1. [Custom error types](01-custom-error-types/README.md)
2. [Using `thiserror` and `anyhow`](02-thiserror-and-anyhow/README.md)
3. [Lifetime basics and elision](03-lifetime-basics-and-elision/README.md)
