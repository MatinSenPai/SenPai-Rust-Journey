# ۲.۵.۴ — طراحیِ رده‌بندیِ خطا برای یک سرویس

## در یک نگاه

بعد از این درس می‌توانی:

- یک enum خطای تودرتو مثلِ `ServiceError` بسازی که سه رده‌ی واقعیِ شکست —
  اعتبارسنجی، پیدا نشدن، داخلی — را جدا نگه می‌دارد، با `thiserror`، نه با
  پیاده‌سازیِ دستیِ `Display` و `Error`.
- برایِ یک خطایِ داخلی، پیامِ امنِ بیرونی (`Display`) را از جزئیاتِ واقعی —
  که فقط از راهِ `source()` در دسترس می‌ماند — جدا نگه داری، بدونِ اینکه
  هیچ‌کدام را گم کنی.
- تشخیص بدهی یک رده‌ی تازه کِی واقعاً به تصمیمِ فراخواننده چیزی اضافه
  می‌کند، و کِی فقط enum را شلوغ می‌کند.

**زمان:** حدود ۵۰ دقیقه · **پیش‌نیاز:**
[۲.۵.۱ — نوع‌های خطای سفارشی](../01-custom-error-types/README.fa.md)،
[۲.۵.۲ — زنجیره‌ی منشأ و `Box<dyn Error>`](../02-error-source-chains/README.fa.md)،
[۲.۵.۳ — `thiserror` در برابرِ `anyhow`](../03-thiserror-and-anyhow/README.fa.md)

---

## چرا اهمیت دارد

سه درسِ قبلی، سه ابزار جدا به‌ات دادند. ۲.۵.۱ یادت داد به‌جایِ برگرداندنِ یک
`String` خام، یک enum بسازی — یک نوع به‌ازایِ هر شکلِ واقعیِ شکست، تا فراخواننده
بتواند رویش `match` کند. ۲.۵.۲ یادت داد آن خطا را دور نریزی: علتِ ریشه‌ای را
از راهِ `source()` زنجیر کنی، و وقتی نوعِ دقیقِ علت از قبل معلوم نیست، آن را در
`Box<dyn Error>` بپیچی. ۲.۵.۳ یادت داد آن enum را با دستِ خودت ننویسی —
`derive(thiserror::Error)` همان `Display` و `Error` را برایت می‌سازد — و
`anyhow` را برایِ لحظه‌ای نگه داری که یک باینری، نه یک کتابخانه، دارد یک خطا
را نهایی گزارش می‌کند.

سه‌تا ابزار. اما هیچ‌کدام به‌ات نگفت وقتی یک تکه‌کدِ واقعی — نه یک تمرینِ
تک‌مسئله‌ای — جلوت است، این‌ها را دقیقاً چطور کنارِ هم بچینی. یک سرویسِ
کوچک را تصور کن: کاری که هم ورودیِ کاربر را اعتبارسنجی می‌کند، هم چیزی را با
شناسه پیدا می‌کند، هم گاهی با دنیایِ بیرون (یک فایل، یک شبکه) کار دارد. این
سه کار، سه‌جور شکست می‌خورند، و طراحیِ درست این نیست که هر شکست یک enum جدا
بگیرد یا همه‌شان یک enum مشترک — این یک سؤالِ طراحی است: **رده‌بندیِ خطا
(error taxonomy)**، یعنی گروه‌بندیِ شکست‌هایِ واقعی زیرِ چند رده‌ی معدود که
برایِ فراخواننده واقعاً فرق می‌کند، نه شمردنِ تک‌تکِ آن‌ها.

این درس چیزِ تازه‌ای از نحو یاد نمی‌دهد. هرچه پایین می‌بینی، همان سه چیزی
است که در ۲.۵.۱ تا ۲.۵.۳ دیدی. کارش این است که آن‌ها را یک‌بار، رویِ یک
مسئله‌ی واقعی، کنارِ هم بگذارد — و همین‌جاست که ماژولِ مدیریتِ خطا بسته
می‌شود.

---

## مفهوم

### یک شکستِ تخت، سه دلیلِ متفاوت

فرض کن یک تابع سه‌جور شکست می‌خورد، ولی همه را با یک `Result<T, String>`
گزارش می‌دهد:

```rust
fn add_entry_stringly(title: &str, rating: u8) -> Result<u64, String> {
    if title.trim().is_empty() {
        return Err("title must not be empty".to_string());
    }
    if rating > 10 {
        return Err(format!("rating {rating} is out of range 0..=10"));
    }
    Ok(0)
}
```

تنها راهِ فراخواننده برایِ رفتارِ متفاوت با هرکدام، کاویدنِ متنِ پیام است:

```rust
match add_entry_stringly("", 5) {
    Ok(id) => println!("added as {id}"),
    Err(msg) if msg.contains("empty") => {
        println!("caller reaction: ask again for a title ({msg})");
    }
    Err(msg) if msg.contains("out of range") => {
        println!("caller reaction: ask again for a rating ({msg})");
    }
    Err(msg) => println!("caller reaction: unknown failure: {msg}"),
}
```

```text
caller reaction: ask again for a title (title must not be empty)
```

امروز کار می‌کند. ولی این دقیقاً همان string-sniffingِ شکننده‌ای است که
۲.۵.۱ در موردش هشدار داد — اگر فردا کلمه‌ی `"empty"` را در پیام به
`"blank"` تغییر بدهی، این `match` بی‌سروصدا شاخه‌ی اشتباه را می‌گیرد و
کامپایلر هیچ‌وقت به‌ات نمی‌گوید. یک enum همین اشتباه را به یک خطایِ کامپایل
تبدیل می‌کند — همان چیزی که بخشِ «خطاهایی که خواهی دید» نشانت می‌دهد.

### سه رده‌یِ طبیعی: چیزی که فراخواننده باید متفاوت باهاش رفتار کند

قاعده‌ی سازمان‌دهی این نیست که «به هر شکلِ متفاوتِ شکست یک گونه بده» — آن
همان چیزی است که ۲.۵.۱ برایِ **درونِ** یک رده به‌ات یاد داد. سؤالِ این درس
یک لایه بالاتر است: **فراخواننده به چند جور رفتارِ متفاوت نیاز دارد؟** برایِ
بیشترِ سرویس‌ها، جواب همیشه همین سه‌تاست:

- **اعتبارسنجی (validation)** — تقصیرِ فراخواننده است. ورودی را عوض کند و
  دوباره امتحان کند، مشکل حل می‌شود.
- **پیدا نشدن** — تقصیرِ کسی نیست. یک غیابِ کاملاً مشخص. فراخواننده شاید
  آن را بسازد، شاید چیزِ دیگری انتخاب کند — ولی رفتارش با اعتبارسنجی یکی
  نیست.
- **داخلی (internal)** — اصلاً تقصیرِ فراخواننده نیست. هیچ تغییری در
  درخواستش این را درست نمی‌کند. تنها کاری که از دستش برمی‌آید این است که
  ببیند شکست خورده و شاید بعداً دوباره امتحان کند.

```senpai-visual
{"kind":"concept","labels":["یک شکست رخ داد","تقصیرِ فراخواننده: Validation","چیزی نبود: NotFound","تقصیرِ ما: Internal","بعداً: یک کدِ HTTP"]}
```

این سه‌تایی آن‌قدر رایج است که (بعداً، وقتی یک API رویِ HTTP ساختی — نه
همین حالا) دقیقاً رویِ سه دسته از کدهای وضعیت می‌نشیند: چیزی شبیهِ ۴۰۰ برایِ
اعتبارسنجی، ۴۰۴ برایِ پیدا نشدن، ۵۰۰ برایِ داخلی. آن نگاشت، کارِ این درس
نیست — کارِ امروز این است که این رده‌بندی را درست، در سمتِ Rust، بسازی.

### شکلِ تودرتو: یک `ServiceError` با `thiserror`، نه با دست

هر رده یک شکلِ متفاوت لازم دارد، و `ServiceError` هرکدام را جور دیگری نگه
می‌دارد:

```rust
#[derive(Debug, thiserror::Error)]
enum ValidationError {
    #[error("title must not be empty")]
    EmptyTitle,
    #[error("rating {rating} is out of range 0..=10")]
    RatingOutOfRange { rating: u8 },
}
```

```rust
#[derive(Debug, thiserror::Error)]
enum ServiceError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error("no entry with id {id}")]
    NotFound { id: u64 },
    #[error("internal error")]
    Internal(#[from] std::io::Error),
}
```

سه گونه، سه تصمیمِ متفاوت:

- **`Validation`** یک **زیرخطایِ رده‌ای (category-specific sub-error)** را
  می‌پیچد — دقیقاً همان چیزی که ۲.۵.۱ یادت داد: یک گونه به‌ازایِ هر قاعده‌ی
  اعتبارسنجی. `#[error(transparent)]` می‌گوید «`Display` و `source()` این
  گونه دقیقاً همانِ چیزی است که `ValidationError` خودش دارد، بدونِ متنِ
  اضافه.»
- **`NotFound`** فقط داده‌ی ساده حمل می‌کند — یک `id`. رده‌ای به این کوچکی
  نیازی به یک نوعِ زیرخطایِ جدا ندارد.
- **`Internal`** یک `std::io::Error` واقعی را می‌پیچد. `#[from]` همان
  کاری را می‌کند که در ۲.۵.۱ دیدی: `?` را قادر می‌کند خودش تبدیل را انجام
  بدهد.

اجرایش را ببین:

```rust
let bad_title = ServiceError::from(ValidationError::EmptyTitle);
let bad_rating = ServiceError::from(ValidationError::RatingOutOfRange { rating: 15 });
let missing = ServiceError::NotFound { id: 7 };

println!("{bad_title}");
println!("{bad_rating}");
println!("{missing}");
```

```text
title must not be empty
rating 15 is out of range 0..=10
no entry with id 7
```

`ServiceError` خودش هیچ منطقی درباره‌یِ *چرا* یک اعتبارسنجی شکست خورد
نمی‌داند — این کارِ `ValidationError` است. کارِ `ServiceError` فقط این است
که بگوید کدام **رده**. همین تفکیک است که enumِ بیرونی را کوچک نگه می‌دارد
حتی وقتی دلایلِ داخلیِ یک رده زیاد می‌شوند.

### `?` رده را حدس نمی‌زند — تو تصمیم می‌گیری

```rust
fn validate(title: &str, rating: u8) -> Result<(), ValidationError> {
    if title.trim().is_empty() {
        return Err(ValidationError::EmptyTitle);
    }
    if rating > 10 {
        return Err(ValidationError::RatingOutOfRange { rating });
    }
    Ok(())
}

fn add_entry(title: &str, rating: u8) -> Result<(), ServiceError> {
    validate(title, rating)?;
    println!("stored: {title} ({rating}/10)");
    Ok(())
}
```

```rust
if let Err(err) = add_entry("Frieren", 10) {
    println!("unexpected: {err}");
}
if let Err(err) = add_entry("", 5) {
    println!("rejected: {err}");
}
if let Err(err) = add_entry("Bocchi", 15) {
    println!("rejected: {err}");
}
```

```text
stored: Frieren (10/10)
rejected: title must not be empty
rejected: rating 15 is out of range 0..=10
```

`validate` یک `ValidationError` برمی‌گرداند؛ `add_entry` یک `ServiceError`
می‌خواهد. `?` این دو را خودش جور می‌کند، فقط چون `#[from]` رویِ `Validation`
این تبدیل را از پیش نوشته. اگر آن سیم‌کشی نبود، `?` هیچ حدسی نمی‌زند —
کامپایلر متوقفت می‌کند و مجبورت می‌کند رده را صریح انتخاب کنی. دقیقاً همین
اتفاق در «خطاهایی که خواهی دید» می‌افتد.

### دو مخاطب برایِ یک خطا: چیزی که بیرون می‌دهی، چیزی که برایِ خودت نگه می‌داری

اینجا نقطه‌ی اصلیِ درس است. `Internal` عمداً یک پیامِ کلی دارد:

```rust
#[derive(Debug, thiserror::Error)]
enum ServiceError {
    #[error("internal error")]
    Internal(#[from] std::io::Error),
}

fn restore_from_file(path: &str) -> Result<String, ServiceError> {
    Ok(std::fs::read_to_string(path)?)
}
```

```rust
use std::error::Error;

let err = restore_from_file("definitely/does/not/exist.txt").unwrap_err();
println!("shown to the caller: {err}");
println!("logged for debugging: {}", err.source().unwrap());
```

```text
shown to the caller: internal error
logged for debugging: The system cannot find the path specified. (os error 3)
```

`{err}` هیچ‌وقت متنِ واقعیِ `io::Error` را نشان نمی‌دهد — فقط `"internal
error"`. این عمدی است: یک فراخواننده‌ی بیرونی (یک کاربر، یک تیمِ دیگر، یک
پاسخِ API عمومی) نباید مسیرِ فایل یا جزئیاتِ زیرساختِ تو را ببیند؛ آن جزئیات
نه امنند و نه به دردِ او می‌خورند. ولی آن جزئیات دور ریخته نشده‌اند — همان
چیزی که ۲.۵.۲ به‌ات داد، `.source()`، هنوز راهِ برگشت به علتِ ریشه‌ای است،
برایِ هرکسی که دارد لاگ‌ها را می‌خواند.

```senpai-visual
{"kind":"result","labels":["io::Error (علتِ ریشه‌ای)","ServiceError::Internal","Display: internal error","source(): همان io::Error"]}
```

این را با `Validation` مقایسه کن: آنجا `#[error(transparent)]` عمداً
*همه‌ی* جزئیات را بیرون می‌دهد، چون آن جزئیات — کدام فیلد، چه مقداری —
دقیقاً همان چیزی است که فراخواننده برایِ درست‌کردنِ ورودی‌اش لازم دارد. هیچ
قاعده‌ی یکسانی برایِ همه‌ی رده‌ها وجود ندارد — «همیشه پنهان کن» یا «همیشه
نشان بده» هر دو غلط‌اند. تصمیم به‌ازایِ هر رده گرفته می‌شود: چه کسی این
پیام را می‌بیند، و آیا جزئیات برایش امن و مفیدند یا نه.

### جایی که رده‌بندی دیگر نمی‌ارزد

می‌شد جلوتر رفت: `Internal` را به `DiskError`، `PermissionError`،
`CorruptStateError` و چند تایِ دیگر شکست. چرا این کار نکردیم؟ چون هیچ
فراخواننده‌ای — و هیچ‌جایِ دیگرِ همین کد — قرار نیست بسته به *کدام* اتفاقِ
داخلی، رفتارِ متفاوتی نشان بدهد. همه‌شان یک واکنش دارند: لاگ کن، شاید هشدار
بده، شاید بعداً دوباره امتحان کن. اضافه‌کردنِ گونه‌های بیشتر آنجا فقط یعنی
چند بازوی `match` بیشتر که همه دقیقاً یک کار می‌کنند.

سؤالی که هر بار باید بپرسی این است: **آیا جایی — کدِ فراخواننده، یا کدِ
خودم — قرار است با این یکی طوری متفاوت رفتار کند که با خواهر و برادرش در
همان رده رفتار نمی‌کند؟** اگر جواب نه است، آن گونه‌ی جداگانه لازم نیست —
ولی جزئیاتش هم نباید گم شود؛ فقط لازم نیست در خودِ نوع اسم داشته باشد،
همان‌طور که `Internal` نشانت داد.

این قاعده برعکسِ خودش هم درست است: دو گونه‌ی `ValidationError` — `EmptyTitle`
و `RatingOutOfRange` — واقعاً جداگانگی‌شان را از جایی گرفته‌اند، چون کدِ
بالادست (یا یک رابطِ کاربری) واقعاً می‌خواهد بسته به این‌که کدام‌یک است،
فیلدِ متفاوتی را در فرم قرمز کند. و اگر روزی `Internal` واقعاً بیش از یک
نوعِ ناهمگون را باید نشان بدهد — نه فقط `io::Error` — همان ابزاری که
۲.۵.۲ به‌ات داد، پیچیدنِ خطا در یک `dyn Error` جعبه‌ای، دقیقاً برایِ آن لحظه
است؛ نه چیزی که پیشاپیش و بدونِ نیازِ واقعی به‌کار ببری.

### یک `ServiceError` در دستِ فراخواننده‌یِ باینری

۲.۵.۳ این قاعده را به‌ات داد: `thiserror` داخلِ کتابخانه، `anyhow` سرِ مرزِ
کتابخانه/باینری. `ServiceError` هیچ تغییری نمی‌کند — همینی که بالا ساختی —
ولی کدی که آن را صدا می‌زند می‌تواند با `anyhow::Context` یک جمله‌ی انسانی
رویش بگذارد:

```rust
use anyhow::Context;

fn add_entry(title: &str) -> Result<(), ServiceError> {
    if title.trim().is_empty() {
        return Err(ValidationError::EmptyTitle.into());
    }
    Ok(())
}

fn seed_startup_data() -> anyhow::Result<()> {
    add_entry("").context("failed to seed the watchlist on startup")?;
    Ok(())
}
```

```rust
if let Err(err) = seed_startup_data() {
    println!("{err:?}");
}
```

```text
failed to seed the watchlist on startup

Caused by:
    title must not be empty
```

همان `ServiceError`، بدونِ هیچ تغییری، به هر دو مخاطب جواب می‌دهد: کدی که
داخلِ کتابخانه صدایش می‌زند و می‌خواهد رویِ رده‌ها `match` کند، و باینری‌ای
که در `main` فقط می‌خواهد بگوید *چه چیزی* شکست خورد و *چرا*. رده‌ها برایِ
اولی زنده ماندند؛ زنجیره برایِ دومی.

---

## دست‌به‌کد

```sh
cargo run -p p2-05-04-error-taxonomy-for-a-service --example 01-flat-error-cant-be-matched
cargo run -p p2-05-04-error-taxonomy-for-a-service --example 02-service-error-shape
cargo run -p p2-05-04-error-taxonomy-for-a-service --example 03-validation-flows-through-question-mark
cargo run -p p2-05-04-error-taxonomy-for-a-service --example 04-internal-error-display-vs-source
cargo run -p p2-05-04-error-taxonomy-for-a-service --example 05-caller-side-anyhow-context
```

بعد دوتایِ خراب:

```sh
cargo run -p p2-05-04-error-taxonomy-for-a-service --example 06-non-exhaustive-after-new-category --features broken
cargo run -p p2-05-04-error-taxonomy-for-a-service --example 07-question-mark-needs-a-category --features broken
```

بعد این‌ها را امتحان کن:

۱. در `02-service-error-shape.rs`، یک `RatingOutOfRange` سوم با عددِ دیگری
   بساز و چاپش کن — مطمئن شو همان عدد در پیام ظاهر می‌شود.
۲. در `04-internal-error-display-vs-source.rs`، به‌جایِ یک مسیرِ ناموجود،
   یک پوشه بده (مثلاً `"src"`). متنِ واقعیِ `io::Error` که از `source()`
   بیرون می‌آید عوض می‌شود — ولی خطِ «shown to the caller» عوض می‌شود یا
   نه؟ چرا؟
۳. در `05-caller-side-anyhow-context.rs`، به‌جایِ `{err:?}`، `{err:#}` را
   چاپ کن — با آنچه بالا از `{err}` و `{err:?}` دیدی مقایسه‌اش کن.

---

## خطاهایی که خواهی دید

### `E0004` — `match`ی که رده‌ی تازه را جا انداخته

```text
error[E0004]: non-exhaustive patterns: `&ServiceError::Internal(_)` not covered
  --> phase2-intermediate\05-error-handling\04-error-taxonomy-for-a-service\examples\06-non-exhaustive-after-new-category.rs:26:11
   |
26 |     match err {
   |           ^^^ pattern `&ServiceError::Internal(_)` not covered
   |
note: `ServiceError` defined here
  --> phase2-intermediate\05-error-handling\04-error-taxonomy-for-a-service\examples\06-non-exhaustive-after-new-category.rs:16:6
   |
16 | enum ServiceError {
   |      ^^^^^^^^^^^^
...
22 |     Internal(#[from] std::io::Error),
   |     -------- not covered
   = note: the matched value is of type `&ServiceError`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
28 ~         ServiceError::NotFound { .. } => "not found",
29 ~         &ServiceError::Internal(_) => todo!(),
   |

For more information about this error, try `rustc --explain E0004`.
error: could not compile `p2-05-04-error-taxonomy-for-a-service` (example "06-non-exhaustive-after-new-category") due to 1 previous error
```

**کامپایلر به چه اعتراض دارد:** `describe` روزی نوشته شده بود که
`ServiceError` فقط دو رده داشت. `Internal` بعداً به رده‌بندی اضافه شد، و
این `match` هیچ‌وقت به‌روز نشد. رده‌بندی به‌عنوانِ یک enum دقیقاً همین را
تضمین می‌کند: اضافه‌کردنِ یک رده، هر جایی که فراموش شده به‌روز شود را به یک
خطایِ کامپایل تبدیل می‌کند — نه یک شکافِ خاموش که فقط وقتی یک `Internal`
واقعی برسد خودش را نشان می‌دهد.

**راه‌حل:** یک بازو برایِ `Internal` هم اضافه کن:

```rust
fn describe(err: &ServiceError) -> &'static str {
    match err {
        ServiceError::Validation(_) => "bad input",
        ServiceError::NotFound { .. } => "not found",
        ServiceError::Internal(_) => "internal error",
    }
}
```

**چرا این راه‌حل است:** پیشنهادِ خودِ کامپایلر (یک بازوی `todo!()`) کارِ
کامپایل‌شدن را انجام می‌دهد ولی به تو می‌گوید هنوز باید تصمیم بگیری این رده
چه واکنشی می‌خواهد — دقیقاً همان چیزی که رده‌بندی قرار است مجبورت کند.
یک `_ => "unknown"` هم کامپایل می‌شود، ولی همان حفره‌ای را که کلِ این درس
درباره‌اش بود دوباره باز می‌کند: یک رده‌ی تازه، بدونِ اینکه هیچ‌جا خبردار
شوی، در یک شاخه‌ی عمومی گم می‌شود.

### `E0277` — `?` نمی‌تواند رده‌ای را که وجود ندارد حدس بزند

```text
error[E0277]: `?` couldn't convert the error to `ServiceError`
  --> phase2-intermediate\05-error-handling\04-error-taxonomy-for-a-service\examples\07-question-mark-needs-a-category.rs:25:49
   |
24 | fn restore_from_file(path: &str) -> Result<String, ServiceError> {
   |                                     ---------------------------- expected `ServiceError` because of this
25 |     let contents = std::fs::read_to_string(path)?;
   |                    -----------------------------^ the trait `From<std::io::Error>` is not implemented for `ServiceError`
   |                    |
   |                    this can't be annotated with `?` because it has type `Result<_, std::io::Error>`
   |
note: `ServiceError` needs to implement `From<std::io::Error>`
  --> phase2-intermediate\05-error-handling\04-error-taxonomy-for-a-service\examples\07-question-mark-needs-a-category.rs:17:1
   |
17 | enum ServiceError {
   | ^^^^^^^^^^^^^^^^^
   = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the `From` trait
help: the trait `From<std::io::Error>` is not implemented for `ServiceError`
      but trait `From<ValidationError>` is implemented for it
  --> phase2-intermediate\05-error-handling\04-error-taxonomy-for-a-service\examples\07-question-mark-needs-a-category.rs:19:18
   |
19 |     Validation(#[from] ValidationError),
   |                  ^^^^
   = help: for that trait implementation, expected `ValidationError`, found `std::io::Error`

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p2-05-04-error-taxonomy-for-a-service` (example "07-question-mark-needs-a-category") due to 1 previous error
```

**کامپایلر به چه اعتراض دارد:** این پیش‌نویسِ `ServiceError` هنوز رده‌ی
`Internal` را ندارد — فقط `Validation` و `NotFound`. `?` رویِ یک
`io::Error` می‌خواهد آن را به `ServiceError` تبدیل کند، ولی هیچ
`From<std::io::Error>`ای برایِ کامپایلر نیست که صدا بزند. حتی خودِ پیام
دقیقاً می‌گوید چه چیزی *هست*: `From<ValidationError>` — یک رده از قبل
سیم‌کشی شده، این یکی نه.

**راه‌حل:** رده‌ی سومی که در این پیش‌نویس جا افتاده را اضافه کن:

```rust
#[error("internal error")]
Internal(#[from] std::io::Error),
```

**چرا این راه‌حل است:** `?` هیچ‌وقت رده‌ای اختراع نمی‌کند — فقط یک `From`ِ
از قبل نوشته‌شده را صدا می‌زند. این محدودیت یک اشکال نیست؛ دقیقاً همان نقطه‌ی
تصمیمی است که این درس درباره‌اش بود: هر جا کد می‌خواهد یک شکستِ تازه را
پرتاب کند، یک انسان باید یک‌بار تصمیم بگیرد این شکست زیرِ کدام رده می‌رود —
کامپایلر آن تصمیم را برایت حدس نمی‌زند، فقط مطمئن می‌شود فراموشش نکرده‌ای.

---

## تمرین

### گرم‌کردن

<details>
<summary>این چه چاپ می‌کند؟</summary>

```rust
let err = ServiceError::NotFound { id: 42 };
println!("{err}");
```

</details>

<details>
<summary>پاسخ</summary>

```text
no entry with id 42
```

`#[error("no entry with id {id}")]` مستقیماً فیلدِ `id` را در پیام
می‌گذارد — `NotFound` داده‌ی ساده حمل می‌کند، نه یک زیرخطا.

</details>

<details>
<summary>این چه چاپ می‌کند؟</summary>

```rust
let err = ServiceError::Internal(
    std::io::Error::new(std::io::ErrorKind::Other, "disk full"),
);
println!("{err}");
```

</details>

<details>
<summary>پاسخ</summary>

```text
internal error
```

نه `"disk full"`. پیامِ `Internal` یک رشته‌ی لیترال است، نه یک قالبی که
فیلدش را داخل بگذارد — همیشه دقیقاً همین متنِ کلی را می‌دهد، هرچه داخلش
باشد.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
fn describe(err: &ServiceError) -> &'static str {
    match err {
        ServiceError::Validation(_) => "bad input",
        ServiceError::NotFound { .. } => "not found",
    }
}
```

(`ServiceError` همان سه‌رده‌ای است که در «مفهوم» ساختی، با `Internal`.)

</details>

<details>
<summary>پاسخ</summary>

نه. `E0004` — `match` رده‌ی `Internal` را پوشش نمی‌دهد.

</details>

<details>
<summary>این کامپایل می‌شود؟ (فرض کن این پیش‌نویسِ <code>ServiceError</code> فقط <code>Validation</code> و <code>NotFound</code> دارد — هنوز <code>Internal</code>ی در کار نیست)</summary>

```rust
fn restore_from_file(path: &str) -> Result<String, ServiceError> {
    let contents = std::fs::read_to_string(path)?;
    Ok(contents)
}
```

</details>

<details>
<summary>پاسخ</summary>

نه. `E0277` — هیچ `From<std::io::Error>`ای برایِ این `ServiceError` وجود
ندارد، پس `?` نمی‌تواند تبدیل را انجام بدهد.

</details>

### تعمیر

هر دو مثالِ خراب را درست کن:

۱. `examples/06-non-exhaustive-after-new-category.rs` را طوری درست کن که
   کامپایل شود — با اضافه‌کردنِ یک بازوی واقعی برایِ `Internal`، نه یک
   جانشینِ `_`.
۲. `examples/07-question-mark-needs-a-category.rs` را طوری درست کن که
   کامپایل شود — با اضافه‌کردنِ رده‌ی `Internal`یِ جا افتاده به
   `ServiceError`، همراه با `#[from]`.

### پیاده‌سازی

پنج تابع/متد در `src/lib.rs`:

```sh
cargo test -p p2-05-04-error-taxonomy-for-a-service
```

`ValidationError` و `ServiceError` از پیش کاملاً نوشته شده‌اند — شکل‌شان
همان چیزی است که «مفهوم» توضیح داد. کاری که مانده، منطقی است که آن‌ها را
تولید و مصرف می‌کند: `validate`، `WatchlistStore::new`، `add_entry`،
`rating_of`، `restore_from_file`. مشخصاتِ دقیقِ هرکدام — از جمله این‌که
شناسه‌ها از کجا شروع می‌شوند و یک خطِ فایلِ بدشکل چه بلایی سرش می‌آید — در
کامنتِ مستنداتِ بالایِ هر تابع است.

### بساز

یک متدِ تازه به `WatchlistStore` اضافه کن:

```rust
pub fn rename(&mut self, id: u64, new_title: &str) -> Result<(), ServiceError>
```

این متد دو جور می‌تواند شکست بخورد. هیچ‌کدام از آن دو به یک رده‌ی چهارم
نیاز ندارد — کارِ تو این است که تشخیص بدهی هرکدام زیرِ کدام‌یک از سه رده‌ی
موجود می‌رود، و امضایش را با همان استدلال بنویسی.

### چالش (اختیاری)

این یکی جلوتر را نگاه می‌کند — به فازِ ۳، جایی که یک API واقعیِ HTTP
می‌سازی. در یک کامنت، برایِ هر سه رده‌ی `ServiceError` بنویس کدام کدِ
وضعیتِ HTTP (۴۰۰، ۴۰۴، یا ۵۰۰) بهش می‌خورد، و یک جمله بگو چرا. بعد به
[۳.۷.۱ — پوشش‌های خطای یکدست](../../../phase3-backend-foundations/07-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.fa.md)
سر بزن و ببین آنجا همین سه‌تایی را دقیقاً به همین شکل به‌کار برده یا نه.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| رده‌بندیِ خطا (error taxonomy) | گروه‌بندیِ شکست‌هایِ واقعی زیرِ چند رده‌ی معدود که برایِ فراخواننده فرق می‌کند | طراحیِ هر enum خطایِ واقعی |
| رده (category) | یک گروه از شکست‌ها که فراخواننده باید یکسان باهاشان رفتار کند | تصمیمِ اولِ رده‌بندی |
| `#[error(transparent)]` | `Display` و `source()` را عیناً به خطایِ توش‌پیچیده‌شده تحویل می‌دهد | وقتی خودِ رده چیزی جز زیرخطایش نمی‌گوید |
| `Display`ِ امن در برابرِ `source()`ِ کامل | یکی برایِ فراخواننده‌ی بیرونی، یکی برایِ لاگ | رده‌یِ `Internal` |

### الان می‌دانی

- رده‌بندیِ خطا یعنی گروه‌بندی بر اساسِ اینکه فراخواننده چند جور رفتار
  می‌کند، نه شمردنِ تک‌تکِ راه‌هایِ شکست.
- اعتبارسنجی، پیدا نشدن، داخلی یک سه‌تاییِ طبیعی‌اند که در بیشترِ سرویس‌ها
  تکرار می‌شود.
- یک enumِ بیرونی می‌تواند یک زیرخطا بپیچد (`Validation`)، داده‌ی ساده حمل
  کند (`NotFound`)، یا یک نوعِ بیرونی مثلِ `io::Error` را بپیچد (`Internal`)
  — و هر سه با `thiserror`، نه با دست.
- `Display` و `source()` دو مخاطبِ جدا دارند؛ می‌توانی یکی را عمداً کلی
  نگه داری و دیگری را کاملاً دقیق، به‌ازایِ هر رده جداگانه.
- بیشترِ گونه، طراحیِ بهتری نیست. سؤالِ واقعی این است که آیا جایی قرار است
  با آن گونه متفاوت رفتار کند یا نه.
- همان `ServiceError`، بدونِ تغییر، هم به فراخواننده‌ای که رویِ رده
  `match` می‌کند جواب می‌دهد، هم به باینری‌ای که با `anyhow::Context` یک
  جمله‌ی انسانی رویش می‌گذارد.

### بعداً کامل‌تر می‌بینی

- **نگاشتِ این سه رده به کدهای وضعیتِ HTTP، پشتِ یک `impl IntoResponse`
  واحد** —
  [فازِ ۳ — پوشش‌های خطای یکدست](../../../phase3-backend-foundations/07-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا رده‌بندی بر اساسِ «فراخواننده چطور واکنش نشان می‌دهد» است، نه بر
  اساسِ «چند جور می‌تواند شکست بخورد»؟
- چرا `Internal` پیامِ کلی دارد ولی `Validation` ندارد؟ کدام تصمیم اشتباه
  بود اگر جایشان را عوض می‌کردی؟
- `.source()` دقیقاً چه چیزی را که `Display` گفته نبود، به‌ات می‌دهد؟
- چه تستی به‌کار می‌بری تا بفهمی یک رده‌ی تازه واقعاً لازم است یا فقط
  enum را شلوغ می‌کند؟
- چرا اضافه‌کردنِ یک رده‌ی تازه به `ServiceError` هر `match`ِ ناقص را به
  یک خطایِ کامپایل تبدیل می‌کند، نه یک باگِ خاموش؟

---

## بیشتر

- [مستنداتِ `thiserror`](https://docs.rs/thiserror/latest/thiserror/) —
  از جمله `#[error(transparent)]`، همان چیزی که `Validation` از آن
  استفاده کرد.
- [مستنداتِ `anyhow`](https://docs.rs/anyhow/latest/anyhow/) — `Context`
  و رفتارِ دقیقِ `{:?}` در برابرِ `{}`.
- [`std::error::Error`](https://doc.rust-lang.org/std/error/trait.Error.html)
  — تعریفِ رسمیِ `source()`.
- [راهنمایِ API رسمیِ Rust — خطاهایِ معنادار و خوش‌رفتار](https://rust-lang.github.io/api-guidelines/interoperability.html#error-types-are-meaningful-and-well-behaved-c-good-err)
  — همین اصول، به‌عنوانِ یک قاعده‌ی رسمی برایِ هر crateای که منتشر می‌کنی.
