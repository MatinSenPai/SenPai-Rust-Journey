# ۱.۶.۳ — `Result` و علامتِ سؤال

## در یک نگاه

بعد از این درس می‌توانی:

- بگویی `Result<T, E>` دقیقاً چه چیزی است، و یک نمونه‌اش را با دست، با `Ok` و `Err`، بسازی.
- هشدارِ واقعیِ `#[must_use]` را بخوانی و بگویی چرا نادیده گرفتنِ یک `Result` صدا دارد.
- از بینِ `.map`، `.map_err`، `.and_then`، `.unwrap_or` و `.ok()` برای یک موقعیتِ مشخص، درست‌ترین را انتخاب کنی.
- یک `match` روی `Result` را با `?` جایگزین کنی، بگویی `?` دقیقاً به چه چیزی باز می‌شود، و `main` را طوری بنویسی که خودش `Result` برگرداند.

**زمان:** حدود ۵۵ دقیقه · **پیش‌نیاز:** [۱.۶.۲ — ترکیب‌گرهای `Option`](../02-option-combinators/README.fa.md)

---

## چرا اهمیت دارد

در [۱.۱.۴](../../01-foundations/04-functions-and-expressions/README.fa.md) یک قول داده شد: `return` زودهنگام یک نسخه‌ی کوتاه‌شده دارد، عملگری که همان کار را پنهانی انجام می‌دهد. این همان درسی است که آن قول را ادا می‌کند.

یک بدهیِ قدیمی‌تر هم هست. از فاز ۰، هر بار که `.parse()` زدی — هر بار یک رشته را به عدد تبدیل کردی — نتیجه‌اش را دیدی، ولی نوعِ واقعی‌اش هیچ‌وقت باز نشد. این درس آن را هم باز می‌کند.

هر دو بدهی از یک جا می‌آیند: `Result<T, E>`.

درسِ قبل، `Option` را بست: یک enum برای «شاید اینجا چیزی نیست». ولی خیلی از شکست‌ها بیشتر از یک «نیست» می‌خواهند بگویند. وقتی `.parse()` روی `"abc"` شکست می‌خورد، خبرِ خوب این نیست که «هیچ عددی نبود» — خبر این است که *چرا*: یک رقمِ نامعتبر پیدا شد. یک `None` آن دلیل را دور می‌ریزد. Rust به‌جایش یک enumِ دوم دارد که آن دلیل را نگه می‌دارد.

اگر از پایتون آمده باشی، معادلِ ذهنی‌ات `try`/`except` است: تابع یا مقدار برمی‌گرداند یا استثنا پرتاب می‌کند، و امضای تابع هیچ‌کدام را نشان نمی‌دهد — باید مستندات را بخوانی یا کد را بخوانی تا بفهمی چه چیزی ممکن است شکست بخورد. `Result<T, E>` همان امکانِ شکست را داخلِ نوعِ بازگشتی می‌گنجاند: وقتی امضا `Result` دارد، از همان لحظه‌ای که تابع را می‌بینی می‌دانی ممکن است شکست بخورد، و کامپایلر مجبورت می‌کند قبل از رسیدن به مقدار، آن حالت را ببینی. شباهت همین‌جا تمام می‌شود — Rust استثنا به معنایِ پایتونی‌اش ندارد؛ نزدیک‌ترین چیز `panic!` است، و اینکه کِی به‌جای `Result` سراغش بروی موضوعِ [۱.۶.۴](../04-panic-vs-result/README.fa.md) است.

---

## مفهوم

### `Option` می‌گفت «شاید هیچی»؛ `Result` می‌گوید «شاید، و این هم دلیلش»

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

همان شکلِ `Option<T>` است — یک enumِ معمولی، دقیقاً هم‌جنسِ چیزی که در [۱.۵.۳](../../05-your-own-types/03-enums-as-data/README.fa.md) دیدی — با یک تفاوت: به‌جای یک بازوی خالی (`None`)، بازوی دومش (`Err`) هم یک مقدار دارد. `T` نوعِ چیزی است که وقتی کار جواب می‌دهد به دست می‌آید؛ `E` نوعِ دلیلی است که وقتی کار جواب نمی‌دهد به دست می‌آید.

| | وقتی کار جواب نمی‌دهد | نوعِ خطا |
|---|---|---|
| `Option<T>` | `None` — هیچ اطلاعاتی نیست | ندارد |
| `Result<T, E>` | `Err(E)` — یک مقدار با دلیل | خودت انتخابش می‌کنی |

### ساختنش با دست: `Ok` و `Err`

```rust
fn safe_divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("cannot divide by zero".to_string())
    } else {
        Ok(a / b)
    }
}

let good = safe_divide(10.0, 2.0);
let bad = safe_divide(10.0, 0.0);
println!("good: {good:?}");
println!("bad:  {bad:?}");
```

```text
good: Ok(5.0)
bad:  Err("cannot divide by zero")
```

اینجا `E` نوعِ `String` است — یک انتخابِ ساده برای شروع. تابع دو راهِ خروج دارد، و هر دو صریح‌اند: یا یک `f64` قابلِ استفاده، یا یک جمله که می‌گوید چرا نه.

### تطبیق با `match`، و دو میان‌بُرِ کوچک

```rust
match safe_divide(20.0, 4.0) {
    Ok(value) => println!("matched ok:  {value}"),
    Err(reason) => println!("matched err: {reason}"),
}

println!("is_ok:  {}", good.is_ok());
println!("is_err: {}", bad.is_err());
```

```text
matched ok:  5
is_ok:  true
is_err: true
```

`match` روی `Result` هم دقیقاً به همان قاعده‌ی [۱.۵.۴](../../05-your-own-types/04-match-in-depth/README.fa.md) پایبند است: دو گونه، دو بازو، وگرنه کامپایل نمی‌شود. `.is_ok()` و `.is_err()` برای وقتی‌اند که فقط شکلش را می‌خواهی، نه خودِ مقدار یا خودِ دلیل را.

### چرا نادیده گرفتنِ یک `Result` صدا دارد: `#[must_use]`

```rust
// نه `let`، نه `?`، هیچ‌چیز — Result دور ریخته می‌شود
safe_divide(1.0, 0.0);
```

```text
warning: unused `Result` that must be used
  --> phase1-fundamentals\06-absence-and-failure\03-result-and-question-mark\examples\01-ok-err-and-must-use.rs:36:5
   |
36 |     safe_divide(1.0, 0.0);
   |     ^^^^^^^^^^^^^^^^^^^^^
   |
   = note: this `Result` may be an `Err` variant, which should be handled
   = note: `#[warn(unused_must_use)]` (part of `#[warn(unused)]`) on by default
help: use `let _ = ...` to ignore the resulting value
   |
36 |     let _ = safe_divide(1.0, 0.0);
   |     +++++++
```

نوعِ `Result` در کتابخانه‌ی استاندارد با یک ویژگی به اسم **استفاده‌ی اجباری (`#[must_use]`)** نشانه‌گذاری شده است. یعنی: «اگر این مقدار را جایی نگذاری، حداقل کامپایلر یک بار به تو خبر بدهد — شاید همین الان یک `Err` را بی‌سروصدا گم کردی.» این یک **هشدار** است، نه یک خطا: کد بالا کامپایل می‌شود و اجرا هم می‌شود، فقط رستِ حرفش را می‌زند. راهِ خاموش کردنِ عمدی‌اش دقیقاً همان چیزی است که خودِ کمکِ کامپایلر پیشنهاد داد: `let _ = ...`، که می‌گوید «دیدمش، و عمداً دورش می‌ریزم.»

### `.parse()` بالاخره توضیح داده می‌شود

```rust
let good: Result<u32, _> = "42".parse::<u32>();
let letters: Result<u32, _> = "abc".parse::<u32>();
let negative: Result<u32, _> = "-5".parse::<u32>();
println!("good:     {good:?}");
println!("letters:  {letters:?}");
println!("negative: {negative:?}");
```

```text
good:     Ok(42)
letters:  Err(ParseIntError { kind: InvalidDigit })
negative: Err(ParseIntError { kind: InvalidDigit })
```

حالا نوعِ واقعی‌اش قابلِ گفتن است: `.parse::<u32>()` یک `Result<u32, ParseIntError>` برمی‌گرداند، نه یک `Option<u32>`. آن `::<u32>` که از فاز ۰ می‌نویسی‌اش — **توربوفیش (turbofish)** — به `.parse()` می‌گوید کدام `T` را هدف بگیرد، چون خودِ متد عمومی است و هیچ‌جای دیگری این را نمی‌گوید. `u32` منفی قبول نمی‌کند، برای همین `"-5"` دقیقاً همان‌طور شکست می‌خورد که `"abc"` می‌خورد — همان گونه‌ی خطا، همان پیام.

```rust
if let Err(e) = "abc".parse::<u32>() {
    println!("message: {e}");
}
```

```text
message: invalid digit found in string
```

`ParseIntError` صفتِ `Display` را دارد، پس `{e}` (نه `{e:?}`) همان جمله‌ای را می‌دهد که `.to_string()` هم می‌داد. این چیزی است که چند خط پایین‌تر، در ترکیب‌گرها، دوباره می‌بینی.

### ترکیب‌گرهای `Result`: پنج راه برای دورزدنِ `match`

همان ترکیب‌گرهایی که در [۱.۶.۲](../02-option-combinators/README.fa.md) روی `Option` دیدی، روی `Result` هم هستند — با یک تفاوت: بعضی‌شان به `Err` هم دسترسی دارند.

```rust
println!("{:?}", safe_divide(10.0, 2.0).map(|v| v * 100.0));
println!("{:?}", safe_divide(10.0, 0.0).map(|v| v * 100.0));

let renamed = safe_divide(10.0, 0.0).map_err(|e| format!("division failed: {e}"));
println!("{renamed:?}");
```

```text
Ok(500.0)
Err("cannot divide by zero")
Err("division failed: cannot divide by zero")
```

`.map()` رویِ `Ok` کار می‌کند و `Err` را دست‌نخورده رد می‌کند — دقیقاً همان شکلی که `Option::map` داشت. `.map_err()` تصویرِ آینه‌ای‌اش است: رویِ `Err` کار می‌کند و `Ok` را دست‌نخورده رد می‌کند. وقتی می‌خواهی نوعِ خطا را عوض کنی (مثلاً از `ParseIntError` به `String`، همان‌طور که چند خط بالاتر دیدی)، این همان ابزار است.

```rust
println!(
    "{:?}",
    safe_divide(10.0, 2.0).and_then(|v| safe_divide(v, 5.0))
);
println!(
    "{:?}",
    safe_divide(10.0, 0.0).and_then(|v| safe_divide(v, 5.0))
);

println!("{}", safe_divide(10.0, 2.0).unwrap_or(0.0));
println!("{}", safe_divide(10.0, 0.0).unwrap_or(0.0));

println!("{:?}", safe_divide(10.0, 2.0).ok());
println!("{:?}", safe_divide(10.0, 0.0).ok());
```

```text
Ok(1.0)
Err("cannot divide by zero")
5
0
Some(5.0)
None
```

`.and_then()` یک گامِ شکست‌پذیرِ دوم را زنجیر می‌کند؛ اگر گامِ اول `Err` بود، بستارِ گامِ دوم اصلاً صدا زده نمی‌شود. `.unwrap_or()` یک مقدارِ ساده بیرون می‌کشد، با یک پیش‌فرض برای `Err` — بدونِ هیچ panicی. و `.ok()` دلیلِ خطا را عمداً دور می‌ریزد و یک `Option` تحویل می‌دهد؛ سراغش برو وقتی واقعاً دیگر برایت مهم نیست چرا شکست خورد.

### عملگرِ `?` — یک بازگشتِ زودهنگامِ پنهان

این همان جایی است که [۱.۱.۴](../../01-foundations/04-functions-and-expressions/README.fa.md) به آن اشاره کرد. اول با دست:

```rust
fn chained_by_hand(a: f64, b: f64, c: f64) -> Result<f64, String> {
    let step1 = match safe_divide(a, b) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };
    let step2 = match safe_divide(step1, c) {
        Ok(value) => value,
        Err(e) => return Err(e),
    };
    Ok(step2)
}
```

هر قدم یک `match` می‌خواهد: اگر `Ok`، مقدار را بیرون بکش و ادامه بده؛ اگر `Err`، همین الان از تابع با همان `Err` برگرد. حالا همان تابع، کلمه‌به‌کلمه همان منطق، با `?`:

```rust
fn chained_with_question_mark(a: f64, b: f64, c: f64) -> Result<f64, String> {
    let step1 = safe_divide(a, b)?;
    let step2 = safe_divide(step1, c)?;
    Ok(step2)
}
```

`expr?` دقیقاً همان `match` بالا است، فشرده در یک کاراکتر: **اگر `expr` مقدارش `Ok(v)` بود، کلِ عبارت را با `v` عوض کن و ادامه بده؛ اگر `Err(e)` بود، از همین تابع، همین الان، با `Err(e)` برگرد.** چیزِ جدیدی اتفاق نمی‌افتد — فقط دیگر لازم نیست هر قدم را با دست بنویسی.

```senpai-visual
{"kind":"result","labels":["expr?","Ok(v)","ادامه","Err(e) → بازگشت"]}
```

```rust
println!("by hand,  ok: {:?}", chained_by_hand(100.0, 2.0, 5.0));
println!("by hand,  err: {:?}", chained_by_hand(100.0, 0.0, 5.0));
println!(
    "with `?`, ok: {:?}",
    chained_with_question_mark(100.0, 2.0, 5.0)
);
println!(
    "with `?`, err: {:?}",
    chained_with_question_mark(100.0, 2.0, 0.0)
);
```

```text
by hand,  ok: Ok(10.0)
by hand,  err: Err("cannot divide by zero")
with `?`, ok: Ok(10.0)
with `?`, err: Err("cannot divide by zero")
```

دو تابع، ورودی‌های یکسان، خروجیِ یکسان. تنها فرقشان طولِ کد است.

دو نکته پیش از رفتن: `?` دقیقاً همین کار را روی `Option<T>` هم می‌کند (`None` به‌جایِ `Err` زودهنگام برمی‌گرداند) — و وقتی نوعِ خطای دو طرفِ `?` یکی نباشد، `?` خودش سعی می‌کند با صفتِ `From` تبدیلش کند؛ اینجا همه‌جا از یک نوعِ خطا (`String`) استفاده کردیم تا آن تبدیل لازم نشود. خودِ `From` و این تبدیلِ خودکار، موضوعِ [۱.۶.۵](../05-from-and-error-conversion/README.fa.md) است.

### `main` هم می‌تواند `Result` برگرداند

```rust
fn main() -> Result<(), String> {
    let result = chained_with_question_mark(100.0, 0.0, 5.0)?;
    println!("unreachable: {result}");
    Ok(())
}
```

```text
Error: "cannot divide by zero"
```

`?` فقط داخلِ توابعی کار می‌کند که خودشان `Result` (یا `Option`) برمی‌گردانند — و این شاملِ خودِ `main` هم می‌شود. اینجا `main` امضایش را به `Result<(), String>` عوض کرده، پس `?` داخلش مجاز است. چون `chained_with_question_mark` این بار `Err` می‌دهد، `?` بلافاصله از `main` هم با همان `Err` برمی‌گردد — خطِ `println!("unreachable: ...")` هرگز اجرا نمی‌شود.

وقتی `main` با `Err` برمی‌گردد، Rust آن را با `{:?}` چاپ می‌کند (برای همین نقل‌قول‌ها دورِ پیام هستند) و فرآیند با کدِ خروجِ ۱ تمام می‌شود — نه صفر. این دقیقاً همان چیزی است که یک برنامه‌ی خط‌فرمان برای گزارشِ شکست به پوسته‌ای که آن را صدا زده لازم دارد. اگر `chained_with_question_mark` موفق شده بود، `main` بی‌سروصدا با `Ok(())` تمام می‌شد و کدِ خروج صفر بود.

---

## دست‌به‌کد

```sh
cargo run -p p1-06-03-result-and-question-mark --example 01-ok-err-and-must-use
cargo run -p p1-06-03-result-and-question-mark --example 02-parse
cargo run -p p1-06-03-result-and-question-mark --example 03-combinators
cargo run -p p1-06-03-result-and-question-mark --example 04-question-mark
```

آخری با کدِ خروجِ ۱ تمام می‌شود — عمدی است؛ همان نکته‌ای است که بالا خواندی.

بعد دو تای خراب:

```sh
cargo run -p p1-06-03-result-and-question-mark --example 05-unwrap-panics --features broken
cargo run -p p1-06-03-result-and-question-mark --example 06-question-mark-needs-result --features broken
```

بعد این‌ها را امتحان کن:

۱. در `01-ok-err-and-must-use`، خطِ آخر را به `let _ = safe_divide(1.0, 0.0);` عوض کن. هشدار می‌رود؟ چرا؟
۲. در `03-combinators`، جای `.map_err()` و `.map()` را عوض کن (یعنی رویِ `Ok` هم `.map_err()` بزن). چه چیزی چاپ می‌شود؟
۳. در `04-question-mark`، آرگومان‌های `chained_with_question_mark` را در آخرین فراخوانی طوری عوض کن که هر دو تقسیم موفق شوند. حالا خروجی و کدِ خروج چه می‌شوند؟

---

## خطاهایی که خواهی دید

### پنیک — `called `Result::unwrap()` on an `Err` value`

```text
thread 'main' (17828) panicked at phase1-fundamentals\06-absence-and-failure\03-result-and-question-mark\examples\05-unwrap-panics.rs:19:40:
called `Result::unwrap()` on an `Err` value: "cannot divide by zero"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**کامپایلر (در واقع، زمانِ اجرا) به چه اعتراض دارد:** `.unwrap()` روی `Ok` مقدار را می‌دهد؛ روی `Err` پنیک می‌کند. پیام همان چیزی است که تویِ `Err(...)` گذاشته بودی — همان `"cannot divide by zero"` که در `safe_divide` نوشتی، حالا در بدنه‌ی خودِ panic.

**راه‌حل:** به‌جای `.unwrap()`، از `match`، `?`، یا یکی از ترکیب‌گرهای بالا استفاده کن — هر کدام که مسیرِ `Err` را واقعاً هندل کند به‌جای فرض کردنِ اینکه هیچ‌وقت اتفاق نمی‌افتد.

**چرا این راه‌حل است:** `.unwrap()` می‌گوید «مطمئنم `Ok` است؛ اگر نبود، همه‌چیز متوقف شود.» این قضاوت گاهی درست است (در تستِ خودت، در نمونه‌کد) و گاهی نه (رویِ ورودیِ کاربر). تشخیصِ این دو از هم موضوعِ [۱.۶.۴](../04-panic-vs-result/README.fa.md) است.

### `E0277` — `?` بیرون از یک تابعی که `Result` برمی‌گرداند

```text
error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option` (or another type that implements `FromResidual`)
  --> phase1-fundamentals\06-absence-and-failure\03-result-and-question-mark\examples\06-question-mark-needs-result.rs:16:39
   |
13 | fn main() {
   | --------- this function should return `Result` or `Option` to accept `?`
...
16 |     let value = safe_divide(10.0, 0.0)?;
   |                                       ^ cannot use the `?` operator in a function that returns `()`
   |
help: consider adding return type
   |
13 ~ fn main() -> Result<(), Box<dyn std::error::Error>> {
14 |     // `main` here returns `()`, not a `Result` — so there is nowhere for
...
17 |     println!("{value}");
18 +     Ok(())
   |

For more information about this error, try `rustc --explain E0277`.
```

**کامپایلر به چه اعتراض دارد:** `?` به یک جایی برای گذاشتنِ `Err` نیاز دارد — یک نوعِ بازگشتیِ `Result` یا `Option` در همان تابع. اینجا `main` نوعِ بازگشتی‌اش `()` است، پس هیچ جایی برای آن `Err` نیست، و کامپایلر همین را دقیقاً می‌گوید: «this function should return `Result` or `Option` to accept `?`».

**راه‌حل:** امضای `main` را به `Result<(), E>` عوض کن، همان‌طور که در بخشِ «مفهوم» دیدی.

**چرا این راه‌حل است:** خودِ کامپایلر یک راهِ دیگر هم پیشنهاد می‌دهد: `Result<(), Box<dyn std::error::Error>>`. **`Box<dyn Error>`** یک نوعِ خطای همه‌کاره است — هر نوعی که صفتِ استانداردِ `Error` را پیاده کرده باشد داخلش جا می‌شود، پس دیگر مجبور نیستی همه‌ی خطاهای برنامه‌ات از یک جنس باشند. برای یک برنامه‌ی کوچک یا یک `main`، این یک انتخابِ سریع‌وکثیفِ رایج است. اینجا فقط اسمش را دانستی؛ خودِ صفتِ `Error` و اینکه `Box<dyn Error>` دقیقاً چطور کار می‌کند، در [نوع‌های خطای سفارشی، فاز ۲](../../../phase2-intermediate/04-error-handling-and-lifetimes/01-custom-error-types/README.fa.md) می‌آید.

---

## تمرین

### گرم‌کردن

<details>
<summary><code>Result<f64, String></code> یعنی چه، در یک جمله؟</summary>

یا یک `f64` قابلِ استفاده (`Ok`)، یا یک `String` که می‌گوید چرا نه (`Err`) — دقیقاً یکی از این دو، هیچ‌وقت هیچ‌کدام و هیچ‌وقت هر دو.

</details>

<details>
<summary>این کامپایل می‌شود؟

```rust
fn area(width: u32) -> Result<u32, String> {
    width * width
}
```
</summary>

نه. عبارتِ پایانی از نوعِ `u32` است، ولی امضا `Result<u32, String>` قول داده. همان خطایی که در [۱.۱.۴](../../01-foundations/04-functions-and-expressions/README.fa.md) دیدی («expected `Result<u32, String>`, found `u32`»)، این‌بار با یک enum به‌جایِ یک عددِ ساده. باید `Ok(width * width)` بنویسی.

</details>

<details>
<summary><code>"7".parse::<u32>()</code> با <code>{:?}</code> چاپ چه می‌شود؟</summary>

`Ok(7)`.

</details>

<details>
<summary>

```rust
fn h(n: i32) -> Result<i32, String> {
    if n < 0 {
        return Err("negative".to_string());
    }
    Ok(n * 2)
}
println!("{:?}", h(-3));
```

چه چاپ می‌شود؟
</summary>

`Err("negative")`. `n < 0` است، پس همان اول با `return` برمی‌گردد؛ خطِ `Ok(n * 2)` هرگز اجرا نمی‌شود.

</details>

<details>
<summary><code>let _ = safe_divide(1.0, 0.0);</code> هم همان هشدارِ <code>unused_must_use</code> را می‌دهد؟</summary>

نه. `let _ = ...` دقیقاً همان چیزی است که خودِ کامپایلر برای خاموش کردنِ عمدیِ هشدار پیشنهاد می‌دهد — می‌گوید «دیدمش، عمداً دورش می‌ریزم»، نه «فراموشش کردم».

</details>

### تعمیر

`examples/06-question-mark-needs-result.rs` را **دو** جور درست کن:

۱. با تغییرِ امضایِ `main` به `Result<(), String>` — همان راهی که در بخشِ «مفهوم» دیدی.
۲. بدونِ دست زدن به امضایِ `main` — با هندل کردنِ `Result` به یک روشِ دیگر (مثلاً `match` یا یکی از ترکیب‌گرها) طوری که دیگر نیازی به `?` نباشد.

بعد `examples/05-unwrap-panics.rs` را طوری درست کن که دیگر پنیک نکند، بدونِ اینکه رفتارِ `safe_divide` را عوض کنی — به‌جای کرش، یک پیامِ دوستانه چاپ کند.

### پیاده‌سازی

پنج تابع در `src/lib.rs`:

```sh
cargo test -p p1-06-03-result-and-question-mark
```

یکی‌شان `Ok`/`Err` را با دست می‌سازد. یکی روی `.parse()` تکیه می‌کند. یکی از `?` برای عبور دادنِ شکست از میانِ دو قدم استفاده می‌کند. دو تا آخر، یک `Result` را عمداً به یک مقدارِ ساده یا یک `Option` تبدیل می‌کنند — دقیقاً همان جایی که دلیلِ خطا دیگر مهم نیست.

### بساز

یک `pub fn parse_all(inputs: &[&str]) -> Result<Vec<i32>, String>` بنویس: هر رشته‌ی داخلِ `inputs` را به‌ترتیب به‌عنوانِ `i32` پارس کن. اگر همه موفق شدند، `Ok` را با همه‌ی اعداد، به همان ترتیب، برگردان. اگر یکی شکست خورد، همان لحظه با `Err` برگرد، حاوی متنِ همان خطای پارس (`.to_string()`)، بدونِ امتحان کردنِ باقیِ رشته‌ها.

بعد یک جمله بنویس درباره‌ی اینکه چرا `?` داخلِ یک حلقه، دقیقاً همین رفتار را رایگان می‌دهد.

### چالش (اختیاری)

**بخشِ یک.** `?` رویِ `Option<T>` هم کار می‌کند، نه فقط رویِ `Result`. این را بنویس، اجرا کن، و ببین حدست درست بود یا نه:

```rust
fn first_char(s: &str) -> Option<char> {
    let c = s.chars().next()?;
    Some(c.to_ascii_uppercase())
}
```

با یک رشته‌ی خالی و یک رشته‌ی معمولی امتحانش کن.

**بخشِ دو.** تابعِ `chained_with_question_mark` را طوری عوض کن که به‌جایِ `String`، نوعِ خطایش `&'static str` باشد. کامپایل می‌شود؟ اگر `safe_divide` را هم به همان نوع عوض نکنی، چه خطایی می‌گیری؟ (این دقیقاً همان جایی است که [۱.۶.۵](../05-from-and-error-conversion/README.fa.md) لازم می‌شود.)

**بخشِ سه.** امضایِ `main` را در `04-question-mark.rs` به `Result<(), Box<dyn std::error::Error>>` عوض کن — همان چیزی که کامپایلر در بخشِ خطاها پیشنهاد داد. کامپایل می‌شود؟ چه چیزی باید اضافه کنی تا کامپایل شود؟

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به کارت می‌آید |
|---|---|---|
| `Result<T, E>` | یا `Ok(T)`، یا `Err(E)` با دلیل | هر جا شکست باید توضیح داشته باشد |
| `#[must_use]` | نادیده گرفتنش هشدار می‌دهد، نه خطا | `let _ = ...` برای خاموش کردنِ عمدی |
| `.parse::<T>()` | `Result<T, ParseIntError>` (یا مشابهش) | تبدیلِ رشته به عدد، از فاز ۰ |
| `.map` / `.map_err` | تبدیلِ `Ok` / تبدیلِ `Err` | تغییرِ شکل بدونِ `match` |
| `.and_then` | زنجیرِ یک قدمِ شکست‌پذیرِ دوم | وقتی گامِ دوم به مقدارِ اول نیاز دارد |
| `.unwrap_or` / `.ok()` | مقدارِ ساده با پیش‌فرض / تبدیل به `Option` | وقتی دیگر دلیلِ خطا مهم نیست |
| `?` | همان `match` بالا، فشرده در یک کاراکتر | فقط داخلِ تابعی که `Result`/`Option` برمی‌گرداند |
| `main() -> Result<...>` | کدِ خروجِ فرآیند را به `Err` گره می‌زند | برنامه‌های خط‌فرمان |

### الان می‌دانی

- `Result<T, E>` می‌گوید «یا موفق، با این مقدار؛ یا شکست، با این دلیل» — و دلیل چیزی است که `Option` نداشت.
- `Result` با `#[must_use]` نشانه‌گذاری شده؛ نادیده گرفتنش هشدار می‌دهد، نه خطا.
- `.parse::<T>()` یک `Result<T, E>` می‌دهد؛ توربوفیش می‌گوید کدام `T`.
- `.map`، `.map_err`، `.and_then`، `.unwrap_or` و `.ok()` پنج راهِ متفاوت برای دورزدنِ `match` روی یک `Result`اند.
- `expr?` یعنی: روی `Ok(v)` با `v` ادامه بده؛ روی `Err(e)` همین الان از تابع با `Err(e)` برگرد.
- `?` فقط داخلِ تابعی کار می‌کند که خودش `Result` یا `Option` برمی‌گرداند — و این شاملِ `main` هم می‌شود.

### بعداً کامل‌تر می‌بینی

- **کِی panic و کِی `Result`** — [۱.۶.۴ — پنیک در برابرِ `Result`](../04-panic-vs-result/README.fa.md)
- **تبدیلِ خودکارِ نوعِ خطا با `From`، همان کاری که `?` پشتِ‌پرده انجام می‌دهد** — [۱.۶.۵ — `From` و تبدیلِ خطا](../05-from-and-error-conversion/README.fa.md)
- **نوع‌های خطای سفارشی که صفتِ `Error` را پیاده می‌کنند، و `Box<dyn Error>`** — [فاز ۲ — نوع‌های خطای سفارشی](../../../phase2-intermediate/04-error-handling-and-lifetimes/01-custom-error-types/README.fa.md)
- **`thiserror` و `anyhow`، برای وقتی نوشتنِ دستیِ خطاها زیاد می‌شود** — [فاز ۲ — `thiserror` و `anyhow`](../../../phase2-intermediate/04-error-handling-and-lifetimes/02-thiserror-and-anyhow/README.fa.md)
- **همین شکلِ `.map`، این‌بار روی ایتریتورها** — [فاز ۲ — کلوژرها و صفت‌های `Fn`](../../../phase2-intermediate/02-iterators-and-closures/01-closures-and-fn-traits/README.fa.md)

### می‌توانی توضیح بدهی؟

- تفاوتِ `Option<T>` و `Result<T, E>` در یک جمله چیست؟
- `#[must_use]` چه چیزی را تضمین می‌کند، و چرا یک هشدار است نه یک خطا؟
- `.parse::<u32>()` روی یک عددِ منفی چه برمی‌گرداند، و چرا؟
- `expr?` دقیقاً معادلِ چه `match`ی است؟
- چرا `?` داخلِ `fn main() { ... }` (بدونِ نوعِ بازگشتی) کامپایل نمی‌شود؟
- وقتی `main` با `Err` برمی‌گردد، کدِ خروجِ فرآیند چند است؟

---

## بیشتر

- [کتابِ Rust — `Result` و بازگشتِ زودهنگام از خطاها](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html) — همین زمین، رسمی، با بخشِ مفصلی درباره‌ی `?`.
- [`std::result::Result`](https://doc.rust-lang.org/std/result/enum.Result.html) — فهرستِ کاملِ ترکیب‌گرها؛ خیلی بیشتر از پنج‌تایی که اینجا دیدی.
- [`std::ops::Try` و `?`](https://doc.rust-lang.org/std/ops/trait.Try.html) — برای وقتی خواستی بدانی `?` دقیقاً چه چیزِ عمومی‌تری را پیاده می‌کند.
