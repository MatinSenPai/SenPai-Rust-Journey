# ۰۳ — مبانی Cargo

`cargo` هم ساخت tool است، هم بسته manager، هم آزمون runner و هم ورودی formatter/linter. اگر در Python با `pip`، `venv` و `Makefile` کار کرده‌ای، Cargo بخش بزرگی از آن گردش کار را یکدست می‌کند.

## ساختار `Cargo.toml`

```toml
[package]
name = "p0-03-cargo-basics"
version = "0.1.0"
edition.workspace = true

[dependencies]
rand.workspace = true
```

`[package]` نام، نسخه و edition همین crate را تعریف می‌کند. `[dependencies]` وابستگی‌ها را می‌آورد. عبارت `rand.workspace = true` یعنی نسخه از `[workspace.dependencies]` در `Cargo.toml` ریشه خوانده شود تا همه‌ی درس‌ها یک نسخه‌ی pinشده داشته باشند.

## نسخه‌گذاری معنایی

در `MAJOR.MINOR.PATCH` پس از `1.0.0`، تغییر PATCH نباید API را بشکند، MINOR قابلیت سازگار می‌افزاید و MAJOR می‌تواند breaking باشد. پیش از `1.0.0`، افزایش MINOR نیز می‌تواند breaking باشد.

## فرمان‌های روزمره

| فرمان | کاربرد |
|---|---|
| `cargo check` | نوع-check سریع، بدون کد generation نهایی |
| `cargo build` | ساخت artifact در `target/debug/` |
| `cargo run` | ساخت در صورت نیاز و سپس اجرا |
| `cargo test` | کامپایل و اجرای تابع‌های `#[test]` |
| `cargo add <crate>` | افزودن dependency |
| `cargo fmt` | قالب‌بندی استاندارد |
| `cargo clippy` | یافتن الگوهای غیراصطلاحی |

## تمرین

تابع `format_greeting` در `src/lib.rs` را پیاده کن و این چرخه را انجام بده:

```sh
cargo check
cargo test
cargo run -- Matin
cargo run -- Matin 3
```

در بک‌اند هم همین چرخه را برای پردازشگرها داری: بررسی سریع، آزمون قرارداد و بعد اجرای سرویس.

```senpai-visual
{"kind":"concept","labels":["check","test","run"]}
```
