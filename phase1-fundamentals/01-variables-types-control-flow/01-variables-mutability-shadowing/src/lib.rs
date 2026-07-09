//! Fill in both `todo!()`s below.

/// Starts a counter at 0 and increments it `times` times using a single
/// mutable variable (no shadowing here — this one's about `mut`).
pub fn increment_n_times(times: u32) -> u32 {
    todo!("declare `let mut count = 0`, loop `times` times incrementing it, return it")
}

/// Takes a numeric string like `"21"`, and using shadowing (re-`let`-ing
/// the same name at least twice, changing type along the way is fine),
/// parses it to an `i32` and returns it doubled.
///
/// # Examples
/// `parse_and_double("21")` returns `42`.
pub fn parse_and_double(input: &str) -> i32 {
    todo!("shadow `input` through a parse step, then a doubling step")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increments_correctly() {
        assert_eq!(increment_n_times(0), 0);
        assert_eq!(increment_n_times(5), 5);
        assert_eq!(increment_n_times(100), 100);
    }

    #[test]
    fn parses_and_doubles() {
        assert_eq!(parse_and_double("21"), 42);
        assert_eq!(parse_and_double("0"), 0);
        assert_eq!(parse_and_double("-3"), -6);
    }
}
