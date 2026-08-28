# ۲.۳.۳ — `From`، `Into`، `TryFrom` و `TryInto`

## در یک نگاه

بعد از این درس می‌توانی:

- بگویی چرا `impl From<T> for U` معنی‌اش «این تبدیل هرگز شکست نمی‌خورد» است، و چرا `i32::from(u8)` وجود دارد ولی `u8::from(i32)` وجود ندارد.
- برایِ یک تبدیلِ عددیِ کوچک‌شونده، بینِ `as` (ساکت، بی‌سروصدا) و `try_into()` (صادق، برگرداننده‌یِ `Result`) یکی را با دلیل انتخاب کنی.
- برایِ نوعِ خودت `TryFrom` پیاده‌سازی کنی، بگویی چرا `TryInto` را مجانی می‌گیری، و بینِ `From` و `TryFrom` برایِ یک نوعِ تازه یکی را با دلیل انتخاب کنی.

**زمان:** حدود ۶۰ دقیقه · **پیش‌نیاز:**
[۱.۶.۵ — `From` و تبدیلِ خطا](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.fa.md)،
[۲.۳.۱ — تعریف و پیاده‌سازیِ traitها](../01-defining-and-implementing-traits/README.fa.md)

---

## چرا اهمیت دارد

[۱.۶.۵](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.fa.md) بهت `From` را داد، ولی دامنه‌اش را رویِ یک کاربردِ خاص نگه داشت: تبدیلِ خطاها، تا `?` خودش تبدیلشان کند. همان‌جا حتی یک تکه هم گفت «`From` فقط برایِ خطاها نیست» و `String::from`، `u64::from(u32)` را مثال زد — ولی همان یک تکه تمامِ چیزی بود که آن درس جا داشت. این درس همان جمله را کامل می‌کند: `From`/`Into` یک جفتِ عمومی‌اند برایِ **هر** تبدیلی که هرگز شکست نمی‌خورَد، نه فقط خطاها.

ولی همه‌ی تبدیل‌ها این‌طور نیستند. یک بک‌اند وقتی دارد یک درخواست را پردازش می‌کند، دائم با ورودی‌هایی روبه‌رو می‌شود که *ممکن است* نامعتبر باشند: یک امتیازِ ستاره‌ای که باید بینِ ۱ تا ۵ باشد ولی کاربر ۹ فرستاده، یک عددِ ۶۴بیتی که باید تویِ یک فیلدِ ۳۲بیتی جا شود ولی شاید جا نشود. `From` برایِ این حالت یک دروغ است: امضایش می‌گوید «همیشه جواب می‌دهم»، ولی تابعش یا باید پنیک کند یا یک مقدارِ من‌درآوردی برگرداند — هیچ‌کدام صادق نیست. پایتون بینِ این دو حالت، تویِ امضایِ تابع، فرقی نمی‌گذارد: `int("12")` جواب می‌دهد، `int("abc")` یک `ValueError` پرتاب می‌کند، و هیچ‌کدامِ این دو تویِ امضایِ خودِ `int()` نوشته نشده — باید مستنداتش را بخوانی. Rust این تفاوت را می‌برد تویِ خودِ نوعِ برگشتی: یک تابعِ `From`-محور همیشه همان نوع را برمی‌گرداند، یک تابعِ `TryFrom`-محور همیشه یک `Result` برمی‌گرداند — دقیقاً همان چیزی که [۱.۶.۱](../../../phase1-fundamentals/06-absence-and-failure/01-option-and-null-safety/README.fa.md) درباره‌ی `Option` گفت: چون نوع فرق دارد، کامپایلر نمی‌گذارد از احتمالِ شکست رد شوی بدونِ روبه‌رو شدن باهاش.

این درس ابزارِ دومی می‌دهد: `TryFrom`، خواهرِ خطاپذیرِ `From`، که همان قول را می‌دهد به‌علاوه‌یِ یک راهِ صادقانه برایِ نه‌گفتن.

---

## مفهوم

### ۱. `From` را می‌شناسی — این‌بار برایِ هر تبدیلِ همیشه‌موفق

۱.۶.۵ نشانت داد `impl From<A> for MyError` می‌نویسی تا `?` بتواند خطاها را تبدیل کند، و بلوکِ فراگیرِ `impl<T, U> Into<U> for T where U: From<T>` را هم دیدی — همان چیزی که `.into()` را مجانی می‌دهد. آن مکانیزم را اینجا از نو نمی‌سازیم؛ فقط دامنه‌اش را باز می‌کنیم. `From<T> for U` یعنی: **هر مقدارِ معتبرِ `T`ای، بدونِ استثنا، به یک `U` تبدیل می‌شود.**

```rust
let score: u8 = 200;
let widened: i32 = i32::from(score);
println!("i32::from(u8): {widened}");
```

```text
i32::from(u8): 200
```

هر `u8`ای (۰ تا ۲۵۵) بدونِ کم‌وکاست تویِ یک `i32` جا می‌شود — هیچ الگویِ بیتی گم نمی‌شود. به همین دلیل کتابخانه‌ی استاندارد این `impl From<u8> for i32` را رایگان به‌ات می‌دهد؛ خودت هیچ‌وقت لازم نیست بنویسیش. به این نوع تبدیل، **پهن‌شوندگی (widening)** می‌گویند: مقصد همیشه جا برایِ مبدأ دارد.

```rust
let delta: i32 = -12_000;
let as_float: f64 = f64::from(delta);
println!("f64::from(i32): {as_float}");
```

```text
f64::from(i32): -12000
```

جفتِ دیگر، همان ایده: مانتیسِ یک `f64` ۵۲ بیت جا دارد — خیلی بیشتر از ۳۲ بیتی که یک `i32` لازم دارد — پس هر `i32`ای هم بدونِ گم‌شدنِ چیزی تویِ یک `f64` جا می‌شود.

### ۲. وقتی جهتِ برعکس ممکن است شکست بخورد: `TryFrom`

راهِ برعکس را امتحان کن: هر `i32`ای تویِ یک `u8` جا نمی‌شود — `u8` فقط ۰ تا ۲۵۵ را نگه می‌دارد. این را **کوچک‌شوندگیِ عددی (narrowing)** می‌گویند، و اینجا دیگر `From` نمی‌تواند صادقانه وجود داشته باشد. کتابخانه‌ی استاندارد به‌جایش `TryFrom` را پیاده‌سازی کرده:

```rust
pub trait TryFrom<T>: Sized {
    type Error;
    fn try_from(value: T) -> Result<Self, Self::Error>;
}
```

همان شکلِ `From`، با دو فرق: اسمِ متد `try_from` است، و به‌جایِ برگرداندنِ `Self` مستقیم، یک `Result<Self, Self::Error>` برمی‌گرداند. آن `type Error` یک **نوعِ وابسته (associated type)** است — یک جایگزین که خودِ trait اعلامش می‌کند و هر پیاده‌کننده یک‌بار پرش می‌کند؛ مکانیزمِ کاملش [۲.۳.۵](../05-associated-types/README.fa.md) است، همین‌قدر بدان که هر `TryFrom` باید بگوید شکستش چه شکلی است. (و چون این دوره از edition ۲۰۲۱ استفاده می‌کند، `TryFrom`/`TryInto` از قبل تویِ پرلود هستند — هیچ‌جایِ این درس لازم نشد `use std::convert::TryFrom;` بنویسیم.)

حالا همان تبدیل، برعکس:

```rust
let fits: Result<u8, _> = u8::try_from(200i32);
let overflow: Result<u8, _> = u8::try_from(300i32);
let negative: Result<u8, _> = u8::try_from(-1i32);

println!("u8::try_from(200i32): {fits:?}");
println!("u8::try_from(300i32): {overflow:?}");
println!("u8::try_from(-1i32):  {negative:?}");
```

```text
u8::try_from(200i32): Ok(200)
u8::try_from(300i32): Err(TryFromIntError(PosOverflow))
u8::try_from(-1i32):  Err(TryFromIntError(NegOverflow))
```

`200` جا می‌شود، `Ok(200)`. `300` و `-1` هر دو جا نمی‌شوند — ولی حتی خطا هم اطلاعات می‌دهد: `PosOverflow` می‌گوید «خیلی بزرگ بود»، `NegOverflow` می‌گوید «خیلی کوچک بود». دقیقاً همان جفتِ `i32`/`u8` که در بخشِ قبل دیدی: یک طرف (`i32::from(u8)`) هرگز شکست نمی‌خورد، طرفِ دیگر (`u8::try_from(i32)`) می‌تواند — همان دو نوع، فقط جهت عوض شده.

```senpai-visual
{"kind":"result","labels":["u8 try_from i32","fits in 0 to 255?","Ok: u8","Err: TryFromIntError"]}
```

### ۳. `as` در برابرِ `try_into`: یکی ساکت است، یکی صادق

[۱.۱.۲](../../../phase1-fundamentals/01-foundations/02-scalar-types-and-overflow/README.fa.md) عملگرِ `as` را نشانت داد. حالا که `TryFrom` را داری، وقتش است این دو را کنارِ هم بگذاری:

```rust
let big: i32 = 300;

let truncated = big as u8;
println!("300i32 as u8:      {truncated}");

let honest: Result<u8, _> = big.try_into();
println!("300i32.try_into(): {honest:?}");
```

```text
300i32 as u8:      44
300i32.try_into(): Err(TryFromIntError(PosOverflow))
```

`as` هیچ‌وقت شکست نمی‌خورد؛ فقط ۳۲ بیتِ یک `i32` را می‌گیرد و پایین‌ترین ۸ تایش را نگه می‌دارد — بقیه دور می‌ریزد. ۳۰۰ در مبنایِ دو یعنی `100101100`؛ فقط ۸ بیتِ آخرش (`00101100` یعنی ۴۴) می‌ماند. `try_into()` همان `try_from` است، خوانده‌شده از سمتِ مقصد — دقیقاً همان استنتاجِ نوعی که [۱.۶.۵](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.fa.md) برایِ `.into()` نشانت داد، این‌بار برایِ جفتِ خطاپذیر: نوعِ نوشته‌شده‌یِ `let honest: Result<u8, _>` است که به کامپایلر می‌گوید کدام `try_from` را صدا بزند.

خطرناکی‌اش دقیقاً همین‌جاست: `256i32 as u8` هم `۰` می‌شود — نه یک خطا، نه یک پنیک، فقط یک عددِ کاملاً معقول که کاملاً غلط است.

```rust
let wrapped = 256i32 as u8;
println!("256i32 as u8: {wrapped}");
```

```text
256i32 as u8: 0
```

**قاعده:** `as` را برایِ تبدیلِ عددی به‌عنوانِ پیش‌فرض به‌کار نبر؛ برایِ جایی نگهش دار که واقعاً بریدنِ بیت‌ها را می‌خواهی. برایِ همه‌جایِ دیگر، `try_into()` است که صادقانه به‌ات می‌گوید تبدیل جواب داد یا نه.

### ۴. پیاده‌سازیِ `TryFrom` برایِ نوعِ خودت: نیوتایپِ اعتبارسنجی‌شده

فرض کن یک امتیازِ ستاره‌ای می‌گیری — یک عددِ ۱ تا ۵ — و نمی‌خواهی هرجایِ کد یک `u8`ِ خام دست‌به‌دست شود که کسی هیچ‌وقت چکش نکرده. آن را تویِ یک نیوتایپ (که [۱.۵.۲](../../../phase1-fundamentals/05-your-own-types/02-tuple-structs-and-newtype/README.fa.md) نشانت داد) بپیچ، با این تفاوت که این‌بار سازنده‌اش خودِ اعتبارسنجی است:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rating(u8);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RatingError {
    OutOfRange(u8),
}
```

```rust
impl TryFrom<u8> for Rating {
    type Error = RatingError;

    fn try_from(raw: u8) -> Result<Self, Self::Error> {
        if (1..=5).contains(&raw) {
            Ok(Rating(raw))
        } else {
            Err(RatingError::OutOfRange(raw))
        }
    }
}
```

```rust
let five_star = Rating::try_from(5);
let zero_star = Rating::try_from(0);
println!("Rating::try_from(5): {five_star:?}");
println!("Rating::try_from(0): {zero_star:?}");
```

```text
Rating::try_from(5): Ok(Rating(5))
Rating::try_from(0): Err(OutOfRange(0))
```

سازه‌ی `Rating` یک فیلدِ خصوصی دارد — `Rating(u8)`، نه یک فیلدِ `pub`. یعنی تنها راهِ ساختنِ یک `Rating` همین `TryFrom::try_from` است؛ هیچ کدی، هیچ‌جایِ برنامه، نمی‌تواند یک `Rating(9)` بسازد بدونِ رد شدن از این `if`. نتیجه: همین که یک `Rating` دستت باشد، خودش سندِ این است که آن مقدار چک شده — نیازی نیست دوباره نگاهش کنی.

این همان چیزی است که در پایتون معمولاً با یک تابعِ بررسی‌کننده یا یک اعتبارسنجِ pydantic انجام می‌دهی: مقدار را همان لبه‌ی ورودی چک می‌کنی. فرقش این‌جاست که چک‌کردنِ pydantic چیزی است که *باید* هرجا صدا بزنی، و mypy کاری به یادت‌آوردنش ندارد — اگر یک مسیر را فراموش کنی، مقدارِ نامعتبر بی‌خبر رد می‌شود. اینجا، چون فیلد خصوصی است، اصلاً امکانِ فراموش‌کردن نیست: کامپایلر تضمین می‌کند، نه انضباطِ نویسنده.

### ۵. `TryInto` را هم مجانی می‌گیری

دقیقاً همان اتفاقی که [۱.۶.۵](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.fa.md) برایِ `Into` نشان داد، اینجا هم می‌افتد — این‌بار برایِ جفتِ خطاپذیر. کتابخانه‌ی استاندارد این را یک‌بار برایِ همیشه نوشته:

```rust
impl<T, U> TryInto<U> for T
where
    U: TryFrom<T>,
{
    type Error = U::Error;
    fn try_into(self) -> Result<U, U::Error> {
        U::try_from(self)
    }
}
```

ما هیچ‌جا `impl TryInto` ننوشتیم — فقط `impl TryFrom<u8> for Rating` را نوشتیم (بخشِ ۴)، و `TryInto` رایگان آمد:

```rust
let from_call: Result<Rating, RatingError> = Rating::try_from(4);
let from_method: Result<Rating, RatingError> = 4u8.try_into();
println!("Rating::try_from(4): {from_call:?}");
println!("4u8.try_into():      {from_method:?}");
```

```text
Rating::try_from(4): Ok(Rating(4))
4u8.try_into():      Ok(Rating(4))
```

عملگرِ `?` هم همان کاری را می‌کند که تویِ [۱.۶.۳](../../../phase1-fundamentals/06-absence-and-failure/03-result-and-question-mark/README.fa.md) یاد گرفتی — رویِ یک `Result<Rating, RatingError>` دقیقاً همان‌طور کار می‌کند که رویِ هر `Result` دیگری:

```rust
fn build(raw: u8) -> Result<Rating, RatingError> {
    let rating: Rating = raw.try_into()?;
    Ok(rating)
}
```

```text
build(3): Ok(Rating(3))
build(7): Err(OutOfRange(7))
```

### ۶. قاعده‌ی انتخاب: کِی `From`، کِی `TryFrom`

قاعده یک جمله است: **اگر تبدیل برایِ هر ورودیِ معتبر همیشه جواب می‌دهد، `From` بنویس؛ اگر حتی یک ورودی هست که باید ردش کنی — اعتبارسنجی، بازه، پارس‌کردن — `TryFrom` بنویس.** یک `impl From` که رویِ ورودیِ «بد» پنیک می‌کند، تویِ سیستمِ نوع یک دروغ است: امضایش قولِ «همیشه» می‌دهد و عمل نمی‌کند.

جهتِ تبدیل هم همین قاعده را دارد، حتی رویِ یک نوعِ واحد. بخشِ ۴ جهتِ ورودی را دید — یک `u8`ِ خام که ممکن است نامعتبر باشد. جهتِ برعکس فرق دارد:

```rust
impl From<Rating> for u8 {
    fn from(value: Rating) -> Self {
        value.0
    }
}
```

هر `Rating`ای که وجود دارد، از قبل از زیرِ دستِ همان `if` تویِ بخشِ ۴ رد شده — چیزی نمانده که رد شود. بیرون‌کشیدنِ `u8`ِ داخلش نمی‌تواند شکست بخورد، پس `From` است، نه `TryFrom`:

```rust
let rating = Rating::try_from(4).unwrap();
let raw: u8 = rating.into();
println!("Rating::try_from(4).unwrap().into(): {raw}");
```

```text
Rating::try_from(4).unwrap().into(): 4
```

```senpai-visual
{"kind":"concept","labels":["every input can succeed?","yes: impl From","no: impl TryFrom","Result Self Error"]}
```

یک نکته‌ی جانبی، برایِ بعداً: هردو `impl` بالا رویِ `Rating` بودند — نوعی که خودمان تعریفش کردیم. Rust یک قاعده دارد درباره‌یِ اینکه کدام طرفِ یک `impl From<X> for Y` باید مالِ خودت باشد تا اصلاً اجازه‌یِ نوشتنش را داشته باشی — همان چیزی که [۲.۳.۶](../06-supertraits-blanket-impls-orphan-rule/README.fa.md) کاملش می‌کند.

---

## دست‌به‌کد

```sh
cargo run -p p2-03-03-from-into-tryfrom --example 01-from-is-still-infallible
cargo run -p p2-03-03-from-into-tryfrom --example 02-numeric-narrowing-tryfrom
cargo run -p p2-03-03-from-into-tryfrom --example 03-as-vs-try-into
cargo run -p p2-03-03-from-into-tryfrom --example 04-tryfrom-for-your-own-type
cargo run -p p2-03-03-from-into-tryfrom --example 05-tryinto-for-free
cargo run -p p2-03-03-from-into-tryfrom --example 06-choosing-from-or-tryfrom
```

بعد سه‌تایِ خراب:

```sh
cargo run -p p2-03-03-from-into-tryfrom --example 07-result-not-a-value --features broken
cargo run -p p2-03-03-from-into-tryfrom --example 08-narrow-panics-on-overflow --features broken
cargo run -p p2-03-03-from-into-tryfrom --example 09-missing-error-type --features broken
```

بعد این‌ها را امتحان کن:

۱. در `02-numeric-narrowing-tryfrom`، `u8::try_from(255i32)` و `u8::try_from(256i32)` را هم چاپ کن. مرزِ دقیقِ بینِ `Ok` و `Err` کجاست؟
۲. در `03-as-vs-try-into`، به‌جایِ `u8`، تبدیل را به `i8` امتحان کن (`300i32 as i8`). همان عددِ ۴۴ را می‌بینی؟ چرا یا چرا نه؟
۳. در `06-choosing-from-or-tryfrom`، یک تابعِ `average(ratings: &[Rating]) -> f64` بنویس که میانگینِ چند `Rating` را برمی‌گرداند — با استفاده از همان `From<Rating> for u8` که آن‌جا نوشتیم.

---

## خطاهایی که خواهی دید

### `E0308` — نتیجه‌ی `try_into` یک مقدار نیست، یک `Result` است

`examples/07-result-not-a-value.rs` دقیقاً همان اشتباهی را دارد که بخشِ ۲ ازش حرف زد: `try_into()` یک `Result<Rating, RatingError>` برمی‌گرداند، نه یک `Rating`ِ خام — ولی این کد طوری نوشته شده انگار `From` است:

```text
error[E0308]: mismatched types
  --> phase2-intermediate\03-traits-and-generics\03-from-into-tryfrom\examples\07-result-not-a-value.rs:29:26
   |
29 |     let rating: Rating = 4u8.try_into();
   |                 ------   ^^^^^^^^^^^^^^ expected `Rating`, found `Result<_, _>`
   |                 |
   |                 expected due to this
   |
   = note: expected struct `Rating`
                found enum `Result<_, _>`
help: consider using `Result::expect` to unwrap the `Result<_, _>` value, panicking if the value is a `Result::Err`
   |
29 |     let rating: Rating = 4u8.try_into().expect("REASON");
   |                                        +++++++++++++++++
```

**کامپایلر به چه اعتراض دارد:** پیام خودش دقیق است — «`Rating` انتظار می‌رفت، `Result<_, _>` پیدا شد». `let rating: Rating` نوعِ دقیقِ `Rating` را می‌خواهد؛ `4u8.try_into()` یک `Result` می‌دهد، نه `Rating` را مستقیم — دقیقاً همان فرقی که `TryFrom` را از `From` جدا می‌کند.

**راه‌حل:** با `Result` همان‌طور رفتار کن که با هر `Result`ِ دیگری — `match`، `?`، یا (وقتی واقعاً مطمئنی جواب `Ok` است) `.unwrap()`:

```rust
let rating: Rating = 4u8.try_into().unwrap();
```

**چرا این راه‌حل است:** پیشنهادِ خودِ کامپایلر (`.expect("REASON")`) هم کار می‌کند، ولی صادق‌تر این است که بدانی این یک تصمیمِ آگاهانه است، نه یک رفعِ خطا: یا واقعاً باور داری اینجا `Err` نمی‌آید (و `.expect()` را با یک دلیل می‌نویسی)، یا باید هر دو حالت را جواب بدهی.

### پنیک — `.unwrap()` رویِ یک `Err` از سرریز

`examples/08-narrow-panics-on-overflow.rs` کامپایل می‌شود — `.unwrap()` رویِ هر `Result<T, E>`ای تایپ‌چک می‌شود — و درست همان‌جایی می‌میرد که `300` تویِ یک `u8` جا نمی‌شود:

```text
thread 'main' (416) panicked at phase2-intermediate\03-traits-and-generics\03-from-into-tryfrom\examples\08-narrow-panics-on-overflow.rs:10:38:
called `Result::unwrap()` on an `Err` value: TryFromIntError(PosOverflow)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**کامپایلر به چه اعتراض دارد:** این حتی خطایِ کامپایلر نیست — برنامه ساخته و اجرا شد، و رویِ اولین `Err`ی که `.unwrap()` دید پنیک گرفت. همان `PosOverflow`ای که بخشِ ۲ نشانت داد، این‌بار به‌جایِ چاپ‌شدنِ آرام، برنامه را متوقف کرد.

**راه‌حل:** یا مطمئن شو مقدار جا می‌شود، یا یک راهِ صادقانه برایِ حالتِ `Err` هم بگذار — همان‌طور که `saturating_narrow` تویِ تمرین‌ها انجامش می‌دهد:

```rust
let narrowed = u8::try_from(300i32).unwrap_or(u8::MAX);
```

**چرا این راه‌حل است:** `.unwrap()` قمار می‌کند که همیشه `Ok` می‌آید؛ وقتی این قمار غلط از آب دربیاید، برنامه به‌جایِ دادنِ یک جوابِ غلط، کاملاً می‌ایستد — که غالباً بهتر است، ولی نه همیشه چیزی است که می‌خواهی. `.unwrap_or(...)` (که [۱.۶.۲](../../../phase1-fundamentals/06-absence-and-failure/02-option-combinators/README.fa.md) بهت یاد داد) یک مسیرِ سومِ صریح می‌دهد: نه پنیک، نه دروغ، یک مقدارِ جایگزینِ صریح.

### `E0046` — `TryFrom` دو چیز لازم دارد، نه یکی

`examples/09-missing-error-type.rs` متدِ `try_from` را می‌نویسد ولی `type Error` را جا می‌اندازد — دقیقاً همان چیزی که بخشِ ۲ نشانت داد trait اعلامش می‌کند:

```text
error[E0046]: not all trait items implemented, missing: `Error`
  --> phase2-intermediate\03-traits-and-generics\03-from-into-tryfrom\examples\09-missing-error-type.rs:16:1
   |
16 | impl TryFrom<u8> for Rating {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `Error` in implementation
   |
   = help: implement the missing item: `type Error = /* Type */;`
```

**کامپایلر به چه اعتراض دارد:** `TryFrom` دو عضو دارد، یک نوعِ وابسته (`Error`) و یک متد (`try_from`) — نوشتنِ فقط یکی‌شان یعنی این `impl` ناقص است، دقیقاً مثلِ [۲.۳.۱](../01-defining-and-implementing-traits/README.fa.md) که نشانت داد هر متدِ بدونِ پیاده‌سازیِ پیش‌فرض اجباری است.

**راه‌حل:** خطِ جا افتاده را اضافه کن:

```rust
impl TryFrom<u8> for Rating {
    type Error = RatingError;
    // ...
}
```

**چرا این راه‌حل است:** پیامِ خطا خودش دقیقاً همین را پیشنهاد می‌دهد — «`type Error = /* Type */;` را پیاده‌سازی کن». وقتی داری `TryFrom` را رویِ یک نوعِ کاملاً تازه، از صفر، دستی می‌نویسی (نه رویِ یک اسکلتِ آماده)، این خط همان‌قدر عادت است که خودِ `try_from` — یادت نرود.

---

## تمرین

### گرم‌کردن

<details>
<summary>این چه چاپ می‌کند؟</summary>

```rust
let x: Result<u8, _> = u8::try_from(-5i32);
println!("{x:?}");
```

</details>

<details>
<summary>پاسخ</summary>

```text
Err(TryFromIntError(NegOverflow))
```

`۵-` منفی است و هیچ `u8`ای منفی نیست، پس سرریز از سمتِ پایین — همان `NegOverflow`ای که بخشِ ۲ نشانت داد.

</details>

<details>
<summary><code>let x: u8 = 10i32.into();</code> کامپایل می‌شود؟</summary>

فکرت را قبل از خواندنِ پاسخ بنویس.

</details>

<details>
<summary>پاسخ</summary>

نه. `E0277` می‌گیری: «`u8: From<i32>` برقرار نیست». هیچ `impl From<i32> for u8`ای وجود ندارد — یک `i32` ممکن است تویِ `u8` جا نشود، و `From` قولِ «همیشه» می‌دهد. باید `.try_into()` بنویسی.

</details>

<details>
<summary><code>256i32 as u8</code> چه چیزی می‌شود؟</summary>

فکرت را بنویس.

</details>

<details>
<summary>پاسخ</summary>

`۰`. `as` فقط پایین‌ترین ۸ بیت را نگه می‌دارد، و ۲۵۶ در آن ۸ بیت همه‌اش صفر است — نه خطا، نه پنیک، فقط یک عددِ غلط.

</details>

<details>
<summary>درست یا نادرست: هر نوعی که به‌عنوانِ <code>Error</code>ِ یک <code>TryFrom</code> استفاده می‌شود، باید <code>std::error::Error</code> را پیاده‌سازی کند.</summary>

فکرت را بنویس.

</details>

<details>
<summary>پاسخ</summary>

نادرست. `RatingError` تویِ همین درس فقط `#[derive(Debug, Clone, PartialEq, Eq)]` دارد و همین برایِ `TryFrom` کاملاً کافی است. `std::error::Error` مالِ [۲.۵.۱](../../05-error-handling/01-custom-error-types/README.fa.md) است، جلوتر.

</details>

<details>
<summary>اگر فیلدِ <code>Rating</code> به‌جایِ خصوصی، <code>pub</code> بود، آیا هر <code>Rating</code>ای که وجود دارد هنوز حتماً معتبر بود؟</summary>

فکرت را بنویس.

</details>

<details>
<summary>پاسخ</summary>

نه. هرکسی می‌توانست مستقیم `Rating(9)` بسازد و از کنارِ `TryFrom::try_from` رد شود. تضمین کاملاً به خصوصی‌بودنِ فیلد وابسته است.

</details>

### تعمیر

هر سه مثالِ خراب را درست کن:

۱. `examples/07-result-not-a-value.rs` را **دو** جور درست کن: یک‌بار با اضافه‌کردنِ `.unwrap()`، یک‌بار با تغییرِ نوعِ `let rating` به `Result<Rating, RatingError>`.
۲. `examples/08-narrow-panics-on-overflow.rs` را طوری درست کن که دیگر پنیک نگیرد — با `.unwrap_or(u8::MAX)` یا یک `match` که هر دو حالت را جواب بدهد.
۳. `examples/09-missing-error-type.rs` را با اضافه‌کردنِ خطِ `type Error = RatingError;` درست کن.

### پیاده‌سازی

چهار چیز در `src/lib.rs`:

```sh
cargo test -p p2-03-03-from-into-tryfrom
```

`impl From<Percentage> for f64` و `impl TryFrom<u8> for Percentage` هردو رویِ یک نوع کار می‌کنند، دو جهتِ مختلف — دقیقاً الگویِ بخشِ ۶. `impl TryFrom<String> for EmailAddress` همان الگویِ نیوتایپِ اعتبارسنجی‌شده‌ی بخشِ ۴ است، این‌بار رویِ یک `String`. `saturating_narrow` همان `try_into` با یک مسیرِ صریح برایِ حالتِ سرریز است که در «خطاهایی که خواهی دید» دیدی، این‌بار بینِ `u64` و `u32`.

هر چهارتا را دقیق از رویِ کامنتِ مستندسازِ بالای هرکدام پیاده کن — چیزی را حدس نزن.

### بساز

یک نیوتایپِ اعتبارسنجی‌شده برایِ دامنه‌ای که خودت انتخاب می‌کنی بنویس — یک کدِ کشورِ دو-حرفی، یک نمره‌ی درسی (`0..=20`)، یک شماره‌یِ پورتِ شبکه (`1..=65535`). یک `TryFrom` برایِ ساختنش از رویِ نوعِ خام بنویس، و یک `From` هم برایِ برگرداندنِ مقدارِ داخلی به بیرون — دقیقاً همان جفتی که `Rating` تویِ این درس نشانت داد.

### چالش (اختیاری)

**بخشِ یک.** یک `pub fn saturating_narrow_to_i8(value: i32) -> i8` بنویس. مواظبِ طرفِ منفی هم باش — `i32::MIN` باید به `i8::MIN` بچسبد، نه فقط طرفِ مثبت به `i8::MAX`.

**بخشِ دو.** فرض کن می‌خواهی `impl From<Rating> for u8` بنویسی — دقیقاً همان چیزی که بخشِ ۶ نوشت — ولی این‌بار `Rating` مالِ کریتِ خودت نیست، از یک کریتِ بیرونی آمده، و `u8` هم که همیشه مالِ کتابخانه‌ی استاندارد بوده، نه مالِ تو. آیا این `impl` را می‌توانی همچنان بنویسی؟ یک جمله حدس بزن چرا یا چرا نه — کدش لازم نیست؛ جوابِ کامل [۲.۳.۶](../06-supertraits-blanket-impls-orphan-rule/README.fa.md) است.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| `From<T> for U` | تبدیلِ همیشه‌موفقِ `T` به `U` (یادآوری از ۱.۶.۵) | نوع‌هایِ خودت، تبدیل‌هایِ عددیِ پهن‌شونده |
| `TryFrom<T> for U` | تبدیلی که ممکن است شکست بخورد؛ `try_from` یک `Result<U, Self::Error>` می‌دهد | اعتبارسنجی، پارس‌کردن، تبدیلِ عددیِ کوچک‌شونده |
| `TryInto` | نتیجه‌ی مجانیِ نوشتنِ `TryFrom` | `.try_into()` — هیچ‌وقت دستی پیاده‌سازی نمی‌شود |
| `as` | تبدیلِ عددیِ ساکت؛ بیت‌هایِ اضافه را بی‌سروصدا دور می‌ریزد | فقط جایی که بریدنِ بیت‌ها واقعاً همان چیزی است که می‌خواهی |
| `TryFromIntError` | خطایِ استانداردِ تبدیل‌هایِ عددیِ کوچک‌شونده | `PosOverflow` / `NegOverflow` |
| `E0046` | یک `impl` که یک عضوِ اجباریِ trait را جا انداخته | فراموش‌کردنِ `type Error = ...;` |

### الان می‌دانی

- `From<T> for U` قولِ «هرگز شکست نمی‌خورد» می‌دهد؛ `TryFrom<T> for U` همان تبدیل را با یک `Result<U, Self::Error>` می‌دهد.
- `TryFrom` و `TryInto` رایگان مالِ همدیگرند — دقیقاً مثلِ `From` و `Into`ای که ۱.۶.۵ نشانت داد.
- `as` هیچ‌وقت شکست نمی‌خورد چون هیچ‌وقت واقعاً چک نمی‌کند — فقط بیت‌ها را می‌بُرد؛ `try_into()` همان تبدیل را صادقانه گزارش می‌دهد.
- یک نیوتایپ با فیلدِ خصوصی و سازنده‌ی `TryFrom`، تضمین می‌کند هر مقدار از نوعش که وجود دارد، اعتبارسنجی شده — بدونِ اینکه لازم باشد دوباره چکش کنی.
- قاعده‌ی انتخاب: اگر هر ورودیِ معتبری همیشه جواب می‌دهد، `From`؛ اگر حتی یکی هست که باید رد شود، `TryFrom` — حتی رویِ یک نوعِ واحد، دو جهتِ متفاوتش می‌توانند دو جوابِ متفاوت داشته باشند.

### بعداً کامل‌تر می‌بینی

- **`Display` دستی، برایِ اینکه خطایت پیامِ خوانا بدهد نه فقط `{:?}`** — [۲.۳.۴ — مشتق‌هایِ استاندارد، دستی پیاده‌سازی‌شده](../04-standard-derives-by-hand/README.fa.md)
- **نوع‌هایِ وابسته (associated types) به‌طورِ کامل** — [۲.۳.۵ — نوع‌هایِ وابسته](../05-associated-types/README.fa.md)
- **قاعده‌ی یتیم — چرا `impl From<Rating> for u8` مجاز بود ولی هر ترکیبی مجاز نیست** — [۲.۳.۶ — ابرصفت‌ها، پیاده‌سازیِ فراگیر و قاعده‌ی یتیم](../06-supertraits-blanket-impls-orphan-rule/README.fa.md)
- **`std::error::Error`، برایِ یک نوعِ خطایِ واقعاً کامل** — [۲.۵.۱ — نوع‌هایِ خطایِ سفارشی](../../05-error-handling/01-custom-error-types/README.fa.md)
- **`thiserror` و `anyhow`، برایِ وقتی نمی‌خواهی این کدهایِ تکراری را خودت بنویسی** — [۲.۵.۳ — `thiserror` و `anyhow`](../../05-error-handling/03-thiserror-and-anyhow/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا `i32::from(u8)` وجود دارد ولی `u8::from(i32)` نه؟
- `try_into()` مقصدش را از کجا می‌فهمد؟ دقیقاً همان سؤالی که ۱.۶.۵ برایِ `.into()` پرسید.
- چرا `as` هیچ‌وقت پنیک نمی‌گیرد، حتی وقتی جواب کاملاً غلط است؟
- نیوتایپِ `Rating` چطور تضمین می‌کند هر نمونه‌اش معتبر است، بدونِ اینکه هیچ‌جایِ دیگرِ کد دوباره چکش کند؟
- برایِ یک نوعِ واحد، چرا یک جهتِ تبدیل می‌تواند `From` باشد و جهتِ دیگرش `TryFrom`؟
- `TryFrom`ای که `type Error` را جا انداخته، دقیقاً کِی و با چه خطایی این را به‌ات می‌گوید؟

---

## بیشتر

- [مستنداتِ `std::convert::From`](https://doc.rust-lang.org/std/convert/trait.From.html) — فهرستِ کاملِ تبدیل‌هایِ عددیِ پهن‌شونده که خودِ کتابخانه‌ی استاندارد پیاده‌سازی کرده.
- [مستنداتِ `std::convert::TryFrom`](https://doc.rust-lang.org/std/convert/trait.TryFrom.html) — همان‌جا، برایِ جفتِ خطاپذیر.
- [راهنمایِ نسخه‌هایِ Rust — افزوده‌هایِ پرلود در ۲۰۲۱](https://doc.rust-lang.org/edition-guide/rust-2021/prelude.html) — چرا امروز هیچ‌جا لازم نبود `use std::convert::TryFrom;` بنویسیم.
- [لینتِ `clippy::cast_possible_truncation`](https://rust-lang.github.io/rust-clippy/master/#cast_possible_truncation) — کلیپی که دقیقاً همین اشتباه را نشانه می‌رود: یک `as` که می‌تواند بی‌سروصدا داده گم کند.
