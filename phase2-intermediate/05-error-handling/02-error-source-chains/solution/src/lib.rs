//! Exercises for 2.5.2 — source chains and `Box<dyn Error>`.
//!
//! `Config`, `ConfigError`'s `Display` impl, and `parse_config_str` are
//! provided in full below — that's 2.5.1's skill, not this lesson's. What's
//! new here: exposing the chain (`source()`), wiring a new failure mode
//! through it (`From<io::Error>`, `load_config`), walking the chain
//! (`error_chain`), and the type-erased return type that lets two unrelated
//! error types share one signature (`effective_max_retries`).

use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

/// A parsed application config — just `name` and `max_retries`, enough
/// fields to make the error handling below interesting.
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub name: String,
    pub max_retries: u32,
}

/// Everything that can go wrong on the way to a [`Config`]: a missing key,
/// a number that doesn't parse, or — new in this lesson — the file itself
/// not opening at all.
#[derive(Debug)]
pub enum ConfigError {
    /// The config file could not be read. Wraps the [`std::io::Error`]
    /// that actually happened (not found, permission denied, ...).
    Io(std::io::Error),
    /// A required `key=value` line was absent entirely. Carries the name
    /// of the field that was missing.
    MissingField(String),
    /// A field that should have parsed as a number didn't. Carries both
    /// the field name and the original [`ParseIntError`].
    InvalidNumber {
        field: String,
        source: ParseIntError,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "could not read config file: {e}"),
            ConfigError::MissingField(field) => write!(f, "missing required field: {field}"),
            ConfigError::InvalidNumber { field, source } => {
                write!(f, "invalid number for field '{field}': {source}")
            }
        }
    }
}

/// Lets `?` convert a `std::io::Error` into a `ConfigError` automatically,
/// anywhere a function returns `Result<_, ConfigError>` — exactly the role
/// 2.5.1 gave `From<ParseIntError>`, now for the failure mode a real file
/// on disk introduces.
impl From<std::io::Error> for ConfigError {
    fn from(source: std::io::Error) -> Self {
        ConfigError::Io(source)
    }
}

impl Error for ConfigError {
    /// The error `self` wraps, if any: `Some` for a variant built from
    /// another error, `None` for a variant that *is* the root cause
    /// itself. The signature is fixed by the trait, not a choice you get
    /// to make: `Option<&(dyn Error + 'static)>`.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),
            ConfigError::MissingField(_) => None,
            ConfigError::InvalidNumber { source, .. } => Some(source),
        }
    }
}

/// Parses `key=value` lines (one per line, blank lines ignored) into a
/// [`Config`]. Required keys: `name`, `max_retries`. Provided in full —
/// this is 2.5.1's skill, not this lesson's.
pub fn parse_config_str(input: &str) -> Result<Config, ConfigError> {
    let mut name = None;
    let mut max_retries = None;
    for line in input.lines() {
        let line = line.trim();
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "name" => name = Some(value.to_string()),
            "max_retries" => {
                let parsed = value
                    .parse::<u32>()
                    .map_err(|source| ConfigError::InvalidNumber {
                        field: "max_retries".to_string(),
                        source,
                    })?;
                max_retries = Some(parsed);
            }
            _ => {}
        }
    }
    Ok(Config {
        name: name.ok_or_else(|| ConfigError::MissingField("name".to_string()))?,
        max_retries: max_retries
            .ok_or_else(|| ConfigError::MissingField("max_retries".to_string()))?,
    })
}

/// Reads the file at `path` and parses it the same way [`parse_config_str`]
/// does.
///
/// Two conversions happen here through the same `?`: a failed read
/// becomes `ConfigError::Io` (via the `From<std::io::Error>` impl above),
/// and `parse_config_str`'s own `Result<Config, ConfigError>` needs no
/// conversion at all.
pub fn load_config(path: &str) -> Result<Config, ConfigError> {
    let contents = std::fs::read_to_string(path)?;
    parse_config_str(&contents)
}

/// Walks `err`'s cause chain, starting with `err` itself.
///
/// The returned `Vec`'s first element is `err`'s own message
/// (`format!("{err}")`). Each element after that is the next link's
/// message, found by calling `.source()` on the previous link, repeated
/// until `.source()` returns `None`. A variant with no source (like
/// `ConfigError::MissingField`) produces a one-element `Vec`.
pub fn error_chain(err: &dyn Error) -> Vec<String> {
    let mut chain = vec![err.to_string()];
    let mut cause = err.source();
    while let Some(source) = cause {
        chain.push(source.to_string());
        cause = source.source();
    }
    chain
}

/// Resolves the effective `max_retries` value: `override_max_retries` wins
/// when present, the config file at `path` is the fallback.
///
/// If `override_max_retries` is `Some(text)`, `path` is never touched —
/// `text` is parsed as `u32` and returned, or its [`ParseIntError`]
/// propagates as-is (**not** wrapped in `ConfigError`). If it is `None`,
/// `path` is loaded with [`load_config`] and its `max_retries` field is
/// returned, or the `ConfigError` propagates.
///
/// Two genuinely different concrete error types can come out of this
/// function, and `?` converts either one into `Box<dyn Error>` without a
/// shared enum — that conversion is what this exercise is testing.
pub fn effective_max_retries(
    path: &str,
    override_max_retries: Option<&str>,
) -> Result<u32, Box<dyn Error>> {
    match override_max_retries {
        Some(text) => Ok(text.parse::<u32>()?),
        None => Ok(load_config(path)?.max_retries),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::path::PathBuf;

    /// A path in the OS temp directory, unique to this test binary's
    /// process so parallel test runs (and the separate `solution/` crate)
    /// never collide on the same file.
    fn temp_config_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("senpai-2-5-2-{label}-{}.cfg", std::process::id()))
    }

    const MISSING_PATH: &str = "senpai-2-5-2-tests/definitely/does/not/exist.cfg";

    #[test]
    fn from_io_error_wraps_it() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "no such file");
        let config_err: ConfigError = io_err.into();
        assert!(matches!(config_err, ConfigError::Io(_)));
    }

    #[test]
    fn source_is_none_for_missing_field() {
        let err = ConfigError::MissingField("name".to_string());
        assert!(err.source().is_none());
    }

    #[test]
    fn source_is_some_for_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "no such file");
        let err = ConfigError::Io(io_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn source_is_some_for_invalid_number() {
        let parse_err = "oops".parse::<u32>().unwrap_err();
        let err = ConfigError::InvalidNumber {
            field: "max_retries".to_string(),
            source: parse_err,
        };
        assert!(err.source().is_some());
    }

    #[test]
    fn load_config_wraps_a_missing_file_as_io_error() {
        let err = load_config(MISSING_PATH).unwrap_err();
        assert!(matches!(err, ConfigError::Io(_)));
        assert!(err.source().is_some());
    }

    #[test]
    fn load_config_reads_and_parses_a_real_file() {
        let path = temp_config_path("load-config-happy");
        std::fs::write(&path, "name=OnePieceTracker\nmax_retries=3\n").unwrap();

        let result = load_config(path.to_str().unwrap());
        let _ = std::fs::remove_file(&path);

        assert_eq!(
            result.unwrap(),
            Config {
                name: "OnePieceTracker".to_string(),
                max_retries: 3,
            }
        );
    }

    #[test]
    fn error_chain_has_one_link_when_there_is_no_source() {
        let err = ConfigError::MissingField("name".to_string());
        assert_eq!(error_chain(&err), vec![err.to_string()]);
    }

    #[test]
    fn error_chain_walks_down_to_the_root_cause() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "no such file");
        let err = ConfigError::Io(io_err);

        let chain = error_chain(&err);

        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0], err.to_string());
        assert_eq!(chain[1], "no such file");
    }

    #[test]
    fn effective_max_retries_prefers_the_override() {
        let result = effective_max_retries(MISSING_PATH, Some("7"));
        assert_eq!(result.unwrap(), 7);
    }

    #[test]
    fn effective_max_retries_propagates_a_bad_override_without_touching_the_file() {
        let err = effective_max_retries(MISSING_PATH, Some("not-a-number")).unwrap_err();
        assert!(err.downcast_ref::<ParseIntError>().is_some());
    }

    #[test]
    fn effective_max_retries_falls_back_to_the_config_file() {
        let path = temp_config_path("effective-retries-happy");
        std::fs::write(&path, "name=OnePieceTracker\nmax_retries=9\n").unwrap();

        let result = effective_max_retries(path.to_str().unwrap(), None);
        let _ = std::fs::remove_file(&path);

        assert_eq!(result.unwrap(), 9);
    }

    #[test]
    fn effective_max_retries_propagates_a_missing_file() {
        let err = effective_max_retries(MISSING_PATH, None).unwrap_err();
        assert!(err.downcast_ref::<ConfigError>().is_some());
    }
}
