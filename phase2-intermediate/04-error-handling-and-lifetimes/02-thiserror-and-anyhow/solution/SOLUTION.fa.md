# پاسخ تشریحی

```rust
impl From<std::num::ParseIntError> for ConfigError {
    fn from(source: std::num::ParseIntError) -> Self {
        ConfigError::InvalidNumber {
            field: "max_retries".to_string(),
            source,
        }
    }
}
```

این پیاده‌سازی با نسخه‌ی دستی درس قبل یکسان است. `thiserror`، `Display` و `Error` را تولید می‌کند، اما تبدیل اختصاصی `From` برای کار خودکار `?` را هنوز خودت می‌نویسی؛ چون فقط منطق برنامه می‌داند هر `ParseIntError` باید به کدام field نسبت داده شود.

```rust
pub fn load_and_parse(input: &str) -> anyhow::Result<Config> {
    use anyhow::Context;
    parse_config(input).context("failed to load application config")
}
```

`.context(...)` متدی است که `anyhow::Context` به هر `Result<T, E>` با شرط `E: std::error::Error + Send + Sync + 'static` اضافه می‌کند. `ConfigError` به کمک `#[derive(thiserror::Error)]` این شرط‌ها را دارد. متد، `Result<T, ConfigError>` را به `anyhow::Result<T>` تبدیل و error اصلی را wrap می‌کند، نه اینکه دور بریزد.

به همین دلیل `err.source()` در تست هنوز `ConfigError` زیرین را پیدا می‌کند. `.context(msg)` زنجیره‌ی کوچکی می‌سازد: پیام `msg` در بالا و error اصلی زیر آن. `Display` معمولی `anyhow::Error` فقط بالای زنجیره را نشان می‌دهد، پس `err.to_string() == "failed to load application config"` است، اما `.source()` اجازه می‌دهد لایه‌های پایین‌تر را پیمایش کنی.

این دقیقاً دلیل اهمیت `std::error::Error` است. `anyhow::Context` و تبدیل `?` به `anyhow::Result` فقط برای errorهای واقعی پیاده‌کننده‌ی آن trait کار می‌کنند؛ `String` نمی‌توانست وارد این زنجیره شود.
