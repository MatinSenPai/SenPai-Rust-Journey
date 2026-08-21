# ۱.۶.۵ — `From` و تبدیلِ خطا

## در یک نگاه

بعد از این درس می‌توانی:

- بگویی وقتی `?` یک `Err` را زودتر برمی‌گرداند دقیقاً چه کاری با آن می‌کند — نه فقط برش می‌گرداند، رویش `From::from` هم صدا می‌زند.
- برای نوعِ خطای خودت `impl From<A> for MyError` بنویسی، و بگویی چرا با نوشتنِ همین یکی، `.into()` هم مجانی به دستت می‌رسد.
- بین جایی که استنتاجِ نوع می‌تواند مقصدِ `.into()` را حدس بزند و جایی که نمی‌تواند فرق بگذاری.
- یک `E0277` از `?` ببینی و به‌جای زدنِ یک `.map_err()` دیگر، `From` گمشده را بنویسی.

**زمان:** حدود ۵۰ دقیقه · **پیش‌نیاز:** [۱.۶.۴ — پنیک در برابرِ `Result`](../04-panic-vs-result/README.fa.md) · [۱.۶.۳ — `Result` و علامتِ سؤال](../03-result-and-question-mark/README.fa.md)

---

## چرا اهمیت دارد

درسِ قبل عملگر `?` را همین‌طور معرفی کرد: «اگر `Err` بود، همان لحظه از تابعِ فعلی `Err` را برگردان.» این توضیح تا وقتی هر تابعی که صدا می‌زنی همان نوعِ خطایی را برمی‌گرداند که خودِ تو هم برمی‌گردانی، کاملاً کافی است.

مشکل از جایی شروع می‌شود که یک تابع باید سه کارِ متفاوت را پشتِ سرِ هم انجام بدهد، و هرکدام به روشِ خودش شکست می‌خورد: یک رشته را به عدد صحیح تبدیل کن (خطایش `ParseIntError`)، رشته‌ی دیگری را به عددِ اعشاری تبدیل کن (خطایش `ParseFloatError`)، یک برچسب را با یک قاعده‌ی کوچک اعتبارسنجی کن (خطایش هرچه خودت طراحی کرده‌ای). تابعت فقط یک `Result<T, E>` برمی‌گرداند — یعنی دقیقاً یک نوعِ `E`، نه سه نوع. اگر همین‌جا بی‌فکر `?` را روی هرسه بگذاری، کامپایلر پیغامی می‌دهد که در نگاهِ اول انگار ربطی به کاری که تو کردی ندارد.

راهِ رایج این است که هرکدام را دستی تبدیل کنی. کار می‌کند، ولی همان کارِ تکراری را سه‌بار می‌نویسی، و هر تابعِ بعدی که سه فراخوانیِ خطاپذیرِ دیگر دارد همین تکرار را از نو می‌خواهد.

نکته‌ای که این درس بازش می‌کند این است: **`?` از همان اول هم دقیقاً همین تبدیل را انجام می‌داده — فقط با نامِ خودش صدایش نکرده بودی.**

---

## مفهوم

### یادآوریِ کوتاهِ `?` — و سؤالی که هنوز جواب نداده

در [۱.۶.۳](../03-result-and-question-mark/README.fa.md) یاد گرفتی که `expr?` یعنی: اگر `Ok(value)` بود، `value` را دربیاور و ادامه بده؛ اگر `Err(e)` بود، همین الان `Err(e)` را از تابعِ فعلی برگردان. این توضیح یک سؤال را باز می‌گذارد: اگر نوعِ خطاییِ `e` با نوعِ خطاییِ تابعِ فعلی یکی نباشد چه؟ کامپایلر که نمی‌تواند دو نوعِ متفاوت را در یک `Result` قاطی کند.

جوابش را همین درس می‌دهد.

### راهِ دستی: یک `match` و دو `.map_err()`

فرض کن یک خطِ متن مثلِ `"12,36.6,C"` را می‌خوانی — شناسه، مقدار، واحد — و سه‌تایش را جدا-جدا پارس می‌کنی. سه فراخوانیِ خطاپذیر، سه نوعِ خطای متفاوت:

```rust
let id = match id_str.parse::<u32>() {
    Ok(id) => id,
    Err(err) => return Err(ReadingError::BadId(err)),
};
```

این همان چیزی است که از [۱.۶.۳](../03-result-and-question-mark/README.fa.md) بلدی: یک `match` روی `Result`، و در بازوی `Err` یک بازگشتِ زودهنگام. کار می‌کند، ولی برای هر فراخوانیِ خطاپذیر باید همین چهار خط را دوباره بنویسی.

می‌شود کوتاهش کرد. `Result` متدی به‌نامِ **`.map_err()`** دارد: اگر `Ok` باشد دست‌نخورده رد می‌شود، اگر `Err(e)` باشد، تابعی که بهش می‌دهی روی `e` اجرا می‌شود و نتیجه‌اش `Err` جدید می‌شود. همان تبدیل، در یک خط:

```rust
let value = value_str.parse::<f64>().map_err(ReadingError::BadValue)?;
let unit = parse_unit(unit_str).map_err(ReadingError::BadUnit)?;

Ok((id, value, unit))
```

`ReadingError::BadValue` اینجا خودِ enum‌ساز (variant) است — Rust هر enum‌سازِ داده‌دار را می‌تواند مثلِ یک تابع صدا بزند، پس همین برای `.map_err()` کافی است.

اجرا کن و ببین:

```sh
cargo run -p p1-06-05-from-and-error-conversion --example 02-error-conversion-by-hand
```

```text
"12,36.6,C" -> Ok((12, 36.6, Celsius))
"x,36.6,C" -> Err(BadId(ParseIntError { kind: InvalidDigit }))
"12,hot,C" -> Err(BadValue(ParseFloatError { kind: Invalid }))
"12,36.6,K" -> Err(BadUnit("K"))
```

کار می‌کند، درست جواب می‌دهد — ولی سه بارِ دیگر که یک تابعِ مشابه بنویسی، همین سه خطِ تبدیل را دوباره تایپ می‌کنی.

### رازی که `?` مخفی کرده بود

این جمله را دقیق بخوان، چون کلِ درس همین است. `expr?` فقط «اگر `Err` بود برگردان» نیست. مفهومش این است:

```rust
// این یعنی چه: `expr?`
match expr {
    Ok(value) => value,
    Err(error) => return Err(From::from(error)),
}
```

آن خطِ آخر را ببین. `?` قبل از برگرداندنِ خطا، رویش `From::from` را صدا می‌زند. یعنی اگر یک راه بگویی «چطور یک `ParseIntError` را به `ReadingError` تبدیل کن»، `?` خودش همان راه را در همان لحظه استفاده می‌کند — بدونِ اینکه در محلِ فراخوانی چیزی بنویسی.

آن راهی که باید بگویی، یک **`impl From<A> for MyError`** است.

### نوشتنِ `From<A> for MyError`

`From` یک صفت (trait) استاندارد است با یک متدِ تنها: `from`. برای هر خطای منبع که `parse_reading` ممکن است ببیند، یک `impl From<آن‌خطا> for ReadingError` می‌نویسیم:

```rust
impl From<ParseIntError> for ReadingError {
    fn from(err: ParseIntError) -> Self {
        ReadingError::BadId(err)
    }
}

impl From<ParseFloatError> for ReadingError {
    fn from(err: ParseFloatError) -> Self {
        ReadingError::BadValue(err)
    }
}
```

و برای سومی — `UnknownUnit`، خطایی که خودمان تعریف کرده‌ایم، نه از کتابخانه‌ی استاندارد — دقیقاً همین شکل، یک `impl` سه‌خطیِ دیگر. `From` فرقی نمی‌گذارد خطای منبع از کجا آمده؛ فقط باید یک راهِ ساختنِ `ReadingError` از رویش بلد باشد.

حالا تنِ اصلیِ تابع همین است — سه `?` خالی، هیچ `.map_err()`ای نیست:

```rust
let id = id_str.parse::<u32>()?;
let value = value_str.parse::<f64>()?;
let unit = parse_unit(unit_str)?;

Ok((id, value, unit))
```

```sh
cargo run -p p1-06-05-from-and-error-conversion --example 03-error-conversion-with-from
```

```text
"12,36.6,C" -> Ok((12, 36.6, Celsius))
"x,36.6,C" -> Err(BadId(ParseIntError { kind: InvalidDigit }))
"12,hot,C" -> Err(BadValue(ParseFloatError { kind: Invalid }))
"12,36.6,K" -> Err(BadUnit(UnknownUnit("K")))
```

همان ورودی‌ها، همان جواب‌ها. تنها فرق این است که هر سه تبدیل را یک‌بار، بالای فایل، نوشته‌ای — نه هر بار که یک تابعِ تازه سراغِ همین سه خطا می‌رود. حالا وقتی تابعِ چهارمی هم بنویسی که `ParseIntError` می‌بیند، همین `impl` که بالا نوشتی برایش کار می‌کند؛ چیزی دوباره‌نویسی نمی‌شود.

### `Into` را مجانی می‌گیری

کتابخانه‌ی استاندارد یک صفتِ دوم هم دارد به‌نامِ `Into`، ولی هیچ‌جای این درس یک‌بار هم `impl Into` ننوشتیم. دلیلش یک پیاده‌سازیِ فراگیر (blanket impl) است که خودِ کتابخانه‌ی استاندارد یک‌بار برای همیشه نوشته:

```rust
impl<T, U> Into<U> for T
where
    U: From<T>,
{
    fn into(self) -> U {
        U::from(self)
    }
}
```

نگرانِ نحوش نباش — نوشتنِ چیزی به این شکل کارِ [فاز ۲](../../../phase2-intermediate/03-generics-and-traits/02-defining-and-implementing-traits/README.fa.md) است. فقط بخوانش: برای هر دو نوعِ `T` و `U`، اگر `U: From<T>` وجود داشته باشد، آن‌وقت `T: Into<U>` هم خودبه‌خود وجود دارد، و پیاده‌سازیِ `into` فقط `U::from(self)` را صدا می‌زند. **هر `From`ای که تو می‌نویسی، یک `Into` رایگان همراهش می‌آید.** برعکسش هم درست است ولی بی‌فایده: چون این پیاده‌سازیِ فراگیر همین الان همه‌جا هست، دیگر هیچ‌کس مستقیم `impl Into` نمی‌نویسد.

نتیجه‌اش `.into()` است — همان تبدیل، از جهتِ مقصد خوانده‌شده:

```rust
let box_weight = Kilograms(5.0);
let in_grams: Grams = box_weight.into();
```

```text
.into() (typed let): 5000
```

هیچ‌جا `impl Into<Grams> for Kilograms` ننوشتیم. فقط `impl From<Kilograms> for Grams` را نوشتیم، و `.into()` همراهش آمد.

### `.into()` — کِی استنتاجِ نوع جواب می‌دهد

`.into()` باید بداند مقصدش چیست، و خودش این را از **جایی که مقدار قرار است بنشیند** می‌گیرد — نه از خودِ `.into()`:

```rust
fn print_grams(amount: Grams) {
    println!("as grams:   {}", amount.0);
}

print_grams(Kilograms(2.0).into());

let in_pounds: Pounds = Kilograms(5.0).into();
```

```text
as grams:   2000
.into() (as Pounds): 11.0231
```

همان `Kilograms`، دو مقصدِ متفاوت. بارِ اول امضای `print_grams` می‌گوید مقصد `Grams` است؛ بارِ دوم نوعِ نوشته‌شده‌ی `let` می‌گوید `Pounds`. کامپایلر مقصد را از این‌جور جاها می‌خواند: نوعِ صریحِ یک `let`، نوعِ پارامترِ تابعی که مقدار به آن داده می‌شود، نوعِ بازگشتیِ تابعی که مقدار از آن برمی‌گردد.

وقتی هیچ‌کدام از این‌ها مقصد را مشخص نکند — یعنی چند `From<Kilograms>` وجود داشته باشد و هیچ‌چیز نگوید کدام‌شان — کامپایلر نمی‌تواند حدس بزند. آن خطا را در بخشِ بعد می‌بینی.

### `From` برای تبدیل‌های بدونِ خطا هم هست

`From` فقط برای خطاها نیست. از فازِ صفر همین یک متد را بدونِ اسمش صدا می‌زدی:

```rust
let name = String::from("Matin");
let wide: u64 = u64::from(42_u32);
```

```text
String::from: Matin
u64::from:  42
```

`String::from("Matin")` دقیقاً `<String as From<&str>>::from` است — تبدیلِ یک `&str` به یک `String` که مالکِ بافرِ خودش است. `u64::from(42_u32)` هم یک `impl From<u32> for u64` است که کتابخانه‌ی استاندارد نوشته، چون هر `u32` جا دارد داخلِ یک `u64`؛ این تبدیل هرگز داده‌ای را گم نمی‌کند.

و همین قاعده توضیح می‌دهد چرا `i32::from(x: i64)` **وجود ندارد**. یک `i64` ممکن است بزرگ‌تر از چیزی باشد که در `i32` جا شود؛ `From` قولِ «هیچ‌وقت شکست نمی‌خورد» می‌دهد، و این تبدیل نمی‌تواند این قول را نگه دارد. برای این جهت دو ابزار داری: `as` (که در [۱.۱.۲](../../01-foundations/02-scalar-types-and-overflow/README.fa.md) دیدی — بی‌سروصدا قطع می‌کند، بدونِ خطا)، یا خانواده‌ی **`TryFrom`**، که دقیقاً همین قول را می‌دهد ولی با یک `Result`: «اگر جا شد، تبدیلت می‌کنم؛ اگر نشد، یک `Err` می‌دهم.» `TryFrom` را در [فاز ۲](../../../phase2-intermediate/08-rust-toolbox/04-tryfrom-fallible-conversions/README.fa.md) کامل می‌بینی؛ همین‌قدر بدان که خواهرِ خطاپذیرِ همین `From` است که امروز یاد گرفتی.

### قانونِ طراحیِ یک enum خطا

الگویی که در `ReadingError` دیدی همه‌جا تکرار می‌شود:

```rust
enum AppError {
    Parse(ParseIntError),   // یک From دارد؛ ? از همان استفاده می‌کند
    Validation(MyOwnError), // این هم From دارد؛ همان مکانیزم
    NotFound,                // مستقیم ساخته می‌شود؛ چیزی برای تبدیل نیست
}
```

قاعده: **یک enum‌ساز به‌ازای هر جور شکست، و یک `impl From` به‌ازای هر خطای بیرونی که با `?` واردش می‌شود.** هر enum‌سازی هم لازم نیست از راهِ `From` بیاید — `NotFound` را جایی که تشخیصش می‌دهی مستقیم می‌سازی، چون خودش از اول همان نوعِ خطای توست.

یک نکته‌ی آخر: اگر دو فراخوانیِ مختلف دقیقاً همان نوعِ خطای منبع را برگردانند (مثلاً پارس‌کردنِ ساعت *و* پارس‌کردنِ دقیقه، هردو `ParseIntError`)، فقط یک `impl From<ParseIntError>` می‌توانی داشته باشی، و آن یکی هر دو را به همان enum‌ساز می‌فرستد — `?` نمی‌تواند بگوید کدام‌یک بود. آن‌جا که این تفاوت برایت مهم است، `.map_err()` هنوز درست‌ترین ابزار است، چون در محلِ فراخوانی مشخص می‌کنی کدام enum‌ساز. `From`/`?` برای وقتی است که خطای منبع خودش کافی است؛ `.map_err()` برای وقتی است که محلِ فراخوانی هم باید در جواب باشد.

---

## دست‌به‌کد

```sh
cargo run -p p1-06-05-from-and-error-conversion --example 01-from-and-into-basics
cargo run -p p1-06-05-from-and-error-conversion --example 02-error-conversion-by-hand
cargo run -p p1-06-05-from-and-error-conversion --example 03-error-conversion-with-from
```

بعد دو تای خراب:

```sh
cargo run -p p1-06-05-from-and-error-conversion --example 04-ambiguous-into --features broken
cargo run -p p1-06-05-from-and-error-conversion --example 05-question-mark-without-from --features broken
```

بعد این‌ها را امتحان کن:

۱. در `02-error-conversion-by-hand`، یک خطِ ورودیِ جدید اضافه کن که سه‌فیلد نداشته باشد (مثلاً `"12,36.6"`). چه `Err`ی می‌گیری، و چرا؟
۲. در `03-error-conversion-with-from`، سطرهای `impl From` را موقتاً کامنت کن و ببین کامپایلر دقیقاً به کدام `?` اعتراض می‌کند.
۳. در `01-from-and-into-basics`، یک نوعِ سومِ مقصد اضافه کن (مثلاً `Ounces`) با `impl From<Kilograms> for Ounces` خودت، و یک `.into()` تازه با نوعِ صریح برایش بنویس.

---

## خطاهایی که خواهی دید

### `E0277` — `?` نمی‌تواند خطا را تبدیل کند

این همان خطایی است که کلِ این درس دورش ساخته شده. `examples/05-question-mark-without-from.rs` یک تابع دارد که `Result<u32, ReadingError>` برمی‌گرداند ولی داخلش از `.parse::<u32>()` که `ParseIntError` می‌دهد، مستقیم با `?` استفاده می‌کند — بدونِ هیچ `impl From<ParseIntError> for ReadingError`ای:

```text
error[E0277]: `?` couldn't convert the error to `ReadingError`
  --> examples\05-question-mark-without-from.rs:12:32
   |
 8 | fn parse_id(raw: &str) -> Result<u32, ReadingError> {
   |                           ------------------------- expected `ReadingError` because of this
...
12 |     let id = raw.parse::<u32>()?;
   |                  --------------^ the trait `From<ParseIntError>` is not implemented for `ReadingError`
   |                  |
   |                  this can't be annotated with `?` because it has type `Result<_, ParseIntError>`
   |
note: `ReadingError` needs to implement `From<ParseIntError>`
  --> examples\05-question-mark-without-from.rs:6:1
   |
 6 | struct ReadingError;
   | ^^^^^^^^^^^^^^^^^^^
   = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the `From` trait
```

**کامپایلر به چه اعتراض دارد:** خطِ آخرِ پیام کاملاً صریح است — «عملگرِ `?` به‌طورِ ضمنی با صفتِ `From` روی مقدارِ خطا یک تبدیل انجام می‌دهد.» یعنی همان رازی که در بخشِ «مفهوم» باز شد، اینجا از زبانِ خودِ `rustc` می‌آید. `?` دید که `raw.parse::<u32>()` یک `Result<u32, ParseIntError>` است، خواست خطایش را به `ReadingError` تبدیل کند تا بتواند `Err`ش را از تابع برگرداند، و هیچ راهی برای این تبدیل پیدا نکرد.

**راه‌حل:** دقیقاً همان سه‌خطی که در بخشِ «نوشتنِ `From<A> for MyError`» نوشتیم:

```rust
impl From<ParseIntError> for ReadingError {
    fn from(err: ParseIntError) -> Self {
        ReadingError::BadId(err)
    }
}
```

**چرا این راه‌حل است:** یادداشتِ کامپایلر می‌گوید «`ReadingError` needs to implement `From<ParseIntError>`» — تحت‌اللفظی همان چیزی که باید بنویسی. این خطا وسوسه می‌کند که یک `.map_err(|_| ReadingError)` بزنی تا خفه شود؛ اگر همین یک‌بار این کار را بکنی مشکلی نیست، ولی اگر همین تابع سه فراخوانیِ دیگر هم داشته باشد که همین خطا را می‌دهند، داری همان کارِ تکراری‌ای را می‌نویسی که این درس آمده حذفش کند.

### `E0282` — `.into()` نمی‌داند مقصدش چیست

```text
error[E0282]: type annotations needed
  --> examples\04-ambiguous-into.rs:24:9
   |
24 |     let result = Kilograms(5.0).into();
   |         ^^^^^^
25 |     println!("{}", result.0);
   |                    ------ type must be known at this point
   |
help: consider giving `result` an explicit type
   |
24 |     let result: /* Type */ = Kilograms(5.0).into();
   |               ++++++++++++
```

**کامپایلر به چه اعتراض دارد:** در `examples/04-ambiguous-into.rs` هم `impl From<Kilograms> for Grams` هست و هم `impl From<Kilograms> for Pounds`. وقتی `Kilograms(5.0).into()` را می‌نویسی و بعد آن را در `let result` بدونِ نوعِ صریح می‌ریزی، هیچ‌چیز نمی‌گوید مقصد کدام‌شان است. `result.0` هم کمکی نمی‌کند، چون هر دو نوع یک فیلدِ `f64` دارند.

**راه‌حل:** همان دو راهی که در بخشِ «`.into()` — کِی استنتاجِ نوع جواب می‌دهد» دیدی — یک نوعِ صریح روی `let` بگذار (`let result: Grams = ...`)، یا مقدار را جایی بده که نوعش از قبل مشخص است (پارامترِ یک تابع، یا بازگشتیِ یک تابع).

**چرا این راه‌حل است:** پیامِ کامپایلر خودش می‌گوید «type annotations needed» — نه «تبدیل ممکن نیست»، فقط «نمی‌دانم کدام تبدیل». همین تفاوت است که این خطا را از `E0277` بالا جدا می‌کند: آن‌جا هیچ `From`ی وجود نداشت، اینجا **بیش‌ازحد** `From` وجود دارد و کامپایلر بینِ‌شان گیر کرده.

---

## تمرین

### گرم‌کردن

<details>
<summary><code>expr?</code> وقتی <code>expr</code> برابرِ <code>Err(e)</code> است، دقیقاً چه چیزی را برمی‌گرداند؟</summary>

`Err(From::from(e))` را برمی‌گرداند، نه خودِ `Err(e)` را. `?` قبل از برگرداندن، رویِ `e` یک `From::from` صدا می‌زند تا آن را به نوعِ خطاییِ تابعِ فعلی تبدیل کند.

</details>

<details>
<summary>اگر <code>impl From&lt;ParseIntError&gt; for AppError</code> بنویسی، آیا هم باید <code>impl Into&lt;AppError&gt; for ParseIntError</code> هم بنویسی تا <code>.into()</code> کار کند؟</summary>

نه. یک پیاده‌سازیِ فراگیر در کتابخانه‌ی استاندارد هست که هر `From<T> for U` را خودکار به یک `Into<U> for T` تبدیل می‌کند. نوشتنِ `impl Into` دستی هم ممکن است ولی هیچ‌وقت لازم نیست.

</details>

<details>
<summary>آیا این کامپایل می‌شود: <code>let x: i32 = i32::from(5_i64);</code></summary>

نه، `E0277` می‌گیری. هیچ `impl From<i64> for i32` وجود ندارد، چون یک `i64` ممکن است بزرگ‌تر از چیزی باشد که در `i32` جا شود و `From` قولِ «هرگز شکست نمی‌خورد» می‌دهد.

</details>

<details>
<summary>اگر <code>Grams</code> و <code>Pounds</code> هردو <code>impl From&lt;Kilograms&gt;</code> داشته باشند، آیا <code>let x = Kilograms(1.0).into();</code> (بدونِ هیچ نوعِ صریحی، جایی) کامپایل می‌شود؟</summary>

نه، `E0282` می‌گیری — «type annotations needed». دو مقصدِ ممکن هست و هیچ‌چیز نمی‌گوید کدام‌شان.

</details>

<details>
<summary>درست یا نادرست: هر enum‌سازِ یک enumِ خطا باید یک <code>impl From</code> داشته باشد.</summary>

نادرست. فقط آن‌هایی که از تبدیلِ یک خطای بیرونی می‌آیند به `From` احتیاج دارند. enum‌سازی که خودت مستقیم می‌سازی (مثلِ یک خطای اعتبارسنجیِ ساده) به هیچ `From`ای احتیاج ندارد.

</details>

<details>
<summary>اگر دو فراخوانیِ مختلف در یک تابع، هر دو <code>ParseIntError</code> برگردانند، آیا <code>?</code> می‌تواند بگوید کدام‌یک شکست خورده؟</summary>

نه، نه با یک `impl From<ParseIntError>` تنها — هر دو به همان enum‌ساز می‌روند. اگر این تفاوت مهم است، باید در محلِ هر فراخوانی از `.map_err()` استفاده کنی، نه از `?`ِ خالی.

</details>

### تعمیر

`examples/04-ambiguous-into.rs` را **دو** جور درست کن:

۱. با گذاشتنِ یک نوعِ صریح روی `let result` (مثلاً `Grams` یا `Pounds`).
۲. با نوشتنِ فرمِ صریحِ `Grams::from(...)` یا `Pounds::from(...)` به‌جایِ `.into()`.

بعد `examples/05-question-mark-without-from.rs` را دو جور درست کن: یک‌بار با نوشتنِ `impl From<ParseIntError> for ReadingError` تا `?` مستقیم کار کند، یک‌بار بدونِ هیچ `From`ی — با `.map_err(...)` که مستقیم داخلش `ReadingError` می‌سازد.

### پیاده‌سازی

چهار چیز در `src/lib.rs` — دو تابع و دو `impl From`:

```sh
cargo test -p p1-06-05-from-and-error-conversion
```

`parse_color` و `impl From<ParseIntError> for ColorError` با هم کار می‌کنند: یکی صفتِ تبدیل را می‌سازد، دیگری با `?` خالی از آن استفاده می‌کند — دقیقاً همان الگویی که در «مفهوم» دیدی. `all_fahrenheit` و `impl From<Celsius> for Fahrenheit` هم همین رابطه را دارند، این‌بار برای یک تبدیلِ بدونِ خطا.

هر چهارتا را دقیق از رویِ کامنتِ مستندساز (doc comment) بالای هرکدام پیاده کن — همان‌جا فرمتِ ورودی، قاعده‌ی تاریکی، و فرمولِ دما دقیق نوشته شده.

### بساز

یک enum خطا و یک تابعِ پارس‌کننده برای دامنه‌ای که خودت انتخاب می‌کنی بنویس — مثلاً یک کسر (`"3/4"`)، یا یک تاریخ (`"2024-01-15"`)، یا یک مختصات (`"12.5,45.0"`). دستِ‌کم دو فراخوانیِ خطاپذیرِ متفاوت داشته باشد، هرکدام با نوعِ خطای متفاوت، و هرکدام یک `impl From` مالِ خودش.

بعد بشمار: نسخه‌ی نهاییِ تو با `?` خالی چند خط شد؟ اگر همان تابع را با `.map_err()` روی هر فراخوانی می‌نوشتی، چند خط می‌شد؟

### چالش (اختیاری)

**بخشِ یک.** یک تابعِ `parse_time(s: &str) -> Result<u32, TimeError>` بنویس که رشته‌ی `"HH:MM"` را به تعدادِ کلِ دقیقه‌ها تبدیل می‌کند، با `TimeError` دارایِ یک enum‌سازِ تنها — `BadNumber(ParseIntError)` — و یک `impl From<ParseIntError>` که هم پارس‌کردنِ ساعت و هم پارس‌کردنِ دقیقه از آن رد می‌شوند. بعد `parse_time("aa:30")` و `parse_time("12:bb")` را امتحان کن. هردو شکست می‌خورند، هردو `TimeError::BadNumber(_)` می‌دهند. آیا از رویِ خودِ `Err` می‌توانی بگویی کدام‌یک بود، ساعت یا دقیقه؟ اگر این تفاوت برایت مهم بود، دقیقاً کجای کد را باید عوض می‌کردی؟

**بخشِ دو.** فرض کن می‌خواهی `Grams::try_from(user_input: f64)` بنویسی که برای مقدارهای منفی یک `Err` بدهد. چرا این کار با `From` قابلِ‌بیان نیست، ولی با `TryFrom` هست؟ یک جمله بنویس — کدش را لازم نیست بنویسی؛ آن [فاز ۲](../../../phase2-intermediate/08-rust-toolbox/04-tryfrom-fallible-conversions/README.fa.md) است.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به کارت می‌آید |
|---|---|---|
| `From<A> for B` | یک راهِ تبدیلِ همیشه‌موفقِ `A` به `B` | یک‌بار بنویس، همه‌جا استفاده شود |
| `Into<B> for A` | نتیجه‌ی مجانیِ نوشتنِ `From<A> for B` | `.into()`، هیچ‌وقت دستی پیاده‌سازی نمی‌شود |
| `.into()` | همان `From::from`، از جهتِ مقصد خوانده‌شده | وقتی مقصد از جایی دیگر معلوم است |
| `?` و `From` | `?` رویِ خطا `From::from` صدا می‌زند، بعد برمی‌گرداند | همان راز |
| `.map_err()` | تبدیلِ دستیِ خطا، در محلِ فراخوانی | وقتی محلِ فراخوانی هم باید در جواب باشد |
| `E0277` (اینجا) | `?` هیچ `From` مناسبی پیدا نکرد | `impl From` گمشده را بنویس |
| `E0282` | `.into()` بینِ چند مقصد گیر کرده | یک نوعِ صریح بگذار |
| `TryFrom` | خواهرِ خطاپذیرِ `From` | [فاز ۲](../../../phase2-intermediate/08-rust-toolbox/04-tryfrom-fallible-conversions/README.fa.md) |

### الان می‌دانی

- `expr?` وقتی `Err(e)` است، `Err(From::from(e))` را برمی‌گرداند — نه خودِ `Err(e)` را.
- نوشتنِ `impl From<A> for B` یعنی `.into()` هم از `A` به `B` مجانی به دست می‌آید؛ کسی `impl Into` دستی نمی‌نویسد.
- `.into()` مقصدش را از جایی که مقدار می‌نشیند می‌گیرد؛ اگر جایی نگفته باشد کدام، `E0282` می‌گیری.
- `From` فقط برای خطا نیست — `String::from` و `u64::from` هم همین صفت‌اند.
- تبدیلِ باریک‌کننده مثلِ `i32::from(x: i64)` وجود ندارد چون ممکن است داده گم شود؛ `as` یا `TryFrom` جای آن است.
- یک enum خطا، یک enum‌ساز به‌ازای هر جور شکست دارد، و یک `impl From` به‌ازای هر خطای بیرونی که با `?` واردش می‌شود — نه لزوماً برایِ همه‌شان.

### بعداً کامل‌تر می‌بینی

- **`TryFrom` و `TryInto`، خواهرِ خطاپذیرِ `From`** — [فاز ۲ — `TryFrom` و تبدیل‌های خطاپذیر](../../../phase2-intermediate/08-rust-toolbox/04-tryfrom-fallible-conversions/README.fa.md)
- **تعریفِ صفتِ خودت، و پیاده‌سازیِ آن برای نوعِ خودت** — [فاز ۲ — تعریف و پیاده‌سازیِ traitها](../../../phase2-intermediate/03-generics-and-traits/02-defining-and-implementing-traits/README.fa.md)
- **`Display` و `std::error::Error`، برای یک خطایِ واقعاً درست و حسابی** — [فاز ۲ — نوع‌های خطای سفارشی](../../../phase2-intermediate/04-error-handling-and-lifetimes/01-custom-error-types/README.fa.md)
- **`thiserror` و `anyhow`، برای وقتی این کدهای تکراری را نمی‌خواهی خودت بنویسی** — [فاز ۲ — `thiserror` و `anyhow`](../../../phase2-intermediate/04-error-handling-and-lifetimes/02-thiserror-and-anyhow/README.fa.md)

### می‌توانی توضیح بدهی؟

- وقتی `expr?` با یک `Err(e)` روبه‌رو می‌شود، دقیقاً چه چیزی را برمی‌گرداند؟
- چرا نوشتنِ `impl From<A> for B` را بلد باشی کافی است تا `.into()` هم کار کند؟
- کامپایلر مقصدِ `.into()` را از کجاها می‌تواند بفهمد؟ کِی نمی‌تواند؟
- چرا `i32::from(x: i64)` وجود ندارد؟ به‌جایش چه داری؟
- چرا هر enum‌سازِ یک enum خطا لازم نیست `impl From` داشته باشد؟
- اگر دو فراخوانی همان نوعِ خطای منبع را بدهند، `?` و `From` چه محدودیتی دارند، و به‌جایش چه می‌نویسی؟

---

## بیشتر

- [کتابِ Rust — یک میان‌بر برایِ انتشارِ خطاها](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#a-shortcut-for-propagating-errors-the--operator) — همین موضوع، از زبانِ رسمی.
- [`std::convert::From`](https://doc.rust-lang.org/std/convert/trait.From.html) — مستنداتِ کاملِ صفت، با فهرستِ تبدیل‌هایی که خودِ کتابخانه‌ی استاندارد پیاده‌سازی کرده.
- [`std::convert::Into`](https://doc.rust-lang.org/std/convert/trait.Into.html) — همان‌جا که پیاده‌سازیِ فراگیر تعریف شده.
- [`clippy::from_over_into`](https://rust-lang.github.io/rust-clippy/master/#from_over_into) — لینتی که اگر جای `From` مستقیم `impl Into` بنویسی، تذکر می‌دهد؛ حالا می‌دانی چرا.
