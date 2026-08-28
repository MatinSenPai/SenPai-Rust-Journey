# ۲.۳.۵ — نوع‌های وابسته در برابرِ پارامترهای جنریک

## در یک نگاه

بعد از این درس می‌توانی:

- توضیح بدهی چرا `type Item;` روی `Iterator` خودِ صفت را جنریک نمی‌کند، ولی `trait Converts<T>` صفت را جنریک می‌کند — و این تفاوت دقیقاً رویِ چه چیزی اثر می‌گذارد.
- بگویی یک نوع چند بار می‌تواند یک صفتِ نوع‌وابسته را پیاده کند، در برابرِ یک صفتِ با پارامترِ جنریک — و این ادعا را با کامپایلرِ واقعی، نه فقط با حرف، ثابت کنی.
- برایِ یک طراحیِ واقعیِ صفت، بینِ نوعِ وابسته و پارامترِ جنریک یکی را انتخاب کنی، و بگویی دقیقاً چرا گزینه‌ی دیگر خراب می‌شد.

**زمان:** حدود ۶۰ دقیقه · **پیش‌نیاز:**
[۲.۳.۱ — تعریف و پیاده‌سازیِ صفت‌ها](../01-defining-and-implementing-traits/README.fa.md)،
[۲.۳.۲ — توابع و ساختارهایِ جنریک](../02-generic-functions-and-structs/README.fa.md)،
[۲.۲.۴ — پیاده‌سازیِ `Iterator` و `IntoIterator` برایِ نوعِ خودت](../../02-iterators-and-closures/04-implementing-iterator/README.fa.md)

---

## چرا اهمیت دارد

صفتِ `Iterator` یک تصمیم را برایت گرفته — بدونِ اینکه حتی یک‌بار ازت بپرسد. در [۲.۲.۴](../../02-iterators-and-closures/04-implementing-iterator/README.fa.md) از کنارِ آن رد شدی: «نوعِ وابسته باعث نمی‌شود خودِ `Iterator` جنریک شود؛ هر پیاده‌ساز فقط می‌گوید 'مالِ من `Item`ش این است'، و همین کافی است.» همان تصمیم دقیقاً همان چیزی است که `iter.next()` را همیشه، همه‌جا، بدونِ هیچ نوع‌نویسیِ صریحی قابلِ صدا زدن نگه می‌دارد.

ولی وقتی خودت صفت طراحی می‌کنی — [۲.۳.۱](../01-defining-and-implementing-traits/README.fa.md) از همان‌جا شروعش کرد — این تصمیم دیگر رایگان نیست. باید انتخاب کنی: `type Target;` بنویسی یا `trait Foo<T>`. کامپایلر این انتخاب را برایت تصحیح نمی‌کند اگر اشتباه بزنی؛ فقط یک طراحیِ دیگر می‌سازد، با محدودیت‌ها و هزینه‌های خودش. انتخابِ اشتباه اینجا هم، درست مثلِ انتخابِ مجموعه‌ی غلط در [۲.۱.۴](../../01-collections/04-choosing-a-collection/README.fa.md)، کامپایل می‌شود و کار می‌کند — فقط بعداً گیر می‌کنی: یا یک نوع را که واقعاً باید چندجور جواب بدهد، قفل می‌کنی به یک جواب؛ یا هر صدا زدنِ یک متدِ ساده را مجبور می‌کنی هربار نوعش را صریح بنویسد، حتی وقتی از اول فقط یک جواب معنا داشت.

این درس دقیقاً همین تصمیم را باز می‌کند — با کامپایلرِ واقعی، نه فقط با تعریف.

---

## مفهوم

### `Iterator::Item`: یک نوع، برایِ همیشه

صفتِ `Iterator` این شکل را دارد — بدونِ هیچ پارامتری رویِ خودِ `trait Iterator`:

```rust
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
```

`type Item;` یک جای‌خالی است، نه یک مقدار. هر پیاده‌سازی این جای‌خالی را دقیقاً یک‌بار پر می‌کند. یک شمارنده‌یِ رو‌به‌پایین بساز که این را نشان بدهد:

```rust
struct Countdown {
    remaining: u8,
}
impl Iterator for Countdown {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.remaining == 0 {
            return None;
        }
        let value = self.remaining;
        self.remaining -= 1;
        Some(value)
    }
}
```

حالا رویش هرچه از [۲.۲](../../02-iterators-and-closures/README.fa.md) یاد گرفتی امتحان کن — بدونِ اینکه هیچ‌جا بگویی `Item` چیست:

```rust
let full = Countdown { remaining: 5 }.count();
let odd = Countdown { remaining: 5 }.filter(|v| v % 2 == 1).count();
println!("full: {full}");
println!("odd: {odd}");
```

```text
full: 5
odd: 3
```

نه در تعریفِ `Countdown`، نه در صدا زدنِ `.count()` یا `.filter()`، هیچ‌جا لازم نشد بنویسی `Item` چیست. کامپایلر از قبل می‌داند: چون `Countdown` دقیقاً یک‌بار `Iterator` را پیاده کرده، `Item`ش هم دقیقاً یک چیز است — `u8` — همیشه، همه‌جا. این «همیشه، همه‌جا» دقیقاً چیزی است که امروز مکانیزمش را باز می‌کنیم.

(صفت‌هایی مثلِ `DoubleEndedIterator` رویِ همین `Iterator` می‌سازند — یک رابطه‌ی کاملاً متفاوت با چیزی که امروز می‌بینیم، و موضوعِ [۲.۳.۶](../06-supertraits-blanket-impls-orphan-rule/README.fa.md) است.)

### همان ایده، به‌عنوانِ یک صفتِ خودت

`Iterator` این تصمیم را برایت گرفت. حالا خودت دو صفت بنویس که تقریباً یک کار می‌کنند — «این مقدار را به یک نوعِ دیگر تبدیل کن» — یکی با نوعِ وابسته، یکی با پارامترِ جنریک:

```rust
trait ConvertsTo {
    type Target;
    fn convert(&self) -> Self::Target;
}

trait Converts<T> {
    fn convert(&self) -> T;
}
```

شکلِ متد تقریباً یکی است — هردو `fn convert(&self) -> ...` هستند. فرق فقط در همین یک جا است: `Self::Target` (چیزی که خودِ نوع، یک‌بار، مشخص می‌کند) در برابرِ `T` (چیزی که خودِ صفت، رویِ هر `impl`، می‌گیرد). همین یک فرقِ کوچک، دو رفتارِ کاملاً متفاوت می‌سازد.

### نوعِ وابسته: یک راه، برایِ همیشه

`ConvertsTo` را برایِ یک نوعِ واقعی پیاده کن — دمایی به سلسیوس، که فارنهایتش را می‌خواهی:

```rust
struct Celsius(f64);
impl ConvertsTo for Celsius {
    type Target = f64;

    fn convert(&self) -> f64 {
        self.0 * 9.0 / 5.0 + 32.0
    }
}
```

```rust
let boiling = Celsius(100.0);
println!("{}", boiling.convert());
```

```text
212
```

کار می‌کند، و همین‌قدر ساده. حالا فرض کن `Celsius` را می‌خواهی به یک رشته‌ی توصیفی هم تبدیل کنی — یک `impl ConvertsTo` دومی برایِ همان `Celsius`، با `type Target = String;`. کامپایل نمی‌شود — `E0119`؛ «خطاهایی که خواهی دید» عینِ پیامش را دارد.

چرا؟ چون خودِ `trait ConvertsTo` هیچ پارامتری نمی‌گیرد. از دیدِ کامپایلر فقط یک جفتِ ممکن هست: «صفتِ `ConvertsTo`، برایِ نوعِ `Celsius`». یک جفت، یک `impl`، یک `Target`. برایِ همیشه.

### پارامترِ جنریک: چند راه، یکی برایِ هر `T`

حالا همان کار را با `Converts<T>` بکن — و این‌بار واقعاً دوبار پیاده‌اش کن:

```rust
impl Converts<f64> for Celsius {
    fn convert(&self) -> f64 {
        self.0 * 9.0 / 5.0 + 32.0
    }
}
impl Converts<String> for Celsius {
    fn convert(&self) -> String {
        format!("{:.1}C", self.0)
    }
}
```

```rust
let fahrenheit: f64 = boiling.convert();
let label: String = boiling.convert();
println!("{fahrenheit}");
println!("{label}");
```

```text
212
100.0C
```

این‌بار کامپایل شد — همان نامِ متد، همان نوع، دو بار پیاده‌سازی‌شده. چرا این یکی اجازه دارد وقتی بخشِ قبل اجازه نداشت؟

چون `Converts<T>` خودش یک پارامتر می‌گیرد. `Converts<f64>` و `Converts<String>` از دیدِ کامپایلر دو چیزِ کاملاً متفاوت‌اند — دو جفتِ متفاوت: «`Converts<f64>`، برایِ `Celsius`» یکی، «`Converts<String>`، برایِ `Celsius`» یکیِ دیگر. هرکدام جفتِ خودش را دارد، پس هرکدام `impl` خودش را می‌گیرد.

اگر این ادعا درست باشد، یک پیش‌بینیِ دقیق هم می‌دهد: دو `impl Converts<f64> for Celsius` — همان `T`، دوبار — باید دقیقاً به همان مشکلِ بخشِ قبل بخورد، چون آن‌وقت واقعاً یک جفتِ تکراری داری. امتحانش کن — `E0119` می‌گیری، این‌بار با `Converts<f64>` توی پیام، نه `ConvertsTo`. «خطاهایی که خواهی دید» عینِ پیامش را دارد.

> **یادت هست؟** یک صفتِ جنریک را قبلاً هم دیده‌ای، بدونِ اینکه اسمش را بگذاری: [۱.۶.۵](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.fa.md)، `From<T>`. `String` هم `From<&str>` را پیاده کرده، هم `From<char>` را، هم چندتایِ دیگر را — همه‌شان هم‌زمان، رویِ همان یک نوع، دقیقاً به همین دلیل.

### هزینه‌اش کجاست: ابهام در نقطه‌ی صدا زدن

بخشِ اول را دوباره نگاه کن: `Countdown { remaining: 5 }.count()` هیچ‌جا نگفت `Item` چیست، و کامپایل شد. حالا همین را با `Converts<T>` امتحان کن — بدونِ نوع‌نویسی:

```rust
let result = boiling.convert();
```

کامپایل نمی‌شود. `E0283` — کامپایلر می‌گوید هم `Converts<f64>` هست هم `Converts<String>`، و هیچ‌جا نگفتی کدام‌شان را می‌خواهی. «خطاهایی که خواهی دید» عینِ پیام را دارد؛ خودِ کامپایلر هم یک راه‌حل پیشنهاد می‌دهد.

این دقیقاً هزینه‌ی طرفِ «چند راه» است: هرجا بیش از یک `impl` هست، نقطه‌ی صدا زدن باید بگوید کدام‌یک را می‌خواهد. دو راه برایِ گفتنش:

```rust
let by_annotation: f64 = boiling.convert();
let by_qualified = <Celsius as Converts<String>>::convert(&boiling);
println!("{by_annotation}");
println!("{by_qualified}");
```

```text
212
100.0C
```

راهِ اول — نوع‌نویسیِ صریح رویِ بایند — همان چیزی است که بخشِ قبل هم استفاده کرد. راهِ دوم — نحوِ کاملاً واجدشده (fully qualified syntax) — دقیقاً می‌گوید کدام صفت، با کدام `T`، برایِ کدام نوع. هردو یک چیز را به کامپایلر می‌گویند: کدام جفت را می‌خواهی.

### کدام‌یک؟ یک سؤال، یک جدول

همه‌ی این‌ها به یک سؤال برمی‌گردد:

```senpai-visual
{"kind":"concept","labels":["این نوع همیشه یک جواب دارد؟","بله → نوعِ وابسته","نه، چند جواب دارد؟","صداکننده انتخاب می‌کند؟","بله → پارامترِ جنریک"]}
```

| | نوعِ وابسته (`type Target;`) | پارامترِ جنریک (`Converts<T>`) |
|---|---|---|
| چند بار می‌شود یک نوع را پیاده‌سازی کرد؟ | فقط یک‌بار | یک‌بار برایِ هر `T`ِ متفاوت |
| نقطه‌ی صدا زدن نوع‌نویسیِ صریح لازم دارد؟ | هرگز | وقتی بیش از یک `impl` هست |
| نمونه از کتابخانه‌ی استاندارد | `Iterator::Item` | `From<T>` / `Into<T>` |
| کِی انتخابش کنی | این نوع، برایِ همیشه، فقط یک‌جور درست جواب می‌دهد | این نوع واقعاً چند جواب دارد، و صداکننده باید انتخاب کند |

قاعده همین‌قدر کوتاه است: اگر نوعت *همیشه*، *برایِ همیشه*، فقط یک جوابِ درست دارد، نوعِ وابسته بگذار — نقطه‌ی صدا زدن هیچ‌وقت مزاحمِ نوع‌نویسی نمی‌شود. اگر واقعاً بیش از یک جوابِ درست دارد و صداکننده باید انتخاب کند، پارامترِ جنریک بگذار — و بپذیر که نقطه‌ی صدا زدن گاهی باید بگوید کدام‌یک را می‌خواهد.

این جدول جوابش را وقتی می‌دهد که کامپایلر، سرِ کامپایل، دقیقاً می‌داند با کدام نوع طرف است. یک سؤالِ دیگر هم هست، برایِ وقتی این را فقط زمانِ اجرا می‌فهمی — آن یکی موضوعِ درسِ بعدی است.

---

## دست‌به‌کد

```sh
cargo run -p p2-03-05-associated-types --example 01-one-way-associated-type
cargo run -p p2-03-05-associated-types --example 02-many-ways-generic-parameter
cargo run -p p2-03-05-associated-types --example 03-iterator-item-is-fixed
cargo run -p p2-03-05-associated-types --example 04-two-ways-to-disambiguate
```

بعد چهارتای خراب:

```sh
cargo run -p p2-03-05-associated-types --example 05-two-impls-conflict --features broken
cargo run -p p2-03-05-associated-types --example 06-ambiguous-convert-call --features broken
cargo run -p p2-03-05-associated-types --example 07-mismatched-target-type --features broken
cargo run -p p2-03-05-associated-types --example 08-same-target-twice-conflicts --features broken
```

بعد این‌ها را امتحان کن:

۱. در `02-many-ways-generic-parameter.rs`، یک `impl Converts<bool>` سوم برایِ `Celsius` اضافه کن (مثلاً: «آیا بالایِ نقطه‌ی جوش است؟»). چند خط از بقیه‌ی فایل واقعاً باید عوض شود تا کامپایل بماند؟
۲. در `04-two-ways-to-disambiguate.rs`، `by_annotation` را هم با نحوِ کاملاً واجدشده بنویس، به‌جایِ نوع‌نویسیِ رویِ بایند. خروجی عوض می‌شود؟
۳. در `03-iterator-item-is-fixed.rs`، یک `impl Iterator for Countdown` دومی اضافه کن که `Item` را `i32` بگذارد. کامپایلر چه می‌گوید، و کدام‌یک از خطاهایِ بخشِ بعد است؟

---

## خطاهایی که خواهی دید

### `E0119` — تضادِ دو پیاده‌سازیِ `ConvertsTo`

```rust
struct Celsius(f64);
impl ConvertsTo for Celsius {
    type Target = f64;
    fn convert(&self) -> f64 {
        self.0 * 9.0 / 5.0 + 32.0
    }
}
impl ConvertsTo for Celsius {
    type Target = String;
    fn convert(&self) -> String {
        format!("{:.1}C", self.0)
    }
}
```

```text
error[E0119]: conflicting implementations of trait `ConvertsTo` for type `Celsius`
  --> phase2-intermediate\03-traits-and-generics\05-associated-types\examples\05-two-impls-conflict.rs:24:1
   |
16 | impl ConvertsTo for Celsius {
   | --------------------------- first implementation here
...
24 | impl ConvertsTo for Celsius {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^ conflicting implementation for `Celsius`

For more information about this error, try `rustc --explain E0119`.
```

**کامپایلر به چه اعتراض دارد:** `ConvertsTo` هیچ پارامتری رویِ خودِ `trait` ندارد. از دیدِ کامپایلر فقط یک جفتِ ممکن هست — «`ConvertsTo`، برایِ `Celsius`» — و تو داری برایِ همان یک جفت دوبار جواب می‌دهی: یک‌بار `Target = f64`، یک‌بار `Target = String`. کامپایلر نمی‌داند کدام‌شان درست است، پس هیچ‌کدام را قبول نمی‌کند.

**راه‌حل:** یکی از دو `impl` را نگه دار — یا اگر واقعاً هردو تبدیل را لازم داری، به بخشِ «پارامترِ جنریک» برگرد: `Converts<T>` دقیقاً برایِ همین ساخته شده.

**چرا این راه‌حل است:** نوعِ وابسته قولِ «یک جوابِ ثابت» می‌دهد. نگه‌داشتنِ دو `impl` یعنی زیرِ آن قول زدن. یا قول را نگه دار (یکی را حذف کن)، یا از اول قولِ درستی نده (به پارامترِ جنریک برو).

### `E0283` — `.convert()` نمی‌داند کدام `impl` را می‌خواهی

```rust
let boiling = Celsius(100.0);
let result = boiling.convert();
println!("{result}");
```

```text
error[E0283]: type annotations needed
  --> phase2-intermediate\03-traits-and-generics\05-associated-types\examples\06-ambiguous-convert-call.rs:28:9
   |
28 |     let result = boiling.convert();
   |         ^^^^^^           ------- type must be known at this point
   |
note: multiple `impl`s satisfying `Celsius: Converts<_>` found
  --> phase2-intermediate\03-traits-and-generics\05-associated-types\examples\06-ambiguous-convert-call.rs:14:1
   |
14 | impl Converts<f64> for Celsius {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
20 | impl Converts<String> for Celsius {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
help: consider giving `result` an explicit type
   |
28 |     let result: /* Type */ = boiling.convert();
   |               ++++++++++++

For more information about this error, try `rustc --explain E0283`.
```

**کامپایلر به چه اعتراض دارد:** خودِ پیام دقیقاً می‌گوید — «multiple `impl`s satisfying `Celsius: Converts<_>` found». هم `Converts<f64>` هست هم `Converts<String>`، و `result` هیچ نوعی نگفته. کامپایلر نمی‌تواند حدس بزند کدام `convert` را می‌خواهی.

**راه‌حل:** بگو کدام‌یک را می‌خواهی:

```rust
let result: f64 = boiling.convert();
```

**چرا این راه‌حل است:** پیشنهادِ خودِ کامپایلر («consider giving `result` an explicit type») دقیقاً همین است. این هزینه‌ای است که با پارامترِ جنریک می‌خری: هرجا بیش از یک `impl` هست، صداکننده باید انتخاب کند — همان چیزی که «هزینه‌اش کجاست» در بخشِ مفهوم گفت.

### `E0308` — بدنه با `Target`ِ اعلام‌شده جور در نمی‌آید

```rust
impl ConvertsTo for Celsius {
    type Target = f64;
    fn convert(&self) -> Self::Target {
        format!("{:.1}C", self.0)
    }
}
```

```text
error[E0308]: mismatched types
  --> phase2-intermediate\03-traits-and-generics\05-associated-types\examples\07-mismatched-target-type.rs:19:9
   |
18 |     fn convert(&self) -> Self::Target {
   |                          ------------ expected `f64` because of return type
19 |         format!("{:.1}C", self.0)
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^ expected `f64`, found `String`

For more information about this error, try `rustc --explain E0308`.
```

**کامپایلر به چه اعتراض دارد:** `type Target = f64;` یک قول است، نه یک کامنت. کامپایلر همان‌جا که `Self::Target` را می‌بیند، آن را با `f64` جایگزین می‌کند و بدنه را با همان معیار می‌سنجد. بدنه یک `String` برمی‌گرداند — قول نقض شده.

**راه‌حل:** یا بدنه را با `f64` جور کن:

```rust
fn convert(&self) -> Self::Target {
    self.0 * 9.0 / 5.0 + 32.0
}
```

یا اگر واقعاً رشته می‌خواهی، خودِ `Target` را عوض کن: `type Target = String;`.

**چرا این راه‌حل است:** نوعِ وابسته فقط یک‌بار مشخص می‌شود؛ ولی همان یک‌بار همه‌جا اعمال می‌شود — رویِ امضایِ متد هم، نه فقط رویِ اسمِ نوع.

### `E0119` — دوباره، و این‌بار دقیق‌تر

```rust
impl Converts<f64> for Celsius {
    fn convert(&self) -> f64 {
        self.0 * 9.0 / 5.0 + 32.0
    }
}
impl Converts<f64> for Celsius {
    fn convert(&self) -> f64 {
        self.0 + 273.15
    }
}
```

```text
error[E0119]: conflicting implementations of trait `Converts<f64>` for type `Celsius`
  --> phase2-intermediate\03-traits-and-generics\05-associated-types\examples\08-same-target-twice-conflicts.rs:21:1
   |
15 | impl Converts<f64> for Celsius {
   | ------------------------------ first implementation here
...
21 | impl Converts<f64> for Celsius {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ conflicting implementation for `Celsius`

For more information about this error, try `rustc --explain E0119`.
```

**کامپایلر به چه اعتراض دارد:** این‌بار پیام `Converts<f64>` را می‌گوید، نه `ConvertsTo`. دقیقاً همان مکانیزمِ خطایِ اول — یک جفتِ تکراری — ولی حالا رویِ جفتِ «`Converts<f64>`، برایِ `Celsius`». دو `impl` مختلف نبودند؛ دو بار همان یکی بودند.

**راه‌حل:** یکی از دو `impl Converts<f64>` را حذف کن — یا اگر واقعاً دو تبدیلِ متفاوت به `f64` می‌خواهی (فارنهایت، کلوین)، این دو دیگر «همان `T`» نیستند از نظرِ کاربرد، پس مجبوری اسمشان را جدا کنی، مثلاً با دو متدِ معمولی به‌جایِ دو `impl` از یک صفت.

**چرا این راه‌حل است:** این خطا ادعایِ بخشِ «مفهوم» را ثابت می‌کند: محدودیت رویِ *جفتِ (صفت، نوع)* است، نه رویِ اسمِ صفت. `Converts<f64>` یک جفت است، `Converts<String>` جفتِ دیگری — و هر جفت دقیقاً یک‌بار.

---

## تمرین

### گرم‌کردن

<details>
<summary>برایِ <code>impl Iterator for Countdown { type Item = u8; ... }</code>، می‌شود یک <code>impl</code>ِ دومی هم نوشت با <code>type Item = i32;</code>؟</summary>

فکرت را قبل از خواندنِ پاسخ بنویس.

</details>

<details>
<summary>پاسخ</summary>

نه. `Iterator` هیچ پارامتری رویِ خودش نمی‌گیرد، پس فقط یک جفتِ «`Iterator`، برایِ `Countdown`» هست — `E0119`.

</details>

<details>
<summary>یک صفتِ فرضی <code>trait Holds&lt;T&gt; { fn get(&self) -> T; }</code> را در نظر بگیر. یک نوع می‌تواند هم <code>Holds&lt;i32&gt;</code> را پیاده کند هم <code>Holds&lt;String&gt;</code> را، هردو هم‌زمان؟</summary>

فکرت را قبل از خواندنِ پاسخ بنویس.

</details>

<details>
<summary>پاسخ</summary>

بله. `Holds<i32>` و `Holds<String>` دو جفتِ متفاوت‌اند از دیدِ کامپایلر، پس هردو `impl` هم‌زمان معتبرند.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
trait Labels<T> {
    fn label(&self) -> T;
}
struct Item;
impl Labels<i32> for Item {
    fn label(&self) -> i32 {
        1
    }
}
impl Labels<bool> for Item {
    fn label(&self) -> bool {
        true
    }
}
let y = Item.label();
```

</details>

<details>
<summary>پاسخ</summary>

نه — `E0283`. هم `Labels<i32>` هست هم `Labels<bool>`، و `y` هیچ نوعی نگفته.

</details>

<details>
<summary>درست یا غلط: صدا زدنِ <code>iter.next()</code> رویِ یک پیمایشگرِ سفارشیِ خودت، هیچ‌وقت به نوع‌نویسیِ صریح نیاز ندارد.</summary>

فکرت را قبل از خواندنِ پاسخ بنویس.

</details>

<details>
<summary>پاسخ</summary>

درست. `Item` دقیقاً یک‌بار، در تعریفِ `impl Iterator`، مشخص می‌شود — نه در نقطه‌ی صدا زدن. همیشه فقط یک جواب هست، پس هیچ‌وقت ابهامی نیست.

</details>

<details>
<summary>درست یا غلط: نوعِ وابسته، خودِ صفت را جنریک می‌کند.</summary>

فکرت را قبل از خواندنِ پاسخ بنویس.

</details>

<details>
<summary>پاسخ</summary>

غلط — دقیقاً برعکسش. نوعِ وابسته باعث می‌شود صفت جنریک *نباشد*. `trait Iterator` هیچ `<T>`ای رویِ خودش ندارد؛ `type Item;` فقط می‌گوید هر پیاده‌ساز یک‌بار چه چیزی را جایگزین می‌کند.

</details>

### تعمیر

هر چهار مثالِ خراب را درست کن:

۱. `examples/05-two-impls-conflict.rs` — یکی از دو `impl` را حذف کن. کدام‌یک را نگه می‌داری؟ در یک کامنت بنویس چرا.
۲. `examples/06-ambiguous-convert-call.rs` — با یک نوع‌نویسیِ صریح یا نحوِ کاملاً واجدشده، بگو کدام تبدیل را می‌خواهی.
۳. `examples/07-mismatched-target-type.rs` — یا بدنه را با `Target` جور کن، یا خودِ `Target` را عوض کن. هردو راه را امتحان کن؛ کدام‌یک منطقی‌تر است وقتی اسمِ نوعت `Celsius` است و `Target = f64`؟
۴. `examples/08-same-target-twice-conflicts.rs` — یکی از دو `impl Converts<f64>` را حذف کن، یا اگر واقعاً هردو تبدیل را می‌خواهی، اسمشان را جدا کن تا دیگر تکرارِ همان `impl` نباشد.

### پیاده‌سازی

دو صفت، از قبل کامل نوشته‌شده، در `src/lib.rs` — کارت این نیست که طراحی‌شان کنی، کارت این است که بدنه‌هایی بنویسی که به قولِ هر شکل وفادار بمانند:

```sh
cargo test -p p2-03-05-associated-types
```

`Measures` نوعِ وابسته دارد — `Rectangle` را دقیقاً یک‌بار پیاده می‌کنی، و `measure()` هیچ‌جا نیاز به نوع‌نویسی ندارد. `DescribesAs<T>` پارامترِ جنریک دارد — `Rectangle` را دوبار پیاده می‌کنی، یک‌بار برایِ `u32` (محیط)، یک‌بار برایِ `String` (برچسب). کامنتِ مستنداتِ هرکدام دقیقاً می‌گوید چه چیزی برمی‌گردد؛ چیزی را حدس نزن.

### بساز

یک `Track` بساز (اسمِ آهنگ، و مدتِ آن به ثانیه). دو بخشِ برنامه‌ات هم‌زمان به یک برچسبِ «الان پخش می‌شود» از رویِ `Track` نیاز دارند: یکی برایِ نوارِ پیشرفت فقط عددِ خامِ ثانیه‌ها را می‌خواهد (`u32`)؛ یکی برایِ نمایش، یک رشته‌ی خوانا مثلِ `"Title — 3:45"` می‌خواهد.

خودت یک صفت طراحی کن — نوعِ وابسته یا پارامترِ جنریک، خودت انتخاب کن — و آن را برایِ `Track` پیاده کن. فرمتِ دقیقِ رشته با خودت است؛ فقط در یک کامنتِ بالایِ صفت، در یکی‌دو جمله، بگو چرا آن یکیِ دیگر این‌جا جواب نمی‌داد.

### چالش (اختیاری)

صفتِ استانداردِ `std::ops::Add` را جست‌وجو کن. امضایِ واقعی‌اش تقریباً این است: `pub trait Add<Rhs = Self> { type Output; fn add(self, rhs: Rhs) -> Self::Output; }` — هم یک پارامترِ جنریک دارد (`Rhs`) هم یک نوعِ وابسته (`Output`)، هردو با هم، رویِ یک صفت.

یک `Money` بساز (مثلاً سنت، به‌عنوانِ `i64`) و `Add` را برایش پیاده کن تا بشود دو `Money` را با `+` جمع زد.

بعد، فقط با فکر کردن — لازم نیست کدش را بنویسی: چرا `Output` باید نوعِ وابسته باشد، نه پارامترِ جنریک؟ و چرا `Rhs` برعکس — پارامترِ جنریک است، نه نوعِ وابسته؟ (سرنخ: کدام‌یک از این دو واقعاً باید بتواند بیش از یک جوابِ درست داشته باشد؟)

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| نوعِ وابسته (`type Target;`) | نوعی که خودِ پیاده‌ساز، یک‌بار، مشخص می‌کند | وقتی یک نوع همیشه فقط یک جوابِ درست دارد |
| پارامترِ جنریکِ صفت (`trait Foo<T>`) | پارامتری رویِ خودِ صفت، نه رویِ تابع یا ساختار | وقتی یک نوع واقعاً چند جوابِ درست دارد |
| نحوِ کاملاً واجدشده (`<Type as Trait<T>>::method()`) | گفتنِ دقیقِ صفت و `T`اش به کامپایلر | رفعِ ابهامِ چند `impl` |
| `E0119` | دو `impl` برایِ یک جفتِ (صفت، نوع) | همیشه نشانه‌ی یک تصمیمِ طراحیِ اشتباه، نه یک تایپو |

### الان می‌دانی

- `type Item;` (و هر نوعِ وابسته‌ی دیگر) خودِ صفت را جنریک نمی‌کند؛ فقط می‌گوید هر پیاده‌ساز، یک‌بار، چه چیزی را جایگزین می‌کند — و برایِ همین، نقطه‌ی صدا زدنش هیچ‌وقت نوع‌نویسی نمی‌خواهد.
- یک پارامترِ جنریک رویِ خودِ صفت (`trait Foo<T>`) خودِ صفت را جنریک می‌کند — `Foo<A>` و `Foo<B>` دو جفتِ متفاوت‌اند، پس یک نوع می‌تواند هردو را هم‌زمان پیاده کند.
- محدودیتِ «فقط یک `impl`» رویِ *جفتِ (صفت، نوع)* است، نه رویِ اسمِ صفت — `Converts<f64>` را دوبار پیاده کردن دقیقاً همان خطایِ `ConvertsTo` را می‌دهد.
- این آزادی رایگان نیست: هرجا بیش از یک `impl` هست، نقطه‌ی صدا زدن باید بگوید کدام‌یک را می‌خواهد — با نوع‌نویسیِ صریح یا با نحوِ کاملاً واجدشده.
- انتخاب بینِ این دو یک سؤال است: این نوع همیشه فقط یک‌جور درست جواب می‌دهد (نوعِ وابسته)، یا واقعاً چند جواب دارد و صداکننده باید انتخاب کند (پارامترِ جنریک)؟

### بعداً کامل‌تر می‌بینی

- **صفت‌هایی که رویِ صفتِ دیگری می‌سازند (مثلِ `DoubleEndedIterator` رویِ `Iterator`)** — [۲.۳.۶ — ابرصفت‌ها، پیاده‌سازیِ فراگیر، قاعده‌ی یتیم](../06-supertraits-blanket-impls-orphan-rule/README.fa.md)
- **وقتی نوعِ دقیق را فقط زمانِ اجرا می‌فهمی، نه سرِ کامپایل** — [۲.۳.۷ — ارسالِ ایستا در برابرِ پویا](../07-static-vs-dynamic-dispatch/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا `type Item;` خودِ `Iterator` را جنریک نمی‌کند، ولی `trait Converts<T>` صفت را جنریک می‌کند؟
- چرا دو `impl ConvertsTo for Celsius` تضاد است، ولی `impl Converts<f64> for Celsius` و `impl Converts<String> for Celsius` نیست؟
- چرا دو `impl Converts<f64> for Celsius` (همان `T`، دوبار) باز هم تضاد است؟
- دو راهِ رفعِ ابهامِ یک صدا زدنِ چندپیاده‌سازی‌ای را نام ببر.
- برایِ یک صفتِ تازه که خودت طراحی می‌کنی، چطور تصمیم می‌گیری بینِ نوعِ وابسته و پارامترِ جنریک؟

---

## بیشتر

- [کتابِ Rust — صفت‌هایِ پیشرفته، نوع‌هایِ وابسته](https://doc.rust-lang.org/book/ch20-02-advanced-traits.html) — دقیقاً همین تضاد را با همان مثالِ `Iterator` توضیح می‌دهد.
- [مستنداتِ صفتِ `Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
- [مستنداتِ صفتِ `std::ops::Add`](https://doc.rust-lang.org/std/ops/trait.Add.html) — همان صفتی که در «چالش» دیدی: هم نوعِ وابسته هم پارامترِ جنریک، با هم، رویِ یک صفت.
- [مرجعِ رسمیِ Rust — آیتم‌هایِ وابسته](https://doc.rust-lang.org/reference/items/associated-items.html) — تعریفِ رسمی‌تر و دقیق‌تر.
