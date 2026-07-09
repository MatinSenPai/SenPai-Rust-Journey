pub fn increment_n_times(times: u32) -> u32 {
    let mut count = 0;
    for _ in 0..times {
        count += 1;
    }
    count
}

// This lesson is specifically about shadowing, so the final shadow is kept
// as its own statement (`let input = input * 2;`) rather than collapsed
// into a tail expression — clippy's `let_and_return` would normally suggest
// exactly that collapse, and it's right that it's *shorter*, but it would
// also hide the third shadow this exercise is asking you to practice.
#[allow(clippy::let_and_return)]
pub fn parse_and_double(input: &str) -> i32 {
    let input: i32 = input.parse().expect("input should be a valid integer");
    let input = input * 2;
    input
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
