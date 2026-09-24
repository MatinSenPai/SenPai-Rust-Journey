# راه‌حل — ۳.۱.۳ چیزهایی از HTTP که باید بدونی

## `is_safe` و `is_idempotent`

```rust
pub fn is_safe(method: &Method) -> bool {
    matches!(method, Method::Get | Method::Head | Method::Options | Method::Trace)
}

pub fn is_idempotent(method: &Method) -> bool {
    matches!(
        method,
        Method::Get | Method::Head | Method::Put | Method::Delete | Method::Options | Method::Trace
    )
}
```

`matches!` یک `match` است که برایِ الگوهایِ فهرست‌شده `true` و برایِ بقیه `false` برمی‌گرداند. دقت کن که فهرستِ `is_idempotent` همان فهرستِ `is_safe` است به‌علاوه‌ی `Put` و `Delete`. این همان قاعده‌ی «هر متدِ ایمنی خودتوان است» است که در خودِ کد دیده می‌شود. می‌شد نوشت `is_safe(method) || matches!(method, Method::Put | Method::Delete)` تا صریح‌تر شود، ولی دو فهرستِ ساده را راحت‌تر می‌شود با جدولِ اسپک مقایسه کرد.

## `status_class`

```rust
pub fn status_class(status: u16) -> Option<StatusClass> {
    match status {
        100..=199 => Some(StatusClass::Informational),
        200..=299 => Some(StatusClass::Success),
        300..=399 => Some(StatusClass::Redirection),
        400..=499 => Some(StatusClass::ClientError),
        500..=599 => Some(StatusClass::ServerError),
        _ => None,
    }
}
```

بازوی `_ => None` همان راه‌حلِ `E0004` در «خطاهایی که خواهی دید» است. جوابِ صادقانه هم همین است: یک `u16` می‌تواند `999` را نگه دارد، و اسپک برایش هیچ رده‌ای تعریف نکرده. `status / 100` با یک `match` رویِ `1..=5` هم کار می‌کند. بازه‌ها فقط کنارِ جدول راحت‌تر خوانده می‌شوند.

## `best_content_type`

راه‌حل هدر را یک بار پارس می‌کند، به یک ساختارِ کوچک برایِ هر مدخل:

```rust
let entries: Vec<AcceptEntry> = accept
    .split(',')
    .map(str::trim)
    .filter(|entry| !entry.is_empty())
    .map(|entry| {
        let (media, q) = match entry.split_once(";q=") {
            Some((media, weight)) => (media.trim(), weight.trim().parse().unwrap_or(1.0)),
            None => (entry, 1.0),
        };
        let (kind, subkind) = media_parts(media);
        AcceptEntry { kind, subkind, q }
    })
    .collect();
```

`unwrap_or(1.0)` همان راه‌حلِ مثالِ `07` است: `q`ِ پارس‌نشدنی به‌جایِ کرش کردنِ هندلر، `1.0` حساب می‌شود. `media_parts` همان `split_once('/')` است، با یک جایگزین برایِ مدخلی که اصلاً `/` ندارد، تا مدخلِ بدشکل بی‌سروصدا با هیچ‌چیز جور نشود.

بعد، برایِ هر نوعی که سرور می‌تواند بسازد، *دقیق‌ترین* مدخلِ جور را پیدا می‌کند. دقت یک عدد است: `2` برایِ تطبیقِ دقیق، `1` برایِ `TYPE/*`، `0` برایِ `*/*`:

```rust
let specificity = if entry.kind.eq_ignore_ascii_case("*") {
    0u8
} else if !entry.kind.eq_ignore_ascii_case(want_kind) {
    continue;
} else if entry.subkind.eq_ignore_ascii_case("*") {
    1
} else if entry.subkind.eq_ignore_ascii_case(want_subkind) {
    2
} else {
    continue;
};
```

تطبیقی با دقتِ بیشتر جایِ تطبیقِ فعلی را می‌گیرد، `q`اش هر چه باشد. این قاعده‌ی ۲ است: برایِ `text/html`، مدخلِ `text/html;q=0.1` از `*/*;q=0.9` می‌برد. فقط وقتی دو تطبیق دقیقاً هم‌دقت‌اند، `q`ِ بیشتر برنده می‌شود.

بعدش دو قدمِ کوچک. وزنِ `0` نوع را کنار می‌گذارد (`continue`)، که قاعده‌ی ۳ است: `q=0` یعنی «قابلِ قبول نیست»، نه «آخرین انتخاب». بعد بهترین نامزد فقط وقتی عوض می‌شود که وزنِ نامزدِ تازه *اکیداً* بیشتر باشد (`q > best_q`). چون `available` به ترتیب پیموده می‌شود، در تساوی نوعِ جلوتر می‌ماند، و این یعنی ترجیحِ خودِ سرور تساوی را می‌شکند.

راه‌حل `entries` را یک بار در یک `Vec` جمع می‌کند، به‌جایِ اینکه برایِ هر نامزد دوباره `accept` را پارس کند. با دو فرمت فرقِ چندانی نمی‌کند، ولی عادتی است که می‌خواهی داشته باشی: ورودی را یک بار پارس کن، بعد رویِ داده‌ی ساختاریافته کار کن.

## `decode_chunked` (تمرینِ «بساز»)

```rust
loop {
    let line_end = find_crlf(rest).ok_or(ChunkedDecodeError::UnexpectedEnd)?;
    let size_line = &rest[..line_end];
    rest = &rest[line_end + 2..];
    // ... size_line -> size (hexadecimal) ...
    if size == 0 {
        let ends_cleanly = rest.len() >= 2 && &rest[..2] == b"\r\n";
        return if ends_cleanly { Ok(out) } else { Err(ChunkedDecodeError::UnexpectedEnd) };
    }
    if rest.len() < size + 2 || &rest[size..size + 2] != b"\r\n" {
        return Err(ChunkedDecodeError::UnexpectedEnd);
    }
    out.extend_from_slice(&rest[..size]);
    rest = &rest[size + 2..];
}
```

(کوتاه‌شده: نسخه‌ی کامل، با پارسِ اندازه، در `src/lib.rs` است.)

`rest` یک برش است که رویِ بدنه جلو می‌رود. جز خودِ داده‌ی تکه‌ها، که در `out` ریخته می‌شود، هیچ‌چیز کپی نمی‌شود. `find_crlf` **فقط** برایِ خطِ اندازه استفاده می‌شود. برایِ داده، راه‌حل به `size` اعتماد می‌کند و دقیقاً همان تعداد بایت را می‌بُرد. همین است که تکه‌ی سومِ تستِ کلاسیک (۱۴ بایت، با دو جفت `\r\n` داخلش) را درست بیرون می‌آورد.

پارسِ خطِ اندازه دو قدم دارد، و هر کدام خطایِ خودش را:

```rust
let size_text = std::str::from_utf8(size_line)
    .map_err(|_| ChunkedDecodeError::InvalidLength(String::from_utf8_lossy(size_line).into_owned()))?;
let size = usize::from_str_radix(size_text, 16)
    .map_err(|_| ChunkedDecodeError::InvalidLength(size_text.to_string()))?;
```

`usize::from_str_radix(text, 16)` عددِ مبنایِ شانزده را پارس می‌کند. هم `"E"` کار می‌کند هم `"e"`. اسپک هر دو را مجاز می‌داند، `from_str_radix` هم همین‌طور. همه‌ی حالت‌هایِ خارج از محدوده (تکه‌ای که زود بریده شده، `\r\n`ِ گم‌شده بعد از داده، تکه‌ی پایانیِ گم‌شده) *قبل از* برش زدن چک می‌شوند. پس بدنه‌ی بدشکل یک `Err` است، هیچ‌وقت یک پنیکِ index-out-of-bounds.

## درباره‌ی چالش (اختیاری)

کوچک‌ترین تغییر در نحوه‌ی پارسِ هر مدخل است. رویِ `;` تکه‌اش کن، هر تکه را trim کن، اولی را نوعِ رسانه بگیر، و بینِ بقیه دنبالِ تکه‌ای بگرد که با `q=` شروع شود:

```rust
let mut parts = entry.split(';').map(str::trim);
let media = parts.next().unwrap_or("");
let q = parts
    .find_map(|param| param.strip_prefix("q="))
    .map(|weight| weight.trim().parse().unwrap_or(1.0))
    .unwrap_or(1.0);
```

هم `text/html ; q=0.5` و هم `text/html;level=1;q=0.5` نتیجه‌ی `("text/html", 0.5)` می‌دهند. پارامترهایِ دیگر، مثلِ `level=1`، نادیده گرفته می‌شوند. اسپک اجازه می‌دهد تطبیق را *دقیق‌تر* کنند، ولی هیچ سروری که در این دوره می‌نویسی لازمش ندارد.

## این درس واقعاً درباره‌ی چه بود

نوشتنِ هیچ‌کدام از این تابع‌ها سخت نیست. سخت دانستنِ قاعده‌ای است که هر کدام پیاده می‌کند. `axum` عبارتِ دلیلِ (reason phrase) خطِ وضعیتت را انتخاب می‌کند، اتصال‌ها را باز نگه می‌دارد، و بدنه‌ی استریمی را برایت تکه‌تکه می‌کند. ولی کدِ وضعیتت، متدت، یا اینکه تکرارِ یک هندلر امن است یا نه را انتخاب نمی‌کند. این انتخاب‌ها تا آخرِ دوره با خودِ توست.
