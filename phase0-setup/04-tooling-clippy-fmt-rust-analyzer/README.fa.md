# ۰۴ — ابزار‌ها: clippy، fmt و rust-analyzer

کد این درس از ابتدا کامپایل می‌شود و آزمون‌ها سبزند. مأموریت تو تغییر رفتار نیست؛ باید با ابزار‌های روزمره‌ی Rust آن را اصطلاحی‌تر کنی.

- `cargo clippy`: linter رسمی Rust؛ شبیه `ruff` یا `pylint`، با پیشنهادهایی برای خوانایی، کارایی و جلوگیری از bug.
- `cargo fmt`: formatter استاندارد؛ نزدیک به `black`. قالب‌بندی در تیم Rust تصمیم شخصی نیست.
- `rust-analyzer`: language سرور برای autocomplete، نوع hint، jump-to-definition و پیام تشخیصی داخل editor.

## تمرین

1. `cargo test` را اجرا و رفتار پایه را ثبت کن.
2. `cargo clippy` را اجرا کن و دلیل هر warning را بخوان.
3. همه‌ی warningهای `src/lib.rs` را بدون تغییر خروجی توابع رفع کن؛ آزمون‌ها باید سبز بمانند.
4. قالب یک تابع را به‌هم بزن و `cargo fmt` را ببین. سپس `cargo fmt -- --check` را اجرا کن.

در یک سرویس بک‌اند، clippy جای آزمون قرارداد API را نمی‌گیرد؛ فقط کلاس دیگری از ایراد‌ها را آشکار می‌کند. پیشنهاد خودکار نیز حکم قطعی نیست و باید در context بررسی شود.

```senpai-visual
{"kind":"result","labels":["source","clippy + fmt","CI"]}
```
