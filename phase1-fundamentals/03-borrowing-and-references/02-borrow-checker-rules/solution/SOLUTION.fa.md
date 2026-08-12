# راه‌حل

```rust
pub fn first_word_len_then_clear(s: &mut String) -> usize {
    let len = s.split_whitespace().next().unwrap_or("").len();
    s.clear();
    len
}
```

slice موقت `&str` همان statement به `usize` مستقل تبدیل می‌شود. پیش از `clear` دیگر borrow زنده‌ای لازم نیست. توجه: `len()` تعداد byteها را می‌دهد؛ نخستین واژه‌ی `"سلام"` طول byte برابر ۸ دارد، نه ۴.

```rust
pub fn make_greeting(name: &str) -> String {
    format!("Hello, {name}!")
}
```

راه‌حل dangling ارجاع یک ترفند نیست: داده‌ی دارای مالکیت برگردان تا مالکیت به فراخوان منتقل شود.

```rust
pub fn describe_and_grow(s: &mut String) -> String {
    let description = {
        let first_char = s.chars().next().unwrap_or(' ');
        format!("{} chars, starts with '{}'", s.len(), first_char)
    };
    s.push_str(" (grown)");
    description
}
```

خروجی `format!` یک `String` دارای مالکیت است. پس از آخرین استفاده از ارجاع‌ها، تغییر مجاز می‌شود. پیام تشخیصی نمونه معمولاً `E0502` است: borrow تغییرپذیر با borrow تغییرناپذیری که بعداً استفاده می‌شود overlap دارد و ممکن بود reallocation، ارجاع قبلی را آویزان کند.
