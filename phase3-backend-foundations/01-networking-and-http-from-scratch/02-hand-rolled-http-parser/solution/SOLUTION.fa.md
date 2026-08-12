# پاسخ تشریحی

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

`match` روی literalهای `&str` است. بازوی `Other(other.to_string())` catch-all است؛ verb ناشناخته هنوز داده‌ی معتبر است، فقط variant نام‌دار ندارد.

```rust
pub fn header(&self, name: &str) -> Option<&str> {
    self.headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}
```

`.find()` در نخستین تطبیق متوقف و `.map()` جفت را به value قرض‌گرفته‌شده تبدیل می‌کند؛ خواندن header به cloneکردن نیاز ندارد.

```rust
pub fn parse_request(raw: &[u8]) -> Result<HttpRequest, HttpParseError> {
    let text = std::str::from_utf8(raw).map_err(|_| HttpParseError::InvalidUtf8)?;
    if text.is_empty() {
        return Err(HttpParseError::EmptyRequest);
    }
    let mut lines = text.split("\r\n");
    let request_line = lines.next().ok_or(HttpParseError::EmptyRequest)?;
    let mut parts = request_line.split(' ');
    let (method_token, target, version) = match (parts.next(), parts.next(), parts.next(), parts.next()) {
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
        if line.is_empty() { break; }
        let (name, value) = line.split_once(':')
            .ok_or_else(|| HttpParseError::MalformedHeaderLine(line.to_string()))?;
        headers.push((name.trim().to_string(), value.trim().to_string()));
    }
    Ok(HttpRequest { method, path, query, version: version.to_string(), headers })
}
```

`text.is_empty()` باید پیش از `split` بررسی شود؛ رشته‌ی خالی در `split("\r\n")` یک item خالی می‌دهد وگرنه error کم‌دقت `MalformedRequestLine` می‌گرفتیم. tuple چهار `Option` دقیقاً سه token را الزام می‌کند؛ `None` چهارم مانع پذیرش token اضافه می‌شود. `split_once('?')` فقط اولین `?` را جدا می‌کند و `split_once(':')` به‌همراه `trim` header با یا بدون space را می‌پذیرد.

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

`String` از قبل byteهای UTF-8 معتبر است، پس `.into_bytes()` در پایان re-encoding ندارد. `.len()` طول byte است: `"café".len()` برابر ۵ و `chars().count()` برابر ۴ است. HTTP length را byte می‌خواهد؛ اعلام ۴ و ارسال ۵ body را وسط character می‌بُرد و یک byte برای پاسخ بعدی روی socket باقی می‌گذارد.

اعتبار UTF-8 تشریفاتی نیست: تمام متدهای `&str` بر این invariant تکیه دارند. تنها راه دورزدنش `from_utf8_unchecked` و یک bug امنیتی است. variantهای مشخص نیز parser و caller را از string matching به `match` کامل و compile-checked می‌برند. نگه‌داشتن case خام header برای log یا proxy اطلاعات را حفظ می‌کند و هزینه‌ی comparison فقط در lookup پرداخت می‌شود. برای query نیز روی `&` و سپس نخستین `=` split کن، key/valueها را URL-decode و در `HashMap` یا برای keyهای تکراری در `Vec<(String, String)>` جمع‌آوری کن.
