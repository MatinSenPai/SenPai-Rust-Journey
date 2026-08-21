//! Reference solution for 1.6.3 — `Result` and the question mark.
//!
//! Five functions. One builds a `Result` by hand. One leans on `.parse()`.
//! One uses `?` to propagate a failure through two steps. Two collapse a
//! `Result` down to a plain value or an `Option`, on purpose, once the exact
//! reason for failure stops mattering.

/// `a` divided by `b`.
///
/// If `b` is exactly `0.0`, returns `Err("cannot divide by zero".to_string())`.
/// Otherwise returns `Ok(a / b)`.
///
/// # Examples
///
/// `safe_divide(10.0, 2.0)` returns `Ok(5.0)`.
/// `safe_divide(10.0, 0.0)` returns `Err("cannot divide by zero".to_string())`.
pub fn safe_divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("cannot divide by zero".to_string())
    } else {
        Ok(a / b)
    }
}

/// `s`, parsed as an unsigned 32-bit integer.
///
/// On success, `Ok` holds the parsed number. On failure — an empty string,
/// non-digit characters, a negative number, or a number too large for
/// `u32` — `Err` holds the same text that calling `.to_string()` on the
/// parse error itself would produce.
///
/// # Examples
///
/// `parse_quantity("42")` returns `Ok(42)`.
/// `parse_quantity("-5")` and `parse_quantity("nope")` both return
/// `Err("invalid digit found in string".to_string())`.
pub fn parse_quantity(s: &str) -> Result<u32, String> {
    s.parse::<u32>().map_err(|e| e.to_string())
}

/// `a` divided by `b`, and then *that* divided by `c` — using the same rule
/// as `safe_divide` at each step.
///
/// If either division fails, returns that failure immediately, without
/// attempting the remaining division.
///
/// # Examples
///
/// `chained_division(100.0, 2.0, 5.0)` returns `Ok(10.0)`.
/// `chained_division(100.0, 0.0, 5.0)` returns
/// `Err("cannot divide by zero".to_string())` — the first division's error.
/// `chained_division(100.0, 2.0, 0.0)` returns
/// `Err("cannot divide by zero".to_string())` — the second division's error.
pub fn chained_division(a: f64, b: f64, c: f64) -> Result<f64, String> {
    let step1 = safe_divide(a, b)?;
    let step2 = safe_divide(step1, c)?;
    Ok(step2)
}

/// Half of `input`, parsed as an `f64`.
///
/// If parsing fails for any reason, returns `0.0` — the specific parse
/// error is discarded.
///
/// # Examples
///
/// `half_or_zero("10")` returns `5.0`.
/// `half_or_zero("7.5")` returns `3.75`.
/// `half_or_zero("oops")` returns `0.0`.
pub fn half_or_zero(input: &str) -> f64 {
    input.parse::<f64>().map(|value| value / 2.0).unwrap_or(0.0)
}

/// `input`, parsed as an `f64` and then divided by `divisor` using the same
/// rule as `safe_divide`.
///
/// Returns `Some` holding the final value if both steps succeed. Returns
/// `None` if either step fails, discarding whatever error text that step
/// would have produced.
///
/// # Examples
///
/// `parsed_and_divided("10", 2.0)` returns `Some(5.0)`.
/// `parsed_and_divided("oops", 2.0)` returns `None` — the parse failed.
/// `parsed_and_divided("10", 0.0)` returns `None` — the division failed.
pub fn parsed_and_divided(input: &str, divisor: f64) -> Option<f64> {
    input
        .parse::<f64>()
        .ok()
        .and_then(|value| safe_divide(value, divisor).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divides_normally() {
        assert_eq!(safe_divide(10.0, 2.0), Ok(5.0));
    }

    #[test]
    fn rejects_division_by_zero() {
        assert_eq!(
            safe_divide(10.0, 0.0),
            Err("cannot divide by zero".to_string())
        );
    }

    #[test]
    fn parses_valid_quantities() {
        assert_eq!(parse_quantity("42"), Ok(42));
    }

    #[test]
    fn reports_the_parse_error_as_the_err() {
        assert_eq!(
            parse_quantity("-5"),
            Err("invalid digit found in string".to_string())
        );
        assert_eq!(
            parse_quantity("nope"),
            Err("invalid digit found in string".to_string())
        );
    }

    #[test]
    fn chains_two_divisions() {
        assert_eq!(chained_division(100.0, 2.0, 5.0), Ok(10.0));
    }

    #[test]
    fn chained_division_propagates_the_first_error() {
        assert_eq!(
            chained_division(100.0, 0.0, 5.0),
            Err("cannot divide by zero".to_string())
        );
    }

    #[test]
    fn chained_division_propagates_the_second_error() {
        assert_eq!(
            chained_division(100.0, 2.0, 0.0),
            Err("cannot divide by zero".to_string())
        );
    }

    #[test]
    fn halves_a_valid_number() {
        assert_eq!(half_or_zero("10"), 5.0);
        assert_eq!(half_or_zero("7.5"), 3.75);
    }

    #[test]
    fn falls_back_to_zero_on_bad_input() {
        assert_eq!(half_or_zero("oops"), 0.0);
        assert_eq!(half_or_zero(""), 0.0);
    }

    #[test]
    fn succeeds_when_both_steps_succeed() {
        assert_eq!(parsed_and_divided("10", 2.0), Some(5.0));
    }

    #[test]
    fn fails_closed_when_either_step_fails() {
        assert_eq!(parsed_and_divided("oops", 2.0), None);
        assert_eq!(parsed_and_divided("10", 0.0), None);
    }
}
