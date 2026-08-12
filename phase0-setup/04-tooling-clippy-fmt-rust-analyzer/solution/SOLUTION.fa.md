# راه‌حل — ابزار‌ها

- `&String` → `&str`: تابع فقط‌خواندنی به مالکیت یا نوع مشخص `String` نیاز ندارد؛ `&str` هم literal و هم قرض‌گرفته‌شده `String` را می‌پذیرد.
- `name.len() == 0` → `name.is_empty()`: قصد کد را مستقیم بیان می‌کند.
- `return x * 2;` → `x * 2`: آخرین expression بدون semicolon خروجی بلوک است؛ `return` برای خروج زودهنگام مفید است.
- شاخه بر سر literalهای boolean → خود `flag`: مقایسه‌ی `flag == true` و بازگرداندن دوباره‌ی boolean زائد است.
- `char_count(&s.to_string())` → `char_count(s)`: ساخت `String` جدید تخصیص حافظه و copy غیرضروری است.
- loop با index → iteration مستقیم روی slice: ریسک off-by-one و indexing زائد حذف می‌شود.
- `.map(|x| double_it(x))` → `.map(double_it)`: closure فقط تابع را forward می‌کرد.
- `match` دستی روی `Option<i32>` → `unwrap_or_default()`: عملیات استاندارد و شناخته‌شده‌ی همان قرارداد است.

هیچ پیشنهاد clippy را کورکورانه نپذیر؛ نوع، قرارداد public و هزینه‌ی تخصیص حافظه را دوباره بررسی کن.
