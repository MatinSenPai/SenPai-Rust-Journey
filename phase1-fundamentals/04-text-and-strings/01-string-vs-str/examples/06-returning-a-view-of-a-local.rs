//! DELIBERATELY BROKEN — expected: E0515
//!
//!     cargo run -p p1-04-01-string-vs-str --example 06-returning-a-view-of-a-local --features broken
//!
//! The reason the signature rule says *return* `String`.

/// Says it hands back a view. Builds an owner and tries to hand back a view
/// of that instead.
fn shout(text: &str) -> &str {
    let loud = text.to_uppercase();
    &loud
}

fn main() {
    println!("{}", shout("senpai"));
}
