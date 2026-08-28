//! Exercises for 04 — Hello, Rust.
//!
//! Each function is fully specified in its doc comment: the exact output for
//! given inputs is written down, so you never need to open the tests to know
//! what to produce.

/// A one-line description of the journey so far.
///
/// # Examples
///
/// `describe_journey("Rust", 1)` returns `"Day 1 of learning Rust: still compiling."`
/// `describe_journey("Rust", 42)` returns `"Day 42 of learning Rust: still compiling."`
pub fn describe_journey(language: &str, day: u32) -> String {
    todo!("build that exact sentence from the two arguments")
}

/// `line` in upper case with a single `!` appended.
///
/// # Examples
///
/// `shout("here we go")` returns `"HERE WE GO!"`
/// `shout("")` returns `"!"`
pub fn shout(line: &str) -> String {
    todo!("upper-case the line, then put one exclamation mark on the end")
}

/// A text progress bar: `done` filled cells, the rest empty, then the count.
///
/// `done` is never greater than `total`.
///
/// # Examples
///
/// `progress_bar(3, 10)` returns `"[###.......] 3/10"`
/// `progress_bar(0, 4)` returns `"[....] 0/4"`
/// `progress_bar(4, 4)` returns `"[####] 4/4"`
pub fn progress_bar(done: usize, total: usize) -> String {
    todo!("one `#` per finished item, one `.` per remaining one, then the two numbers")
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
