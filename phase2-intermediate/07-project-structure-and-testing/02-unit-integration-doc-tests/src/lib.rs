//! Exercises for 2.7.2 — unit, integration, and doc tests.
//!
//! Every doc comment below is a full specification AND a real, runnable
//! test: `cargo test` compiles and runs every fenced example inside a `///`
//! comment, exactly like any other test in this crate.

// NOT `pub` — reachable only from code inside this crate, including this
// file's own unit tests below. `tests/public_api.rs` cannot name it at all;
// see examples/02-private-item-is-unreachable.rs and "Errors you will meet".
fn round_to_one_decimal(x: f64) -> f64 {
    todo!("round x to one decimal place")
}

/// Converts `c` from Celsius to Fahrenheit, rounded to one decimal place.
///
/// # Examples
///
/// ```
/// # use p2_07_02_unit_integration_doc_tests::celsius_to_fahrenheit;
/// assert_eq!(celsius_to_fahrenheit(0.0), 32.0);
/// assert_eq!(celsius_to_fahrenheit(100.0), 212.0);
/// ```
pub fn celsius_to_fahrenheit(c: f64) -> f64 {
    todo!("convert c from Celsius to Fahrenheit, then round with round_to_one_decimal")
}

/// The average of `readings`, rounded to one decimal place.
///
/// # Examples
///
/// ```
/// # use p2_07_02_unit_integration_doc_tests::average_celsius;
/// assert_eq!(average_celsius(&[10.0, 20.0, 30.0]), 20.0);
/// assert_eq!(average_celsius(&[-5.0, 5.0]), 0.0);
/// ```
///
/// # Panics
///
/// Panics with the message `readings must not be empty` if `readings` is
/// empty.
///
/// ```should_panic
/// # use p2_07_02_unit_integration_doc_tests::average_celsius;
/// average_celsius(&[]);
/// ```
pub fn average_celsius(readings: &[f64]) -> f64 {
    todo!("panic with \"readings must not be empty\" if readings is empty; otherwise return its mean, rounded with round_to_one_decimal")
}

/// Parses `input` as a Celsius reading.
///
/// # Examples
///
/// ```
/// # use p2_07_02_unit_integration_doc_tests::parse_celsius;
/// assert_eq!(parse_celsius("36.6"), Ok(36.6));
/// ```
///
/// # Errors
///
/// Returns `Err` with the message `not a valid number: "<input>"` (`input`
/// formatted with `{input:?}`) when `input` cannot be parsed as a
/// floating-point number.
///
/// ```
/// # use p2_07_02_unit_integration_doc_tests::parse_celsius;
/// assert_eq!(
///     parse_celsius("hot"),
///     Err("not a valid number: \"hot\"".to_string())
/// );
/// ```
pub fn parse_celsius(input: &str) -> Result<f64, String> {
    todo!("parse input as a floating-point number; on failure return the exact Err shown above")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Reaches the PRIVATE function directly — only possible because this
    // test compiles as part of the same crate.
    #[test]
    fn rounds_correctly() {
        assert_eq!(round_to_one_decimal(1.24), 1.2);
        assert_eq!(round_to_one_decimal(1.26), 1.3);
    }

    #[test]
    fn converts_freezing_and_boiling_points() {
        assert_eq!(celsius_to_fahrenheit(0.0), 32.0);
        assert_eq!(celsius_to_fahrenheit(100.0), 212.0);
    }

    #[test]
    fn averages_multiple_readings() {
        assert_eq!(average_celsius(&[10.0, 20.0, 30.0]), 20.0);
        assert_eq!(average_celsius(&[-5.0, 5.0]), 0.0);
    }

    #[test]
    #[should_panic(expected = "readings must not be empty")]
    fn average_of_nothing_panics() {
        average_celsius(&[]);
    }

    #[test]
    fn parses_valid_and_invalid_input() {
        assert_eq!(parse_celsius("36.6"), Ok(36.6));
        assert_eq!(
            parse_celsius("hot"),
            Err("not a valid number: \"hot\"".to_string())
        );
    }
}
