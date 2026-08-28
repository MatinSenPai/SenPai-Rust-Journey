# راه‌حل

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

این کد دقیقاً همون نسخه‌ی دستی‌نویس از درس ۱ هستش — کریت `thiserror` زحمتِ `Display`/`Error` رو برات می‌کشه و تولیدشون می‌کنه، اما نوشتن یه پیاده‌سازی `From`ِ سفارشی که با عملگر `?` بتونه به صورت خودکار تبدیل خطا رو انجام بده، کماکان به عهده‌ی *خودته*؛ چرا که فقط خودت می‌دونی هر `ParseIntError` قرار بوده به کدوم فیلد نسبت داده بشه.

```rust
pub fn load_and_parse(input: &str) -> anyhow::Result<Config> {
    use anyhow::Context;
    parse_config(input).context("failed to load application config")
}
```

متدِ `.context(...)` متدیه که `anyhow::Context` به هر نوعِ `<Result<T, E`ای که شرطِ `E: std::error::Error + Send + Sync + 'static` رو داشته باشه اضافه می‌کنه (که اینجا به لطف `#[derive(thiserror::Error)]` نوعِ `ConfigError` ما این شرط رو داره). این متد در واقع `<Result<T, ConfigError` رو می‌گیره و با پوشوندن (wrapping) خطای اصلی به جای دور ریختنش، اونو تبدیل می‌کنه به یه `<anyhow::Result<T` — به همین خاطره که فراخوانی `()err.source` تو اون تست هنوز هم خطای زیربناییِ `ConfigError` رو پیدا می‌کنه: دستورِ `.context(msg)` میاد یه زنجیره‌ی کوچیک می‌سازه، طوری که پیامِ `msg` میاد رو و خطای اصلی هم قرار می‌گیره زیرش، در حالی که خروجیِ متد `Display` از نوعِ `anyhow::Error` فقط بخش بالاییِ این زنجیره رو نشون می‌ده (مثلاً `"err.to_string() == "failed to load application config`)، اما متد `()source.` بهت اجازه می‌ده که تو این زنجیره پایین‌تر بری و ببینی زیرش چیه.

کلِ دلیل اینکه تو درس ۱ انقدر به خودمون زحمت دادیم تا `std::error::Error` رو با دقت و اصول پیاده‌سازی کنیم (به جای اینکه خیلی ساده از `String` برای خطاها استفاده بشه) همین بود: `anyhow::Context` — و به‌طور کلی تبدیل شدنِ خطا از طریق `?` به `anyhow::Result` — فقط و فقط واسه نوع‌هایی کار می‌کنه که traitِ واقعیِ `Error` رو پیاده‌سازی کرده باشن. یه خطایِ مبتنی بر `String` اصلاً نمی‌تونست وصل بشه به این زنجیره و خودش رو نشون بده.