use std::collections::HashMap;

/// A required startup config value — if it's missing, that's a bug in how
/// the program was deployed/configured, not something the caller can
/// meaningfully recover from. **Panic**, with a message explaining the
/// assumption that broke.
pub fn required_config(vars: &HashMap<String, String>, key: &str) -> String {
    todo!("vars.get(key).cloned().expect(&format!(\"missing required config key: {{key}}\"))")
}

/// Parses user-submitted text as an age (0-150, say). This is ordinary,
/// expected, external input — it will absolutely be invalid sometimes, and
/// the caller (e.g. an HTTP handler in Phase 3) needs to handle that
/// gracefully. **Never panic** here — return `Result`.
pub fn parse_user_age(input: &str) -> Result<u8, String> {
    todo!("input.parse::<u8>().map_err(|e| e.to_string())")
}

/// Averages `nums`. This function's documented contract (as an internal
/// helper, not user-facing) is that it's only ever called with a non-empty
/// slice — an empty slice here means a bug in the *caller*, not bad
/// external input. **Panic** (with `.expect`) if that invariant is broken.
pub fn average_of_nonempty(nums: &[f64]) -> f64 {
    todo!("assert or expect that nums is not empty, then compute the average")
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
