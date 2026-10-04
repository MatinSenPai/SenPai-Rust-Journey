# راه‌حل — ۳.۷.۱ هش‌کردنِ پسورد با argon2

کدِ کامل و تست‌شده در `src/lib.rs` کنارِ همین فایل است؛ `cargo test` در این پوشه هر ۱۹ تست را اجرا می‌کند. اینجا می‌گوییم هر قطعه چرا این شکل را دارد.

## `hash_password`

```rust
pub fn hash_password(password: &str, params: &Params) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::from(params.clone());
    argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("hashing an in-memory password with valid params cannot fail")
        .to_string()
}
```

`Argon2::from(params)` یک hasherِ `Argon2id` با نسخه‌یِ `0x13` و هزینه‌یِ تو می‌دهد. نمکِ تازه در هر فراخوانی باعث می‌شود یک پسورد هر بار هشِ متفاوتی بدهد. `.expect(...)` اینجا صادقانه است: ورودی یک `&str` در حافظه است و `Params` هنگامِ ساخته‌شدن اعتبارسنجی شده، پس شکستِ قابل‌بازیابی‌ای نمانده. با `verify_password` مقایسه کن که رشته‌یِ ذخیره‌شده از بیرونِ تابع می‌آید.

## `verify_password`

```rust
pub fn verify_password(password: &str, phc: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(phc) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}
```

`Argon2::default()` به‌عنوانِ verifier مشکلی ندارد چون الگوریتم، نسخه، هزینه و نمک از `parsed` خوانده می‌شوند؛ پیش‌فرض‌ها فقط جایی را پر می‌کنند که رشته چیزی نگفته. نوعِ برگشتی عمداً `bool` است: ردیفِ خراب و پسوردِ غلط به یک تصمیم می‌رسند، «رد کن»، پس caller کاری با تفاوتشان ندارد. این راهِ حلِ مثالِ `06` است. مقایسه‌یِ داخلی در زمانِ ثابت است (`Output` در کریتِ `password-hash` با `subtle` مقایسه می‌کند).

## `describe_hash` و `needs_rehash`

```rust
let parsed = PasswordHash::new(phc).ok()?;
let params = Params::try_from(&parsed).ok()?;
Some(HashInfo {
    algorithm: parsed.algorithm.to_string(),
    version: parsed.version?,
    memory_kib: params.m_cost(),
    /* iterations, lanes, salt, hash_len the same way */
})
```

`?` اینجا روی `Option` کار می‌کند، پس رشته‌ای که نسخه، نمک یا هش ندارد به‌سادگی `None` می‌دهد. `needs_rehash` بعدش سه مقایسه است به‌علاوه‌یِ «اگر توصیف نشد یا argon2id نبود، یعنی بله». از `<` استفاده می‌کند نه `!=`: هشی که از هدف قوی‌تر است مشکلی ندارد و تنزلش یک باگ است.

## `constant_time_eq`

```rust
if a.len() != b.len() {
    return false;
}
let mut diff = 0u8;
for (x, y) in a.iter().zip(b) {
    diff |= x ^ y;
}
diff == 0
```

بررسیِ طول می‌تواند زود خارج شود: طولِ یک هش یا توکن راز نیست. بعد از آن هر جفت XOR می‌شود و با OR در یک انباره جمع می‌شود، پس حلقه چه بایتِ اول فرق کند چه هیچ‌کدام، همان‌قدر کار می‌کند. این نسخه برایِ یادگیری است؛ کدِ تولید از کریتِ `subtle` استفاده می‌کند که جلوی تبدیلِ دوباره‌یِ این الگو به خروجِ زودهنگام را هم می‌گیرد.

## `UserStore`

```rust
pub fn login(&mut self, username: &str, password: &str) -> Result<(), AuthError> {
    let Some(stored) = self.users.get(username) else {
        verify_password(password, &self.dummy_hash);
        return Err(AuthError::InvalidCredentials);
    };
    if !verify_password(password, stored) {
        return Err(AuthError::InvalidCredentials);
    }
    // challenge: upgrade a weak stored hash here
    Ok(())
}
```

شاخه‌یِ کاربرِ ناشناس یک verifyِ واقعی انجام می‌دهد و جواب را دور می‌اندازد، پس هر دو شکست در زمانی تقریباً برابر `InvalidCredentials` برمی‌گردانند. هشِ ساختگی یک بار در `new` با پارامترهایِ خودِ store ساخته می‌شود تا هزینه‌اش با یک ردیفِ واقعی برابر باشد. `register` اول طول را می‌سنجد (با `chars().count()` نه `len()`، چون هر حرفِ فارسی دو بایت است)، بعد نام را، و فقط `hash_password(...)` را ذخیره می‌کند.

نسخه‌یِ چالش در `src/lib.rs` بعد از بررسیِ موفق یک آزمونِ `needs_rehash(stored, &self.params)` اضافه می‌کند و اگر درست بود ردیف را با هشِ تازه‌یِ پسوردِ خامی که همان لحظه در دست داری عوض می‌کند. ورودِ ناموفق زودتر برمی‌گردد، پس هرگز چیزی را ارتقا نمی‌دهد.

## درباره‌یِ پرسش‌هایِ «می‌توانی توضیح بدهی؟»

- **هش، نه رمزنگاری:** ورود فقط می‌پرسد «می‌خواند؟»، پس پسوردِ اصلی را هرگز دوباره لازم نداری. رمزنگاری کلید می‌خواهد و کلید رازِ دیگری است که دزدیدنش همه‌ی پسوردها را یکجا لو می‌دهد.
- **نمک:** تصادفی برایِ هر هش، پس پسوردهایِ برابر فرق می‌کنند و جدولِ از پیش‌ساخته باید برایِ هر ردیف دوباره ساخته شود. راز نیست و برای همین داخلِ رشته ذخیره می‌شود.
- **`SHA-256` در برابرِ `bcrypt` در برابرِ `Argon2id`:** سریع، کند با حافظه‌یِ کم، کند و حافظه‌سخت. RAM روی GPU یا ASIC ارزان موازی نمی‌شود.
- **کاربرِ ناشناس:** متنِ خطایِ یکسان کافی نیست؛ کارِ یکسان هم لازم است، وگرنه ساعت لو می‌دهد.
- **`==` روی رازها:** زمان به تعدادِ بایت‌هایِ ابتدایی که یکی بودند بستگی دارد. زمانِ ثابت همه‌ی بایت‌ها را می‌بیند.
