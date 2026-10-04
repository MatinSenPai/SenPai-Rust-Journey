//! Exercises for 3.4.1 — 12-factor config and secrets.
//!
//! Layered configuration (default < file < environment) parsed into one typed
//! `Config`, with a password that cannot leak through `Debug`. Nothing here
//! touches the real environment: the environment is a `HashMap` you pass in,
//! so every test builds its own.

use std::collections::HashMap;
use std::fmt;

use axum::http::{header, HeaderValue, Method};
use axum::{routing::get, Router};
use secrecy::SecretString;
use serde::Deserialize;
use tower_http::cors::{AllowOrigin, CorsLayer};

/// Port used when neither the file nor the environment sets one.
pub const DEFAULT_PORT: u16 = 8080;
/// The one allowed CORS origin used when neither the file nor the environment
/// sets a list.
pub const DEFAULT_ORIGIN: &str = "http://localhost:5173";

/// Everything that can go wrong while building a [`Config`]. Each variant
/// names the thing that was wrong, so a startup log line can say exactly what
/// to fix.
#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    /// A required setting was set by no layer. `key` is the setting's name in
    /// the config file (for example `"db_password"`).
    Missing { key: &'static str },
    /// A setting was present but could not be understood. `key` is the name
    /// the operator typed (for example `"APP_PORT"`), `value` is exactly what
    /// they set it to, `expected` is a short description of what would have
    /// been accepted.
    Invalid {
        key: &'static str,
        value: String,
        expected: &'static str,
    },
    /// The TOML file could not be parsed or had an unknown/mistyped key. The
    /// string is the TOML parser's own message.
    File(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Missing { key } => write!(f, "missing required setting `{key}`"),
            ConfigError::Invalid {
                key,
                value,
                expected,
            } => write!(f, "invalid value {value:?} for {key}: expected {expected}"),
            ConfigError::File(msg) => write!(f, "config file error: {msg}"),
        }
    }
}

impl std::error::Error for ConfigError {}

/// The finished, validated configuration. Every field is the real type the
/// rest of the program wants, not a string.
#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub allowed_origins: Vec<String>,
    pub debug: bool,
    pub db_password: SecretString,
}

/// One layer of configuration: every setting is optional because a single
/// layer (the file, or the environment) usually sets only some of them.
#[derive(Debug, Default, Clone)]
pub struct Layer {
    pub port: Option<u16>,
    pub allowed_origins: Option<Vec<String>>,
    pub debug: Option<bool>,
    pub db_password: Option<SecretString>,
}

/// What the TOML file is allowed to contain. `deny_unknown_fields` turns a
/// typo such as `prot = 9000` into an error instead of a silently ignored line.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FileLayer {
    port: Option<u16>,
    allowed_origins: Option<Vec<String>>,
    debug: Option<bool>,
    db_password: Option<String>,
}

impl Layer {
    /// The file layer. Given for you: parses `text` as TOML into a `Layer`.
    /// Any parse failure, wrong type, or unknown key becomes
    /// `ConfigError::File` carrying the parser's message.
    pub fn from_toml(text: &str) -> Result<Layer, ConfigError> {
        let file: FileLayer = toml::from_str(text).map_err(|e| ConfigError::File(e.to_string()))?;
        Ok(Layer {
            port: file.port,
            allowed_origins: file.allowed_origins,
            debug: file.debug,
            db_password: file.db_password.map(SecretString::from),
        })
    }

    /// The environment layer: reads these four variables out of `env` and
    /// leaves every other variable alone.
    ///
    /// | variable | field | parsing |
    /// |---|---|---|
    /// | `APP_PORT` | `port` | a `u16`; otherwise `Invalid { key: "APP_PORT", value: <the text>, expected: "a port number from 0 to 65535" }` |
    /// | `APP_ALLOWED_ORIGINS` | `allowed_origins` | `parse_origins` of the text |
    /// | `APP_DEBUG` | `debug` | `parse_bool("APP_DEBUG", <the text>)` |
    /// | `APP_DB_PASSWORD` | `db_password` | the text, wrapped in a `SecretString`, as-is |
    ///
    /// A variable that is absent gives `None` for its field. A variable that
    /// is present but empty counts as present: an empty `APP_PORT` is
    /// `Invalid`, an empty `APP_ALLOWED_ORIGINS` is `Some(vec![])`. If several
    /// variables are bad, return the error for the first one in the table's
    /// order.
    pub fn from_env(env: &HashMap<String, String>) -> Result<Layer, ConfigError> {
        let port = match env.get("APP_PORT") {
            Some(raw) => Some(raw.parse::<u16>().map_err(|_| ConfigError::Invalid {
                key: "APP_PORT",
                value: raw.clone(),
                expected: "a port number from 0 to 65535",
            })?),
            None => None,
        };
        let allowed_origins = env.get("APP_ALLOWED_ORIGINS").map(|raw| parse_origins(raw));
        let debug = match env.get("APP_DEBUG") {
            Some(raw) => Some(parse_bool("APP_DEBUG", raw)?),
            None => None,
        };
        let db_password = env.get("APP_DB_PASSWORD").cloned().map(SecretString::from);
        Ok(Layer {
            port,
            allowed_origins,
            debug,
            db_password,
        })
    }

    /// Stacks two layers: `self` is the higher-priority one. For each field,
    /// use `self`'s value if it is `Some`, otherwise `lower`'s value (which may
    /// itself be `None`).
    pub fn over(self, lower: Layer) -> Layer {
        Layer {
            port: self.port.or(lower.port),
            allowed_origins: self.allowed_origins.or(lower.allowed_origins),
            debug: self.debug.or(lower.debug),
            db_password: self.db_password.or(lower.db_password),
        }
    }

    /// Turns the stacked layers into a [`Config`], filling the gaps with
    /// defaults: `port` becomes [`DEFAULT_PORT`], `allowed_origins` becomes a
    /// list holding only [`DEFAULT_ORIGIN`], `debug` becomes `false`.
    /// `db_password` has no default: if it is `None`, return
    /// `ConfigError::Missing { key: "db_password" }`.
    pub fn finish(self) -> Result<Config, ConfigError> {
        Ok(Config {
            port: self.port.unwrap_or(DEFAULT_PORT),
            allowed_origins: self
                .allowed_origins
                .unwrap_or_else(|| vec![DEFAULT_ORIGIN.to_string()]),
            debug: self.debug.unwrap_or(false),
            db_password: self
                .db_password
                .ok_or(ConfigError::Missing { key: "db_password" })?,
        })
    }
}

/// Parses a boolean setting. Exactly `"true"` or `"1"` (any letter case) is
/// `true`; exactly `"false"` or `"0"` (any letter case) is `false`. Nothing is
/// trimmed. Anything else is
/// `ConfigError::Invalid { key, value: raw.to_string(), expected: "true, false, 1 or 0" }`.
pub fn parse_bool(key: &'static str, raw: &str) -> Result<bool, ConfigError> {
    if raw.eq_ignore_ascii_case("true") || raw == "1" {
        Ok(true)
    } else if raw.eq_ignore_ascii_case("false") || raw == "0" {
        Ok(false)
    } else {
        Err(ConfigError::Invalid {
            key,
            value: raw.to_string(),
            expected: "true, false, 1 or 0",
        })
    }
}

/// Splits a comma-separated list of origins. Each piece has its surrounding
/// whitespace removed, and pieces that are empty after that are dropped.
/// `"a, b,,c "` gives `["a", "b", "c"]`; `""` gives an empty `Vec`.
pub fn parse_origins(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|piece| !piece.is_empty())
        .map(String::from)
        .collect()
}

impl Config {
    /// The whole pipeline in one call. Builds the file layer from `file` (if
    /// it is `Some`) and the environment layer from `env`, stacks them so that
    /// the environment beats the file and the file beats the defaults, and
    /// finishes the result.
    ///
    /// The file is parsed first and the environment second; the first error
    /// encountered is the one returned, and nothing is returned half-built.
    pub fn load(file: Option<&str>, env: &HashMap<String, String>) -> Result<Config, ConfigError> {
        let file_layer = match file {
            Some(text) => Layer::from_toml(text)?,
            None => Layer::default(),
        };
        Layer::from_env(env)?.over(file_layer).finish()
    }
}

/// Builds the CORS layer for this configuration: exactly the origins in
/// `config.allowed_origins` may read responses, for `GET` requests with a
/// `content-type` header. An origin that is not a valid header value is
/// `ConfigError::Invalid { key: "APP_ALLOWED_ORIGINS", .. }`. Given for you.
pub fn cors_layer(config: &Config) -> Result<CorsLayer, ConfigError> {
    let mut origins = Vec::new();
    for origin in &config.allowed_origins {
        let value = HeaderValue::from_str(origin).map_err(|_| ConfigError::Invalid {
            key: "APP_ALLOWED_ORIGINS",
            value: origin.clone(),
            expected: "a valid origin such as https://example.com",
        })?;
        origins.push(value);
    }
    Ok(CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET])
        .allow_headers([header::CONTENT_TYPE]))
}

/// A one-route app (`GET /` answers `ok`) behind the configured CORS policy.
/// Given for you.
pub fn app(config: &Config) -> Result<Router, ConfigError> {
    Ok(Router::new()
        .route("/", get(|| async { "ok\n" }))
        .layer(cors_layer(config)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;

    fn env(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn parse_bool_accepts_both_spellings_in_any_case() {
        assert_eq!(parse_bool("K", "true"), Ok(true));
        assert_eq!(parse_bool("K", "TRUE"), Ok(true));
        assert_eq!(parse_bool("K", "1"), Ok(true));
        assert_eq!(parse_bool("K", "False"), Ok(false));
        assert_eq!(parse_bool("K", "0"), Ok(false));
    }

    #[test]
    fn parse_bool_rejects_everything_else_without_trimming() {
        let err = ConfigError::Invalid {
            key: "K",
            value: "yes".to_string(),
            expected: "true, false, 1 or 0",
        };
        assert_eq!(parse_bool("K", "yes"), Err(err));
        assert!(parse_bool("K", " true").is_err());
        assert!(parse_bool("K", "").is_err());
    }

    #[test]
    fn parse_origins_trims_and_drops_empties() {
        assert_eq!(parse_origins("a, b,,c "), ["a", "b", "c"]);
        assert!(parse_origins("").is_empty());
        assert!(parse_origins(" , ,").is_empty());
    }

    #[test]
    fn from_env_reads_the_four_variables() {
        let layer = Layer::from_env(&env(&[
            ("APP_PORT", "9000"),
            (
                "APP_ALLOWED_ORIGINS",
                "https://a.example, https://b.example",
            ),
            ("APP_DEBUG", "1"),
            ("APP_DB_PASSWORD", " s3cret "),
            ("PATH", "/usr/bin"),
        ]))
        .unwrap();
        assert_eq!(layer.port, Some(9000));
        assert_eq!(
            layer.allowed_origins,
            Some(vec!["https://a.example".into(), "https://b.example".into()])
        );
        assert_eq!(layer.debug, Some(true));
        assert_eq!(layer.db_password.unwrap().expose_secret(), " s3cret ");
    }

    #[test]
    fn from_env_absent_is_none_and_empty_origins_is_some_empty() {
        let layer = Layer::from_env(&env(&[("APP_ALLOWED_ORIGINS", "")])).unwrap();
        assert_eq!(layer.port, None);
        assert_eq!(layer.debug, None);
        assert!(layer.db_password.is_none());
        assert_eq!(layer.allowed_origins, Some(vec![]));
    }

    #[test]
    fn from_env_reports_the_first_bad_variable_in_table_order() {
        let err =
            Layer::from_env(&env(&[("APP_PORT", "eighty"), ("APP_DEBUG", "maybe")])).unwrap_err();
        assert_eq!(
            err,
            ConfigError::Invalid {
                key: "APP_PORT",
                value: "eighty".to_string(),
                expected: "a port number from 0 to 65535",
            }
        );
        for bad in ["", "70000", "-1", "80 "] {
            assert!(
                Layer::from_env(&env(&[("APP_PORT", bad)])).is_err(),
                "{bad:?}"
            );
        }
        let err = Layer::from_env(&env(&[("APP_DEBUG", "maybe")])).unwrap_err();
        assert!(matches!(
            err,
            ConfigError::Invalid {
                key: "APP_DEBUG",
                ..
            }
        ));
    }

    #[test]
    fn over_prefers_self_and_falls_back_field_by_field() {
        let high = Layer {
            port: Some(1),
            debug: Some(true),
            ..Layer::default()
        };
        let low = Layer {
            port: Some(2),
            allowed_origins: Some(vec!["o".into()]),
            db_password: Some("pw".into()),
            ..Layer::default()
        };
        let merged = high.over(low);
        assert_eq!(merged.port, Some(1));
        assert_eq!(merged.debug, Some(true));
        assert_eq!(merged.allowed_origins, Some(vec!["o".to_string()]));
        assert_eq!(merged.db_password.unwrap().expose_secret(), "pw");
    }

    #[test]
    fn finish_fills_defaults_and_requires_the_password() {
        let cfg = Layer {
            db_password: Some("pw".into()),
            ..Layer::default()
        }
        .finish()
        .unwrap();
        assert_eq!(cfg.port, DEFAULT_PORT);
        assert_eq!(cfg.allowed_origins, [DEFAULT_ORIGIN]);
        assert!(!cfg.debug);
        assert_eq!(
            Layer::default().finish().unwrap_err(),
            ConfigError::Missing { key: "db_password" }
        );
    }

    #[test]
    fn load_environment_beats_file_beats_default() {
        let file = "port = 7000\ndebug = true\ndb_password = \"from-file\"\n";
        let cfg = Config::load(Some(file), &env(&[("APP_PORT", "9000")])).unwrap();
        assert_eq!(cfg.port, 9000);
        assert!(cfg.debug);
        assert_eq!(cfg.allowed_origins, [DEFAULT_ORIGIN]);
        assert_eq!(cfg.db_password.expose_secret(), "from-file");
    }

    #[test]
    fn load_without_a_file_uses_the_environment_alone() {
        let cfg = Config::load(None, &env(&[("APP_DB_PASSWORD", "pw")])).unwrap();
        assert_eq!(cfg.port, DEFAULT_PORT);
        assert_eq!(
            Config::load(None, &env(&[])).unwrap_err(),
            ConfigError::Missing { key: "db_password" }
        );
    }

    #[test]
    fn load_returns_the_file_error_before_the_environment_error() {
        let err = Config::load(Some("prot = 1"), &env(&[("APP_PORT", "x")])).unwrap_err();
        assert!(matches!(err, ConfigError::File(_)));
        let err =
            Config::load(Some("db_password = \"p\""), &env(&[("APP_PORT", "x")])).unwrap_err();
        assert!(matches!(
            err,
            ConfigError::Invalid {
                key: "APP_PORT",
                ..
            }
        ));
    }

    #[test]
    fn debug_output_never_contains_the_password() {
        let cfg = Config::load(None, &env(&[("APP_DB_PASSWORD", "hunter2-xyz")])).unwrap();
        let printed = format!("{cfg:?}");
        assert!(!printed.contains("hunter2-xyz"), "{printed}");
        assert!(printed.contains("REDACTED"), "{printed}");
    }
}
