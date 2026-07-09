//! A Telegram bot as a ChatOps admin client for `taskforge-api` — reuses
//! the `teloxide` skills from `side-quests/sq-02-telegram-quiz-bot` at
//! production scale. Split the same way that side-quest is: `format.rs`
//! holds pure, fully-unit-tested formatting logic with zero I/O; `client.rs`
//! is a thin `reqwest`-based HTTP client; `main.rs` (not part of the
//! library — see `src/main.rs`) is the actual `teloxide` wiring, which
//! needs a real bot token and network access to run and so isn't exercised
//! by `cargo test`.

pub mod client;
pub mod format;

pub use client::ApiClient;
