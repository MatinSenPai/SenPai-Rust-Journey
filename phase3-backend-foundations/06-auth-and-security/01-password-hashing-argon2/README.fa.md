# ۰۶.۱ — هش‌کردن password با `argon2`

## password plaintext هرگز ذخیره نمی‌شود

اگر ستون `password` در جدول `users` خودِ password را داشته باشد، نشت جدول یعنی نشت رمز واقعی کاربران؛ رمزی که احتمالاً جای دیگری هم تکرار کرده‌اند. راه‌حل encrypt کردنِ چیزی که بعداً لازم نیست بازش کنی نیست. ما فقط می‌خواهیم بدانیم ورودی با مقدار ذخیره‌شده می‌خواند یا نه، پس **هش یک‌طرفه (one-way hash)** نگه می‌داریم. همان غریزه‌ای که در Django در `User.objects.create_user(...)` و `check_password()` داشته‌ای.

```senpai-visual
{"kind":"concept","labels":["password ورود","salt تصادفی","PHC hash ذخیره‌شده"]}
```

## salt و rainbow table

هش ساده مثل `sha256("hunter2")` قطعی است؛ همین امکان ساختن **rainbow table**، یعنی جدول آمادهٔ `hash → password` برای میلیون‌ها password رایج، را می‌دهد. salt دادهٔ تصادفی تازه برای هر password است. با آن، دو کاربر با password یکسان hashهای کاملاً متفاوت می‌گیرند. salt مخفی نیست و در کنار hash ذخیره می‌شود؛ ارزشش در تصادفی و یکتابودن برای هر hash است. مهاجم باید جدول آماده را برای تک‌تک saltها از نو بسازد، که همان هزینهٔ brute-force هر ردیف است.

## چرا Argon2، نه SHA-256 یا bcrypt؟

- `SHA-256` عمداً سریع است؛ برای checksum خوب، برای password فاجعه است چون GPU می‌تواند میلیاردها حدس در ثانیه بزند.
- `bcrypt` عمداً کند است، اما فقط CPU-hard است و حافظهٔ ثابتِ کمی می‌خواهد؛ سخت‌افزار موازی هنوز از آن سود می‌برد.
- `Argon2id` memory-hard است: هر حدس باید مقدار قابل‌تنظیمی حافظهٔ واقعی مصرف کند. تکثیر حافظه برای GPU/FPGA/ASIC ارزان و ساده نیست؛ بنابراین حملهٔ موازی را گران می‌کند. این انتخاب جدید و پیشنهادی OWASP است و Django نیز پشتیبانی درجه‌یک از آن دارد.

## خواندن hash ذخیره‌شده

`hash_password` رشته‌ای در قالب PHC برمی‌گرداند:

```
$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$RdescudvJCsgt3ub+b+dWRWJTmaaJObG
```

یک ستون `TEXT` کافی است: variant الگوریتم، نسخه، هزینه‌های memory/time/parallelism، salt و خروجی hash همگی در همین رشته هستند. چون parameterها همراه hash می‌آیند، بعداً هزینه را بالا می‌بری ولی hashهای قدیمی همچنان verify می‌شوند؛ هنگام login موفق می‌توانی تدریجی rehash کنی.

## کد starter را دنبال کن

- `hash_password(password: &str) -> String`: با `SaltString::generate(&mut OsRng)` salt می‌سازد، سپس `Argon2::default().hash_password(...)` را به رشتهٔ PHC تبدیل می‌کند.
- `verify_password(password: &str, hash: &str) -> bool`: `PasswordHash::new(hash)` را parse می‌کند و `verify_password(...)` را اجرا می‌کند. password غلط و hash خراب، هر دو برای caller فقط «login رد شود» هستند.

در `src/lib.rs` این دو `todo!()` را کامل کن و اجرا کن:

```sh
cargo test -p p3-06-01-password-hashing-argon2
```

## قدم بعدی

بعد از تست، `solution/SOLUTION.fa.md` را بخوان.
