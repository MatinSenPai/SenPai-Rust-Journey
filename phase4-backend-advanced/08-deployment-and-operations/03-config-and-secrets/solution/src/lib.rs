use std::collections::HashMap;
use std::fmt;

/// Parse the contents of a `.env` file into key/value pairs.
pub fn parse_env_file(contents: &str) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue; // malformed: no '=' at all
        };
        let key = key.trim();
        if key.is_empty() {
            continue; // malformed: `=value` with no name
        }
        vars.insert(key.to_string(), value.trim().to_string());
    }
    vars
}

/// A `String` that refuses to show itself in `Debug` output.
pub struct SecretString(String);

impl SecretString {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// The one deliberate, grep-able way to read the value.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    Missing(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Missing(name) => {
                write!(f, "missing required config variable: {name}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// The hardcoded, lowest-precedence layer — safe non-secrets only, and
/// deliberately no `DATABASE_URL`.
pub fn defaults() -> HashMap<String, String> {
    HashMap::from([
        ("BIND_ADDR".to_string(), "127.0.0.1:3000".to_string()),
        ("LOG_LEVEL".to_string(), "info".to_string()),
    ])
}

/// Remove `key` from the merged map, or fail with an error that NAMES it.
fn required(merged: &mut HashMap<String, String>, key: &str) -> Result<String, ConfigError> {
    merged
        .remove(key)
        .ok_or_else(|| ConfigError::Missing(key.to_string()))
}

#[derive(Debug)]
pub struct Config {
    pub database_url: SecretString,
    pub bind_addr: String,
    pub log_level: String,
}

impl Config {
    /// Merge the three layers (`defaults` < `file_vars` < `env_vars`),
    /// then build the struct, failing fast on the first missing key.
    pub fn resolve(
        defaults: HashMap<String, String>,
        file_vars: HashMap<String, String>,
        env_vars: HashMap<String, String>,
    ) -> Result<Config, ConfigError> {
        let mut merged = defaults;
        merged.extend(file_vars);
        merged.extend(env_vars);

        Ok(Config {
            database_url: SecretString::new(required(&mut merged, "DATABASE_URL")?),
            bind_addr: required(&mut merged, "BIND_ADDR")?,
            log_level: required(&mut merged, "LOG_LEVEL")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    // --- (a) parse_env_file ---

    #[test]
    fn parses_simple_key_value_pairs() {
        let vars = parse_env_file("BIND_ADDR=0.0.0.0:8080\nLOG_LEVEL=debug");
        assert_eq!(
            vars.get("BIND_ADDR").map(String::as_str),
            Some("0.0.0.0:8080")
        );
        assert_eq!(vars.get("LOG_LEVEL").map(String::as_str), Some("debug"));
    }

    #[test]
    fn skips_blank_lines_and_comments() {
        let contents = "\n# local overrides — never commit this file\n\nLOG_LEVEL=trace\n   # indented comment\n";
        let vars = parse_env_file(contents);
        assert_eq!(vars.len(), 1);
        assert_eq!(vars.get("LOG_LEVEL").map(String::as_str), Some("trace"));
    }

    #[test]
    fn splits_on_the_first_equals_only() {
        let vars = parse_env_file("DATABASE_URL=postgres://app:pw@localhost/dev?sslmode=disable");
        assert_eq!(
            vars.get("DATABASE_URL").map(String::as_str),
            Some("postgres://app:pw@localhost/dev?sslmode=disable")
        );
    }

    #[test]
    fn trims_whitespace_around_keys_and_values() {
        let vars = parse_env_file("  LOG_LEVEL =  warn  ");
        assert_eq!(vars.get("LOG_LEVEL").map(String::as_str), Some("warn"));
    }

    #[test]
    fn ignores_malformed_lines() {
        let vars = parse_env_file("just some words\n=value-with-no-key\nLOG_LEVEL=info");
        assert_eq!(vars.len(), 1);
        assert_eq!(vars.get("LOG_LEVEL").map(String::as_str), Some("info"));
    }

    #[test]
    fn keeps_an_explicitly_empty_value() {
        let vars = parse_env_file("FEATURE_FLAGS=");
        assert_eq!(vars.get("FEATURE_FLAGS").map(String::as_str), Some(""));
    }

    // --- (b) SecretString ---

    #[test]
    fn debug_output_contains_no_secret_material() {
        let secret = SecretString::new("hunter2-do-not-print");
        let debugged = format!("{secret:?}");
        assert!(!debugged.contains("hunter2"), "leaked: {debugged}");
        assert!(debugged.contains("[REDACTED]"));
    }

    #[test]
    fn expose_returns_the_real_value() {
        let secret = SecretString::new("hunter2");
        assert_eq!(secret.expose(), "hunter2");
    }

    // --- (c) Config::resolve ---

    #[test]
    fn real_environment_beats_file_beats_defaults() {
        let file_vars = map(&[
            ("LOG_LEVEL", "debug"),
            ("DATABASE_URL", "postgres://app:pw@localhost/dev"),
        ]);
        let env_vars = map(&[
            ("LOG_LEVEL", "warn"),
            ("DATABASE_URL", "postgres://app:hunter2@db/prod"),
        ]);

        let config = Config::resolve(defaults(), file_vars, env_vars).unwrap();
        assert_eq!(config.log_level, "warn");
        assert_eq!(
            config.database_url.expose(),
            "postgres://app:hunter2@db/prod"
        );
        assert_eq!(config.bind_addr, "127.0.0.1:3000");
    }

    #[test]
    fn file_beats_defaults_when_the_environment_is_silent() {
        let file_vars = map(&[
            ("BIND_ADDR", "0.0.0.0:8080"),
            ("DATABASE_URL", "postgres://app:pw@localhost/dev"),
        ]);
        let config = Config::resolve(defaults(), file_vars, HashMap::new()).unwrap();
        assert_eq!(config.bind_addr, "0.0.0.0:8080");
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn missing_database_url_fails_fast_and_names_the_variable() {
        let err = Config::resolve(defaults(), HashMap::new(), HashMap::new()).unwrap_err();
        assert_eq!(err, ConfigError::Missing("DATABASE_URL".to_string()));
        assert!(err.to_string().contains("DATABASE_URL"));
    }

    #[test]
    fn any_missing_required_key_names_itself() {
        let env_vars = map(&[
            ("DATABASE_URL", "postgres://app:pw@db/prod"),
            ("LOG_LEVEL", "info"),
        ]);
        let err = Config::resolve(HashMap::new(), HashMap::new(), env_vars).unwrap_err();
        assert_eq!(err, ConfigError::Missing("BIND_ADDR".to_string()));
    }

    #[test]
    fn a_full_config_debug_dump_is_safe_to_log() {
        let env_vars = map(&[("DATABASE_URL", "postgres://app:hunter2@db/prod")]);
        let config = Config::resolve(defaults(), HashMap::new(), env_vars).unwrap();

        let debugged = format!("{config:?}");
        assert!(!debugged.contains("hunter2"), "leaked: {debugged}");
        assert!(debugged.contains("[REDACTED]"));
        assert!(debugged.contains("127.0.0.1:3000"));
    }
}
