# ۰۱.۱ — متغیر، mutability و shadowing

## immutable به‌صورت پیش‌فرض

```rust
let x = 5;
x = 6; // compile error
```

`let` یک اتصال تغییرناپذیر می‌سازد. این پیش‌فرض خواندن کد را ساده می‌کند: اگر `x` یک‌بار مقدار گرفته، بعداً بی‌خبر عوض نمی‌شود. برای تغییر همان مقدار، `mut` را صریح کن.

```rust
let mut x = 5;
x = 6;
```

## shadowing با تغییر فرق دارد

```rust
let x = 5;
let x = x * 2;
let x = x.to_string();
```

هر `let` اتصال تازه‌ای می‌سازد و نام قبلی را می‌پوشاند. بنابراین نوع می‌تواند از `i32` به `String` عوض شود؛ یک اتصال دارای `mut` نمی‌تواند نوع خود را تغییر دهد.

برای counter و accumulator از `mut` استفاده کن. برای pipeline تبدیل—مثلاً ورودی خام، trimشده و parseشده—shadowing نام‌ها را کم می‌کند.

```rust
const MAX_RETRIES: u32 = 3;
```

`const` همیشه immutable، دارای نوع صریح و قابل محاسبه در زمان کامپایل است.

مثال بک‌اند: شمارنده‌ی retry یک مقدار متغیر است، ولی تبدیل درخواست parameter از `&str` به `u32` می‌تواند shadowing باشد. تشبیه shadowing به «پاک‌کردن مقدار قبلی» کامل نیست؛ lifetime و drop واقعی به ownership و استفاده‌های مقدار بستگی دارد.

```senpai-visual
{"kind":"concept","labels":["let","let mut","shadow"]}
```

توابع `src/lib.rs` را پیاده و `cargo test -p p1-01-01-variables-mutability-shadowing` را اجرا کن.
