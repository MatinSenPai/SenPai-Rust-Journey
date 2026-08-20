//! Reference solution for 04 — Hello, Rust.

/// A one-line description of the journey so far.
pub fn describe_journey(language: &str, day: u32) -> String {
    format!("Day {day} of learning {language}: still compiling.")
}

/// `line` in upper case with a single `!` appended.
pub fn shout(line: &str) -> String {
    format!("{}!", line.to_uppercase())
}

/// A text progress bar: `done` filled cells, the rest empty, then the count.
pub fn progress_bar(done: usize, total: usize) -> String {
    format!(
        "[{}{}] {done}/{total}",
        "#".repeat(done),
        ".".repeat(total - done)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_the_journey() {
        assert_eq!(
            describe_journey("Rust", 1),
            "Day 1 of learning Rust: still compiling."
        );
        assert_eq!(
            describe_journey("Rust", 42),
            "Day 42 of learning Rust: still compiling."
        );
    }

    #[test]
    fn shouts() {
        assert_eq!(shout("here we go"), "HERE WE GO!");
        assert_eq!(shout(""), "!");
    }

    #[test]
    fn draws_a_progress_bar() {
        assert_eq!(progress_bar(3, 10), "[###.......] 3/10");
        assert_eq!(progress_bar(0, 4), "[....] 0/4");
        assert_eq!(progress_bar(4, 4), "[####] 4/4");
    }
}
