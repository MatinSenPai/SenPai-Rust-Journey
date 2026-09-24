# Solution — 3.1.2 Hand-rolled HTTP parser

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

A plain `match` on `&str` literals. The `other => Method::Other(...)` arm is the catch-all, so this function can never fail — an unrecognized verb is still valid *data*, just not one of the named variants. And because the match is on exact strings, it's naturally case-sensitive too: `"get"` matches no arm, so it falls to `Other("get".to_string())` — which is correct per spec, unlike header names, HTTP methods are case-sensitive.

```rust
pub fn header(&self, name: &str) -> Option<&str> {
    self.headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}
```

`.find()` short-circuits on the first case-insensitive match; `.map()` just projects out the value, borrowed (`&str`) rather than cloned — reading a header doesn't need to own it.

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

Four deliberate choices here:

- **`text.is_empty()` is checked before splitting**, not after. `"".split("\r\n")` yields one item — an empty string — not zero; without this early check, `lines.next()` would return `Some("")` instead of `None`, and the request line would fail as `MalformedRequestLine("")` rather than the more specific `EmptyRequest`.
- **The request line is matched as a 4-tuple of `Option`s**, requiring exactly `(Some, Some, Some, None)`. That last `None` matters just as much as the three `Some`s: without it, `"GET / HTTP/1.1 extra"` (four space-separated tokens) would silently parse as if the fourth token didn't exist. Requiring the *fourth* `.next()` to be `None` is what enforces "exactly three tokens, no more, no fewer."
- **`target.split_once('?')`** splits on the *first* `?` only — a query string can itself legally contain `?` characters (typically URL-encoded ones), so splitting on the first occurrence rather than the last (or every occurrence) is the correct choice.
- **`line.split_once(':')`, not `split_once(": ")`**. Real HTTP allows a colon with no following space, or extra whitespace before the value; splitting on just `:` and then `.trim()`-ing both sides handles all of those uniformly, instead of assuming the exact two-character `": "` separator every time.

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

Builds the response as a `String` (easier to reason about line by line) and converts to `Vec<u8>` only at the very end with `.into_bytes()` — nearly free, since a Rust `String` is already valid UTF-8 bytes underneath, no re-encoding needed. `.len()` gives the byte length, not `.chars().count()`: a client reads exactly that many **bytes** off the wire to find the end of the body, and a multi-byte UTF-8 character is more than one byte.

## The "Build" exercise — `query_params`

```rust
pub fn query_params(&self) -> Vec<(String, String)> {
    let Some(query) = &self.query else {
        return Vec::new();
    };
    query
        .split('&')
        .map(|pair| match pair.split_once('=') {
            Some((key, value)) => (key.to_string(), value.to_string()),
            None => (pair.to_string(), String::new()),
        })
        .collect()
}
```

The same `split_once('?')` rule from above repeats one layer down: each pair splits on its *first* `=` (a value can itself contain `=`). The return type is a `Vec`, not a `HashMap`, on purpose: a repeated key like `tag=a&tag=b` needs to keep both values, and a plain `HashMap` would silently lose one.

## On the challenge (optional)

If you went for the stricter request-line variant, the smallest fix is swapping `request_line.split(' ')` for `request_line.split_whitespace()` — it treats any run of consecutive spaces as one separator, so `"GET  / HTTP/1.1"` (two spaces) still produces exactly the same three tokens. The cost is that you can no longer see a genuinely empty token *inside* the line; for an HTTP request line that's a harmless trade, since none of the three tokens is legally allowed to contain a space itself.

## What this lesson was really about

All four functions share one idea: every place the input could be malformed gets an explicit branch — a named `Err`, not a `panic!` and not an optimistic assumption. That's exactly what `axum`, two lessons from now, does for you automatically: write `Json<T>` or `Path<T>` in a handler's signature, and a body or path that doesn't match gets a 400, not a crash. Today you found out where that 400 actually comes from, because you built one by hand.
