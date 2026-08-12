# ۰۴.۲ — `thiserror` و `anyhow`

## `thiserror`: تولید خودکار boilerplate درس قبل

`ConfigError` در درس قبل به `impl Display` دستی و `impl std::error::Error for ConfigError {}` خالی نیاز داشت. الگوی enum به‌اضافه‌ی `Display` و `Error` در پروژه‌های واقعی Rust آن‌قدر تکرار می‌شود که crate به نام `thiserror` فقط برای تولید آن از روی attributeها ساخته شده است:

```rust
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("missing required field: {0}")]
    MissingField(String),

    #[error("invalid number for field '{field}': {source}")]
    InvalidNumber {
        field: String,
        #[source]
        source: std::num::ParseIntError,
    },
}
```

`#[derive(thiserror::Error)]` همان `impl Display` و `impl std::error::Error` درس قبل را می‌سازد:

- `#[error("...")]` روی هر variant به پیام `Display` آن تبدیل می‌شود. `{0}` نخستین field در tuple variant و `{field}` یا `{source}` fieldهای نام‌دار struct variant هستند؛ همان syntax مربوط به `format!`، این بار داخل attribute.
- `#[source]` یک field را علت زیربنایی معرفی می‌کند و `Error::source()` را متصل می‌سازد. در درس قبل نیازی به override این متد نداشتیم، چون زنجیره‌ی خطایی برای نمایش نداشتیم.
- اگر `#[from]` را روی field آن نوع بگذاری، thiserror پیاده‌سازی `From` را هم خودکار می‌سازد؛ یک attribute به‌جای یک بلوک کامل `impl From<...> for ...`.

رفتار همان درس قبل است، اما کد نگهداری‌پذیرتر می‌شود. `thiserror` ماهیت خطای Rust را عوض نمی‌کند؛ همچنان `Display + Error` است و crate فقط boilerplate را می‌نویسد.

```senpai-visual
{"kind":"result","labels":["ConfigError","thiserror","anyhow::Context","گزارش نهایی"]}
```

## `anyhow`: وقتی فراخواننده لازم نیست نوع خطا را match کند

`thiserror` برای **کد کتابخانه** است؛ جایی که فراخواننده شاید بخواهد `match err { ConfigError::MissingField(f) => ..., ... }` بنویسد و هر شکست را جدا مدیریت کند.

اما در بالاترین لایه‌ی برنامه معمولاً فراخواننده‌ی دیگری وجود ندارد. `main()`، منطق بالای CLI یا handlerای که فقط باید خطای ۵۰۰ برگرداند و جزئیات را log کند، لازم نیست بداند کدام‌یک از چهل نوع خطا رخ داده؛ فقط می‌خواهد خطا را تا یک محل واحد برای ثبت و توقف بالا ببرد.

اینجا `anyhow::Result<T>` به کار می‌آید؛ نام کوتاه `Result<T, anyhow::Error>`. نوع `anyhow::Error` می‌تواند **هر** نوع پیاده‌کننده‌ی `std::error::Error` را wrap کند. به همین دلیل در دو درس این trait را درست پیاده کردیم و فقط `String` برنگرداندیم. `?` داخل تابع دارای خروجی `anyhow::Result<T>` هر error استاندارد را خودکار به `anyhow::Error` تبدیل می‌کند و به `From` جداگانه برای هر نوع نیازی ندارد.

قاعده‌ی عملی پروژه‌های حرفه‌ای: **`thiserror` در `lib.rs` و `anyhow` در `main.rs`.** کتابخانه enum دقیق و قابل‌match ارائه می‌دهد تا مصرف‌کننده در صورت نیاز واکنش اختصاصی داشته باشد. برنامه در بیرونی‌ترین لایه همه‌چیز را در `anyhow::Error` جمع می‌کند، چون دیگر فراخواننده‌ای برای match باقی نمانده است.

## `anyhow::Context`: خرده‌نان‌های مسیر انتشار خطا

```rust
use anyhow::Context;

pub fn load_and_parse(input: &str) -> anyhow::Result<Config> {
    parse_config(input).context("failed to load application config")
}
```

متد `.context("...")` بدون دورریختن error اصلی، یک پیام خوانا دور آن می‌پیچد. چاپ `anyhow::Error` پیام context را نشان می‌دهد و نمایش `{:#}` یا `{:?}` زنجیره‌ی زیرش را نیز در اختیار می‌گذارد.

متد `.with_context(|| format!("..."))` نسخه‌ی تنبل است. وقتی ساخت پیام هزینه دارد—مثلاً باید مقداری format شود—اجازه می‌دهد فقط در مسیر خطا آن هزینه را بدهی. در backend واقعی با call stack چندلایه، `.context(...)` در هر لایه پیام مبهم `invalid digit found in string` را به روایت مفیدی مانند `failed to load application config: invalid number for field 'max_retries': invalid digit found in string` تبدیل می‌کند؛ تفاوت میان stack trace و یک داستان قابل‌پیگیری.

این زنجیره را می‌توان مثل یادداشت هر ایستگاه روی بسته‌ی مرجوعی دید. هر لایه توضیح خودش را اضافه می‌کند، اما علت اصلی داخل بسته حفظ می‌شود. مرز تشبیه: `anyhow` جای error enum دقیق در API کتابخانه را نمی‌گیرد؛ اگر فراخواننده باید بر اساس نوع خطا تصمیم بگیرد، پاک‌کردن نوع با `anyhow` مناسب نیست.

## تمرین تو

`ConfigError` را با `#[derive(thiserror::Error)]` بازنویسی کن، رفتار `parse_config` را دقیقاً مانند درس قبل نگه دار و `load_and_parse` را با `anyhow::Context` پیاده کن.

## ایست بازرسی

پس از تمرین، `CHECKPOINT.fa.md` و سپس `solution/SOLUTION.fa.md` را بخوان.
