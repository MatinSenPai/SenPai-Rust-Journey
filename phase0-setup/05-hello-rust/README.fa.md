# ۰۵ — سلام Rust

نخستین برنامه‌ی واقعی تو عمداً کوچک است تا چرخه‌ی `cargo new → edit → cargo run` را یک‌بار کامل ببینی.

## `fn main`

هر فایل اجرایی crate دقیقاً یک `fn main() { ... }` دارد؛ entry point برنامه. library crate به `main` نیاز ندارد و API خود را از `lib.rs` ارائه می‌کند.

## چرا `println!` تابع نیست؟

علامت `!` یعنی macro. `println!` هنگام کامپایل گسترش می‌یابد و می‌تواند سازگاری format string و argumentها را پیش از اجرا بررسی کند. `vec!`، `format!` و `todo!` نیز macro هستند.

## تمرین

تابع `describe_journey` در `src/lib.rs` را پیاده کن؛ `main.rs` را تغییر نده.

```sh
cargo test -p p0-05-hello-rust
cargo run -p p0-05-hello-rust
```

تشبیه macro به «تابعی که زودتر اجرا می‌شود» کامل نیست: macro روی syntax کار می‌کند و کد تولید می‌کند؛ صرفاً اجرای زودهنگام یک تابع معمولی نیست.

```senpai-visual
{"kind":"concept","labels":["fn main","println!","terminal"]}
```
