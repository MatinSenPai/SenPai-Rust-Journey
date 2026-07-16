/// How severe a log message is. The exercises match on these variants
/// directly (nested inside `LogEvent` patterns) rather than comparing them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

/// One event pulled off a web service's log stream. Every exercise below
/// pattern-matches this enum from a different angle.
#[derive(Debug, Clone)]
pub enum LogEvent {
    /// An HTTP request that completed with `status`.
    Request {
        method: String,
        path: String,
        status: u16,
    },
    /// A free-form log line with a severity level.
    Message { severity: Severity, text: String },
    /// A periodic "still alive" ping; `uptime_secs` is seconds since boot.
    Heartbeat { uptime_secs: u64 },
}

/// Describes an HTTP status code. Exercises or-patterns, range patterns,
/// `@` bindings, and arm order. Expected output, in match-arm order:
///
/// - exactly `200` → `"OK"`
/// - any *other* `200..=299` → `"success (<code>)"` — needs `@` to keep the code
/// - `301`, `302`, `307`, or `308` → `"redirect"` — one arm, or-pattern
/// - `400..=499` → `"client error (<code>)"`
/// - `500..=599` → `"server error (<code>)"`
/// - anything else → `"unrecognized status <code>"`
///
/// Arm order matters: the literal `200` arm must sit *above* the
/// `200..=299` range arm, or the range swallows it first.
pub fn describe_status(status: u16) -> String {
    todo!(
        "match status: 200 => OK, n @ 200..=299 => format!(\"success ({{n}})\"), 301 | 302 | 307 | 308 => redirect, n @ 400..=499 / n @ 500..=599 similarly, other => format!(\"unrecognized status {{other}}\")"
    )
}

/// Returns a short alert string for events worth waking somebody up over,
/// `None` for everything else. Exercises match guards and nested
/// destructuring:
///
/// - a `Request` with `status >= 500` → `"server error <status> on <path>"`
/// - a `Message` with `Severity::Error` → `"error: <text>"`
/// - a `Message` with `Severity::Warning` whose text contains `"disk"`
///   → `"disk warning: <text>"` (other warnings are not noteworthy)
/// - a `Heartbeat` with `uptime_secs == 0` → `"service just restarted"`
/// - everything else → `None`
///
/// Matching `severity: Severity::Error` *inside* the struct pattern is the
/// nested-destructuring part — no `if severity == ...` guard needed there.
pub fn noteworthy(event: &LogEvent) -> Option<String> {
    todo!(
        "arms like `LogEvent::Request {{ status, path, .. }} if *status >= 500 => Some(format!(...))` and `LogEvent::Message {{ severity: Severity::Error, text }} => ...`, ending with `_ => None`"
    )
}

/// Returns the HTTP method if `event` is a `Request`, `None` otherwise.
///
/// Nearly a one-liner, but it exercises *binding modes*: because `event`
/// is a `&LogEvent`, the `method` binding inside the pattern is a
/// `&String`, not a `String` — no `ref` keyword, no `&` in the pattern.
/// You'll need `.as_str()` to produce the `&str` the signature promises.
pub fn method_of(event: &LogEvent) -> Option<&str> {
    todo!(
        "match event {{ LogEvent::Request {{ method, .. }} => Some(method.as_str()), _ => None }}"
    )
}

/// Summarizes a slice of latency samples (in milliseconds) using slice
/// patterns:
///
/// - `[]` → `"no samples"`
/// - `[only]` → `"1 sample: <only>ms"`
/// - `[first, .., last]` → `"<len> samples, first <first>ms, last <last>ms"`
///
/// Those three patterns cover lengths 0, 1, and 2+, and the compiler
/// *verifies* that — an `if/else if` chain on `.len()` with indexing gets
/// no such proof.
pub fn summarize_samples(samples: &[u64]) -> String {
    todo!(
        "match samples {{ [] => .., [only] => .., [first, .., last] => format!(\"{{}} samples, first {{first}}ms, last {{last}}ms\", samples.len()) }}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(method: &str, path: &str, status: u16) -> LogEvent {
        LogEvent::Request {
            method: method.to_string(),
            path: path.to_string(),
            status,
        }
    }

    fn message(severity: Severity, text: &str) -> LogEvent {
        LogEvent::Message {
            severity,
            text: text.to_string(),
        }
    }

    #[test]
    fn describe_status_matches_the_exact_literal_before_the_range() {
        assert_eq!(describe_status(200), "OK");
    }

    #[test]
    fn describe_status_captures_range_matches_with_at_bindings() {
        assert_eq!(describe_status(204), "success (204)");
        assert_eq!(describe_status(404), "client error (404)");
        assert_eq!(describe_status(503), "server error (503)");
    }

    #[test]
    fn describe_status_groups_redirects_with_an_or_pattern() {
        for code in [301, 302, 307, 308] {
            assert_eq!(describe_status(code), "redirect");
        }
    }

    #[test]
    fn describe_status_has_a_fallback_arm() {
        assert_eq!(describe_status(42), "unrecognized status 42");
        assert_eq!(describe_status(700), "unrecognized status 700");
    }

    #[test]
    fn noteworthy_flags_5xx_requests_via_guard() {
        assert_eq!(
            noteworthy(&request("GET", "/api/jobs", 503)),
            Some("server error 503 on /api/jobs".to_string())
        );
    }

    #[test]
    fn noteworthy_ignores_healthy_requests() {
        assert_eq!(noteworthy(&request("GET", "/api/jobs", 200)), None);
        assert_eq!(noteworthy(&request("GET", "/missing", 404)), None);
    }

    #[test]
    fn noteworthy_flags_error_messages_via_nested_destructuring() {
        assert_eq!(
            noteworthy(&message(Severity::Error, "db connection lost")),
            Some("error: db connection lost".to_string())
        );
    }

    #[test]
    fn noteworthy_flags_disk_warnings_but_not_other_warnings() {
        assert_eq!(
            noteworthy(&message(Severity::Warning, "disk 91% full")),
            Some("disk warning: disk 91% full".to_string())
        );
        assert_eq!(
            noteworthy(&message(Severity::Warning, "cache miss rate high")),
            None
        );
        assert_eq!(
            noteworthy(&message(Severity::Info, "disk check done")),
            None
        );
    }

    #[test]
    fn noteworthy_flags_only_fresh_heartbeats() {
        assert_eq!(
            noteworthy(&LogEvent::Heartbeat { uptime_secs: 0 }),
            Some("service just restarted".to_string())
        );
        assert_eq!(noteworthy(&LogEvent::Heartbeat { uptime_secs: 3600 }), None);
    }

    #[test]
    fn method_of_extracts_the_method_from_requests_only() {
        assert_eq!(method_of(&request("POST", "/login", 401)), Some("POST"));
        assert_eq!(method_of(&message(Severity::Info, "hi")), None);
        assert_eq!(method_of(&LogEvent::Heartbeat { uptime_secs: 5 }), None);
    }

    #[test]
    fn summarize_samples_handles_empty_and_single() {
        assert_eq!(summarize_samples(&[]), "no samples");
        assert_eq!(summarize_samples(&[42]), "1 sample: 42ms");
    }

    #[test]
    fn summarize_samples_reports_first_and_last_of_longer_slices() {
        assert_eq!(
            summarize_samples(&[5, 80, 9, 12]),
            "4 samples, first 5ms, last 12ms"
        );
        assert_eq!(summarize_samples(&[7, 3]), "2 samples, first 7ms, last 3ms");
    }
}
