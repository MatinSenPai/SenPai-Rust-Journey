# ۰۲ — نصب Rust و toolchainها

دستور‌های دقیق در [`docs/setup-guide.md`](../../docs/setup-guide.md) نگهداری می‌شوند. نصب را از همان‌جا انجام بده و بعد برگرد.

## `rustup` فقط installer نیست

`rustup` مدیر toolchain رسمی Rust است؛ نزدیک‌ترین نمونه در دنیای Python، `pyenv` است. یک toolchain شامل `rustc`، `cargo` و ابزار‌های هم‌نسخه‌ی دیگر است.

## channelها

- `stable`: انتخاب این دوره و تقریباً همه‌ی پروژه‌های محیط عملیاتی؛ انتشار پایدار دوره‌ای.
- `beta`: نسخه‌ی بعدی stable برای آزمایش.
- `nightly`: ساخت روزانه با قابلیت‌های آزمایشی پشت `#![feature(...)]`.

فایل `rust-toolchain.toml` در ریشه، channel این مخزن را pin می‌کند. بنابراین اجرای `cargo` در این پروژه به تنظیم سراسری دستگاه وابسته نمی‌ماند؛ شبیه `.python-version` در `pyenv`.

## edition چیست؟

edition راهی برای فعال‌کردن تغییر‌های محدود و opt-in زبان است. crateهای editionهای متفاوت می‌توانند در یک dependency graph کنار هم باشند. edition با نسخه‌ی کامپایلر یکی نیست و برخلاف مهاجرت Python 2 به 3 برای شکستن اکوسیستم طراحی نشده است.

## بررسی نصب

```sh
rustc --version
cargo --version
```

هر دو باید نسخه چاپ کنند و editor نیز برای فایل `.rs` تشخیص نوع و diagnostics مربوط به `rust-analyzer` را نشان دهد.

## مثال و مرز تشبیه

مثل این است که برای یک کارگاه، جعبه‌ابزار شماره‌دار داشته باشی؛ فایل پروژه می‌گوید از کدام جعبه استفاده شود. اما toolchain محیط کاملاً ایزوله‌ای مانند container نیست و وابستگی‌های سیستم‌عامل همچنان اثر دارند.

```senpai-visual
{"kind":"roadmap","labels":["rustup","toolchain","project"]}
```
