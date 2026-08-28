# ۲.۵.۳ — `thiserror` در برابرِ `anyhow` و مرزِ کتابخانه/باینری

## در یک نگاه

بعد از این درس می‌توانی:

- دقیقاً توضیح بدهی `#[derive(thiserror::Error)]` چه کدی تولید می‌کند — با گذاشتنش کنارِ همان `Display`/`Error`ی که در ۲.۵.۱ و ۲.۵.۲ با دست نوشتی.
- برایِ یک گونه (variant) بینِ `#[source]` و `#[from]` یکی را انتخاب کنی، و بگویی چرا `#[from]` وقتی گونه به یک فیلدِ اضافه (مثلِ شماره‌ی خط) هم نیاز دارد، جواب نمی‌دهد.
- بینِ `thiserror` و `anyhow` برایِ یک تابعِ مشخص یکی را انتخاب کنی — بر اساسِ اینکه فراخواننده‌اش واقعاً لازم دارد رویِ نوعِ خطا `match` کند یا نه — و با `anyhow::Context` یک پیامِ خواناتر به خطا بچسبانی، دقیقاً همان لحظه که دارد بالا می‌رود.

**زمان:** حدود ۷۵ دقیقه · **پیش‌نیاز:**
[۲.۵.۲ — زنجیره‌ی منشأ و `Box<dyn Error>`](../02-error-source-chains/README.fa.md)، و به‌طورِ خاص [۲.۵.۱ — نوع‌های خطای سفارشی](../01-custom-error-types/README.fa.md) برایِ همان شکلِ دستی‌نویسی که امروز کنارِ نسخه‌ی ماکرویش می‌بینی

---

## چرا اهمیت دارد

۲.۵.۱ ازت خواست یک `enum` خطای خودت را بسازی و برایش با دست `Display` بنویسی — یک `match` با یک بازو به‌ازایِ هر گونه — به‌علاوه‌ی یک `impl std::error::Error` (حتی اگر خالی) و یک `impl From<...>` برایِ آن گونه‌ای که تبدیلش ساده بود. ۲.۵.۲ یک پله جلوتر برد: حالا `source()` را واقعی override کردی تا یک فراخواننده بتواند زنجیره‌ی خطا را تا ته دنبال کند، و `Box<dyn Error>` را دیدی برایِ وقتی که فقط می‌خواهی بگویی «همینجا یک خطا برگردان، فرقی نمی‌کند از چه نوعی».

حالا فرض کن این کار را بارِ دهم انجام می‌دهی. همان چهار تکه — enum، `match`ِ `Display`، `impl Error`، `impl From` — دوباره، برایِ یک نوعِ خطایِ کاملاً متفاوت. هیچ‌کدامشان سخت نیست؛ مشکل این است که *مکانیکی*اند. هر بار همان شکل را تایپ می‌کنی، و هر بار احتمال دارد یک بازوی `match` را فراموش کنی یا پیامش را کمی جور دیگری بنویسی. این دقیقاً همان دسته کاری است که کامپیوترها بهتر از آدم‌ها انجامش می‌دهند — و کریتِ `thiserror` دقیقاً همین را تولید می‌کند: از رویِ چند خط attribute، همان `Display` و `Error` و (وقتی بخواهی) `From` را برایت می‌سازد.

یک نخِ دومِ درس هم هست، از یک زاویه‌ی کاملاً متفاوت. فرضِ ۲.۵.۱ و ۲.۵.۲ این بود که فراخواننده‌ی تابعت شاید بخواهد رویِ نوعِ خطا `match` کند — برایِ همین یک `enum` با گونه‌های جدا ساختی، نه یک `String`. ولی همیشه این‌طور نیست. تابعِ `main`، منطقِ بالایِ یک ابزارِ CLI، یک پردازشگرِ HTTP که فقط می‌خواهد یک کدِ وضعیتِ ۵۰۰ برگرداند و جزئیات را لاگ کند — این‌ها معمولاً هیچ فراخواننده‌ای *ندارند* که بخواهد رویِ نوعِ خطا شاخه برود. برایِ این‌ها، ساختنِ یک `enum` دقیق، هزینه‌ای است بدونِ فایده. کریتِ `anyhow` دقیقاً برایِ همین‌جا ساخته شده.

امروز هر دویِ این‌ها را می‌بینی، و یک قاعده که می‌گوید کجا کدامش را انتخاب کنی.

---

## مفهوم

### یادآوری: همان شکل، این‌بار فقط برایِ به‌خاطر آوردن

قبل از دیدنِ ماکرو، بیا دوباره همان شکل را با دست بسازیم — این‌بار برایِ فایلی از امتیازهایِ یک انیمه، خط‌به‌خط `title,score`. همان سه تکه‌ای که در ۲.۵.۱ دیدی، به‌علاوه‌ی یک `source()`ِ واقعی مثلِ ۲.۵.۲:

```rust
#[derive(Debug)]
pub enum RatingsError {
    Io(std::io::Error),
    MissingScore {
        line: usize,
    },
    InvalidScore {
        line: usize,
        source: std::num::ParseIntError,
    },
}
```

```rust
impl fmt::Display for RatingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RatingsError::Io(source) => write!(f, "could not read ratings file: {source}"),
            RatingsError::MissingScore { line } => write!(f, "line {line}: missing score"),
            RatingsError::InvalidScore { line, source } => {
                write!(f, "line {line}: invalid score: {source}")
            }
        }
    }
}
```

```rust
impl std::error::Error for RatingsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RatingsError::Io(source) => Some(source),
            RatingsError::MissingScore { .. } => None,
            RatingsError::InvalidScore { source, .. } => Some(source),
        }
    }
}
```

```rust
impl From<std::io::Error> for RatingsError {
    fn from(source: std::io::Error) -> Self {
        RatingsError::Io(source)
    }
}
```

هیچ‌کدامِ این چهار تکه غریبه نیست — دقیقاً همان کاری است که در ۲.۵.۱ و ۲.۵.۲ یاد گرفتی. اجرا کن و ببین:

```sh
cargo run -p p2-05-03-thiserror-and-anyhow --example 01-hand-written-error
```

```text
line 2: missing score
line 5: invalid score: invalid digit found in string
  caused by: invalid digit found in string
could not read ratings file: The system cannot find the file specified. (os error 2)
  caused by: The system cannot find the file specified. (os error 2)
```

### همان چیز، پشتِ `#[derive(thiserror::Error)]`

حالا همان `enum`، همان پیام‌ها، همان `source()`، همان `From` — ولی این‌بار همه‌شان از رویِ چند attribute تولید می‌شوند:

```rust
#[derive(Debug, thiserror::Error)]
pub enum RatingsError {
    #[error("could not read ratings file: {0}")]
    Io(#[from] std::io::Error),

    #[error("line {line}: missing score")]
    MissingScore { line: usize },

    #[error("line {line}: invalid score: {source}")]
    InvalidScore {
        line: usize,
        #[source]
        source: std::num::ParseIntError,
    },
}
```

سه چیز دارد اینجا اتفاق می‌افتد:

- `#[error("...")]` رویِ هر گونه، همان پیامِ `Display` است — سینتکسِ جاگذاری‌اش دقیقاً همانِ `format!` است. `{0}` به اولین (و اینجا تنها) فیلدِ یک گونه‌ی توپلی اشاره می‌کند؛ `{line}` و `{source}` به فیلدهایِ نام‌دارِ یک گونه‌ی ساختاری.
- `#[source]` رویِ یک فیلد، آن را به‌عنوانِ چیزی که `Error::source()` باید برگرداند نشانه‌گذاری می‌کند — دقیقاً همان `match`ی که بالاتر با دست نوشتی.
- `#[from]` روی فیلدِ `Io`، هم آن را `#[source]` حساب می‌کند، هم علاوه بر آن `impl From<std::io::Error> for RatingsError` را هم می‌سازد — همان بلوکِ چهارم از بالا، رایگان.

اجرایش کن و مقایسه کن:

```sh
cargo run -p p2-05-03-thiserror-and-anyhow --example 02-thiserror-derive
```

```text
line 2: missing score
line 5: invalid score: invalid digit found in string
  caused by: invalid digit found in string
could not read ratings file: The system cannot find the file specified. (os error 2)
  caused by: The system cannot find the file specified. (os error 2)
```

حرف‌به‌حرف همان چیزی که بالا از نسخه‌ی دستی‌نویس گرفتی. `thiserror` ماهیتِ خطا را عوض نکرده — هنوز همان `Display` + `Error` است، همان دو trait که در ۲.۵.۱ شناختی — فقط دیگر لازم نیست خودت تایپشان کنی.

### `#[source]` بدونِ `#[from]` — جایی که `?` کافی نیست

`#[from]` فقط رویِ فیلدی کار می‌کند که *تمامِ* گونه است، چون `From::from` فقط همان یک مقدار را می‌گیرد و باید کلِ گونه را از دلش بسازد. گونه‌ی `Io` بالا همین شکل را دارد؛ گونه‌ی `InvalidScore` نه — یک `line` هم لازم دارد که هیچ `ParseIntError`ای نمی‌تواند تأمینش کند. مثالِ زیر همان `RatingsError` است، این‌بار فقط با همین دو گونه — `MissingScore` این‌جا لازم نیست:

```rust
fn read_len(path: &str) -> Result<usize, RatingsError> {
    let text = std::fs::read_to_string(path)?; // bare `?` — `#[from]` covers it
    Ok(text.len())
}

fn parse_score(line: usize, raw: &str) -> Result<u8, RatingsError> {
    raw.trim()
        .parse()
        .map_err(|source| RatingsError::InvalidScore { line, source })
}
```

```sh
cargo run -p p2-05-03-thiserror-and-anyhow --example 03-from-attribute
```

```text
read_len error: could not read ratings file: The system cannot find the file specified. (os error 2)
parse_score error: line 3: invalid score: invalid digit found in string
```

`read_len` یک `?` خامِ ساده می‌نویسد چون `#[from]` تبدیلِ لازم را از قبل تولید کرده. `parse_score` نمی‌تواند — `line` باید از جایی بیاید که فقط خودِ تابع می‌داند، نه `From::from`. این دقیقاً همان مرزی است که ۲.۵.۱ و ۲.۵.۲ هم، هر بار با دست، نشانت دادند: یک تبدیل فقط وقتی می‌تواند خودکار باشد که هیچ‌چیزِ اضافه‌ای — یک نامِ فیلد، یک شماره‌ی خط — لازم نداشته باشد. `#[from]` فقط نسخه‌ی ماکرودارِ همان مرز است.

### `anyhow::Error` — یک نوعِ خطایِ پویا، برایِ هر چیزی که `Error` باشد

۲.۵.۲ به تو `Box<dyn Error>` را داد: یک راه برایِ گفتنِ «همینجا یک خطا برمی‌گردد، فرقی نمی‌کند دقیقاً از چه نوعی» بدونِ اینکه نوعِ ملموسش را هم بنویسی. `anyhow::Error` همان ایده را می‌گیرد و رویش دو چیز اضافه می‌کند که بعداً می‌بینی — امکانِ چسباندنِ پیام موقعِ بالا رفتن، و برگشتن به نوعِ ملموس اگر لازم شد.

```rust
fn parse_score(raw: &str) -> Result<u8, std::num::ParseIntError> {
    raw.trim().parse()
}

fn run() -> anyhow::Result<()> {
    let score = parse_score("87")?;
    println!("parsed score: {score}");
    let score = parse_score("oops")?; // ParseIntError -> anyhow::Error, no From needed
    println!("parsed score: {score}");
    Ok(())
}
```

`anyhow::Result<T>` فقط خلاصه‌نویسیِ `Result<T, anyhow::Error>` است. نکته‌ی مهم اینجاست: `parse_score` اصلاً `RatingsError` برنمی‌گرداند — یک `std::num::ParseIntError`ِ خامِ کتابخانه‌ی استاندارد است، همان چیزی که خودِ من نه صاحبشم نه برایش `From` نوشته‌ام. با این‌حال `?` داخلِ تابعی که `anyhow::Result<T>` برمی‌گرداند، بدونِ هیچ کدِ اضافه‌ای تبدیلش می‌کند، چون هر نوعی که `std::error::Error` را پیاده کرده باشد، همین‌جوری قبول می‌شود:

```sh
cargo run -p p2-05-03-thiserror-and-anyhow --example 04-anyhow-basics
```

```text
parsed score: 87
Display : invalid digit found in string
Debug   : invalid digit found in string
```

اینجا `{}` و `{:?}` یک چیز را نشان می‌دهند، چون هنوز فقط یک لایه خطا داریم — هیچ `.context(...)`ی رویش نیامده. این دقیقاً چیزی است که زیربخشِ بعدی عوضش می‌کند.

### `anyhow::Context` — پیام‌گذاری روی خطا، همان لحظه‌ی بالا رفتنش

```rust
use anyhow::Context;

fn load_score(raw: &str) -> anyhow::Result<u8> {
    parse_score(raw).context("failed to load score from config")
}
```

`.context("...")` یک پیامِ خواناترِ انسانی را روی خطا می‌گذارد — بدونِ اینکه خطایِ اصلی را دور بیندازد. `{}` فقط همان پیامِ تازه را نشان می‌دهد؛ `{:?}` کلِ زنجیره را:

```sh
cargo run -p p2-05-03-thiserror-and-anyhow --example 05-anyhow-context
```

```text
Display : failed to load score from config
Debug   : failed to load score from config

Caused by:
    invalid digit found in string
```

حالا مقایسه کن با خروجیِ زیربخشِ قبل: همان `ParseIntError`، ولی این‌بار `{:?}` یک بخشِ «`Caused by:`» هم دارد، چون این‌بار واقعاً یک زنجیره داریم — پیامِ تازه بالایِ آن، خطایِ اصلی زیرش. `.with_context(|| ...)` نسخه‌ی تنبلِ همین است: وقتی ساختنِ پیام خودش هزینه دارد (مثلاً باید چیزی را `format!` کنی)، فقط زمانی که مسیرِ خطا واقعاً طی شود آن هزینه را می‌پردازی.

```senpai-visual
{"kind":"result","labels":["parse_score(raw)?","Err: رقمِ نامعتبر","context: شکستِ بارگذاری","Debug: پیام و زنجیره‌ی Caused by"]}
```

### مرزِ کتابخانه/باینری

حالا هر دو تکه را کنارِ هم بگذار. یک تابعِ کتابخانه‌ای — یعنی تابعی که *ممکن است* یک فراخواننده داشته باشد که می‌خواهد رویِ نوعِ خطا شاخه برود — همان `RatingsError`ِ دقیق را برمی‌گرداند. تابعی که هیچ فراخواننده‌ی دیگری ندارد — چون خودش آخرِ خط است — همه‌چیز را در `anyhow::Result` جمع می‌کند. این‌بار `RatingsError` فقط دو گونه دارد، `MissingScore` و `InvalidScore` — همان دو تایی که این مثال واقعاً لازم دارد:

```rust
fn parse_line(line: usize, text: &str) -> Result<(String, u8), RatingsError> {
    let (title, score) = text
        .split_once(',')
        .ok_or(RatingsError::MissingScore { line })?;
    let score = score
        .trim()
        .parse()
        .map_err(|source| RatingsError::InvalidScore { line, source })?;
    Ok((title.to_string(), score))
}
```

و باینری‌اش، رویِ همان تابع:

```rust
fn load_ratings(input: &str) -> anyhow::Result<Vec<(String, u8)>> {
    input
        .lines()
        .enumerate()
        .map(|(index, line)| parse_line(index + 1, line))
        .collect::<Result<Vec<_>, RatingsError>>()
        .context("failed to load ratings")
}
```

`load_ratings` دقیقاً همان تله‌ی توقفِ زودهنگام (short-circuiting) را به کار می‌برد که در ۲.۲.۳ با `.collect::<Result<Vec<_>, _>>()` دیدی: اولین `Err` کلِ نتیجه می‌شود، و بقیه‌ی خط‌ها اصلاً پردازش نمی‌شوند. نتیجه‌اش را اجرا کن:

```sh
cargo run -p p2-05-03-thiserror-and-anyhow --example 06-library-vs-binary-boundary
```

```text
Ok([("Frieren", 96), ("Bocchi the Rock!", 90)])
Display : failed to load ratings
Debug   : failed to load ratings

Caused by:
    0: line 2: invalid score: invalid digit found in string
    1: invalid digit found in string
```

این‌بار زنجیره دو لایه‌ی زیرِ پیامِ context دارد، نه یکی: پیامِ خودِ `RatingsError::InvalidScore` (لایه‌ی صفر)، بعد `ParseIntError`ی که آن از قبل به‌عنوانِ `source()`ش نگه داشته بود (لایه‌ی یک). `anyhow` این زنجیره را خودش پیدا می‌کند، از راهِ `Error::source()` — همان متدی که در ۲.۵.۲ یاد گرفتی و بالاتر دوباره override کردی.

**قاعده:** یک کتابخانه نوعِ خطایِ دقیق و قابلِ‌match نشان می‌دهد، چون نمی‌داند فراخواننده‌اش لازم دارد رویِ چه چیزی شاخه برود یا نه. یک باینری — لایه‌ی بیرونی که واقعاً اجرا می‌شود و خودش دیگر فراخواننده‌ای ندارد — می‌تواند به‌جایش `anyhow` را بردارد، چون پایین‌دستِ آن دیگر کسی نیست که بخواهد رویِ نوعِ خطا `match` کند.

```senpai-visual
{"kind":"concept","labels":["تابعِ خطاپذیر در کتابخانه","فراخواننده شاید match لازم داشته باشد؟","بله: enumِ thiserror","فراخواننده‌ی دیگری نیست: main()","باینری: anyhow::Error"]}
```

### کِی این قانون را می‌شکنی

این یک قانونِ مطلق نیست، یک پیش‌فرضِ خوب است. جایی که واقعاً می‌شکندش این است: یک «باینری» که در واقع خودش هم یک کتابخانه است. یک ابزارِ CLI که `main.rs`اش نازک است ولی منطقِ اصلی‌اش تویِ `lib.rs`ای زندگی می‌کند که کریت‌هایِ دیگر هم `use`ش می‌کنند؛ یا یک سرویس که خطاهایش آخرش به یک بدنه‌ی HTTP تبدیل می‌شوند و آن طرفِ خط — نه کدِ Rust، بلکه کلاینتِ API — واقعاً باید بتواند بینِ «۴۰۴» و «۵۰۰» فرق بگذارد. اینجا سؤالِ درست دیگر «این باینری است یا کتابخانه؟» نیست، بلکه «آیا *چیزی*، هرچه باشد، بعداً می‌خواهد رویِ این خطا شاخه برود؟» — و اگر جواب بله است، حتی تویِ یک `main.rs` هم `thiserror` را انتخاب کن. ۲.۵.۴ دقیقاً همین‌جا ادامه پیدا می‌کند: طراحیِ یک رده‌بندیِ خطا برایِ یک سرویسِ واقعی، جایی که این تصمیم دیگر یک‌بار برایِ همیشه گرفته نمی‌شود.

---

## دست‌به‌کد

```sh
cargo run -p p2-05-03-thiserror-and-anyhow --example 01-hand-written-error
cargo run -p p2-05-03-thiserror-and-anyhow --example 02-thiserror-derive
cargo run -p p2-05-03-thiserror-and-anyhow --example 03-from-attribute
cargo run -p p2-05-03-thiserror-and-anyhow --example 04-anyhow-basics
cargo run -p p2-05-03-thiserror-and-anyhow --example 05-anyhow-context
cargo run -p p2-05-03-thiserror-and-anyhow --example 06-library-vs-binary-boundary
```

بعد سه‌تای خراب:

```sh
cargo run -p p2-05-03-thiserror-and-anyhow --example 07-derive-error-needs-debug --features broken
cargo run -p p2-05-03-thiserror-and-anyhow --example 08-context-needs-trait-import --features broken
cargo run -p p2-05-03-thiserror-and-anyhow --example 09-from-needs-single-field --features broken
```

بعد این‌ها را امتحان کن:

۱. در `01-hand-written-error.rs` و `02-thiserror-derive.rs`، یک گونه‌ی تازه اضافه کن (مثلاً `EmptyFile`) — یک‌بار تویِ نسخه‌ی دستی‌نویس، یک‌بار تویِ نسخه‌ی `derive`. چند خط تویِ هرکدام لازم داشتی؟
۲. در `03-from-attribute.rs`، `#[from]` را از رویِ `Io` بردار و سعی کن `read_len` را همان‌طور با یک `?` خام نگه داری. چه خطایی می‌گیری، و چرا دقیقاً همان خطاست؟
۳. در `06-library-vs-binary-boundary.rs`، ورودیِ `bad` را طوری عوض کن که خطِ *اول* خراب باشد، نه دومی. شماره‌ی خط در پیام درست خودش را پیدا می‌کند؟

---

## خطاهایی که خواهی دید

### `E0277` — `#[derive(thiserror::Error)]` بدونِ `#[derive(Debug)]`

```text
error[E0277]: `RatingsError` doesn't implement `Debug`
  --> phase2-intermediate\05-error-handling\03-thiserror-and-anyhow\examples\07-derive-error-needs-debug.rs:11:10
   |
10 | #[derive(thiserror::Error)]
   |          ---------------- in this derive macro expansion
11 | pub enum RatingsError {
   |          ^^^^^^^^^^^^ the trait `Debug` is not implemented for `RatingsError`
   |
   = note: add `#[derive(Debug)]` to `RatingsError` or manually `impl Debug for RatingsError`
note: required by a bound in `std::error::Error`
  --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\error.rs:59:18
   |
59 | pub trait Error: Debug + Display {
   |                  ^^^^^ required by this bound in `Error`
   = note: this error originates in the derive macro `thiserror::Error` (in Nightly builds, run with -Z macro-backtrace for more info)
help: consider annotating `RatingsError` with `#[derive(Debug)]`
   |
11 + #[derive(Debug)]
12 | pub enum RatingsError {
   |

For more information about this error, try `rustc --explain E0277`.
```

**کامپایلر به چه اعتراض دارد:** `#[derive(thiserror::Error)]` دارد `impl std::error::Error for RatingsError` را می‌سازد، ولی خودِ trait اعلام کرده `Error: Debug + Display` — یعنی هر نوعی که `Error` باشد، باید از قبل `Debug` هم باشد. ۲.۵.۱ هم دقیقاً همین را نشان داد: `Error` به هر دو ابرصفتش نیاز دارد، و نوشتنِ یکی هیچ‌وقت دیگری را مجانی نمی‌دهد؛ `thiserror` هیچ‌کدام از دو طرفِ آن قاعده را عوض نکرده — فقط `Display` را برایت می‌سازد، نه `Debug` را.

**راه‌حل:** دقیقاً پیشنهادِ خودِ کامپایلر:

```rust
#[derive(Debug, thiserror::Error)]
pub enum RatingsError {
    #[error("line {0}: missing score")]
    MissingScore(usize),
}
```

**چرا این راه‌حل است:** `#[derive(Debug)]` یک پیاده‌سازیِ مکانیکی و رایگان است — دقیقاً همان چیزی که در ۲.۳.۴ دیدی. حالا هر دو نیمه‌یِ کرانِ `Error` برآورده شده‌اند، و `thiserror` می‌تواند بدونِ شکایت، `impl Error` را کاملش کند.

### `E0599` — `.context(...)` بدونِ `use anyhow::Context;`

```text
error[E0599]: no method named `context` found for enum `Result<T, E>` in the current scope
   --> phase2-intermediate\05-error-handling\03-thiserror-and-anyhow\examples\08-context-needs-trait-import.rs:15:22
    |
 15 |     parse_score(raw).context("failed to load score")
    |                      ^^^^^^^
    |
   ::: C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\anyhow-1.0.103\src\lib.rs:618:8
    |
618 |     fn context<C>(self, context: C) -> Result<T, Error>
    |        ------- the method is available for `Result<u8, ParseIntError>` here
    |
    = help: items from traits can only be used if the trait is in scope
help: trait `Context` which provides `context` is implemented but not in scope; perhaps you want to import it
    |
 10 + use anyhow::Context;
    |
help: there is a method `with_context` with a similar name
    |
 15 |     parse_score(raw).with_context("failed to load score")
    |                      +++++

For more information about this error, try `rustc --explain E0599`.
```

**کامپایلر به چه اعتراض دارد:** `.context()` روی خودِ `Result` تعریف نشده — یک متدِ اضافه‌شده به `Result<T, E>` است، از راهِ صفتِ `anyhow::Context`، فقط وقتی آن صفت در دامنه باشد. بدونِ `use anyhow::Context;`، متد از نظرِ کامپایلر اصلاً وجود ندارد.

**راه‌حل:** ایمپورت را اضافه کن:

```rust
use anyhow::Context;
```

**چرا این راه‌حل است:** پیشنهادِ اولِ کامپایلر همین است. پیشنهادِ دومش — تعویض به `.with_context()` — به‌تنهایی کمکی نمی‌کند: `with_context` رویِ دقیقاً همان صفتِ `Context`ی زندگی می‌کند که `context`، پس بدونِ آن `use`، دوباره همان خطایِ E0599 (صفت در دامنه نیست) را می‌گیری. (وقتی صفت واقعاً در دامنه باشد، `.with_context()` هم یک کلوژر می‌خواهد — `F: FnOnce() -> C` — نه یک رشته‌ی خام، پس `"failed to load score"` باید به `|| "failed to load score"` تبدیل شود — ولی این یک واقعیتِ جداگانه درباره‌ی امضایِ متد است، نه چیزی که این‌جا خرابش می‌کند.) راهِ درست همیشه اول همان است: صفتِ مناسب را در دامنه بیاور.

### خطایِ ماکرویِ `thiserror` — `#[from]` رویِ گونه‌ای با فیلدِ اضافه

```text
error: deriving From requires no fields other than source and backtrace
  --> phase2-intermediate\05-error-handling\03-thiserror-and-anyhow\examples\09-from-needs-single-field.rs:16:9
   |
16 |         #[from]
   |         ^^^^^^^
```

**کامپایلر به چه اعتراض دارد:** این یکی کدِ `E`ی ندارد — پیامِ خودِ ماکروی `thiserror` است، تشخیص‌داده‌شده هنگامِ expand شدنِ `#[derive]`، نه توسطِ خودِ `rustc`. `#[from]` قول می‌دهد `impl From<ParseIntError> for RatingsError` بسازد؛ ولی این تابع فقط همان یک مقدارِ `ParseIntError` را می‌گیرد و باید *تمامِ* گونه را از دلش بسازد. گونه‌ی `InvalidScore` هم به یک `line: usize` نیاز دارد که هیچ `ParseIntError`ای نمی‌تواند تأمینش کند — پس این قول از اساس قابلِ‌وفا نیست.

**راه‌حل:** `#[from]` را بردار، `#[source]` را نگه دار، و تبدیل را همان‌طور که در «مفهوم» دیدی با دست بنویس:

```rust
InvalidScore {
    line: usize,
    #[source]
    source: std::num::ParseIntError,
},
```

بعد در نقطه‌ی صدا زدن:

```rust
.map_err(|source| RatingsError::InvalidScore { line, source })?
```

**چرا این راه‌حل است:** `#[source]` هنوز `Error::source()` را برایت سیم‌کشی می‌کند — همان زنجیره را داری. فقط دیگر نمی‌توانی از `?` خامِ خودکار استفاده کنی، چون هیچ تبدیلِ خودکاری *نمی‌تواند* وجود داشته باشد وقتی یک فیلدِ اضافه لازم است. این همان مرزی است که زیربخشِ «`#[source]` بدونِ `#[from]`» نشانت داد.

---

## تمرین

### گرم‌کردن

<details>
<summary>برایِ <code>#[error("missing required field: {0}")] MissingField(String)</code>، فراخوانیِ <code>MissingField("name".to_string()).to_string()</code> دقیقاً چه رشته‌ای برمی‌گرداند؟</summary>

فکرت را قبل از دیدنِ پاسخ بنویس.

</details>

<details>
<summary>پاسخ</summary>

```text
missing required field: name
```

`{0}` به همان اولین (و تنها) فیلدِ گونه‌ی توپلی اشاره می‌کند — دقیقاً همان‌جوری که `format!` عمل می‌کرد.

</details>

<details>
<summary><code>#[derive(thiserror::Error)] enum Foo { #[error("bad")] Bad }</code> — بدونِ <code>#[derive(Debug)]</code> — کامپایل می‌شود؟</summary>

فکرت را بنویس — `std::error::Error` چه چیزی را پیش‌نیاز اعلام کرده؟

</details>

<details>
<summary>پاسخ</summary>

نه. `std::error::Error: Debug + Display` — و `#[derive(thiserror::Error)]` فقط `Display` را می‌سازد. کدِ خطا `E0277` است.

</details>

<details>
<summary>گونه‌ای داری با <code>Io(#[from] std::io::Error)</code>. یک <code>?</code>ِ خام روی یک <code>&lt;Result&lt;T, std::io::Error&gt;</code>، داخلِ تابعی که <code>&lt;Result&lt;T, Foo&gt;</code> برمی‌گرداند، کامپایل می‌شود؟</summary>

فکرت را بنویس — `#[from]` دقیقاً چه چیزی تولید می‌کند؟

</details>

<details>
<summary>پاسخ</summary>

بله. `#[from]` یک `impl From<std::io::Error> for Foo` می‌سازد، و همین کافی است تا `?` تبدیل را خودکار انجام بدهد.

</details>

<details>
<summary>درست یا غلط: <code>anyhow::Error</code> فقط می‌تواند نوع‌هایِ خطایی را بپیچد که در همان crate تعریف شده‌اند.</summary>

فکرت را بنویس.

</details>

<details>
<summary>پاسخ</summary>

غلط. `anyhow::Error` هر نوعی را می‌پذیرد که `std::error::Error` را پیاده کرده باشد — حتی یک نوعِ کاملاً بیگانه مثلِ `std::num::ParseIntError`، همان‌طور که در `04-anyhow-basics` دیدی.

</details>

<details>
<summary>یک تابعِ کتابخانه‌ای خطایِ اعتبارسنجی می‌دهد که فراخواننده‌اش شاید بخواهد رویِ آن <code>match</code> کند. باینری‌ای که آن کتابخانه را صدا می‌زند، هیچ فراخواننده‌ای برایِ خودش ندارد. کدام یکی <code>thiserror</code> برمی‌گرداند و کدام <code>anyhow</code>؟</summary>

فکرت را بنویس — قاعده‌ی مرزِ کتابخانه/باینری را به زبانِ خودت بگو.

</details>

<details>
<summary>پاسخ</summary>

کتابخانه `thiserror` (یک enum دقیق) برمی‌گرداند، چون فراخواننده‌اش شاید لازم داشته باشد شاخه برود. باینری `anyhow::Result` برمی‌گرداند، چون پایین‌دستِ آن دیگر کسی نیست که بخواهد `match` کند.

</details>

### تعمیر

هر سه مثالِ خراب را درست کن:

۱. `examples/07-derive-error-needs-debug.rs` را طوری درست کن که کامپایل شود — بدونِ اینکه هیچ `#[error("...")]`ای را عوض کنی.
۲. `examples/08-context-needs-trait-import.rs` را طوری درست کن که کامپایل شود. پیشنهادِ دومِ کامپایلر (`.with_context()`) را امتحان نکن — قبلش پیش‌بینی کن چرا کار نمی‌کند، بعد خودت را با اجرا کردنش تصحیح کن.
۳. `examples/09-from-needs-single-field.rs` را طوری درست کن که کامپایل شود، بدونِ اینکه `line` را از گونه‌ی `InvalidScore` حذف کنی.

### پیاده‌سازی

دو تابع در `src/lib.rs`، رویِ نوعِ `WatchNoteError` که از قبل کاملاً نوشته شده — چون پر کردنِ `#[error("...")]` کارِ یک `todo!()` نیست، یک attribute که کدِ اجرایی نیست:

```sh
cargo test -p p2-05-03-thiserror-and-anyhow
```

`parse_watch_note()` یک خطِ `"<episode>:<note>"` را پارس می‌کند؛ `load_watch_notes()` همان تابع را رویِ هر خط از یک ورودیِ چندخطی صدا می‌زند و نتیجه را در `anyhow::Result` جمع می‌کند. کامنتِ مستنداتِ هرکدام دقیقاً می‌گوید انتظارِ خروجی و پیامِ خطا چیست — چیزی را حدس نزن.

### بساز

یک گونه‌ی تازه به `WatchNoteError` اضافه کن، برایِ یک نوعِ خرابیِ دیگر که این فرمت می‌تواند داشته باشد (مثلاً یادداشتِ خالی بعد از `:`، یا شماره‌ی اپیزودِ صفر). خودت تصمیم بگیر پیامِ `#[error("...")]`اش چه باشد، و اینکه اصلاً به `#[source]` یا `#[from]` نیاز دارد یا نه — بیشترِ خرابی‌ها ندارند. `parse_watch_note()` را طوری عوض کن که گونه‌ی تازه را واقعاً برگرداند، و در یک کامنت بنویس چرا آن تصمیم را گرفتی.

### چالش (اختیاری)

یکی از گونه‌های `WatchNoteError` (یا آن یکی که در «بساز» نوشتی) را طوری عوض کن که به‌جایِ پیچیدنِ یک خطایِ استانداردِ کتابخانه (مثلِ `ParseIntError`)، یک نوعِ خطایِ *دیگرِ خودت* را با `#[from]` بپیچد — یک `enum` کوچکِ تازه، دو-سه گونه، فقط برایِ همین. بعد بررسی کن `anyhow::Context` هنوز هم زنجیره را تا سه لایه پایین درست نشان می‌دهد یا نه. این دقیقاً همان مسئله‌ای است که ۲.۵.۴ به‌طورِ کامل حلش می‌کند: ترکیبِ چند نوعِ خطایِ کتابخانه‌ای در یک رده‌بندیِ خطایِ سراسریِ سرویس.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| `thiserror` | ماکروی derive که `Display`، `Error` و (با `#[from]`) `From` را از رویِ attributeها می‌سازد | نوعِ خطایِ کتابخانه‌ای، به‌جایِ نوشتنِ دستی |
| `#[error("...")]` | پیامِ `Display`، به‌ازایِ هر گونه | همان سینتکسِ `format!`، رویِ خودِ گونه |
| `#[source]` | این فیلد را به‌عنوانِ `Error::source()` نشانه‌گذاری کن | زنجیره‌ی خطا (۲.۵.۲) |
| `#[from]` | `#[source]` به‌علاوه‌ی یک `From` خودکار | فقط وقتی فیلد تمامِ گونه است |
| `anyhow::Error` | یک نوعِ خطایِ پویایِ یکتا، برایِ هرچه `std::error::Error` باشد | آخرین لایه، جایی که کسی روی نوعِ خطا `match` نمی‌کند |
| `.context(...)` / `.with_context(...)` | پیامِ خواناترِ انسانی روی خطا، بدونِ دور انداختنِ اصلی | خطا موقعِ بالا رفتن از میانِ چند لایه |
| مرزِ کتابخانه/باینری | کتابخانه دقیق و قابل‌match، باینری با `anyhow` جمعش می‌کند | تصمیمِ نوعِ برگشتیِ هر تابع |

### الان می‌دانی

- `#[derive(thiserror::Error)]` همان `Display` و `Error`ی را می‌سازد که در ۲.۵.۱/۲.۵.۲ با دست نوشتی — نه بیشتر، نه کمتر — و هنوز به `#[derive(Debug)]`ِ خودت نیاز دارد.
- `#[error("...")]` همان سینتکسِ `format!` را دارد؛ `#[source]` متدِ `source()` را سیم‌کشی می‌کند؛ `#[from]` علاوه بر آن یک `From` هم می‌سازد — فقط وقتی فیلد تمامِ گونه باشد.
- `anyhow::Error` هر نوعی را می‌پذیرد که `std::error::Error` باشد، حتی نوع‌هایِ کاملاً بیگانه؛ `?` داخلِ تابعی که `anyhow::Result<T>` برمی‌گرداند، این تبدیل را همیشه رایگان انجام می‌دهد.
- `.context(...)` یک پیامِ تازه روی زنجیره می‌گذارد، بدونِ دور انداختنِ خطایِ اصلی؛ `{}` فقط پیامِ تازه را نشان می‌دهد، `{:?}` کلِ زنجیره را.
- کتابخانه نوعِ خطایِ دقیق و قابلِ‌match نشان می‌دهد چون نمی‌داند فراخواننده‌اش لازم دارد شاخه برود یا نه؛ باینری — لایه‌ی آخر، بدونِ فراخواننده‌ای برایِ خودش — می‌تواند به `anyhow` جمعش کند.
- این قانون مطلق نیست: هر جا *چیزی*، حتی داخلِ همان باینری، ممکن است بعداً بخواهد رویِ خطا شاخه برود، جوابِ درست همان `thiserror` است.

### بعداً کامل‌تر می‌بینی

- **طراحیِ یک رده‌بندیِ خطا برایِ یک سرویسِ واقعی، وقتی چند نوعِ خطایِ کتابخانه‌ای باید کنارِ هم زندگی کنند** — [۲.۵.۴ — طراحیِ رده‌بندیِ خطا برای یک سرویس](../04-error-taxonomy-for-a-service/README.fa.md)
- **این دقیقاً همان قاعده، رویِ یک سرویسِ واقعیِ Axum که خطاهایش باید به بدنه‌هایِ HTTP سازگار تبدیل شوند** — [۳.۷.۱ — پاکت‌هایِ خطایِ یکدست](../../../phase3-backend-foundations/07-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.fa.md)

### می‌توانی توضیح بدهی؟

- `#[derive(thiserror::Error)]` دقیقاً چه چیزی را برایت می‌سازد، و چه چیزی را نه؟
- چرا `#[from]` رویِ گونه‌ای که یک فیلدِ اضافه (مثلِ `line`) هم دارد، کامپایل نمی‌شود؟
- چرا `?` داخلِ تابعی که `anyhow::Result<T>` برمی‌گرداند، حتی برایِ یک نوعِ خطایِ کاملاً بیگانه هم کار می‌کند؟
- `.context(...)` دقیقاً چه چیزی را عوض می‌کند که `{}` و `{:?}` رویِ خطایِ نتیجه فرق می‌کند؟
- قاعده‌ی مرزِ کتابخانه/باینری را با یک مثالِ واقعی از این درس توضیح بده — و بعد یک مثال بزن از جایی که این قاعده را می‌شکنی.

---

## بیشتر

- [مستنداتِ `thiserror`](https://docs.rs/thiserror/latest/thiserror/) — فهرستِ کاملِ attributeها، از جمله `#[error(transparent)]` که امروز ندیدیمش.
- [مستنداتِ `anyhow`](https://docs.rs/anyhow/latest/anyhow/) — به‌خصوص بخشِ مقایسه با `thiserror`، از زبانِ خودِ نویسنده‌ی هر دو کریت.
- [مستنداتِ صفتِ `anyhow::Context`](https://docs.rs/anyhow/latest/anyhow/trait.Context.html) — امضایِ دقیقِ `.context()` و `.with_context()`.
- [کتابِ Rust CLI — رسیدگی به خطا](https://rust-cli.github.io/book/tutorial/errors.html) — همین قاعده‌ی مرزِ کتابخانه/باینری، از زاویه‌ی یک ابزارِ خط‌فرمانِ واقعی.
