# ۰۴.۲ — sliceها

slice نمای قرض‌گرفته‌شده از یک بازه‌ی پیوسته‌ی عضو‌ها در مجموعه بزرگ‌تر است: تخصیص حافظه و copy ندارد و از اشاره‌گر همراه طول تشکیل می‌شود.

```rust
let s = String::from("hello world");
let hello: &str = &s[0..5];
let world: &str = &s[6..11];
let whole: &str = &s[..];
```

rangeها بر اساس byte هستند. اگر مرز وسط character چند-byteای بیفتد، slicing در runtime panic می‌کند. مثلاً `&"🦀ust"[0..1]` معتبر نیست؛ Rust داده‌ی خراب را بی‌صدا تولید نمی‌کند.

```rust
let nums = [1, 2, 3, 4, 5];
let middle: &[i32] = &nums[1..4];
```

`&[i32]` می‌تواند به بخشی از آرایه، `Vec<i32>` یا slice دیگر اشاره کند. تابع فقط‌خواندنی sequence معمولاً باید `&[T]` بگیرد، نه `&Vec<T>`؛ درست مانند ترجیح `&str` به `&String`.

مثال بک‌اند: parser می‌تواند sliceای از buffer شبکه را بدون copy بخواند. این نما مالک buffer نیست؛ اگر مالک از بین برود یا تغییر باعث جابه‌جایی شود، ارجاع نباید زنده بماند و کامپایلر همین رابطه را بررسی می‌کند.

```senpai-visual
{"kind":"borrowing","labels":["buffer","pointer + length","بدون copy"]}
```
