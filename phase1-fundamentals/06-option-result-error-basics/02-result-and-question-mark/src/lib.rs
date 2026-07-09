/// Divides `a` by `b`, or returns an `Err` describing the problem if
/// `b == 0.0`.
pub fn safe_divide(a: f64, b: f64) -> Result<f64, String> {
    todo!("if b == 0.0 return Err(...), else Ok(a / b)")
}

/// Parses `s` as a `u32`, converting any parse failure into a `String`
/// error via `.map_err(...)`. (Parsing directly to `u32` already rejects
/// negative numbers and non-numeric text — no extra check needed.)
pub fn parse_positive(s: &str) -> Result<u32, String> {
    todo!("s.parse::<u32>().map_err(|e| e.to_string())")
}

/// Divides `a` by `b`, then divides *that* result by `c`, propagating
/// either division's error immediately via `?`.
pub fn chained_division(a: f64, b: f64, c: f64) -> Result<f64, String> {
    todo!("let step1 = safe_divide(a, b)?; let step2 = safe_divide(step1, c)?; Ok(step2)")
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
    fn parses_valid_positive_numbers() {
        assert_eq!(parse_positive("42"), Ok(42));
    }

    #[test]
    fn rejects_invalid_or_negative_input() {
        assert!(parse_positive("not a number").is_err());
        assert!(parse_positive("-5").is_err());
    }

    #[test]
    fn chains_two_divisions() {
        assert_eq!(chained_division(100.0, 2.0, 5.0), Ok(10.0));
    }

    #[test]
    fn chained_division_propagates_first_error() {
        assert_eq!(
            chained_division(100.0, 0.0, 5.0),
            Err("cannot divide by zero".to_string())
        );
    }

    #[test]
    fn chained_division_propagates_second_error() {
        assert_eq!(
            chained_division(100.0, 2.0, 0.0),
            Err("cannot divide by zero".to_string())
        );
    }
}
