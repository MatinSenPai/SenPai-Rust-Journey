# راه‌حل

```rust
pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("hashing an in-memory password should never fail")
        .to_string()
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}
```

در `hash_password` ورودی یک `&str` معتبر و درون حافظه است و با parameterهای معمول `Argon2::default()` شکست قابل‌بازیابیِ واقع‌بینانه‌ای ندارد؛ بنابراین `.expect(...)` همان invariant را آشکار می‌کند. اما `verify_password`، `hash` را از بیرون تابع می‌گیرد: شاید ستون database قطع شده یا مقدار PHC نباشد. `let-else` آن را به `false` تبدیل می‌کند، نه panic.

caller در login فقط یک تصمیم دارد: ورود یا رد. password غلط و hash خراب هر دو باید `401` شوند؛ جداکردنشان هم کمکی به کاربر نمی‌کند و هم جزئیات نامفید به مهاجم می‌دهد. `SaltString::generate(&mut OsRng)` هر بار salt تازه می‌سازد، اما salt در PHC string ذخیره می‌شود تا verify همان salt اولیه را بازخوانی کند؛ salt تازه هنگام verify هیچ‌وقت match نمی‌شود.

این را به زبان عملی نگه دار: encryption برای داده‌ای است که باید بعداً بازیابی شود و key آن نیز هدف حمله است. password را هرگز نباید بازیابی کنی. salt اقتصادِ rainbow table را خراب می‌کند و Argon2 با حافظهٔ زیاد، موازی‌سازی گستردهٔ حدس‌ها را گران می‌سازد. parameterهای جاسازی‌شده هم اجازه می‌دهند hash قدیمی با هزینهٔ زمان خودش verify شود و بعد از login موفق، تدریجی با هزینهٔ جدید rehash شود.
