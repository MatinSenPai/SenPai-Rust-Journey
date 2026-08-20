/// Builds a one-line description of the journey, e.g. for `language = "Rust"`
/// and `day = 1`: `"Day 1 of learning Rust: still compiling."`
pub fn describe_journey(language: &str, day: u32) -> String {
    format!("Day {day} of learning {language}: still compiling.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_day_one() {
        assert_eq!(
            describe_journey("Rust", 1),
            "Day 1 of learning Rust: still compiling."
        );
    }

    #[test]
    fn describes_other_days() {
        assert_eq!(
            describe_journey("Rust", 42),
            "Day 42 of learning Rust: still compiling."
        );
    }
}
