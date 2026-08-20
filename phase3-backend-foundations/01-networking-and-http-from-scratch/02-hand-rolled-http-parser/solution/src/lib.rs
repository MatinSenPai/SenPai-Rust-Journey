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
    /// Parses a single request-line token, e.g. `"GET"`.
    pub fn parse(token: &str) -> Method {
        match token {
            "GET" => Method::Get,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "DELETE" => Method::Delete,
            "HEAD" => Method::Head,
            other => Method::Other(other.to_string()),
        }
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
    /// Case-insensitive header lookup — HTTP header *names* don't carry
    /// meaning in their casing (`Host` and `host` are the same header),
    /// even though this struct stores them exactly as the client sent them.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
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
/// Lines are separated by `\r\n`, not just `\n` — don't forget the `\r` or
/// header values will end up with a trailing carriage-return character
/// baked in.
pub fn parse_request(raw: &[u8]) -> Result<HttpRequest, HttpParseError> {
    let text = std::str::from_utf8(raw).map_err(|_| HttpParseError::InvalidUtf8)?;
    if text.is_empty() {
        return Err(HttpParseError::EmptyRequest);
    }

    let mut lines = text.split("\r\n");

    // `lines.next()` can't be `None` here (we just checked `text` isn't
    // empty, and `str::split` always yields at least one item), but
    // `ok_or` keeps this robust instead of `.unwrap()`ing.
    let request_line = lines.next().ok_or(HttpParseError::EmptyRequest)?;

    let mut parts = request_line.split(' ');
    let (method_token, target, version) =
        match (parts.next(), parts.next(), parts.next(), parts.next()) {
            (Some(method), Some(target), Some(version), None) => (method, target, version),
            _ => {
                return Err(HttpParseError::MalformedRequestLine(
                    request_line.to_string(),
                ))
            }
        };

    let method = Method::parse(method_token);
    let (path, query) = match target.split_once('?') {
        Some((path, query)) => (path.to_string(), Some(query.to_string())),
        None => (target.to_string(), None),
    };

    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            break; // the blank line marks the end of the headers
        }
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| HttpParseError::MalformedHeaderLine(line.to_string()))?;
        headers.push((name.trim().to_string(), value.trim().to_string()));
    }

    Ok(HttpRequest {
        method,
        path,
        query,
        version: version.to_string(),
        headers,
    })
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

    /// Serializes this response into raw HTTP/1.1 wire bytes: status line,
    /// headers, an automatically-computed `Content-Length`, a blank line,
    /// then the body — the exact mirror image of `parse_request`.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = format!("HTTP/1.1 {} {}\r\n", self.status, self.reason);
        for (name, value) in &self.headers {
            out.push_str(&format!("{name}: {value}\r\n"));
        }
        // Byte length, not `.chars().count()` — see recall questions.md question
        // 4. A client reads exactly this many *bytes* off the wire to find
        // the end of the body, and a multi-byte UTF-8 character is more
        // than one byte.
        out.push_str(&format!("Content-Length: {}\r\n", self.body.len()));
        out.push_str("\r\n");
        out.push_str(&self.body);
        out.into_bytes()
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
}
