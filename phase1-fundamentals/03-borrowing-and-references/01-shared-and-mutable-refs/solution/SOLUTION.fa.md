# راه‌حل

```rust
pub fn count_vowels(s: &str) -> usize {
    s.chars().filter(|c| "aeiouAEIOU".contains(*c)).count()
}
```

تابع فقط می‌خواند، پس اشتراکی borrow کافی است و فراخوان مالکیت `text` را نگه می‌دارد. برای شمارش واکه‌های فارسی باید تعریف واکه را جداگانه مشخص کنی؛ این پیاده‌سازی عمداً فقط واکه‌های لاتین را می‌شمارد.

```rust
pub fn append_exclamation(s: &mut String) {
    s.push_str("!");
}
```

`&mut String` دسترسی موقت و انحصاری می‌دهد و تغییر روی مقدار خود فراخوان اعمال می‌شود.

```rust
pub fn swap_and_report(a: &mut i32, b: &mut i32) -> String {
    let (orig_a, orig_b) = (*a, *b);
    std::mem::swap(a, b);
    format!("swapped {orig_a} and {orig_b}")
}
```

عملیات dereference در `*a` و `*b` مقدارهای `Copy` را می‌خواند. داشتن دو ارجاع تغییرپذیر مجاز است، چون هرکدام یک `i32` جدا را قرض گرفته‌اند؛ این محدودیت برای هر مقدار بررسی می‌شود، نه برای کل برنامه.
