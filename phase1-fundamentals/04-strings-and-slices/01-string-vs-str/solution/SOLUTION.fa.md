# راه‌حل

```rust
pub fn longest_word(text: &str) -> &str {
    text.split_whitespace()
        .max_by_key(|word| word.len())
        .unwrap_or("")
}
```

امضای تابع از خود پیمایشگر مهم‌تر است: خروجی برشی از `text` است و فقط تا وقتی ورودی معتبر باشد اعتبار دارد. به‌دلیل حذف ضمنی طول عمر (Lifetime Elision)، وقتی یک ارجاع ورودی و یک ارجاع خروجی داریم، کامپایلر رابطه‌ی طول عمرشان را خودش استنباط می‌کند.

در `greet(&owned)`، نوع `String` صفت `Deref<Target = str>` را پیاده می‌کند و deref coercion، `&String` را رایگان به `&str` تبدیل می‌کند.

`max_by_key(|word| word.len())` طول byte را مقایسه می‌کند. برای متن فارسی، اگر معیار «تعداد حرف» باشد باید معیار را آگاهانه مثلاً با `chars().count()` تغییر دهی؛ برای grapheme کامل به crate مناسب Unicode نیاز است.
