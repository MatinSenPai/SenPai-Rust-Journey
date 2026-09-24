//! Hand-rolled HTTP parser — exercise skeleton.
//!
//! Every function's doc comment is its full specification. The tests below
//! only check what those doc comments already describe — you should never
//! need to open this module's test block to know what to build.

use std::fmt;

/// The HTTP method from a request line. Only a handful of variants are
/// named explicitly; anything else (`PATCH`, a typo, a made-up verb) falls
/// back to `Other`, so parsing never fails just because of an unusual verb
/// — that's a *semantic* question for whoever routes the request, not a
/// syntax error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Other(String),
}

impl Method {
    /// Parses a single request-line token, e.g. `"GET"`, into a `Method`.
    ///
    /// Matching is case-sensitive: HTTP method names are case-sensitive by
    /// spec (unlike header *names* — see `HttpRequest::header` below), so
    /// `"get"` is not `Method::Get`, it's `Method::Other("get".to_string())`.
    pub fn parse(token: &str) -> Method {
        todo!(
            "map each known method token (GET, POST, PUT, DELETE, HEAD) to its matching Method \
             variant, case-sensitively; anything else is a valid but unrecognized method, so it \
             becomes Other holding that exact token as an owned String"
        )
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Method::Get => write!(f, "GET"),
            Method::Post => write!(f, "POST"),
            Method::Put => write!(f, "PUT"),
            Method::Delete => write!(f, "DELETE"),
            Method::Head => write!(f, "HEAD"),
            Method::Other(s) => write!(f, "{s}"),
        }
    }
}

/// A parsed HTTP/1.1 request: request line + headers. No body support —
/// this lesson is scoped to `GET`-style requests, see the README.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: Method,
    pub path: String,
    pub query: Option<String>,
    pub version: String,
    pub headers: Vec<(String, String)>,
}

impl HttpRequest {
    /// Looks up a header by name, case-insensitively — HTTP header *names*
    /// don't carry meaning in their casing (`Host` and `host` are the same
    /// header), even though `headers` stores each one exactly as the client
    /// sent it. Returns the value of the *first* header whose name matches,
    /// or `None` if nothing does.
    pub fn header(&self, name: &str) -> Option<&str> {
        todo!(
            "find the first (name, value) pair in self.headers whose name matches the name \
             argument case-insensitively, and return a reference to its value; None if nothing \
             matches"
        )
    }
}

/// Everything that can go wrong turning raw bytes into an `HttpRequest`.
/// Each failure mode is its own variant (not one generic "parse failed")
/// so a caller — or a test — can tell exactly what was malformed.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum HttpParseError {
    #[error("request bytes are not valid UTF-8")]
    InvalidUtf8,
    #[error("request is empty")]
    EmptyRequest,
    #[error("malformed request line: {0:?}")]
    MalformedRequestLine(String),
    #[error("malformed header line: {0:?}")]
    MalformedHeaderLine(String),
}

/// Parses a raw HTTP/1.1 request (request line + headers only — no body)
/// from bytes straight off a socket.
///
/// Full specification:
///
/// - The bytes must be valid UTF-8, checked *before* anything else runs —
///   otherwise `HttpParseError::InvalidUtf8`.
/// - Empty input is `HttpParseError::EmptyRequest`.
/// - Lines are separated by `\r\n`, **not** bare `\n` — splitting on the
///   wrong one leaves a stray `\r` baked into whatever comes right before
///   it (see "Errors you will meet").
/// - The first line is the request line: exactly three space-separated
///   tokens — method, target, version. Anything other than exactly three
///   is `HttpParseError::MalformedRequestLine`, carrying the exact text of
///   that line.
/// - The target's path and query string split on the *first* `?` (a query
///   string can itself legally contain further `?` characters): everything
///   before it is `path`, everything after is `Some(query)`; no `?` at all
///   means `path` is the whole target and `query` is `None`.
/// - Every line after the request line, up to (but not including) the
///   first empty line, is a header line: it must contain a `:`, and its
///   name and value are the text before and after it with surrounding
///   whitespace trimmed. A line with no `:` at all is
///   `HttpParseError::MalformedHeaderLine`, carrying the exact text of that
///   line. Headers keep the order the client sent them in, duplicates
///   included — nothing here deduplicates them.
pub fn parse_request(raw: &[u8]) -> Result<HttpRequest, HttpParseError> {
    todo!(
        "turn raw into text (rejecting invalid UTF-8), split it into the request line and the \
         header lines per the doc comment above, validate and extract each piece, and build the \
         HttpRequest — or return the specific HttpParseError variant naming whichever piece was \
         malformed"
    )
}

/// A response ready to be serialized back into HTTP/1.1 wire bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub reason: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl HttpResponse {
    pub fn new(status: u16, reason: &str, body: impl Into<String>) -> Self {
        HttpResponse {
            status,
            reason: reason.to_string(),
            headers: Vec::new(),
            body: body.into(),
        }
    }

    pub fn ok(body: impl Into<String>) -> Self {
        Self::new(200, "OK", body)
    }

    pub fn not_found() -> Self {
        Self::new(404, "Not Found", "Not Found\n")
    }

    /// Serializes this response into raw HTTP/1.1 wire bytes — the exact
    /// mirror image of `parse_request`:
    ///
    /// 1. the status line, `"HTTP/1.1 {status} {reason}\r\n"`;
    /// 2. one `"{name}: {value}\r\n"` line per entry in `self.headers`, in
    ///    order;
    /// 3. a `"Content-Length: {n}\r\n"` line, where `n` is the **byte**
    ///    length of `self.body` (`.len()`, not `.chars().count()` — a
    ///    multi-byte UTF-8 character is more than one byte, and a client
    ///    reads exactly `n` bytes off the wire to find the end of the
    ///    body);
    /// 4. a blank `"\r\n"` line;
    /// 5. `self.body` itself, verbatim.
    ///
    /// Returns the whole thing as bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!(
            "build the status line, then one header line per entry in self.headers, then a \
             Content-Length line computed from the body's byte length, then a blank line, then \
             the body itself, in that exact order, and return it all as bytes"
        )
    }
}

#[cfg(test)]
mod method_tests {
    use super::*;

    #[test]
    fn parses_known_methods() {
        assert_eq!(Method::parse("GET"), Method::Get);
        assert_eq!(Method::parse("POST"), Method::Post);
    }

    #[test]
    fn falls_back_to_other_for_unknown_methods() {
        assert_eq!(Method::parse("PATCH"), Method::Other("PATCH".to_string()));
    }

    #[test]
    fn method_matching_is_case_sensitive() {
        assert_eq!(Method::parse("get"), Method::Other("get".to_string()));
    }
}
