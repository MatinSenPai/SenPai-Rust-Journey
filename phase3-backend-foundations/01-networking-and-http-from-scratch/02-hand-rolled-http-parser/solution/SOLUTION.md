# Solution

```rust
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
```

A plain `match` on `&str` literals — `Other(other.to_string())` is the
catch-all arm, so this function can never fail; an unrecognized verb is
still valid *data*, just not one of the named variants.

```rust
pub fn header(&self, name: &str) -> Option<&str> {
    self.headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}
```

`.find()` short-circuits on the first case-insensitive match and `.map()`
projects the matched pair down to just the value, borrowed (`&str`) rather
than cloned — the caller doesn't need ownership just to read a header.

```rust
pub fn parse_request(raw: &[u8]) -> Result<HttpRequest, HttpParseError> {
    let text = std::str::from_utf8(raw).map_err(|_| HttpParseError::InvalidUtf8)?;
    if text.is_empty() {
        return Err(HttpParseError::EmptyRequest);
    }

    let mut lines = text.split("\r\n");
    let request_line = lines.next().ok_or(HttpParseError::EmptyRequest)?;

    let mut parts = request_line.split(' ');
    let (method_token, target, version) =
        match (parts.next(), parts.next(), parts.next(), parts.next()) {
            (Some(method), Some(target), Some(version), None) => (method, target, version),
            _ => return Err(HttpParseError::MalformedRequestLine(request_line.to_string())),
        };

    let method = Method::parse(method_token);
    let (path, query) = match target.split_once('?') {
        Some((path, query)) => (path.to_string(), Some(query.to_string())),
        None => (target.to_string(), None),
    };

    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| HttpParseError::MalformedHeaderLine(line.to_string()))?;
        headers.push((name.trim().to_string(), value.trim().to_string()));
    }

    Ok(HttpRequest { method, path, query, version: version.to_string(), headers })
}
```

A few deliberate choices here:

- **`text.is_empty()` is checked before splitting**, not after. `"".split("\r\n")`
  actually yields one item — an empty string — so without this early check,
  `lines.next()` would return `Some("")` instead of `None`, and the request
  line would fail as `MalformedRequestLine("".to_string())` rather than the
  more specific `EmptyRequest` the tests expect.
- **The request line is matched as a 4-tuple of `Option`s**, requiring
  exactly `(Some, Some, Some, None)`. That last `None` matters just as much
  as the three `Some`s: without it, `"GET / HTTP/1.1 extra-garbage"` (four
  space-separated tokens) would silently parse as if the fourth token
  didn't exist. Requiring the *fourth* `.next()` to be `None` is what
  enforces "exactly three tokens, no more, no fewer."
- **`target.split_once('?')`** splits on the *first* `?` only — a query
  string can itself legally contain `?` characters (URL-encoded ones,
  typically), so splitting on the first occurrence rather than the last (or
  splitting on every occurrence) is the correct choice.
- **`line.split_once(':')`**, not `split_once(": ")`. Real HTTP allows
  (and clients sometimes send) a colon with no space, or extra whitespace,
  before the value — splitting on just `:` and then `.trim()`-ing both
  sides handles all of those uniformly instead of assuming the exact
  two-character `": "` separator every time.

```rust
pub fn to_bytes(&self) -> Vec<u8> {
    let mut out = format!("HTTP/1.1 {} {}\r\n", self.status, self.reason);
    for (name, value) in &self.headers {
        out.push_str(&format!("{name}: {value}\r\n"));
    }
    out.push_str(&format!("Content-Length: {}\r\n", self.body.len()));
    out.push_str("\r\n");
    out.push_str(&self.body);
    out.into_bytes()
}
```

Builds the response as a `String` (easier to reason about line by line)
and converts to `Vec<u8>` only at the very end with `.into_bytes()` — cheap,
since a Rust `String` is already valid UTF-8 bytes under the hood, no
re-encoding needed.

## On the checkpoint questions

**Q1 (skipping UTF-8 validation):** `str::from_utf8` isn't just a formality
— a `&[u8]` that isn't valid UTF-8 genuinely cannot be treated as `&str` in
Rust; every `&str` method (`.split`, `.trim`, indexing by byte-range) relies
on the invariant that the bytes are valid UTF-8, and violating it is
undefined behavior if forced via `unsafe`. Skipping the check and using
`from_utf8_unchecked` (the only way to skip it and still get a `&str`)
would be a real security bug: a client sending malformed bytes could cause
the server to panic, read garbage, or worse. This is exactly why the type
system makes you handle the `Result` before a single byte of "text" logic
runs.

**Q2 (specific error variants):** `assert!(matches!(result,
Err(HttpParseError::MalformedRequestLine(_))))` proves the parser correctly
identified *which stage* failed. If there were only one `ParseFailed`
variant, that assertion — and any real caller trying to decide "should I
return a 400 with a helpful message, or is this a different kind of bug
entirely?" — would have no way to distinguish "the request line had the
wrong number of tokens" from "a header line had no colon" from "this wasn't
UTF-8 at all." Named variants turn debugging (and, in a real server,
constructing a useful error response) from string-matching into exhaustive,
compiler-checked pattern matching.

**Q3 (why not lowercase up front):** Lowercasing at parse time would work
functionally, but it throws away information for no benefit: `headers`
stores what the client *actually sent*, byte for byte, which matters if
you ever need to log the raw request, re-serialize it verbatim (a proxy
forwarding the request onward), or debug a client that's sending unusual
casing. The lookup is the *only* place casing genuinely doesn't matter (per
the HTTP spec), so that's the only place that pays the
`eq_ignore_ascii_case` cost — and it only runs when someone actually calls
`.header(...)`, not once per header on every single parse regardless of
whether anyone ever looks it up.

**Q4 (byte length vs. char count):** `"café".len()` is `5` (the `é` is two
UTF-8 bytes) while `"café".chars().count()` is `4`. A client reading the
response reads exactly `Content-Length` **bytes** off the TCP stream to
know where the body ends — it has no way to know encoding boundaries ahead
of time. If `to_bytes` used `.chars().count()` for a body containing `é`,
it would advertise `4` but actually write `5` bytes, and any client reading
exactly 4 bytes would get a body cut off mid-character, with one stray byte
left dangling on the wire (which would then get misread as the start of the
*next* response, on a connection that reuses the socket). `.len()` (byte
length) is the only value that matches what's actually written to the
socket.

**Q5 (from raw query string to something DRF-like):** You'd split the raw
string on `&` to get individual `key=value` pairs, then split each pair on
the first `=` (mirroring the `split_once(':')` choice above, since a value
can itself contain `=` after URL-encoding), URL-*decode* each key and value
(`%20` → space, `+` → space in some contexts, `%3D` → `=`, etc. — this
lesson's `parse_request` doesn't do this at all, deliberately, to keep the
scope to line-oriented text parsing), and collect the pairs into a
`HashMap<String, String>` (or a `Vec<(String, String)>` if a key can
legitimately repeat, e.g. `?tag=a&tag=b`, which a plain `HashMap` would
silently only keep one of).
