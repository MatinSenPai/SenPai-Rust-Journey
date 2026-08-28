# ۲.۵.۱ — نوع‌های خطای سفارشی و `std::error::Error`

## در یک نگاه

بعد از این درس می‌توانی:

- یک enum خطا طراحی کنی که هر گونه‌اش دقیقاً یک جور شکست را نشان می‌دهد — نه یک ساختارِ تکی با یک پیامِ رشته‌ای — و بگویی این طراحی چه چیزی به فراخواننده می‌دهد که یک `String` یا یک `Box<dyn Error>` نمی‌دهد.
- صفتِ `std::error::Error` را برایِ نوعِ خودت پیاده کنی، و دقیقاً بگویی این صفت چه دو صفتِ دیگری را پیش‌نیاز می‌گیرد و چه یک متدی خودش دارد.
- کنارِ یک `Debug` مشتق‌شده، یک `Display` دستی بنویسی، و یک `match` بسازی که بسته به اینکه کدام گونه شکست خورده، پاسخِ متفاوتی بدهد.

**زمان:** حدود ۵۰ دقیقه · **پیش‌نیاز:**
[۲.۳.۴ — مشتق‌های استاندارد، دستی پیاده‌سازی‌شده](../../03-traits-and-generics/04-standard-derives-by-hand/README.fa.md) ·
[۱.۶.۵ — `From` و تبدیلِ خطا](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.fa.md)

---

## چرا اهمیت دارد

از فاز ۱ تا این‌جا، دو بار دقیقاً همین دیوار جلوی راهت سبز شده و هر دو بار درس یک قدم رفته و ایستاده. در [۱.۶.۳](../../../phase1-fundamentals/06-absence-and-failure/03-result-and-question-mark/README.fa.md) با `Result<T, String>` شروع کردی — برایِ اولین مواجهه با `?` انتخابِ درستی بود. همان‌جا کامپایلر یک راهِ دیگر هم پیشنهاد داد: `Result<(), Box<dyn std::error::Error>>`، و درس گفت خودِ صفتِ `Error` و اینکه `Box<dyn Error>` دقیقاً چطور کار می‌کند، امروز می‌آید. در [۱.۶.۵](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.fa.md) یک قدم جلوتر رفتی: یک enum خطا با چند گونه‌ی مجزا نوشتی (`ReadingError`)، با یک `impl From` که `?` را وادار به تبدیلِ خودکار می‌کرد. آن enum ساختار داشت — ولی هنوز «یک خطایِ واقعیِ Rust» نبود، چون هیچ‌کدام از `Display` یا `std::error::Error` رویش پیاده نشده بود؛ فقط `#[derive(Debug)]` داشت.

امروز همان قرض را صاف می‌کنی. اول ببین چرا یک `String` اصلاً کافی نیست: فراخواننده‌ای که یک `String` می‌گیرد، هیچ کارِ ساختاریافته‌ای باهاش نمی‌تواند بکند جز چاپش کردن یا — بدتر — گشتنِ متنش با `contains`، دقیقاً همان رشته‌کاویِ شکننده‌ای که پایتون با `except Exception as e: if "not found" in str(e):` انجام می‌دهد. یک `Box<dyn Error>` یک قدم جلوتر است — لااقل پشتش یک صفتِ واقعی هست — ولی همچنان نوعِ ملموسش را پاک می‌کند: بدونِ یک عملیاتِ اضافه‌ی downcast، فراخواننده نمی‌تواند `match` کند کدام گونه‌ی خطا رخ داده. آن downcast کردن و دنبال‌کردنِ زنجیره‌ی منشأ، تمامِ موضوعِ [۲.۵.۲](../02-error-source-chains/README.fa.md) است؛ امروز روی حالتِ ساده‌تر و رایج‌ترِ یک خطایِ خودکفا می‌مانیم — یک enum که خودش، بدونِ هیچ جعبه‌ای دورش، ساختار را نگه می‌دارد و `match` می‌شود.

سؤالِ دومی هم هست: چرا اصلاً باید یک صفتِ استاندارد به‌نامِ `Error` پیاده کنی، وقتی یک enum ساده با `Debug` مشتق‌شده به همان اندازه چاپ می‌شود؟ چون «خطا بودن» در Rust یک قراردادِ اکوسیستمی است. [۲.۳.۳](../../03-traits-and-generics/03-from-into-tryfrom/README.fa.md) یک نوعِ خطا نشانت داد که هیچ‌چیز جز چندتا `derive` نداشت — برایِ `TryFrom` همین کافی بود، چون آن صفت به `std::error::Error` نیازی ندارد. آنچه امروز به‌دست می‌آوری یک الزامِ کامپایلری نیست؛ ورود به یک قراردادِ گسترده‌تر است: کتابخانه‌هایی مثلِ `anyhow`، ابزارهایِ لاگ‌گیری، و هر تابعی که `Box<dyn Error>` می‌خواهد، همه انتظار دارند نوعِ خطایت این صفت را داشته باشد.

---

## مفهوم

### طراحیِ enum خطا: یک گونه به‌ازایِ هر جور شکست

فرض کن یک خطِ متن مثلِ `"Frieren:96"` را پارس می‌کنی — یک عنوان و یک امتیاز از ۰ تا ۱۰۰. سه جور می‌تواند خراب شود: عنوان خالی باشد، امتیاز اصلاً عدد نباشد، یا امتیاز عدد باشد ولی از ۱۰۰ بیشتر. به‌جایِ یک ساختارِ تکی با یک فیلدِ `message: String`، یک enum می‌نویسی که هر گونه‌اش دقیقاً یکی از این سه را نشان می‌دهد:

```rust
#[derive(Debug, PartialEq)]
pub struct Review {
    pub title: String,
    pub score: u8,
}

#[derive(Debug)]
pub enum ReviewError {
    MissingTitle,
    InvalidScore(std::num::ParseIntError),
    ScoreOutOfRange(u8),
}

println!("{:?}", ReviewError::ScoreOutOfRange(150));
```

```text
ScoreOutOfRange(150)
```

`InvalidScore` خودِ [`ParseIntError`](https://doc.rust-lang.org/std/num/struct.ParseIntError.html)ِ زیربنایی را نگه می‌دارد — هیچ اطلاعاتی گم نمی‌شود. `ScoreOutOfRange` عددِ واقعی‌ای که رد شده را حمل می‌کند، نه فقط یک پیام که می‌گوید «یک عدد رد شد». این دقیقاً همان چیزی است که [۱.۶.۵](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.fa.md) با قاعده‌اش گفت — یک گونه به‌ازایِ هر جور شکست — امروز فقط یک قدم جلوتر می‌رویم: خودِ enum را «یک خطایِ واقعیِ Rust» می‌کنیم.

تابعِ پارس‌کننده، فعلاً به‌شکلِ دستی (بدونِ `?` — چند سطر جلوتر، همان راه‌حلِ `From` که ۱.۶.۵ یادت داد برمی‌گردد):

```sh
cargo run -p p2-05-01-custom-error-types --example 01-designing-the-error-enum
```

```text
"Frieren:96" -> Ok(Review { title: "Frieren", score: 96 })
":90" -> Err(MissingTitle)
"Bocchi:oops" -> Err(InvalidScore(ParseIntError { kind: InvalidDigit }))
"Bocchi:150" -> Err(ScoreOutOfRange(150))
```

هر سه شکست، هر سه یک enum. فراخواننده می‌تواند دقیقاً بگوید کدام‌یک بود — نه با گشتنِ یک رشته، با یک `match` معمولی.

### `Debug` مجانی، `Display` دستی

[۲.۳.۴](../../03-traits-and-generics/04-standard-derives-by-hand/README.fa.md) این تقسیم را از قبل یادت داد: `Debug` (`{:?}`) مکانیکی است و تقریباً همیشه `derive` می‌شود — همان چیزی که بالا دیدی. `Display` (`{}`) یک تصمیمِ انسانی است و هیچ‌وقت `derive` نمی‌شود. همان تقسیم، همین‌جا هم عوض نمی‌شود؛ فقط این‌بار نوعی که `Display` می‌نویسی یک خطاست:

```rust
impl std::fmt::Display for ReviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewError::MissingTitle => write!(f, "review is missing a title"),
            ReviewError::InvalidScore(source) => write!(f, "invalid score: {source}"),
            ReviewError::ScoreOutOfRange(score) => {
                write!(f, "score {score} is out of range (must be 0-100)")
            }
        }
    }
}
```

```sh
cargo run -p p2-05-01-custom-error-types --example 02-debug-and-display
```

```text
Display: review is missing a title
Debug:   MissingTitle
Display: invalid score: invalid digit found in string
Debug:   InvalidScore(ParseIntError { kind: InvalidDigit })
Display: score 150 is out of range (must be 0-100)
Debug:   ScoreOutOfRange(150)
```

بازوی `InvalidScore` یک نکته دارد: `{source}` مستقیم پیامِ `Display` خودِ `ParseIntError` را («invalid digit found in string») تویِ پیامِ خودت می‌چیند. هیچ اطلاعاتی از خطایِ زیربنایی دور ریخته نمی‌شود، حتی در سطحِ پیامِ متنی.

### `std::error::Error`: دو ابرصفت، و یک متد

اسمِ `std::error::Error` را در [۱.۶.۳](../../../phase1-fundamentals/06-absence-and-failure/03-result-and-question-mark/README.fa.md) دیدی؛ حالا خودش. اعلانِ واقعی‌اش — از کتابخانه‌ی استانداردِ خودِ Rust، منهایِ چند متدِ منسوخ‌شده که دیگر کسی نمی‌نویسدشان — همین است:

```rust
pub trait Error: Debug + Display {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
```

`Error: Debug + Display` یعنی `Error` دو تا **ابرصفت** دارد — دقیقاً همان اصطلاحی که [۲.۳.۶](../../03-traits-and-generics/06-supertraits-blanket-impls-orphan-rule/README.fa.md) بهت داد: هیچ نوعی نمی‌تواند `Error` را پیاده کند مگر از قبل `Debug` و `Display` را داشته باشد. `ReviewError` بالا هر دو را دارد — یکی مشتق‌شده، یکی دستی. پس پیاده‌سازیِ `Error` همین می‌شود:

```rust
impl std::error::Error for ReviewError {}
```

```sh
cargo run -p p2-05-01-custom-error-types --example 03-implementing-the-error-trait
```

```text
err.source(): None
```

یک بدنه‌ی کاملاً خالی. نه چون تنبلی کرده‌ای — چون هرچه لازم بود، جای دیگری از قبل تأمین شده: دو ابرصفتِ بالا، و تنها متدِ خودِ `Error` — `source()` — یک بدنه‌ی پیش‌فرض دارد که `None` برمی‌گرداند. `source()` برایِ زنجیره‌کردنِ خطاهاست — «این خطا خودش نتیجه‌ی کدام خطایِ دیگر بود؟» — و برایِ یک خطایِ ریشه‌ای مثلِ `ReviewError` (که خودش علتِ چیزِ دیگری نیست)، `None` دقیقاً جوابِ درست است. زنجیره‌کردنِ واقعی — override کردنِ `source()` وقتی خطا خودش یک خطایِ دیگر را در دلش دارد — موضوعِ [۲.۵.۲](../02-error-source-chains/README.fa.md) است.

### `From` همان کاری را می‌کند که ۱.۶.۵ یاد داد

[۱.۶.۵](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.fa.md) نشانت داد `?` قبل از برگرداندنِ یک `Err`، رویش `From::from` صدا می‌زند. همان مکانیزم، بدونِ هیچ تغییری، اینجا هم کار می‌کند:

```rust
impl From<std::num::ParseIntError> for ReviewError {
    fn from(source: std::num::ParseIntError) -> Self {
        ReviewError::InvalidScore(source)
    }
}
```

با این `impl`، تابعِ `parse_review` دیگر به آن `match`ِ دستیِ بالا نیاز ندارد؛ فقط همین یک خط عوض می‌شود — `let score: u8 = score_str.parse()?;`، یک `?` خالی، بدونِ هیچ `.map_err(...)`ای:

```sh
cargo run -p p2-05-01-custom-error-types --example 04-from-lets-question-mark-convert
```

```text
"Frieren:96" -> Ok(Review { title: "Frieren", score: 96 })
":90" -> Err(MissingTitle)
"Bocchi:oops" -> Err(InvalidScore(ParseIntError { kind: InvalidDigit }))
"Bocchi:150" -> Err(ScoreOutOfRange(150))
```

همان جواب‌ها، همان مثالِ ۰۱. توجه کن: `MissingTitle` هیچ‌وقت از راهِ `From` نمی‌آید — مستقیم ساخته می‌شود، همان‌جا که تشخیصش می‌دهی. ۱.۶.۵ همین را گفت: نه هر گونه‌ای به یک `impl From` نیاز دارد، فقط آن‌هایی که از تبدیلِ یک خطایِ بیرونی می‌آیند.

### پاسخِ ساختاریافته: `match` کن، رفتار را عوض کن

این‌جا سودِ اصلیِ کلِ درس است. یک `String` یا یک `Box<dyn Error>` را فقط می‌شود چاپ کرد. یک enum را می‌شود `match` کرد:

```rust
fn guidance(err: &ReviewError) -> &'static str {
    match err {
        ReviewError::MissingTitle => "ask them to add a title",
        ReviewError::InvalidScore(_) => "ask them to type digits only",
        ReviewError::ScoreOutOfRange(_) => "ask them for a score between 0 and 100",
    }
}
```

```sh
cargo run -p p2-05-01-custom-error-types --example 05-matching-to-respond-differently
```

```text
MissingTitle -> ask them to add a title
InvalidScore(ParseIntError { kind: InvalidDigit }) -> ask them to type digits only
ScoreOutOfRange(150) -> ask them for a score between 0 and 100
```

سه پاسخِ کاملاً متفاوت، از رویِ یک `match`. این دقیقاً همان چیزی است که یک `String` هیچ‌وقت نمی‌توانست بدهد — نه بدونِ اینکه فراخواننده مجبور شود متنش را با `contains` بگردد.

```senpai-visual
{"kind":"result","labels":["parse_review(line)","Err(MissingTitle)","Err(InvalidScore)","Err(ScoreOutOfRange)","پاسخی متفاوت برای هرکدام"]}
```

---

## دست‌به‌کد

```sh
cargo run -p p2-05-01-custom-error-types --example 01-designing-the-error-enum
cargo run -p p2-05-01-custom-error-types --example 02-debug-and-display
cargo run -p p2-05-01-custom-error-types --example 03-implementing-the-error-trait
cargo run -p p2-05-01-custom-error-types --example 04-from-lets-question-mark-convert
cargo run -p p2-05-01-custom-error-types --example 05-matching-to-respond-differently
```

بعد دو تای خراب:

```sh
cargo run -p p2-05-01-custom-error-types --example 06-error-needs-debug-and-display --features broken
cargo run -p p2-05-01-custom-error-types --example 07-source-needs-the-trait-in-scope --features broken
```

بعد این‌ها را امتحان کن:

۱. در `01-designing-the-error-enum`، یک ورودیِ چهارم اضافه کن که به‌جایِ `:` از `;` استفاده کند (مثلاً `"Frieren;96"`). چه `Err`ی می‌گیری، و چرا دقیقاً همان است، نه یک گونه‌ی تازه؟
۲. در `03-implementing-the-error-trait`، یک `println!("{}", err);` هم اضافه کن. چاپش با چاپِ `err.source()` چه فرقی دارد؟
۳. در `05-matching-to-respond-differently`، یک گونه‌ی چهارم به `ReviewError` اضافه کن (مثلاً `DuplicateTitle`) و ببین کامپایلر دقیقاً کجایِ `guidance` شاکی می‌شود.

---

## خطاهایی که خواهی دید

### `E0277` — `Error` بدونِ `Debug` و بدونِ `Display`

```text
error[E0277]: `ReviewError` doesn't implement `std::fmt::Display`
  --> phase2-intermediate\05-error-handling\01-custom-error-types\examples\06-error-needs-debug-and-display.rs:9:28
   |
 9 | impl std::error::Error for ReviewError {}
   |                            ^^^^^^^^^^^ unsatisfied trait bound
   |
help: the trait `std::fmt::Display` is not implemented for `ReviewError`
  --> phase2-intermediate\05-error-handling\01-custom-error-types\examples\06-error-needs-debug-and-display.rs:5:1
   |
 5 | pub struct ReviewError;
   | ^^^^^^^^^^^^^^^^^^^^^^
note: required by a bound in `std::error::Error`
  --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\error.rs:59:26
   |
59 | pub trait Error: Debug + Display {
   |                          ^^^^^^^ required by this bound in `Error`

error[E0277]: `ReviewError` doesn't implement `Debug`
  --> phase2-intermediate\05-error-handling\01-custom-error-types\examples\06-error-needs-debug-and-display.rs:9:28
   |
 9 | impl std::error::Error for ReviewError {}
   |                            ^^^^^^^^^^^ the trait `Debug` is not implemented for `ReviewError`
   |
   = note: add `#[derive(Debug)]` to `ReviewError` or manually `impl Debug for ReviewError`
note: required by a bound in `std::error::Error`
  --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\error.rs:59:18
   |
59 | pub trait Error: Debug + Display {
   |                  ^^^^^ required by this bound in `Error`
help: consider annotating `ReviewError` with `#[derive(Debug)]`
   |
 5 + #[derive(Debug)]
 6 | pub struct ReviewError;
   |

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p2-05-01-custom-error-types` (example "06-error-needs-debug-and-display") due to 2 previous errors
```

**کامپایلر به چه اعتراض دارد:** `examples/06-error-needs-debug-and-display.rs` یک `ReviewError` می‌سازد که هیچ‌چیز ندارد — نه `#[derive(Debug)]`، نه `impl Display`. خطِ `impl std::error::Error for ReviewError {}` می‌خواهد دقیقاً همان دو ابرصفتی را که «مفهوم» نشانت داد فعال کند، و کامپایلر هر دو تخطی را جداگانه گزارش می‌کند — یکی برایِ `Display`، یکی برایِ `Debug` — هر دو با اشاره به همان خطِ `pub trait Error: Debug + Display` تویِ خودِ کتابخانه‌ی استاندارد.

**راه‌حل:** هر دو ابرصفت را واقعاً بده:

```rust
#[derive(Debug)]
pub struct ReviewError;

impl std::fmt::Display for ReviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "review is missing a title")
    }
}
```

**چرا این راه‌حل است:** همین که `Debug` و `Display` هردو واقعاً پیاده شوند، همان `impl std::error::Error for ReviewError {}`ِ خالی، بدونِ هیچ تغییرِ دیگری، کامپایل می‌شود — دقیقاً همان چیزی که در «مفهوم» با `ReviewError`ِ سه‌گونه‌ای دیدی.

### `E0599` — `source()` بدونِ اینکه صفت در دامنه باشد

```text
error[E0599]: no method named `source` found for struct `ReviewError` in the current scope
   --> phase2-intermediate\05-error-handling\01-custom-error-types\examples\07-source-needs-the-trait-in-scope.rs:21:26
    |
  9 | pub struct ReviewError;
    | ---------------------- method `source` not found for this struct
...
 21 |     println!("{:?}", err.source());
    |                          ^^^^^^ method not found in `ReviewError`
    |
   ::: C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\error.rs:111:8
    |
111 |     fn source(&self) -> Option<&(dyn Error + 'static)> {
    |        ------ the method is available for `ReviewError` here
    |
    = help: items from traits can only be used if the trait is in scope
help: trait `Error` which provides `source` is implemented but not in scope; perhaps you want to import it
    |
  8 + use std::error::Error;
    |

For more information about this error, try `rustc --explain E0599`.
error: could not compile `p2-05-01-custom-error-types` (example "07-source-needs-the-trait-in-scope") due to 1 previous error
```

**کامپایلر به چه اعتراض دارد:** `examples/07-source-needs-the-trait-in-scope.rs` واقعاً `impl std::error::Error for ReviewError {}` دارد — `source()` واقعاً رویِ این نوع وجود دارد. مشکل جایِ دیگری است: هیچ‌جایِ این فایل `use std::error::Error;` نیامده. `source()` یک متدِ صفتی است، نه یک متدِ خودِ نوع، و فراخوانیِ یک متدِ صفتی به وجودِ خودِ صفت در دامنه نیاز دارد — همان‌طور که پیامِ کامپایلر هم می‌گوید: «items from traits can only be used if the trait is in scope».

**راه‌حل:** خطِ `use` را اضافه کن:

```rust
use std::error::Error;
```

**چرا این راه‌حل است:** با این `use`، `Error` در دامنه است، و `err.source()` دقیقاً همان متدی را پیدا می‌کند که همین حالا هم رویِ `ReviewError` پیاده شده بود — فقط تا این لحظه کامپایلر نمی‌دانست کجا دنبالش بگردد. این تله مخصوصِ `source()` نیست: هر متدی که از یک صفت می‌آید، نه از خودِ نوع، همین قاعده را دارد.

---

## تمرین

### گرم‌کردن

<details>
<summary>یک enum خطا سه گونه دارد: <code>MissingTitle</code>، <code>InvalidScore(ParseIntError)</code>، <code>ScoreOutOfRange(u8)</code>. بهش فقط <code>#[derive(Debug)]</code> می‌دهی، بدونِ <code>Display</code>. آیا <code>impl std::error::Error for X {}</code> کامپایل می‌شود؟</summary>

نه. `Error: Debug + Display` هر دو ابرصفت را می‌خواهد. `Debug` هست، `Display` نیست — `E0277` می‌گیری، دقیقاً مثلِ نیمه‌ی دومِ مثالِ ۰۶.

</details>

<details>
<summary>یک نوع <code>Debug</code> و <code>Display</code> هر دو را دارد، و <code>impl std::error::Error for X {}</code> را هم نوشته‌ای. آیا <code>source()</code> را هم باید بنویسی تا کامپایل شود؟</summary>

نه. `source()` یک متدِ پیش‌فرض است — بدنه‌اش از قبل `None` را برمی‌گرداند. یک بدنه‌ی کاملاً خالی برایِ `impl Error` کافی است.

</details>

<details>
<summary>چرا <code>MissingTitle</code> هیچ‌وقت از راهِ <code>impl From</code> ساخته نمی‌شود؟</summary>

چون از یک خطایِ بیرونی نمی‌آید — تویِ `parse_review` مستقیم تشخیصش می‌دهی (`title.is_empty()`) و مستقیم می‌سازی‌اش. `From` فقط برایِ تبدیلِ یک خطایِ دیگر لازم است.

</details>

<details>
<summary>یک تابع <code>E: std::error::Error</code> را کران می‌گیرد. یک <code>String</code> را می‌شود بهش داد؟</summary>

نه. مشکل این‌جاست که کتابخانه‌ی استاندارد `Error` را برایِ `String` پیاده نکرده. `Debug`+`Display` داشتنِ یک نوع خودبه‌خود به این معنا نیست که `Error`ش هم پیاده شده — `impl Error for X` همیشه باید صریح نوشته شود.

</details>

<details>
<summary>یک enum خطا <code>impl Error</code> دارد ولی هیچ‌جا <code>use std::error::Error;</code> ننوشته‌ای. آیا <code>format!("{}", err)</code> کار می‌کند؟</summary>

بله. `{}` از `Display` استفاده می‌کند، و فرمت‌ماکروها همیشه `Display`/`Debug` را می‌شناسند، بدونِ اینکه لازم باشد جایی `use`شان کنی. مشکلِ «صفت در دامنه نیست» فقط وقتی سر می‌رسد که یک متدِ صفتی را با `.` صدا بزنی — مثلِ `err.source()`.

</details>

### تعمیر

هر دو مثالِ خراب را درست کن:

۱. `examples/06-error-needs-debug-and-display.rs` را طوری درست کن که کامپایل شود — `#[derive(Debug)]` را اضافه کن و یک `impl Display` واقعی بنویس، بدونِ اینکه `impl std::error::Error for ReviewError {}` را دست بزنی.
۲. `examples/07-source-needs-the-trait-in-scope.rs` را با یک `use std::error::Error;` درست کن.

### پیاده‌سازی

سه چیز در `src/lib.rs` — یک `Display`، یک `impl From`، و تابعِ پارس‌کننده:

```sh
cargo test -p p2-05-01-custom-error-types
```

`impl std::error::Error for EntryError {}` از قبل آن‌جاست، کاملاً خالی — دقیقاً همان چیزی که در «مفهوم» دیدی، همین که `Debug` (مشتق‌شده) و `Display` (چیزی که می‌نویسی) هر دو باشند. هر سه چیزِ دیگر را دقیقاً از رویِ کامنتِ مستندساز (doc comment) بالای هرکدام پیاده کن — فرمتِ متنِ `Display`، ترتیبِ چک‌ها، و سقفِ عددی همه آن‌جا دقیق نوشته شده.

### بساز

یک enum خطا برایِ دامنه‌ای که خودت انتخاب می‌کنی طراحی کن — یک رمزِ تخفیف، یک آدرسِ ایمیل، یک شماره‌ی تلفن، هرچه دوست داری — با دستِ‌کم دو گونه‌ی متفاوت. `#[derive(Debug)]`، یک `Display` دستی، و یک `impl std::error::Error` خالی برایش بنویس. بعد یک تابع بنویس که یکی از این گونه‌ها را می‌گیرد و بسته به گونه، رفتارِ واقعاً متفاوتی نشان می‌دهد — نه فقط یک پیامِ متفاوت.

### چالش (اختیاری)

**بخشِ یک.** enumِ بخشِ «بساز» را بردار. یک گونه‌ی تازه به آن اضافه کن که خودش یک enum خطایِ دیگر را در دلش نگه می‌دارد (مثلاً یک خطایِ پارسِ یک زیرفیلد). `impl Error`ت را همان بدنه‌ی خالیِ همیشگی نگه دار — یعنی `source()` هنوز پیش‌فرضش را دارد و `None` برمی‌گرداند، حتی وقتی گونه‌ی جدید واقعاً یک خطایِ دیگر را حمل می‌کند. آیا این دروغ است؟

**بخشِ دو.** (این یکی جلوتر را نگاه می‌کند.) مستنداتِ [`std::error::Error::source`](https://doc.rust-lang.org/std/error/trait.Error.html#tymethod.source) را باز کن و مثالِ کدش را بخوان — دو نوع به نام‌هایِ `SuperError` و `SuperErrorSideKick`. حدس بزن `source()` رویِ `SuperError` دقیقاً چه چیزی برمی‌گرداند، بعد در [۲.۵.۲](../02-error-source-chains/README.fa.md) جوابت را بررسی کن.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| enum خطا | یک گونه به‌ازایِ هر جور شکست، نه یک `String` | هر تابعی که چند جور می‌تواند شکست بخورد |
| `std::error::Error` | صفتِ استانداردِ «این یک خطایِ واقعیِ Rust است» | هر نوعِ خطایی که قرار است با بقیه‌ی اکوسیستم ترکیب شود |
| ابرصفت (اینجا) | `Error: Debug + Display` — هر دو باید از قبل باشند | چرا `impl Error for X {}` گاهی همین‌قدر کوچک است |
| `source()` | تنها متدِ خودِ `Error`، پیش‌فرضش `None` | یک خطایِ ریشه‌ای که خودش علتِ چیزِ دیگری نیست |
| `Display` دستی + `Debug` مشتق‌شده | همان تقسیمِ ۲.۳.۴، رویِ یک خطا | پیامِ کاربر در برابرِ دامپِ برنامه‌نویس |

### الان می‌دانی

- یک enum خطا یک گونه به‌ازایِ هر جور شکست دارد؛ فراخواننده با `match` می‌تواند دقیقاً بگوید کدام‌یک بود، نه فقط بخواندش.
- `std::error::Error` دو ابرصفت می‌خواهد — `Debug` و `Display` — و فقط یک متدِ خودش دارد، `source()`، با یک پیش‌فرضِ `None`.
- برایِ یک خطایِ ریشه‌ای، `impl std::error::Error for X {}` می‌تواند کاملاً خالی باشد، همین که `Debug` و `Display` هر دو موجود باشند.
- یک `String` هیچ ساختاری برایِ `match`کردن ندارد؛ یک `Box<dyn Error>` هم، بدونِ downcast، همان مشکل را دارد.
- `From` همان کاری را می‌کند که [۱.۶.۵](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.fa.md) گفت — نه هر گونه‌ای به آن نیاز دارد، فقط آن‌هایی که از یک خطایِ بیرونی می‌آیند.
- `source()` یک متدِ صفتی است؛ فراخوانی‌اش با `.` به `use std::error::Error;` نیاز دارد، حتی وقتی `impl` از قبل هست.

### بعداً کامل‌تر می‌بینی

- **زنجیره‌ی منشأ: override کردنِ `source()`، و `Box<dyn Error>` برایِ نگه‌داشتنِ خطاهایِ ناهم‌جنس** — [۲.۵.۲ — زنجیره‌ی منشأ](../02-error-source-chains/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا یک enum خطا از یک `Result<T, String>` بیشتر به فراخواننده می‌دهد؟
- `std::error::Error` دقیقاً چه دو صفتِ دیگری را پیش‌نیاز می‌گیرد، و چرا؟
- چرا `impl std::error::Error for X {}` می‌تواند کاملاً خالی باشد؟
- `source()` پیش‌فرضش چیست، و کِی این پیش‌فرض دقیقاً درست است؟
- چرا `err.source()` بدونِ `use std::error::Error;` کامپایل نمی‌شود، ولی `format!("{}", err)` بدونش هم کار می‌کند؟

---

## بیشتر

- [کتابِ Rust — خطایابیِ قابلِ‌بازیافت با `Result`](https://doc.rust-lang.org/book/ch09-00-error-handling.html) — همین زمین، رسمی.
- [`std::error::Error`](https://doc.rust-lang.org/std/error/trait.Error.html) — مستنداتِ کاملِ صفت، همراه با مثالِ `source()` که در چالش بهش اشاره شد.
- [`std::fmt::Display`](https://doc.rust-lang.org/std/fmt/trait.Display.html) و [`std::fmt::Debug`](https://doc.rust-lang.org/std/fmt/trait.Debug.html) — مستنداتِ همان دو ابرصفت.
