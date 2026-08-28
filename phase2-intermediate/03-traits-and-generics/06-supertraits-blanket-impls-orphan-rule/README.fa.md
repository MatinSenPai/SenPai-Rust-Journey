# ۲.۳.۶ — ابرصفت‌ها، پیاده‌سازیِ فراگیر، قاعده‌ی یتیم و راهِ فرارِ newtype

## در یک نگاه

بعد از این درس می‌توانی:

- یک ابرصفت تعریف کنی — صفتی که نوع را مجبور می‌کند پیش از پیاده‌سازیِ خودش، صفتِ دیگری را هم پیاده‌سازی کرده باشد — و بگویی چرا این کار «ارث‌بری» نیست.
- بندِ `where` در پیاده‌سازیِ فراگیرِ `Into` را که در ۱.۶.۵ دیده بودی، این‌بار واقعاً بخوانی و بفهمی چه می‌گوید، و یک پیاده‌سازیِ فراگیرِ کوچکِ مالِ خودت بنویسی.
- قاعده‌ی یتیم را از حفظ بگویی، توضیح بدهی چرا وجود دارد، و با پیچیدنِ یک نوعِ بیگانه در newtypeِ خودت — همان الگویی که در ۱.۵.۲ برایِ ایمنیِ نوع دیدی — دورش بزنی.
- `E0277` روی یک ابرصفتِ گمشده، و `E0117` روی نقضِ قاعده‌ی یتیم را خودت بخوانی و رفع کنی.

**زمان:** حدود ۶۰ دقیقه · **پیش‌نیاز:**
[۲.۳.۱ — تعریف و پیاده‌سازیِ traitها](../01-defining-and-implementing-traits/README.fa.md)، و به‌طورِ خاص
[۲.۳.۲ — توابع و ساختارهای جنریک](../02-generic-functions-and-structs/README.fa.md)،
[۱.۵.۲ — ساختارهای تاپلی و الگوی newtype](../../../phase1-fundamentals/05-your-own-types/02-tuple-structs-and-newtype/README.fa.md) و
[۱.۶.۵ — `From` و تبدیلِ خطا](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.fa.md)

---

## چرا اهمیت دارد

دو وعده‌ی ناتمام از قبل داری.

اولی از [۱.۶.۵](../../../phase1-fundamentals/06-absence-and-failure/05-from-and-error-conversion/README.fa.md) است. آن‌جا این کد را دیدی:

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

و درس همان‌جا گفت: «نگرانِ نحوش نباش — نوشتنِ چیزی به این شکل کارِ فاز ۲ است. فقط بخوانش.» آن روز فقط خواندی. امروز قرار است بفهمی این نحو دقیقاً چه می‌گوید، و خودت یکی از این‌ها را بنویسی.

دومی از [۱.۵.۲](../../../phase1-fundamentals/05-your-own-types/02-tuple-structs-and-newtype/README.fa.md) است. آن‌جا `AccountId(u64)` و `Rial(i64)` را ساختی تا کامپایلر نگذارد یک شناسه‌ی حساب جای یک مبلغِ پول بنشیند. آن درس گفت newtype یک نوعِ تازه می‌سازد که کامپایلر می‌شناسدش. چیزی که نگفت این بود: گاهی newtype را نه برای ایمنیِ نوع، بلکه چون **راهِ دیگری نداری** می‌سازی.

فرض کن می‌خواهی یک `Vec<i32>` را به‌شکلِ دلخواهِ خودت چاپ کنی — با `println!("{}", numbers)`، نه با `{:?}`. این یعنی `impl std::fmt::Display for Vec<i32>`. بنویسش و کامپایل کن:

```text
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
```

نه `Vec` مالِ توست، نه `Display`. هر دو را کتابخانه‌ی استاندارد نوشته. کامپایلر با یک قانون این را رد می‌کند که اسمش **قاعده‌ی یتیم (orphan rule)** است، و رد کردنش دلیلِ خوبی دارد — دلیلی که تا آخرِ این درس می‌فهمی. و راهِ دورزدنش، جالب اینجاست، همان ابزاری است که در ۱.۵.۲ یاد گرفتی، این‌بار برای یک کارِ کاملاً متفاوت.

---

## مفهوم

### ابرصفت: صفتی که به صفتِ دیگری تکیه دارد

یک صفتِ کوچک برای «هرچیزی که اسم دارد»:

```rust
trait Named {
    fn name(&self) -> String;
}
```

و حالا صفتِ دومی که می‌خواهد سلام بدهد — ولی برای اینکه بتواند این کار را بکند، باید بداند طرف اسمی دارد:

```rust
trait Greet: Named {
    fn greet(&self) -> String {
        format!("Hello, I'm {}!", self.name())
    }
}
```

`trait Greet: Named` را ببین. این را **ابرصفت (supertrait)** می‌گویند: `Named` ابرصفتِ `Greet` است، و این خط یعنی «هیچ نوعی نمی‌تواند `Greet` را پیاده‌سازی کند مگر اینکه از قبل `Named` را هم پیاده‌سازی کرده باشد.» بدنه‌ی پیش‌فرضِ `greet` همین الان دارد `self.name()` را صدا می‌زند — متدی که خودِ `Greet` تعریفش نکرده، `Named` تعریفش کرده. کامپایلر این فراخوانی را قبول می‌کند چون همان خطِ `: Named` تضمین داده هر `Self`ای که به اینجا برسد، یک `name()` هم دارد.

حالا دو نوع که هر دو صفت را پیاده می‌کنند:

```rust
struct Villager {
    name: String,
}

impl Named for Villager {
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl Greet for Villager {}
```

`impl Greet for Villager {}` بدنه‌اش خالی است — همان پیش‌فرض را می‌گیرد. اجرا کن:

```sh
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 01-supertraits-basics
```

```text
Hello, I'm Rin!
Hello, I'm Old Tom of the Blacksmith!
```

خطِ دوم از یک نوعِ دومِ همان فایل می‌آید — `Merchant`، که `name()`ِ خودش را جوردیگری می‌سازد (اسم به‌علاوه‌ی نامِ مغازه). `Greet` برایش مهم نیست `name()` از کجا آمده؛ فقط باید وجود داشته باشد.

### این ارث‌بری نیست — یک وابستگی است

اینجا جایی است که وسوسه می‌شوی فکر کنی `Greet: Named` یعنی همان چیزی که در پایتون یا جاوا با ارث‌بریِ کلاس می‌شناسی. نیست، و فرقش مهم است.

در ارث‌بریِ شیءگرا، یک زیرکلاس **رایگان** چیزی از والدش می‌گیرد — فیلدها، متدهای پیاده‌سازی‌شده، همه‌چیز، بدونِ اینکه دوباره بنویسی‌شان. اینجا هیچ‌چیز رایگان نیست. `impl Greet for Villager {}` هیچ‌کدام از فیلدها یا رفتارِ `Named` را به `Villager` **نمی‌دهد**. تو باید جداگانه `impl Named for Villager` را بنویسی — همان‌طور که بالا نوشتی — و اگر ننویسی‌اش، «خطاهایی که خواهی دید» نشانت می‌دهد دقیقاً چه اتفاقی می‌افتد.

ابرصفت فقط یک **شرط** است: «تا وقتی `Named` را پیاده نکرده‌ای، اجازه نداری `Greet` را پیاده کنی.» یک وابستگیِ سطحِ صفت، نه هدیه‌ای که از بالا می‌رسد.

### از داخلِ کراندِ فرزند هم به متدهای ابرصفت می‌رسی

یک نکته‌ی عملی که مستقیم از تعریفِ بالا نتیجه می‌شود: تابعی که جنریک است رویِ `T: Greet`، می‌تواند متدهای `Named` را هم رویِ همان `T` صدا بزند — بدونِ اینکه لازم باشد بنویسی `T: Greet + Named`:

```rust
fn print_intro<T: Greet>(entity: &T) {
    println!("{}", entity.greet());
    println!("(bare name: {})", entity.name());
}
```

```sh
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 02-supertraits-through-the-bound
```

```text
Hello, I'm Rin!
(bare name: Rin)
```

`print_intro` فقط `T: Greet` نوشته، ولی `entity.name()` — متدِ `Named` — را هم صدا زد. کامپایلر این را رد نکرد چون خودِ `trait Greet: Named` از قبل ثابت کرده: هر `T`ای که `Greet` را ارضا می‌کند، `Named` را هم ارضا کرده. نوشتنِ `+ Named` زائد بود.

### ابرصفت‌هایی که از قبل، بی‌خبر، ازشان استفاده می‌کردی

این الگو تویِ کتابخانه‌ی استاندارد همه‌جا هست. همان `+` که در [۲.۳.۲](../02-generic-functions-and-structs/README.fa.md) برایِ ترکیبِ چند کراند رویِ یک پارامترِ جنریک دیدی (`T: Display + Clone`)، دقیقاً همان `+` است که بینِ چند ابرصفت می‌آید:

```text
pub trait Eq: PartialEq<Self> {}
pub trait Ord: Eq + PartialOrd<Self> { /* ... */ }
pub trait Copy: Clone {}
```

`Copy` یک ابرصفت دارد: `Clone`. برایِ همین هر نوعی که `#[derive(Copy)]` می‌زنی، باید `Clone` را هم بزند — دقیقاً همان چیزی که در [۱.۲.۳](../../../phase1-fundamentals/02-ownership-and-memory/03-clone-and-copy/README.fa.md) دیدی، بدونِ اینکه آن‌جا اسمش «ابرصفت» بود. `Ord` دو ابرصفت دارد، با `+` کنارِ هم — همان نحوی که برایِ کراندهایِ جنریک بلدی، اینجا رویِ خودِ `Self` به‌کار رفته.

### پیاده‌سازیِ فراگیر، دوباره — این‌بار می‌توانی where را بخوانی

برگرد سراغِ چیزی که بالایِ همین درس، از ۱.۶.۵، نقل کردیم:

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

حالا از [۲.۳.۲](../02-generic-functions-and-structs/README.fa.md) بلدی `<T>` و `where` یعنی چه. بخوانش، این‌بار کلمه‌به‌کلمه: «برایِ **هر** دو نوعِ `T` و `U`، اگر `U: From<T>` برقرار باشد، آن‌وقت `T: Into<U>`.» کلمه‌ی کلیدی «هر» است — نه یک `T` و `U`ِ خاص که خودِ کتابخانه انتخاب کرده باشد، بلکه همه‌شان، یک‌جا. به این می‌گویند **پیاده‌سازیِ فراگیر (blanket impl)**: یک `impl` که به‌جایِ یک نوعِ ملموسِ تنها، رویِ هر نوعی که یک کراند را ارضا می‌کند اجرا می‌شود.

نتیجه‌اش همانی بود که در ۱.۶.۵ دیدی: **هیچ‌کس دستی `impl Into` نمی‌نویسد**، چون این یک `impl` — همین بالا — از قبل هر `From` را به یک `Into` تبدیل کرده.

### یک پیاده‌سازیِ فراگیرِ خودت

حالا نوبتِ توست که یکی از این‌ها را بسازی. برگرد به `Named`/`Greet`. هر نوعی که می‌خواهد `Greet` داشته باشد، مجبور بود یک خطِ خالیِ `impl Greet for X {}` هم بنویسد — حتی با اینکه بدنه‌اش همیشه همان پیش‌فرض است. اگر ده نوع داشتی، ده بار همان خطِ خالی را می‌نوشتی.

به‌جایش، یک بار این را بنویس:

```rust
impl<T: Named> Greet for T {}
```

همین. حالا **هر** نوعی که `Named` را پیاده‌سازی کند، خودکار `Greet` را هم دارد — بدونِ اینکه جایی `impl Greet for Villager {}` یا `impl Greet for Merchant {}` بنویسی:

```rust
struct Villager {
    name: String,
}

impl Named for Villager {
    fn name(&self) -> String {
        self.name.clone()
    }
}
```

```sh
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 03-blanket-impl-of-your-own
```

```text
Hello, I'm Rin!
Hello, I'm Old Tom of the Blacksmith!
```

عینِ همان خروجیِ مثالِ اول — همان دو نوع، همان `name()`ها — ولی این‌بار هیچ‌جایِ فایل عبارتِ `impl Greet for` دوباره ظاهر نشده. `impl<T: Named> Greet for T {}` دقیقاً همان کاری را کرد که پیاده‌سازیِ فراگیرِ `Into` برایِ `From` کرد: یک `impl` نوشتی، برایِ همه‌ی نوع‌ها یک‌جا کار کرد.

(یک نکته‌ی جانبی، برایِ بعد: حالا که `Greet` را با پیاده‌سازیِ فراگیر گرفته‌ای، دیگر نمی‌توانی برایِ یک نوعِ خاص یک `greet` سفارشی بنویسی — آن `impl` با همین یکی تداخل می‌کند. تویِ گرم‌کردنِ همین درس بیشتر می‌بینی‌اش.)

### قاعده‌ی یتیم، و دو شکلِ مجازش

برگردیم سراغِ چیزی که در «چرا اهمیت دارد» شکست خورد: `impl Display for Vec<i32>`. حالا وقتِ قاعده‌ی دقیق است:

> **می‌توانی یک صفت را برایِ یک نوع پیاده‌سازی کنی، فقط اگر دستِ‌کم یکی از این دو — خودِ صفت یا خودِ نوع — در crateِ خودت تعریف شده باشد.**

دو شکلِ مجاز، هر دو در یک فایل:

```rust
trait Summarized {
    fn summarize(&self) -> String;
}

// نوعِ بیگانه (`Vec` مالِ std است)، صفتِ محلی (`Summarized` مالِ ماست).
impl Summarized for Vec<i32> {
    fn summarize(&self) -> String {
        let total: i32 = self.iter().sum();
        format!("{} numbers, sum {}", self.len(), total)
    }
}
```

```rust
struct Point {
    x: i32,
    y: i32,
}

// نوعِ محلی (`Point` مالِ ماست)، صفتِ بیگانه (`Display` مالِ std است).
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
```

```sh
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 04-orphan-rule-legal-forms
```

```text
4 numbers, sum 10
(3, 4)
```

هر دو کامپایل می‌شوند، چون هر دو دقیقاً یک طرفِ محلی دارند. `impl Display for Vec<i32>` — چیزی که در بالایِ درس شکست خورد — هیچ طرفِ محلی ندارد: نه `Display` مالِ توست، نه `Vec`. دقیقاً همین یکی خطِ فاصل است.

```senpai-visual
{"kind":"concept","labels":["Villager فقط Greet دارد","کامپایلر Named را هم می‌خواهد","نبودنش یعنی E0277","حالا Named برای Villager نوشته شد","Greet هم کامپایل می‌شود"]}
```

### چرا این قاعده وجود دارد

فرض کن قاعده‌ی یتیم نبود. دو کتابخانه‌ی کاملاً بی‌ربط — بگو `crate_weather` و `crate_finance` — هر دو تصمیم می‌گیرند `impl std::fmt::Display for Vec<i32>` بنویسند، هرکدام به سلیقه‌ی خودشان (یکی با کاما جدا می‌کند، دیگری با فاصله). حالا برنامه‌ی تو هر دو کتابخانه را import می‌کند و یک‌جا `println!("{}", my_vec)` می‌نویسد. کدام `impl` باید اجرا شود؟ نه کامپایلر می‌داند، نه تو، نه حتی نویسنده‌ی هیچ‌کدام از آن دو کتابخانه — چون هیچ‌کدامشان از وجودِ دیگری خبر نداشت.

قاعده‌ی یتیم این وضعیت را از ریشه غیرممکن می‌کند. یک `impl Trait for Type` فقط جایی مجاز است که دستِ‌کم یکی از `Trait` یا `Type` در همان crate تعریف شده باشد — یعنی همیشه یک نفرِ مشخص، صاحبِ آن `impl` است: یا نویسنده‌ی صفت، یا نویسنده‌ی نوع. دو crateِ بی‌ربط هرگز نمی‌توانند هر دو صاحبِ همان ترکیب باشند، پس این تصادم اصلاً رخ نمی‌دهد. (قاعده‌ی دقیق برایِ نوع‌های جنریک کمی ظریف‌تر است؛ چیزی که امروز لازم داری همین یک جمله است.)

### راهِ فرارِ newtype: پوششِ محلی برایِ یک نوعِ بیگانه

پس اگر واقعاً بخواهی `Vec<i32>` را با `Display` چاپ کنی چه؟ همان الگویی که در [۱.۵.۲](../../../phase1-fundamentals/05-your-own-types/02-tuple-structs-and-newtype/README.fa.md) یاد گرفتی، این‌بار برایِ یک کارِ دوم: `Vec<i32>` را در یک ساختارِ تاپلیِ محلیِ خودت بپیچ.

```rust
struct Numbers(Vec<i32>);

impl fmt::Display for Numbers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total: i32 = self.0.iter().sum();
        write!(f, "{} numbers, sum {}", self.0.len(), total)
    }
}
```

```sh
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 05-newtype-escape-hatch
```

```text
4 numbers, sum 10
```

`Numbers` یک `Vec<i32>` است، فقط با یک لایه‌ی نازکِ محلی دورش. همین لایه کافی است: `Numbers` در crateِ خودت تعریف شده، پس `impl Display for Numbers` قاعده‌ی یتیم را کاملاً ارضا می‌کند — با اینکه چیزی که واقعاً داخلش است، عینِ همان `Vec<i32>`یِ بیگانه است.

در [۱.۵.۲](../../../phase1-fundamentals/05-your-own-types/02-tuple-structs-and-newtype/README.fa.md)، newtype را برایِ این ساختی که کامپایلر یک `AccountId` را با یک `Rial` قاطی نکند — یعنی **ایمنیِ نوع**. اینجا newtype دارد کارِ کاملاً دیگری می‌کند: نه برایِ اینکه دو چیزِ شبیه‌به‌هم را از هم جدا کند، بلکه برایِ اینکه یک نوعِ بیگانه را **محلی** کند تا قاعده‌ی یتیم اجازه بدهد. همان ابزار، دومین کاربردش — و کاربردی که خیلی از کدهای واقعیِ Rust به همین دلیل newtype می‌زنند.

یک هزینه دارد: `Numbers` هیچ متدی از `Vec` به ارث نمی‌برد. `numbers.push(5)` کامپایل نمی‌شود؛ باید `numbers.0.push(5)` بنویسی، یا یک متدِ forwarding خودت اضافه کنی. راهی هست که این متدها را خودکار عبور بدهی — `Deref` — و آن را در [۲.۴.۳](../../04-lifetimes-and-conversion/03-deref-asref-borrow/README.fa.md) کامل می‌بینی.

```senpai-visual
{"kind":"concept","labels":["Display برای Vec مستقیم","هر دو بیگانه‌اند","یعنی E0117","پیچیدنش در Numbers","حالا Display کامپایل می‌شود"]}
```

---

## دست‌به‌کد

```sh
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 01-supertraits-basics
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 02-supertraits-through-the-bound
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 03-blanket-impl-of-your-own
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 04-orphan-rule-legal-forms
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 05-newtype-escape-hatch
```

بعد دو تای خراب:

```sh
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 06-missing-supertrait-impl --features broken
cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 07-orphan-rule-violation --features broken
```

بعد این‌ها را امتحان کن:

۱. در `01-supertraits-basics`، یک نوعِ سومی اضافه کن — مثلاً `Guard` — که `Named` را پیاده‌سازی می‌کند و بعد `Greet` را (بدونِ بازنویسیِ `greet`). آیا `greet`ی که می‌گیرد همان پیش‌فرض است؟
۲. در `03-blanket-impl-of-your-own`، سعی کن یک `impl Greet for Villager { ... }` دستی هم اضافه کنی، کنارِ پیاده‌سازیِ فراگیرِ `impl<T: Named> Greet for T {}` که از قبل هست. چه خطایی می‌گیری؟ کدِ خطا را نگه دار — «گرم‌کردن» همین درس همین را می‌پرسد.
۳. در `04-orphan-rule-legal-forms`، یک `impl Summarized for Vec<i32>` دومی اضافه کن، درست زیرِ اولی. چه خطایی می‌گیری، و چرا این یکی به قاعده‌ی یتیم ربطی ندارد؟

---

## خطاهایی که خواهی دید

### `E0277` — ابرصفتِ گمشده

`examples/06-missing-supertrait-impl.rs` یک `impl Greet for Villager {}` دارد، بدونِ اینکه `Villager` هیچ‌جا `Named` را پیاده‌سازی کند:

```text
error[E0277]: the trait bound `Villager: Named` is not satisfied
  --> phase2-intermediate\03-traits-and-generics\06-supertraits-blanket-impls-orphan-rule\examples\06-missing-supertrait-impl.rs:20:16
   |
20 | impl Greet for Villager {}
   |                ^^^^^^^^ unsatisfied trait bound
   |
help: the trait `Named` is not implemented for `Villager`
  --> phase2-intermediate\03-traits-and-generics\06-supertraits-blanket-impls-orphan-rule\examples\06-missing-supertrait-impl.rs:15:1
   |
15 | struct Villager {
   | ^^^^^^^^^^^^^^^
help: this trait has no implementations, consider adding one
  --> phase2-intermediate\03-traits-and-generics\06-supertraits-blanket-impls-orphan-rule\examples\06-missing-supertrait-impl.rs:5:1
   |
 5 | trait Named {
   | ^^^^^^^^^^^
note: required by a bound in `Greet`
  --> phase2-intermediate\03-traits-and-generics\06-supertraits-blanket-impls-orphan-rule\examples\06-missing-supertrait-impl.rs:9:14
   |
 9 | trait Greet: Named {
   |              ^^^^^ required by this bound in `Greet`
```

همین فایل یک `E0277`ِ دوم هم می‌دهد — سرِ `rin.greet()` در `main`، نه سرِ خطِ `impl`:

```text
error[E0277]: the trait bound `Villager: Named` is not satisfied
  --> phase2-intermediate\03-traits-and-generics\06-supertraits-blanket-impls-orphan-rule\examples\06-missing-supertrait-impl.rs:26:24
   |
26 |     println!("{}", rin.greet());
   |                        ^^^^^ unsatisfied trait bound
   |
help: the trait `Named` is not implemented for `Villager`
  --> phase2-intermediate\03-traits-and-generics\06-supertraits-blanket-impls-orphan-rule\examples\06-missing-supertrait-impl.rs:15:1
   |
15 | struct Villager {
   | ^^^^^^^^^^^^^^^
help: this trait has no implementations, consider adding one
  --> phase2-intermediate\03-traits-and-generics\06-supertraits-blanket-impls-orphan-rule\examples\06-missing-supertrait-impl.rs:5:1
   |
 5 | trait Named {
   | ^^^^^^^^^^^
note: required by a bound in `Greet::greet`
  --> phase2-intermediate\03-traits-and-generics\06-supertraits-blanket-impls-orphan-rule\examples\06-missing-supertrait-impl.rs:9:14
   |
 9 | trait Greet: Named {
   |              ^^^^^ required by this bound in `Greet::greet`
10 |     fn greet(&self) -> String {
   |        ----- required by a bound in this associated function
```

**کامپایلر به چه اعتراض دارد:** پیامِ اول دقیقاً همان جمله‌ای است که در «مفهوم» خواندی: «the trait bound `Villager: Named` is not satisfied». `trait Greet: Named` قولی داده بود: «هر `Self`ای که به اینجا می‌رسد، `Named` هم هست.» `Villager` این قول را نگه نداشته، پس حتی خطِ `impl Greet for Villager {}` — پیش از اینکه هیچ متدی صدا زده شود — رد می‌شود. پیامِ دوم همان مشکل را، این‌بار سرِ فراخوانیِ `greet()`، دوباره نشان می‌دهد.

**راه‌حل:** آن `impl Named` که یادت رفته بود را بنویس:

```rust
impl Named for Villager {
    fn name(&self) -> String {
        self.name.clone()
    }
}
```

**چرا این راه‌حل است:** پیامِ کمکِ کامپایلر خودش می‌گوید «the trait `Named` is not implemented for `Villager`» — تحت‌اللفظی همان چیزی که باید بنویسی. همین که این `impl` سرِ جایش باشد، هم خطِ `impl Greet for Villager {}` کامپایل می‌شود، هم `rin.greet()` — چون هر دو خطا از یک ریشه بودند.

### `E0117` — نقضِ قاعده‌ی یتیم

`examples/07-orphan-rule-violation.rs` سعی می‌کند `Display` — صفتِ بیگانه — را مستقیم برایِ `Vec<i32>` — نوعِ بیگانه — پیاده‌سازی کند:

```text
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
 --> phase2-intermediate\03-traits-and-generics\06-supertraits-blanket-impls-orphan-rule\examples\07-orphan-rule-violation.rs:9:1
  |
9 | impl fmt::Display for Vec<i32> {
  | ^^^^^^^^^^^^^^^^^^^^^^--------
  |                       |
  |                       `Vec` is not defined in the current crate
  |
  = note: impl doesn't have any local type before any uncovered type parameters
  = note: for more information see https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules
  = note: define and implement a trait or new type instead
```

**کامپایلر به چه اعتراض دارد:** خطِ آخرِ خودِ کامپایلر راه‌حل را لو می‌دهد: «define and implement a trait or new type instead» — یک صفتِ خودت تعریف کن، یا یک نوعِ تازه. زیرِ همان خط، کامپایلر مستقیم روی `Vec` انگشت گذاشته و نوشته «not defined in the current crate» — دقیقاً همان چیزی که در «مفهوم» بهش رسیدی: نه `Display`، نه `Vec`، هیچ‌کدام مالِ این crate نیستند.

**راه‌حل:** یکی از دو راهی که در «مفهوم» دیدی — یا صفت را محلی کن، یا نوع را:

```rust
struct Numbers(Vec<i32>);

impl fmt::Display for Numbers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total: i32 = self.0.iter().sum();
        write!(f, "{} numbers, sum {}", self.0.len(), total)
    }
}
```

**چرا این راه‌حل است:** `Numbers` در همین crate تعریف شده، پس یک طرفِ `impl fmt::Display for Numbers` محلی است — قاعده‌ی یتیم دقیقاً همین را می‌خواست. اگر ترجیح می‌دادی به‌جایِ پیچیدنِ نوع، صفت را محلی کنی، راهِ دیگر هم بود: به‌جایِ `Display`، صفتِ خودت را بنویس (`trait Summarized { ... }`) و مستقیم برایِ `Vec<i32>` پیاده‌اش کن — دقیقاً چیزی که `04-orphan-rule-legal-forms.rs` نشان داد. کدام یکی را انتخاب می‌کنی به این بستگی دارد که کدام سمت واقعاً مالِ توست: اگر می‌خواهی رفتارِ استانداردِ `Display` را (که همه‌جا `println!("{}", ...)` می‌شناسدش) داشته باشی، newtype درست است؛ اگر فقط یک متدِ خودت لازم داری، یک صفتِ محلیِ تازه ساده‌تر است.

---

## تمرین

### گرم‌کردن

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
trait A {}
trait B: A {}

struct S;
impl B for S {}
```

</details>

<details>
<summary>پاسخ</summary>

نه، `E0277`. `B: A` یعنی `S` باید پیش از `impl B for S`، `A` را هم پیاده‌سازی کرده باشد — و اینجا هیچ `impl A for S`ای نیست.

</details>

<details>
<summary>اگر <code>impl Greet for Cat {}</code> بنویسی، آیا این خودکار <code>Cat</code> را هم صاحبِ <code>Named</code> می‌کند؟</summary>

</details>

<details>
<summary>پاسخ</summary>

نه. باید `impl Named for Cat` را جداگانه، خودت بنویسی. ابرصفت فقط چک می‌کند این کار را کرده‌ای؛ چیزی را برایت انجام نمی‌دهد.

</details>

<details>
<summary>با <code>impl&lt;T: Named&gt; Greet for T {}</code> از قبل نوشته‌شده، این هم کامپایل می‌شود؟</summary>

```rust
impl Greet for Villager {
    fn greet(&self) -> String {
        "Meow!".to_string()
    }
}
```

</details>

<details>
<summary>پاسخ</summary>

نه، `E0119` — «conflicting implementations of trait `Greet` for type `Villager`». پیاده‌سازیِ فراگیر از قبل به `Villager` (چون `Named` را دارد) یک `Greet` داده؛ این `impl` دومی همان ترکیب را دوباره ادعا می‌کند. یک نوع فقط یک‌بار می‌تواند یک صفت را پیاده‌سازی کند — همان قاعده‌ای که پشتِ خودِ قاعده‌ی یتیم است، این‌بار بینِ دو `impl` در همان crate.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
struct MyVec(Vec<i32>);

impl std::fmt::Display for MyVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} items", self.0.len())
    }
}
```

</details>

<details>
<summary>پاسخ</summary>

بله. `Display` بیگانه است، ولی `MyVec` محلی است — دقیقاً یک طرف کافی است.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
impl std::fmt::Display for Vec<i32> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} items", self.len())
    }
}
```

</details>

<details>
<summary>پاسخ</summary>

نه، `E0117`. هیچ طرف محلی نیست: نه `Display`، نه `Vec`.

</details>

### تعمیر

`examples/06-missing-supertrait-impl.rs` را با نوشتنِ `impl Named for Villager` درست کن.

بعد `examples/07-orphan-rule-violation.rs` را **دو** جور درست کن:

۱. با پیچیدنِ `Vec<i32>` در یک `struct Numbers(Vec<i32>);` محلی و پیاده‌سازیِ `Display` برایِ `Numbers`.
۲. بدونِ هیچ newtypeای — به‌جایِ `Display`، یک صفتِ خودت بنویس (مثلاً `trait Summarized { fn summarize(&self) -> String; }`) و مستقیم برایِ `Vec<i32>` پیاده‌اش کن.

بعد یک جمله بنویس: این دو راه‌حل کدام طرفِ قاعده‌ی یتیم را محلی می‌کنند — صفت را، یا نوع را؟

### پیاده‌سازی

چهار چیز در `src/lib.rs`:

```sh
cargo test -p p2-03-06-supertraits-blanket-impls-orphan-rule
```

`Discounted` ابرصفتِ `Priced` است؛ بدنه‌ی پیش‌فرضش را کامل کن. `amount_saved` یک تابعِ جنریک است که فقط `T: Discounted` می‌خواهد ولی به متدهایِ هر دو صفت نیاز دارد. `Cart` یک newtype دورِ `Vec<Pen>` است — `total_cents` یک متدِ عادی است، و `Display for Cart` همان جایی است که کلِ درس به هم می‌رسد: `Cart` تنها دلیلی است که این `impl` اصلاً کامپایل می‌شود.

هر چهارتا را دقیق از رویِ کامنتِ مستندساز بالای هرکدام پیاده کن — فرمولِ تخفیف، و فرمتِ دقیقِ متنِ `Cart` همان‌جا نوشته شده.

### بساز

یک صفتِ اول با یک متدِ اجباری تعریف کن (دامنه‌اش هرچه خودت انتخاب کنی — بازی، آشپزی، هر چیزی). یک صفتِ دوم بساز که اولی را به‌عنوانِ ابرصفت بخواهد، با یک متدِ پیش‌فرض که از متدِ صفتِ اول استفاده می‌کند. بعد، به‌جایِ اینکه برایِ هر نوع یک `impl صفتِ‌دوم for نوع {}` جدا بنویسی، یک پیاده‌سازیِ فراگیرِ تنها بنویس که آن را به هر نوعی که صفتِ اول را دارد، رایگان بدهد. با دستِ‌کم دو نوعِ مختلف امتحانش کن.

### چالش (اختیاری)

**بخشِ یک.** فرض کن می‌خواهی یک لیست بسازی که هم `Book` و هم `Pen` را کنارِ هم نگه دارد و رویِ هرکدام `.discounted_cents()` صدا بزند — بدونِ اینکه از قبل بدانی کدام کدام است. `Vec<T>` این کار را نمی‌کند، چون `T` باید یک نوعِ ملموس و یکتا باشد. یک راه هست — یک ترکیبی به اسمِ **شیءِ صفتی (trait object)** — که دقیقاً همین کار را ممکن می‌کند، به قیمتِ یک تصمیمِ متفاوت سرِ کامپایل. آن ترکیب موضوعِ درسِ بعدی است: [۲.۳.۷ — شیءِ صفتی در برابرِ تخصیصِ ایستا](../07-static-vs-dynamic-dispatch/README.fa.md).

**بخشِ دو.** (این جلوتر از امروز را نگاه می‌کند و می‌دانیم.) در مستنداتِ استاندارد تعریفِ واقعیِ `std::cmp::Ord` را جست‌وجو کن. چند ابرصفت دارد؟ چرا هم به `Eq` نیاز دارد هم به `PartialOrd` — این دو چه فرقی با هم دارند که یکی جایِ دیگری کافی نیست؟

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| ابرصفت (supertrait) | `trait B: A` — پیاده‌سازیِ `B` نیازمندِ پیاده‌سازیِ قبلیِ `A` است | سلسله‌مراتبِ صفت‌ها، مثلِ `Ord: Eq + PartialOrd` |
| پیاده‌سازیِ فراگیر (blanket impl) | `impl<T: Bound> Trait for T` — یک `impl`، برایِ هر نوعی که کراند را دارد | `impl<T, U> Into<U> for T where U: From<T>` |
| قاعده‌ی یتیم (orphan rule) | صفت یا نوع، دستِ‌کم یکی باید محلی باشد | جلوگیری از تصادمِ دو crateِ بی‌ربط |
| راهِ فرارِ newtype | پوششِ محلی دورِ یک نوعِ بیگانه، برایِ محلی‌کردنِ آن طرف | `impl Display for Vec<T>` غیرمستقیم |
| `E0277` (اینجا) | ابرصفتِ لازم پیاده‌سازی نشده | `impl` گمشده را بنویس |
| `E0117` | نه صفت محلی است نه نوع | newtype بساز، یا صفتِ خودت را بنویس |

### الان می‌دانی

- `trait B: A` یعنی هیچ نوعی نمی‌تواند `B` را پیاده‌سازی کند مگر اینکه از قبل `A` را پیاده‌سازی کرده باشد — یک وابستگی، نه ارث‌بری؛ هیچ‌چیزی رایگان منتقل نمی‌شود.
- تابعِ جنریکی که فقط `T: B` می‌خواهد، می‌تواند متدهایِ `A` را هم رویِ همان `T` صدا بزند.
- `impl<T: Bound> Trait for T` یک صفت را یک‌بار برایِ همه‌ی نوع‌هایِ واجدِ شرایط پیاده‌سازی می‌کند — دقیقاً همان کاری که کتابخانه‌ی استاندارد با `Into` کرد.
- قاعده‌ی یتیم: `impl Trait for Type` فقط اگر `Trait` یا `Type` محلی باشد مجاز است — برایِ اینکه دو crateِ بی‌ربط هرگز نتوانند صاحبِ همان `impl` باشند.
- پیچیدنِ یک نوعِ بیگانه در یک ساختارِ تاپلیِ محلی، آن را برایِ قاعده‌ی یتیم محلی می‌کند — همان الگویِ newtypeِ ۱.۵.۲، این‌بار برایِ دورزدنِ یک قانون، نه فقط برایِ ایمنیِ نوع.

### بعداً کامل‌تر می‌بینی

- **شیءِ صفتی، برایِ نگه‌داشتنِ چند نوعِ متفاوت پشتِ یک صفتِ مشترک** — [۲.۳.۷ — شیءِ صفتی در برابرِ تخصیصِ ایستا](../07-static-vs-dynamic-dispatch/README.fa.md)
- **`Deref`، تا newtype متدهایِ نوعِ داخلی‌اش را هم رایگان بدهد** — [۲.۴.۳ — `Deref`، `AsRef` و `Borrow`](../../04-lifetimes-and-conversion/03-deref-asref-borrow/README.fa.md)

### می‌توانی توضیح بدهی؟

- تفاوتِ ابرصفت با ارث‌بریِ شیءگرا چیست؟
- چرا تابعی که فقط `T: Discounted` نوشته، هنوز می‌تواند `.price_cents()` را رویِ همان `T` صدا بزند؟
- پیاده‌سازیِ فراگیرِ `Into` را با صدایِ بلند، کلمه‌به‌کلمه، توضیح بده — انگار داری به کسی می‌گویی که تازه `where` را یاد گرفته.
- قاعده‌ی یتیم دقیقاً چه چیزی را ممنوع می‌کند؟ چرا وجود دارد؟
- newtype در ۱.۵.۲ چه مشکلی را حل کرد، و امروز چه مشکلِ دیگری را حل کرد؟

---

## بیشتر

- [کتابِ Rust — استفاده از ابرصفت‌ها](https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#using-supertraits-to-require-one-traits-functionality-within-another-trait) — همین موضوع، از زبانِ رسمی.
- [مرجعِ Rust — قواعدِ یتیم](https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules) — تعریفِ دقیق و رسمیِ قاعده، همان‌جایی که پیامِ `E0117` هم ارجاعش می‌دهد.
- [`std::convert::Into`](https://doc.rust-lang.org/std/convert/trait.Into.html) — همان‌جا که پیاده‌سازیِ فراگیرِ امروز تعریف شده.
- [Rust API Guidelines — نوع‌های newtype](https://rust-lang.github.io/api-guidelines/type-safety.html#newtypes-provide-static-distinctions-c-newtype) — همان راهنمایی که ۱.۵.۲ معرفی‌اش کرد، برایِ کاربردِ ایمنیِ‌نوعِ newtype. راهِ فرارِ قاعده‌ی یتیم که این درس اضافه می‌کند، کارِ دومی است که همین الگو در آن هم خوب است، نه چیزی که خودِ آن صفحه دربارهاش حرف بزند.
