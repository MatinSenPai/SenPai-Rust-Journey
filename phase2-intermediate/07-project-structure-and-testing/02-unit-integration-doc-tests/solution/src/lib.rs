//! Reference solution for 2.7.2 — unit, integration, and doc tests.

fn round_to_one_decimal(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

/// Converts `c` from Celsius to Fahrenheit, rounded to one decimal place.
///
/// # Examples
///
/// ```
/// # use p2_07_02_unit_integration_doc_tests_solution::celsius_to_fahrenheit;
/// assert_eq!(celsius_to_fahrenheit(0.0), 32.0);
/// assert_eq!(celsius_to_fahrenheit(100.0), 212.0);
/// ```
pub fn celsius_to_fahrenheit(c: f64) -> f64 {
    round_to_one_decimal(c * 9.0 / 5.0 + 32.0)
}

/// The average of `readings`, rounded to one decimal place.
///
/// # Examples
///
/// ```
/// # use p2_07_02_unit_integration_doc_tests_solution::average_celsius;
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
/// # use p2_07_02_unit_integration_doc_tests_solution::average_celsius;
/// average_celsius(&[]);
/// ```
pub fn average_celsius(readings: &[f64]) -> f64 {
    assert!(!readings.is_empty(), "readings must not be empty");
    let sum: f64 = readings.iter().sum();
    round_to_one_decimal(sum / readings.len() as f64)
}

/// Parses `input` as a Celsius reading.
///
/// # Examples
///
/// ```
/// # use p2_07_02_unit_integration_doc_tests_solution::parse_celsius;
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
/// # use p2_07_02_unit_integration_doc_tests_solution::parse_celsius;
/// assert_eq!(
///     parse_celsius("hot"),
///     Err("not a valid number: \"hot\"".to_string())
/// );
/// ```
pub fn parse_celsius(input: &str) -> Result<f64, String> {
    input
        .parse::<f64>()
        .map_err(|_| format!("not a valid number: {input:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

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
