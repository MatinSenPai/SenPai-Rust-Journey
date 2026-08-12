# ۰۶.۱ — module، سطح دسترسی و workspace

## `mod`: ساختن درخت کد

Rust کد crate را به درخت moduleها تقسیم می‌کند. `mod catalog { ... }` یک module فرزند می‌سازد و می‌توان همان تعریف را با `mod catalog;` به فایل `catalog.rs` منتقل کرد. مسیرها از ریشه‌ی crate با `crate::`، از parent با `super::` و از module فعلی با `self::` آغاز می‌شوند.

## همه‌چیز به‌طور پیش‌فرض private است

item بدون `pub` فقط در module خودش و فرزندان آن قابل‌دسترسی است. `pub(crate)` دسترسی را به کل crate می‌دهد، اما آن را وارد API عمومی مصرف‌کنندگان نمی‌کند. fieldهای struct نیز جداگانه سطح دسترسی دارند؛ publicبودن خود struct به‌معنای publicبودن fieldها نیست.

```senpai-visual
{"kind":"concept","labels":["crate root","catalog خصوصی","pub Anime","pub(crate) rating"]}
```

## `use` و بازصادرکردن با `pub use`

`use` فقط نامی کوتاه در scope می‌آورد. `pub use catalog::Anime;` علاوه بر آن، مسیر عمومی تازه‌ای می‌سازد تا کاربر `my_crate::Anime` بنویسد، بی‌آنکه ساختار داخلی `catalog` بخشی از قرارداد API شود. در backend این جداسازی اجازه می‌دهد فایل‌ها و moduleها را بعداً جابه‌جا کنی، بی‌آنکه import مصرف‌کنندگان بشکند.

## workspace

Cargo workspace چند package را با یک `Cargo.lock` و معمولاً یک پوشه‌ی `target` مدیریت می‌کند. dependencyها یک‌بار resolve می‌شوند، buildهای مشترک reuse می‌شوند و فرمان‌هایی مانند `cargo test --workspace` همه را یکجا بررسی می‌کنند.

تشبیه module به بخش‌های یک اداره کمک می‌کند: مراجعه‌کننده فقط با باجه‌ی عمومی کار دارد. مرز تشبیه این است که privacy در Rust مرز امنیت runtime نیست؛ قراردادی compile-time برای API است.

## تمرین تو

ساختار module، سطح دسترسی و re-exportهای `src/lib.rs` را کامل کن؛ سپس `CHECKPOINT.fa.md` و پاسخ تشریحی را بخوان.
