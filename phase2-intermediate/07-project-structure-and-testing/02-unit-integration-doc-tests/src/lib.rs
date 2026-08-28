/// Converts Celsius to Fahrenheit, rounded to one decimal place.
///
/// # Examples
///
/// ```
/// # use p2_06_02_unit_integration_doc_tests::celsius_to_fahrenheit;
/// assert_eq!(celsius_to_fahrenheit(0.0), 32.0);
/// assert_eq!(celsius_to_fahrenheit(100.0), 212.0);
/// ```
pub fn celsius_to_fahrenheit(c: f64) -> f64 {
    todo!("round_to_one_decimal(c * 9.0 / 5.0 + 32.0)")
}

// NOT `pub` — only reachable from code inside this crate (including this
// file's own unit tests below). tests/conversion_test.rs cannot call this
// function at all; try it and read the compiler error.
fn round_to_one_decimal(x: f64) -> f64 {
    todo!("(x * 10.0).round() / 10.0")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_correctly() {
        // Reaches the PRIVATE function directly — only possible because
        // this test compiles as part of the same crate.
        assert_eq!(round_to_one_decimal(1.24), 1.2);
        assert_eq!(round_to_one_decimal(1.26), 1.3);
    }

    #[test]
    fn converts_freezing_and_boiling_points() {
        assert_eq!(celsius_to_fahrenheit(0.0), 32.0);
        assert_eq!(celsius_to_fahrenheit(100.0), 212.0);
    }
}
