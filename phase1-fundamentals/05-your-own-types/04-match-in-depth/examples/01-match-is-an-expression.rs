//! `match` is an expression: it produces a value, and every arm produces a
//! value of the same type.
//!
//!     cargo run -p p1-05-04-match-in-depth --example 01-match-is-an-expression

/// One band per score.
///
/// `1..=3` is a range pattern. `7 | 8` is two patterns in one arm. `_` is the
/// catch-all, and for a `u8` it is what covers 11 through 255.
fn band(stars: u8) -> String {
    match stars {
        0 => "unrated".to_string(),
        1..=3 => "weak".to_string(),
        4..=6 => "watchable".to_string(),
        7 | 8 => "good".to_string(),
        9 | 10 => "top shelf".to_string(),
        _ => "not a score".to_string(),
    }
}

fn main() {
    // The whole `match` is one expression, so a `let` can take its value.
    let mood = match 3 {
        0 => "nothing on the shelf",
        1..=5 => "a manageable pile",
        _ => "too many open tabs",
    };
    println!("mood:  {mood}");

    // No `return`, no semicolon after the `match` in `band` — the same rule as
    // the last expression of any function.
    for stars in [0, 2, 5, 8, 10, 200] {
        println!("{stars:>3}  -> {}", band(stars));
    }

    // An arm's body can be a block; the block's last expression is its value.
    let note = match 9_u8 {
        s @ 9..=10 => {
            let shelf = "favourites";
            format!("{shelf} ({s}/10)")
        }
        s => format!("ordinary ({s}/10)"),
    };
    println!("note:  {note}");

    // And because it is an expression, it composes: a `match` inside a
    // `format!`, with no temporary variable in between.
    println!(
        "check: {}",
        match band(7).len() {
            0 => "empty",
            1..=4 => "short",
            _ => "long enough",
        }
    );
}
