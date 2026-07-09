use std::collections::HashMap;

/// A parsed application config — just enough fields to make the error
/// handling below interesting.
#[derive(Debug)]
pub struct Config {
    pub name: String,
    pub max_retries: u32,
    pub timeout_secs: u32,
}

/// Everything that can go wrong while parsing a [`Config`] from text.
///
/// One variant per distinct *kind* of failure, instead of one `String` for
/// everything — so callers can `match` on this and react differently
/// per-kind, instead of grepping a message for substrings.
#[derive(Debug)]
pub enum ConfigError {
    /// A required `key=value` line was absent entirely. Carries the name of
    /// the field that was missing.
    MissingField(String),
    /// A field that should have parsed as a number didn't. Carries both the
    /// field name (so the `Display` message can say *which* field) and the
    /// original [`std::num::ParseIntError`] (so no information from the
    /// underlying failure is thrown away).
    InvalidNumber {
        field: String,
        source: std::num::ParseIntError,
    },
}

/// The human-readable message shown to an end user (via `{}`), as opposed
/// to `Debug`'s `{:?}` developer dump, which `#[derive(Debug)]` above
/// already gives us for free. Display has no derive — you write it by hand,
/// because only you know what's actually worth telling someone.
impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingField(field) => write!(f, "missing required field: {field}"),
            ConfigError::InvalidNumber { field, source } => {
                write!(f, "invalid number for field '{field}': {source}")
            }
        }
    }
}

/// The marker trait that makes `ConfigError` "a proper Rust error" — it lets
/// this type compose with `?`, `Box<dyn std::error::Error>`, and (next
/// lesson) `anyhow::Error`. `Error` requires `Debug` + `Display` (both
/// above) and provides a default `source()` that returns `None`; you don't
/// need to override it for this exercise.
impl std::error::Error for ConfigError {}

/// Lets `?` convert a `ParseIntError` into a `ConfigError` automatically,
/// *when the field name can be hardcoded at the conversion site*. Look at
/// how `parse_config` uses this for `max_retries` via a bare `?`, then
/// contrast it with `timeout_secs`, which needs an explicit `.map_err(...)`
/// instead — `From::from` only receives the `ParseIntError`, so it can't
/// know which field a *different* call site was parsing.
impl From<std::num::ParseIntError> for ConfigError {
    fn from(source: std::num::ParseIntError) -> Self {
        ConfigError::InvalidNumber {
            field: "max_retries".to_string(),
            source,
        }
    }
}

/// Parses `key=value` lines (one per line, blank lines ignored) into a
/// [`Config`]. Required keys: `name`, `max_retries`, `timeout_secs`.
///
/// Example input:
/// ```text
/// name=OnePieceTracker
/// max_retries=3
/// timeout_secs=30
/// ```
pub fn parse_config(input: &str) -> Result<Config, ConfigError> {
    let mut fields: HashMap<&str, &str> = HashMap::new();
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            fields.insert(key.trim(), value.trim());
        }
    }

    let name = fields
        .get("name")
        .ok_or_else(|| ConfigError::MissingField("name".to_string()))?
        .to_string();

    let max_retries_raw = fields
        .get("max_retries")
        .ok_or_else(|| ConfigError::MissingField("max_retries".to_string()))?;
    let max_retries: u32 = max_retries_raw.parse()?;

    let timeout_secs_raw = fields
        .get("timeout_secs")
        .ok_or_else(|| ConfigError::MissingField("timeout_secs".to_string()))?;
    let timeout_secs: u32 =
        timeout_secs_raw
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
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn missing_name_is_reported_by_field_name() {
        let input = "max_retries=3\ntimeout_secs=30\n";
        match parse_config(input) {
            Err(ConfigError::MissingField(field)) => assert_eq!(field, "name"),
            other => panic!("expected MissingField(\"name\"), got {other:?}"),
        }
    }

    #[test]
    fn invalid_max_retries_reports_its_field_via_automatic_from() {
        let input = "name=OnePieceTracker\nmax_retries=not-a-number\ntimeout_secs=30\n";
        match parse_config(input) {
            Err(ConfigError::InvalidNumber { field, .. }) => assert_eq!(field, "max_retries"),
            other => panic!("expected InvalidNumber, got {other:?}"),
        }
    }

    #[test]
    fn invalid_timeout_secs_reports_its_own_field_via_manual_map_err() {
        let input = "name=OnePieceTracker\nmax_retries=3\ntimeout_secs=oops\n";
        match parse_config(input) {
            Err(ConfigError::InvalidNumber { field, .. }) => assert_eq!(field, "timeout_secs"),
            other => panic!("expected InvalidNumber, got {other:?}"),
        }
    }

    #[test]
    fn display_messages_are_human_readable() {
        let missing = ConfigError::MissingField("name".to_string());
        assert_eq!(missing.to_string(), "missing required field: name");

        let source = "abc".parse::<u32>().unwrap_err();
        let invalid = ConfigError::InvalidNumber {
            field: "max_retries".to_string(),
            source,
        };
        assert!(invalid.to_string().contains("max_retries"));
        assert!(invalid.to_string().contains("invalid"));
    }

    #[test]
    fn config_error_is_a_proper_std_error() {
        fn assert_is_error<E: std::error::Error>() {}
        assert_is_error::<ConfigError>();
    }
}
