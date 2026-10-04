//! A tiny 12-factor config loader, stdlib only. Real projects usually
//! reach for `dotenvy` or `figment` — but a dotenv parser is ~20 lines,
//! and building the whole loader once by hand demystifies exactly what
//! those crates do: read, layer, validate.

use std::collections::HashMap;
use std::fmt;

/// Parse the contents of a `.env` file into key/value pairs.
///
/// The rules — deliberately the subset every dotenv library agrees on:
/// - blank lines (after trimming) are skipped
/// - lines whose first non-whitespace character is `#` are comments
/// - everything else splits on the **first** `=`: left is the key, right
///   is the value, both trimmed
/// - lines with no `=` at all, or with an empty key, are silently ignored
///
/// What this deliberately does *not* handle (and `dotenvy` does): quoting,
/// escape sequences, multi-line values, `export ` prefixes. That would be
/// the missing 80% of code for 20% of cases — fine to skip in a file
/// format you fully control.
pub fn parse_env_file(contents: &str) -> HashMap<String, String> {
    todo!(
        "for each contents.lines(): trim it; `continue` if empty or starts_with('#'); \
         line.split_once('=') — None means no '=', malformed, skip it; Some((key, value)) means \
         trim both, skip if the trimmed key is empty, otherwise insert owned Strings into the map"
    )
}

/// A `String` that refuses to show itself in `Debug` output.
///
/// The threat isn't an attacker decrypting your memory — it's you, six
/// months from now, writing `tracing::info!(?config, "starting up")` and
/// shipping the database password to the log aggregator forever. The
/// newtype makes the *default* behavior safe, and the unsafe behavior
/// (`expose`) loud and grep-able.
pub struct SecretString(String);

impl SecretString {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// The one deliberate way to read the value. Named to be impossible
    /// to type by accident — `rg 'expose\(\)'` finds every place secret
    /// material leaves the wrapper.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!(
            "f.write_str(\"[REDACTED]\") — never touch self.0 in here. Because `Config` derives \
             Debug and derives delegate to each field's own impl, this one method is what makes \
             a whole Config safe to log."
        )
    }
}

/// Everything that can go wrong while resolving configuration. One
/// variant today — the important part is that it carries the variable's
/// NAME, so the startup crash tells the reader exactly what to set.
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

/// The hardcoded, lowest-precedence layer. Provided.
///
/// Note what's *not* here: `DATABASE_URL`. Two rules decide what gets a
/// default: secrets never do (a default secret is a vulnerability with a
/// version number), and neither does anything with no truthful universal
/// answer — there is no database this binary can honestly assume.
pub fn defaults() -> HashMap<String, String> {
    HashMap::from([
        ("BIND_ADDR".to_string(), "127.0.0.1:3000".to_string()),
        ("LOG_LEVEL".to_string(), "info".to_string()),
    ])
}

/// The fully-resolved, typed configuration the rest of a service passes
/// around. Derives `Debug` *on purpose* — the whole point of
/// `SecretString` is that this stays safe to print.
#[derive(Debug)]
pub struct Config {
    pub database_url: SecretString,
    pub bind_addr: String,
    pub log_level: String,
}

impl Config {
    /// Merge the three layers — lowest precedence first — then build the
    /// struct, failing fast with the NAME of the first missing key.
    ///
    /// Precedence: `defaults` < `file_vars` < `env_vars`. The closer a
    /// value lives to the actual deployment, the more it should win.
    ///
    /// Deliberately takes plain `HashMap`s instead of touching `std::env`
    /// or the filesystem itself — that's what makes every precedence rule
    /// below unit-testable with no environment mutation.
    pub fn resolve(
        defaults: HashMap<String, String>,
        file_vars: HashMap<String, String>,
        env_vars: HashMap<String, String>,
    ) -> Result<Config, ConfigError> {
        todo!(
            "let mut merged = defaults; merged.extend(file_vars); merged.extend(env_vars); — \
             HashMap::extend overwrites existing keys, which IS the precedence rule. Then pull \
             each key out: merged.remove(key).ok_or_else(|| ConfigError::Missing(key.to_string())) \
             — a tiny helper fn taking (&mut merged, key) keeps that from repeating three times. \
             DATABASE_URL gets wrapped in SecretString::new; BIND_ADDR and LOG_LEVEL stay plain."
        )
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
        // Real values contain '=' all the time — query strings, base64
        // padding. Splitting on anything but the FIRST '=' corrupts them.
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
        // `KEY=` is not malformed — it's a deliberate "set to empty",
        // which is how a layer blanks out a value a lower layer set.
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
        assert_eq!(config.log_level, "warn"); // env wins over file and default
        assert_eq!(
            config.database_url.expose(),
            "postgres://app:hunter2@db/prod"
        );
        assert_eq!(config.bind_addr, "127.0.0.1:3000"); // nothing overrode the default
    }

    #[test]
    fn file_beats_defaults_when_the_environment_is_silent() {
        let file_vars = map(&[
            ("BIND_ADDR", "0.0.0.0:8080"),
            ("DATABASE_URL", "postgres://app:pw@localhost/dev"),
        ]);
        let config = Config::resolve(defaults(), file_vars, HashMap::new()).unwrap();
        assert_eq!(config.bind_addr, "0.0.0.0:8080");
        assert_eq!(config.log_level, "info"); // untouched default
    }

    #[test]
    fn missing_database_url_fails_fast_and_names_the_variable() {
        let err = Config::resolve(defaults(), HashMap::new(), HashMap::new()).unwrap_err();
        assert_eq!(err, ConfigError::Missing("DATABASE_URL".to_string()));
        // The Display message names it too — this exact string is what
        // someone bleary-eyed at 3am reads in the crash log.
        assert!(err.to_string().contains("DATABASE_URL"));
    }

    #[test]
    fn any_missing_required_key_names_itself() {
        // No defaults layer at all: DATABASE_URL and LOG_LEVEL supplied,
        // so the one and only missing key is BIND_ADDR — and the error
        // must say so, whatever order the keys are checked in.
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
        assert!(debugged.contains("127.0.0.1:3000")); // non-secrets still visible
    }
}
