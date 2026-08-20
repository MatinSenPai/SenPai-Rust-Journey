//! Exercises for 04 — Cargo basics.
//!
//! The point of this lesson is the tool, not the language: the code here is
//! deliberately small so your attention stays on `Cargo.toml`, dependencies
//! and the command line.

/// A greeting for `name`, repeated `times` times, one per line, with no
/// trailing newline.
///
/// # Examples
///
/// `format_greeting("Matin", 1)` returns `"Hello, Matin!"`
/// `format_greeting("Matin", 2)` returns `"Hello, Matin!\nHello, Matin!"`
/// `format_greeting("Matin", 0)` returns `""`
pub fn format_greeting(name: &str, times: u32) -> String {
    todo!("build `times` copies of the greeting and put a newline between them")
}

/// One line of encouragement, chosen at random.
///
/// The point of this one is the *dependency*, not the logic: it uses the
/// `rand` crate declared in this lesson's `Cargo.toml`. Any of the three
/// lines below is a correct answer.
///
/// # Examples
///
/// `pick_encouragement()` returns one of `"Keep going."`,
/// `"One todo!() at a time."` or `"You've got this."`
pub fn pick_encouragement() -> String {
    let lines = ["Keep going.", "One todo!() at a time.", "You've got this."];
    todo!("use `rand` to choose one of `lines` and hand it back as an owned String")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greets_once() {
        assert_eq!(format_greeting("Matin", 1), "Hello, Matin!");
    }

    #[test]
    fn greets_multiple_times() {
        assert_eq!(
            format_greeting("Matin", 3),
            "Hello, Matin!\nHello, Matin!\nHello, Matin!"
        );
    }

    #[test]
    fn zero_times_is_an_empty_string() {
        assert_eq!(format_greeting("Matin", 0), "");
    }

    #[test]
    fn encouragement_is_one_of_the_three() {
        let line = pick_encouragement();
        assert!(
            line == "Keep going." || line == "One todo!() at a time." || line == "You've got this.",
            "got an unexpected line: {line}"
        );
    }
}
