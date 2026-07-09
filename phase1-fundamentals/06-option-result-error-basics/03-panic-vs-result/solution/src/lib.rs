use std::collections::HashMap;

pub fn required_config(vars: &HashMap<String, String>, key: &str) -> String {
    vars.get(key)
        .cloned()
        .unwrap_or_else(|| panic!("missing required config key: {key}"))
}

pub fn parse_user_age(input: &str) -> Result<u8, String> {
    input.parse::<u8>().map_err(|e| e.to_string())
}

pub fn average_of_nonempty(nums: &[f64]) -> f64 {
    assert!(
        !nums.is_empty(),
        "average_of_nonempty called with an empty slice — caller bug"
    );
    let sum: f64 = nums.iter().sum();
    sum / nums.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_required_config() {
        let mut vars = HashMap::new();
        vars.insert(
            "DATABASE_URL".to_string(),
            "postgres://localhost".to_string(),
        );
        assert_eq!(
            required_config(&vars, "DATABASE_URL"),
            "postgres://localhost"
        );
    }

    #[test]
    #[should_panic(expected = "missing required config key")]
    fn panics_on_missing_required_config() {
        let vars = HashMap::new();
        required_config(&vars, "DATABASE_URL");
    }

    #[test]
    fn parses_valid_ages() {
        assert_eq!(parse_user_age("25"), Ok(25));
    }

    #[test]
    fn returns_err_for_invalid_ages_without_panicking() {
        assert!(parse_user_age("not an age").is_err());
        assert!(parse_user_age("-5").is_err());
    }

    #[test]
    fn averages_nonempty_slice() {
        assert_eq!(average_of_nonempty(&[2.0, 4.0, 6.0]), 4.0);
    }

    #[test]
    #[should_panic]
    fn panics_on_empty_slice_invariant_violation() {
        average_of_nonempty(&[]);
    }
}
