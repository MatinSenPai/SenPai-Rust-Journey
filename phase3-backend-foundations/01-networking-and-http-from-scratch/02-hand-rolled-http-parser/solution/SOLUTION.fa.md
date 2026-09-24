# راه‌حل — ۳.۱.۲ پارسرِ دست‌سازِ HTTP

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

یک `match` ساده روی لیترال‌های `&str`. بازوی `other => Method::Other(...)` نقشِ بازویِ همه‌گیر را بازی می‌کند، پس این تابع هرگز نمی‌تواند شکست بخورد — یک فعل ناشناخته هنوز هم داده‌ی معتبر است، فقط جزوِ گونه‌های اسم‌دار نیست. و چون تطبیق روی رشته‌های دقیق است، این کار به‌طور طبیعی حساس به حروف کوچیک/بزرگ هم هست: `"get"` هیچ بازویی را جور در نمی‌آورد، پس می‌افتد به `Other("get".to_string())`. طبقِ اسپک هم دقیقاً همین درست است — برخلافِ نام‌های هدر، متدهای HTTP حساس به حروفند.

```rust
pub fn header(&self, name: &str) -> Option<&str> {
    self.headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}
```

`.find()` به محضِ اولین تطبیقِ حساس‌نبودن‌به‌حروف متوقف می‌شود؛ `.map()` فقط مقدار را بیرون می‌کشد، به‌شکلِ قرضی (`&str`)، نه کلون‌شده — خواننده‌ی هدر برای فقط *دیدن* یک مقدار نیازی به مالکیتش ندارد.

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

چهار انتخابِ عمدی اینجاست:

- **`text.is_empty()` قبل از تکه‌کردن چک شده، نه بعدش.** عبارتِ `"".split("\r\n")` یک آیتم می‌دهد — یک رشته‌ی خالی — نه صفر آیتم؛ بدونِ این چکِ زودهنگام، `lines.next()` مقدارِ `Some("")` برمی‌گرداند نه `None`، و خطِ درخواست به‌جایِ `EmptyRequest`ِ دقیق‌تر، با `MalformedRequestLine("")` رد می‌شد.
- **خطِ درخواست به‌شکلِ یک تاپلِ چهارتاییِ `Option` تطبیق داده شده**، که دقیقاً `(Some, Some, Some, None)` را می‌خواهد. آن `None` چهارم به‌اندازه‌ی سه `Some` قبلی اهمیت دارد: بدونش، چیزی مثلِ `"GET / HTTP/1.1 extra"` (چهار توکن) بی‌سروصدا انگار توکنِ چهارم اصلاً وجود نداشته پارس می‌شد. اجبارِ اینکه چهارمین `.next()` باید `None` باشد، دقیقاً همان چیزی است که «دقیقاً سه توکن، نه بیشتر نه کمتر» را تضمین می‌کند.
- **`target.split_once('?')` فقط رویِ اولین `?`** تکه می‌کند — خودِ رشته‌ی کوئری هم می‌تواند قانوناً `?` داشته باشد (معمولاً URL-encoded)، پس تکه‌کردن رویِ اولین برخورد، نه آخرین یا همه، انتخابِ درست است.
- **`line.split_once(':')`، نه `split_once(": ")`.** دنیایِ واقعیِ HTTP اجازه می‌دهد یک دونقطه بدونِ فاصله بیاید، یا فاصله‌ی اضافه قبل از مقدار باشد؛ تکه‌کردن فقط رویِ `:` و بعد `.trim()` کردنِ هر دو طرف، همه‌ی این حالت‌ها را یک‌دست پوشش می‌دهد، به‌جایِ اینکه فرض کند همیشه دقیقاً `": "` دو-کاراکتری بینِ اسم و مقدار است.

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

پاسخ را به‌شکلِ یک `String` سرهم می‌کند (خط‌به‌خط فکر کردن راحت‌تر است) و فقط در گامِ آخر با `.into_bytes()` تبدیلش می‌کند به `Vec<u8>` — تقریباً رایگان، چون یک `String`ِ Rust از پایه همان بایت‌هایِ معتبرِ UTF-8 است، بدونِ هیچ رمزگذاریِ دوباره. `.len()` طولِ بایت را می‌دهد، نه `.chars().count()`: کلاینت دقیقاً همان‌قدر **بایت** از رویِ سیم می‌خواند تا بفهمد بدنه کجا تمام می‌شود، و یک کاراکترِ چندبایتیِ UTF-8 بیش از یک بایت است.

## تمرینِ «بساز» — `query_params`

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

همان قانونِ `split_once('?')` بالا اینجا هم تکرار می‌شود، فقط یک لایه پایین‌تر: هر جفت رویِ *اولین* `=` تکه می‌شود (یک مقدار می‌تواند خودش هم `=` داشته باشد). خروجی یک `Vec` است نه یک `HashMap`، عمداً: یک کلیدِ تکراری مثلِ `tag=a&tag=b` باید هر دو مقدار را نگه دارد، و یک `HashMap`ِ ساده بی‌سروصدا یکی‌شان را گم می‌کرد.

## دربابِ چالش (اختیاری)

اگر رفتی سراغِ نسخه‌ی سخت‌گیرترِ خطِ درخواست، ساده‌ترین راه این است: `request_line.split(' ')` را با `request_line.split_whitespace()` عوض کن — این یکی خودش هر دنباله‌ای از فاصله‌های پیاپی را یک جداکننده حساب می‌کند، پس `"GET  / HTTP/1.1"` (دو فاصله) هم دقیقاً همان سه توکن را می‌دهد. قیمتش این است که دیگر نمی‌توانی یک فاصله‌ی خالی *داخلِ* یک توکن را ببینی؛ برایِ خطِ درخواستِ HTTP این معامله‌ی بی‌ضرری است، چون هیچ‌کدام از سه توکن خودش قانوناً فاصله ندارد.

## چیزی که این درس واقعاً درباره‌اش بود

هر چهار تابع یک ایده‌ی مشترک دارند: هر جایی که ورودی می‌تواند بدشکل باشد، یک شاخه‌ی صریح برایش هست — یک `Err` با اسمِ خودش، نه یک `panic!` یا یک فرضِ خوش‌بینانه. این همان اتفاقی است که `axum`، دو درسِ بعد، برایت خودکار انجام می‌دهد: وقتی `Json<T>` یا `Path<T>` را می‌نویسی و بدنه/مسیر جور در نمی‌آید، یک ۴۰۰ می‌گیری، نه یک کرش. امروز فهمیدی آن ۴۰۰ از کجا می‌آید، چون خودت یکی از آن‌ها را دستی ساختی.
