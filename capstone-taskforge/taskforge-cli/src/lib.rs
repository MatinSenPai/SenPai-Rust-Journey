//! A `clap`-based CLI client for `taskforge-api` — demonstrates "thin
//! client, thick core" directly: this crate holds no domain logic at all
//! beyond formatting terminal output; everything real (the job state
//! machine, storage, worker behavior) lives in `taskforge-core` and its
//! siblings. `taskforge-admin-bot` is a second, independent thin client
//! over the exact same API surface — proof the architecture supports
//! multiple front-ends without duplicating logic anywhere but the HTTP
//! plumbing itself.

pub mod client;
pub mod format;

pub use client::ApiClient;
