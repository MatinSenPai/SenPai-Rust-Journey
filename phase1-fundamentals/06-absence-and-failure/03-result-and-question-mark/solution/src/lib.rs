pub fn safe_divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("cannot divide by zero".to_string())
    } else {
        Ok(a / b)
    }
}

pub fn parse_positive(s: &str) -> Result<u32, String> {
    s.parse::<u32>().map_err(|e| e.to_string())
}

pub fn chained_division(a: f64, b: f64, c: f64) -> Result<f64, String> {
    let step1 = safe_divide(a, b)?;
    let step2 = safe_divide(step1, c)?;
    Ok(step2)
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
