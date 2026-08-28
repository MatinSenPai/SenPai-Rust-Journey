//! `.context("...")` wraps a lower-level error with a human-readable
//! message *without* discarding the original — `{}` shows only the new
//! message, `{:?}` walks the whole chain down to the real cause. Compare
//! this `{:?}` output to `04-anyhow-basics.rs`'s: one layer of `.context()`
//! is the difference between a bare message and a "Caused by:" story.

use anyhow::Context;

fn parse_score(raw: &str) -> Result<u8, std::num::ParseIntError> {
    raw.trim().parse()
}

fn load_score(raw: &str) -> anyhow::Result<u8> {
    parse_score(raw).context("failed to load score from config")
}

fn main() {
    match load_score("oops") {
        Ok(score) => println!("score: {score}"),
        Err(err) => {
            println!("Display : {err}");
            println!("Debug   : {err:?}");
        }
    }
}
