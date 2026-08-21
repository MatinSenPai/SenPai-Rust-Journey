# راه‌حل — ۱.۴.۴ برشِ امنِ متن

```rust
pub fn char_boundaries(text: &str) -> Vec<usize> {
    let mut found = Vec::new();
    for index in 0..=text.len() {
        if text.is_char_boundary(index) {
            found.push(index);
        }
    }
    found
}

pub fn safe_prefix(text: &str, max_bytes: usize) -> &str {
    &text[..text.floor_char_boundary(max_bytes)]
}

pub fn truncate_to_chars(text: &str, max_chars: usize) -> &str {
    let mut seen = 0;
    for (index, _) in text.char_indices() {
        if seen == max_chars {
            return &text[..index];
        }
        seen += 1;
    }
    text
}

pub fn truncated_with_ellipsis(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let kept = truncate_to_chars(text, max_chars);
    format!("{kept}…")
}

pub fn split_at_char(text: &str, char_index: usize) -> (&str, &str) {
    let mut seen = 0;
    for (index, _) in text.char_indices() {
        if seen == char_index {
            return text.split_at(index);
        }
        seen += 1;
    }
    (text, "")
}
```

هیچ‌کدام از این پنج تابع روی هیچ ورودی‌ای پنیک نمی‌کند، و این تصادفی نیست: هر برشی که در این فایل نوشته شده، عددش را از جایی گرفته که تضمین می‌کند مرز است.

## `char_boundaries` — چرا `0..=text.len()` و نه `0..text.len()`

```rust
for index in 0..=text.len() {
```

بازه‌ی شاملِ انتها است، چون `text.len()` خودش یک مرزِ مجاز است. با `0..text.len()` رشته‌ی خالی فهرستِ خالی می‌داد در حالی که جوابِ درست `[0]` است، و هر رشته‌ی دیگری هم آخرین مرزش را از دست می‌داد — یعنی امکانِ «برش تا آخر» را.

`is_char_boundary` برای اندیسِ بزرگ‌تر از طول فقط `false` می‌دهد و پنیک نمی‌کند، پس این حلقه هیچ‌وقت خطرناک نیست.

می‌شد همین را با `char_indices` هم نوشت و در عمل سریع‌تر هم بود:

```rust
for (index, _) in text.char_indices() {
    found.push(index);
}
found.push(text.len());
```

هر دو درست‌اند. نسخه‌ی اول مستقیماً همان چیزی را می‌گوید که تعریفِ «مرز» است و برای یاد گرفتن بهتر است؛ نسخه‌ی دوم به‌جای `n` بار پرسیدن، فهرست را یک بار می‌سازد.

## `safe_prefix` — یک خط، چون کتابخانه کارش را کرده

```rust
&text[..text.floor_char_boundary(max_bytes)]
```

سه حالتِ لبه در همین یک خط حل شده‌اند و هیچ‌کدام `if` لازم ندارند:

- `max_bytes` بزرگ‌تر از طول: `floor_char_boundary` به `text.len()` بسنده می‌کند، پس کلِ متن برمی‌گردد.
- `max_bytes` وسطِ یک حرف: عقب می‌رود تا اولین مرز.
- `max_bytes` برابرِ صفر: صفر خودش همیشه مرز است، پس `""`.

و برشِ `&text[..cut]` هرگز پنیک نمی‌کند، چون `cut` را از تابعی گرفته‌ایم که تعریفش «یک مرز بده» است.

اگر روی کامپایلرِ قبل از ۱.۹۱ هستی، معادلِ دستی‌اش این است:

```rust
let mut cut = if max_bytes > text.len() {
    text.len()
} else {
    max_bytes
};
while !text.is_char_boundary(cut) {
    cut -= 1;
}
&text[..cut]
```

آن `if` اول لازم است: بدونش `is_char_boundary` برای عددِ بزرگ `false` می‌دهد و حلقه از بالای طول شروع به شمردنِ معکوس می‌کند — که کار می‌کند ولی بی‌جهت کند است.

## `truncate_to_chars` — شمارنده، نه حساب

```rust
let mut seen = 0;
for (index, _) in text.char_indices() {
    if seen == max_chars {
        return &text[..index];
    }
    seen += 1;
}
text
```

نکته‌ی مرکزیِ کلِ درس همین‌جاست: `index` از خودِ `char_indices` بیرون آمده، پس **حتماً** مرزِ یک حرف است. برش امن است، بدونِ اینکه لازم باشد چیزی را بررسی کنی.

ترتیبِ داخلِ حلقه مهم است. اول بررسی، بعد شمردن. اگر جایشان را عوض کنی، برای `max_chars` برابرِ ۳ چهار حرف برمی‌گردانی.

و اگر حلقه تا آخر برود یعنی متن `max_chars` حرف یا کمتر داشته، پس کلِ `text` جواب است. توجه کن که هیچ‌جا `chars().count()` را از قبل حساب نکردیم — این کار یک گذرِ کاملِ اضافه روی متن است که در بیشترِ موارد لازم نیست.

خروجی `&str` است. هیچ تخصیصی نگرفتیم؛ فقط یک پنجره‌ی کوچک‌تر به همان بافر دادیم. اگر امضا را `String` می‌گذاشتی، هر بار صدا زدنش یک تخصیص می‌شد — و این تابع همان چیزی است که در یک حلقه روی هزار ردیف صدا زده می‌شود.

اگر روی این crate دستورِ `cargo clippy` را بگیری، اعتراض می‌کند:

```text
warning: the variable `seen` is used as a loop counter
  --> src\lib.rs:61:5
   |
61 |     for (index, _) in text.char_indices() {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: consider using: `for (seen, (index, _)) in text.char_indices().enumerate()`
```

حق هم دارد، و شمارنده‌ی دستی هنوز چیزی است که امروز باید بنویسی: `.enumerate()` یک آداپتورِ ایتریتور است و آن‌ها مالِ فاز ۲ هستند. همین بازنویسی بخشِ سومِ چالشِ این درس است، و وقتی ابزارش را داشتی ارزشِ برگشتن دارد.

## `truncated_with_ellipsis` — شرط روی حرف، نه بایت

```rust
if text.chars().count() <= max_chars {
    return text.to_string();
}
```

اینجا `chars().count()` لازم است و راهِ میان‌بری هم ندارد: باید بدانی *آیا* چیزی بریده شد یا نه، و این سؤال درباره‌ی تعدادِ حرف است.

نوشتنِ `text.len() <= max_chars` تستِ زیر را می‌شکند:

```rust
assert_eq!(truncated_with_ellipsis("سلام", 4), "سلام");
```

«سلام» چهار حرف است ولی هشت بایت. با شرطِ بایتی، `8 <= 4` غلط است، پس تابع فکر می‌کند باید ببرد، چهار حرف را برمی‌دارد (یعنی کلِ متن) و یک سه‌نقطه‌ی دروغ به آخرش می‌چسباند. کاربر می‌بیند «سلام…» و فکر می‌کند چیزی پنهان شده.

این دقیقاً همان تستی است که وجود دارد تا این اشتباه را بگیرد، و همان چیزی است که «حدِ ۲۰ کاراکتر» را در دنیای واقعی خراب می‌کند.

سه‌نقطه یک نویسه‌ی `…` است (`U+2026`) نه سه تا `.`. تستِ «هرگز بیشتر از `max_chars + 1` حرف» با سه نقطه رد می‌شود.

## `split_at_char` — همان الگو، با `split_at`

```rust
if seen == char_index {
    return text.split_at(index);
}
```

`split_at` معمولاً خطرناک است، ولی اینجا نیست — به همان دلیلِ همیشگی: `index` از `char_indices` آمده.

می‌شد به‌جایش دو برش نوشت: `(&text[..index], &text[index..])`. `split_at` همان کار را می‌کند، فقط یک بار بررسی می‌کند به‌جای دو بار، و صریح‌تر می‌گوید که دو تکه‌ی کنارِ هم می‌خواهی.

`(text, "")` در انتها حالتی است که `char_index` از تعدادِ حروف بزرگ‌تر بوده. تستی هست که برای هر متن و هر برشی بررسی می‌کند دو نیمه دوباره کلِ متن را می‌سازند:

```rust
let (head, tail) = split_at_char(sample, cut);
assert_eq!(format!("{head}{tail}"), sample);
```

اگر انتها را `("", text)` می‌گذاشتی این تست هم پاس می‌شد — و رفتارِ تابع برای `char_index` بزرگ عوض می‌شد بی‌آنکه کسی بفهمد. برای همین تستِ جداگانه‌ای هم هست که `split_at_char("سلام", 99)` را دقیقاً `("سلام", "")` می‌خواهد. مشخصات را از doc comment می‌خوانی، نه از تست.

## این درس واقعاً درباره‌ی چه بود

- **`&text[a..b]` بایت می‌شمارد.** در ASCII این نامرئی است؛ در فارسی نیست.
- **برشِ روی مرزِ بد پنیک می‌کند، نه خروجیِ خراب.** و این طراحی است، نه سخت‌گیری.
- **هر عددی که از `char_indices` بیرون بیاید امن است.** بیشترِ کدِ امنِ این فایل از همین یک حقیقت می‌آید.
- **`.get()` امن است چون نوعِ دیگری می‌دهد.** `Option<&T>` یعنی «شاید نباشد»، و کامپایلر مجبورت می‌کند به آن حالت فکر کنی.
- **«حداکثر N کاراکتر» باید با حرف پیاده شود، وگرنه یک باگ است که فقط کاربرِ غیرانگلیسی‌زبانت می‌بیند.**
