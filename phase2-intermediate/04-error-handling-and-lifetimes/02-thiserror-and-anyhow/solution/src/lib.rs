use std::collections::HashMap;

#[derive(Debug)]
pub struct Config {
    pub name: String,
    pub max_retries: u32,
    pub timeout_secs: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required field: {0}")]
    MissingField(String),

    #[error("invalid number for field '{field}': {source}")]
    InvalidNumber {
        field: String,
        #[source]
        source: std::num::ParseIntError,
    },
}

impl From<std::num::ParseIntError> for ConfigError {
    fn from(source: std::num::ParseIntError) -> Self {
        ConfigError::InvalidNumber {
            field: "max_retries".to_string(),
            source,
        }
    }
}

pub fn parse_config(input: &str) -> Result<Config, ConfigError> {
    let fields: HashMap<&str, &str> = input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter_map(|line| line.split_once('='))
        .collect();

    let name = fields
        .get("name")
        .ok_or_else(|| ConfigError::MissingField("name".to_string()))?
        .to_string();

    let max_retries: u32 = fields
        .get("max_retries")
        .ok_or_else(|| ConfigError::MissingField("max_retries".to_string()))?
        .parse()?;

    let timeout_secs: u32 = fields
        .get("timeout_secs")
        .ok_or_else(|| ConfigError::MissingField("timeout_secs".to_string()))?
        .parse()
        .map_err(|source| ConfigError::InvalidNumber {
            field: "timeout_secs".to_string(),
            source,
        })?;

    Ok(Config {
        name,
        max_retries,
        timeout_secs,
    })
}

pub fn load_and_parse(input: &str) -> anyhow::Result<Config> {
    use anyhow::Context;
    parse_config(input).context("failed to load application config")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> &'static str {
        "name=OnePieceTracker\nmax_retries=3\ntimeout_secs=30\n"
    }

    #[test]
    fn parses_a_valid_config() {
        let config = parse_config(valid_input()).unwrap();
        assert_eq!(config.name, "OnePieceTracker");
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn thiserror_generated_display_matches_hand_written_messages() {
        let missing = ConfigError::MissingField("name".to_string());
        assert_eq!(missing.to_string(), "missing required field: name");

        let source = "abc".parse::<u32>().unwrap_err();
        let invalid = ConfigError::InvalidNumber {
            field: "max_retries".to_string(),
            source,
        };
        assert!(invalid.to_string().contains("max_retries"));
    }

    #[test]
    fn invalid_max_retries_uses_the_generated_from_impl() {
        let input = "name=OnePieceTracker\nmax_retries=oops\ntimeout_secs=30\n";
        match parse_config(input) {
            Err(ConfigError::InvalidNumber { field, .. }) => assert_eq!(field, "max_retries"),
            other => panic!("expected InvalidNumber, got {other:?}"),
        }
    }

    #[test]
    fn load_and_parse_succeeds_on_valid_input() {
        let config = load_and_parse(valid_input()).unwrap();
        assert_eq!(config.name, "OnePieceTracker");
    }

    #[test]
    fn load_and_parse_wraps_the_error_with_context() {
        let input = "max_retries=3\ntimeout_secs=30\n";
        let err = load_and_parse(input).unwrap_err();

        assert_eq!(err.to_string(), "failed to load application config");
        let source = err
            .source()
            .expect("context should preserve the original error");
        assert!(source.to_string().contains("missing required field: name"));
    }
}
