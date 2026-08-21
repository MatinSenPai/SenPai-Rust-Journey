# ۱.۵.۳ — شمارشی‌ها به‌عنوان داده

## در یک نگاه

بعد از این درس می‌توانی:

- یک `enum` تعریف کنی که هر گونه‌اش داده‌ی متفاوتی حمل می‌کند — و هر سه شکلِ گونه را بنویسی و بسازی.
- حالتِ نامعتبر را طوری مدل کنی که اصلاً *نوشتنی* نباشد، به‌جای اینکه در زمانِ اجرا بررسی‌اش کنی.
- بگویی `Option<T>` و `Result<T, E>` واقعاً چه هستند، و اندازه‌ی یک شمارشی را از روی گونه‌هایش پیش‌بینی کنی.

**زمان:** حدود ۵۰ دقیقه · **پیش‌نیاز:**
[۱.۵.۲ — ساختارهای تاپلی و الگوی newtype](../02-tuple-structs-and-newtype/README.fa.md) ·
[۱.۲.۱ — پشته و هیپ](../../02-ownership-and-memory/01-stack-and-heap/README.fa.md)

---

## چرا اهمیت دارد

در جنگو یک مدل می‌نویسی و وضعیتش را در یک `CharField` نگه می‌داری، و بعد ستون‌هایی اضافه می‌کنی که فقط برای *بعضی* از آن وضعیت‌ها معنا دارند:

```text
status  = "planned" | "watching" | "rated" | "dropped"
episode = null
score   = null
reason  = ""
```

هیچ چیزی جلوی `status="planned"` را با `score=9` نمی‌گیرد. جلوی `"wathcing"` را هم نمی‌گیرد. پس هر جا این رکورد را می‌خوانی باید دوباره بپرسی «الان کدام ستون‌ها معتبرند؟» — و روزی که یکی از این پرسش‌ها را جا بیندازی، باگ متولد می‌شود.

`enum` در Rust همان مسئله را از جای دیگری می‌گیرد: به‌جای اینکه ترکیبِ غلط را *تشخیص بدهد*، کاری می‌کند که ترکیبِ غلط اصلاً ساختنی نباشد.

و این با `enum` در C یا Java یکی نیست. آنجا شمارشی فقط فهرستی از نام‌های شماره‌دار است. در Rust **هر گونه می‌تواند داده‌ی خودش را حمل کند** — و همین یک تفاوت، شمارشی را از یک ابزارِ حاشیه‌ای به اصلی‌ترین ابزارِ مدل‌سازیِ زبان تبدیل می‌کند.

---

## مفهوم

### شمارشی یعنی «دقیقاً یکی از این‌ها»

```rust
#[derive(Debug)]
enum Medium {
    Anime,
    Manga,
    Webtoon,
}

let medium = Medium::Manga;
println!("{medium:?}");
```

```text
Manga
```

`Medium` یک نوعِ تازه است، مثل `struct` در ۱.۵.۱ — با یک تفاوتِ اساسی: یک `struct` همه‌ی فیلدهایش را *با هم* دارد، ولی یک `Medium` دقیقاً یکی از آن سه است. نه هیچ‌کدام، نه دوتا، نه یک چهارمی.

این ساده‌ترین شکلِ **شمارشی (enum)** است، و به هر یک از آن سه نام یک **گونه (variant)** می‌گوییم. گونه‌ای که هیچ داده‌ای حمل نمی‌کند **گونه‌ی واحد (unit variant)** نام دارد. تا همین‌جا، این همان چیزی است که یک برنامه‌نویسِ C از قبل می‌شناسد.

دقت کن که گونه را همیشه از راهِ نوعش صدا می‌زنی: `Medium::Manga`، نه `Manga`. نامِ شمارشی بخشی از مسیر است.

### یک گونه می‌تواند داده حمل کند

```rust
#[derive(Debug)]
enum Episode {
    Numbered(u32),
    Special(String),
}

println!("{:?}", Episode::Numbered(12));
println!("{:?}", Episode::Special(String::from("OVA")));
```

```text
Numbered(12)
Special("OVA")
```

اینجا شمارشیِ Rust از شمارشیِ C جدا می‌شود. `Numbered` یک عدد حمل می‌کند و `Special` یک رشته — **دو نوعِ کاملاً متفاوت، داخلِ یک نوع**. هر دو خطِ بالا یک `Episode` ساختند.

به این شکل **گونه‌ی تاپلی (tuple variant)** می‌گوییم، چون داده‌اش مثل یک تاپلِ بی‌نام است — همان چیزی که در ساختارهای تاپلیِ درسِ قبل دیدی.

و یک نکته‌ی ظریف که بعداً به کارت می‌آید: `Episode::Numbered` خودش یک تابع است که یک `u32` می‌گیرد و یک `Episode` می‌دهد. برای همین `Episode::Numbered(12)` شبیهِ صدا زدنِ تابع به نظر می‌رسد — چون هست.

### یک گونه می‌تواند داده‌ی نام‌دار حمل کند

```rust
#[derive(Debug)]
enum Entry {
    Planned,
    Watching(u32),
    Rated { score: u8 },
    Dropped { episode: u32, reason: String },
}

println!("{:?}", Entry::Rated { score: 9 });
```

```text
Rated { score: 9 }
```

`Rated` و `Dropped` فیلدِ نام‌دار دارند، درست مثل بدنه‌ی یک `struct`. به این‌ها **گونه‌ی ساختاری (struct variant)** می‌گوییم، و وقتی داده بیش از یک تکه است — یا وقتی نامِ فیلد خودش نصفِ توضیح است — از تاپل خواناتر است.

ولی مهم‌تر از هر سه اسم، این است: **هر گونه شکلِ خودش را انتخاب می‌کند.** `Entry` بالا هم گونه‌ی واحد دارد، هم تاپلی، هم ساختاری — و هر چهار مقدار از یک نوع‌اند:

```rust
let library = vec![
    Entry::Planned,
    Entry::Watching(7),
    Entry::Rated { score: 9 },
];
println!("{library:?}");
```

```text
[Planned, Watching(7), Rated { score: 9 }]
```

یک `Vec<Entry>` هر سه را کنارِ هم نگه می‌دارد، چون هر سه یک نوع دارند. تابعی که `Entry` می‌گیرد هر چهار حالت را می‌پذیرد، و **شکلِ پنجمی وجود ندارد که بشود به آن داد**. به این خانواده از نوع‌ها **نوع جمعی (sum type)** می‌گویند: مجموعه‌ی مقدارهای ممکنش، جمعِ مقدارهای گونه‌هایش است.

### حالتِ نامعتبر را نانوشتنی کن

حالا همان مدلِ جنگو را در Rust بنویس، ولی بدونِ شمارشی:

```rust
#[derive(Debug)]
struct LooseEntry {
    status: String,
    episode: u32,
    score: u8,
    reason: String,
}
```

و یک مقدارِ بی‌معنا بساز:

```rust
let nonsense = LooseEntry {
    status: String::from("planned"),
    episode: 40,
    score: 9,
    reason: String::from("too slow"),
};
println!("{nonsense:?}");
```

```text
LooseEntry { status: "planned", episode: 40, score: 9, reason: "too slow" }
```

«برنامه‌ریزی‌شده»، ولی روی قسمتِ ۴۰، با نمره‌ی ۹، و با دلیلی برای رها کردنش. سه وضعیت هم‌زمان. کامپایلر کوچک‌ترین اعتراضی نکرد — و نباید هم می‌کرد، چون هر `String` یک `String` معتبر است و هر `u8` یک `u8` معتبر.

با `Entry` هیچ خطی وجود ندارد که چنین مقداری بسازد. نه اینکه گرفته می‌شود؛ **بیان‌شدنی نیست**. `Planned` جایی برای نمره ندارد و `Rated` جایی برای دلیل.

> **قاعده‌ی عملی:** اگر برای فهمیدنِ اینکه کدام فیلدها معتبرند مجبوری به یک فیلدِ دیگر نگاه کنی، آن فیلد دارد نقشِ شناسه‌ی گونه را بازی می‌کند. یعنی همان‌جا یک شمارشی پنهان شده.

**پُلِ پایتون/جنگو، و جایی که پاره می‌شود:** `choices` در جنگو دقیقاً همین کار را تا نیمه انجام می‌دهد — فهرستِ وضعیت‌های مجاز را می‌بندد. ولی `choices` فقط برچسب را محدود می‌کند؛ ستون‌های `episode` و `score` بیرونِ آن می‌مانند و همچنان هر ترکیبی می‌توانند داشته باشند. شمارشیِ Rust برچسب و داده را با هم می‌بندد، و شباهت دقیقاً همین‌جا تمام می‌شود.

### شمارشی هم یک نوع است، پس `impl` می‌گیرد

هر چیزی که در ۱.۵.۱ درباره‌ی `impl` یاد گرفتی اینجا هم برقرار است. تابعِ وابسته (associated function)، بدونِ `self`:

```rust
impl Entry {
    fn new() -> Self {
        Entry::Planned
    }

    fn start(episode: u32) -> Self {
        Entry::Watching(episode)
    }
}
```

و متدهایی که یک پرسش می‌پرسند. برای پرسیدنِ «این مقدار کدام گونه است؟» ماکروی `matches!` کافی است:

```rust
impl Entry {
    fn is_watching(&self) -> bool {
        matches!(self, Entry::Watching(_))
    }

    fn is_done(&self) -> bool {
        matches!(self, Entry::Rated { .. } | Entry::Dropped { .. })
    }

    fn is_favourite(&self) -> bool {
        matches!(self, Entry::Rated { score } if *score >= 8)
    }
}
```

```text
Planned
    watching: false  done: false  favourite: false
Watching(7)
    watching: true  done: false  favourite: false
Rated { score: 9 }
    watching: false  done: true  favourite: true
Rated { score: 4 }
    watching: false  done: true  favourite: false
Dropped { episode: 3, reason: "too slow" }
    watching: false  done: true  favourite: false
```

سه الگویی که آنجا دیدی: `_` یعنی «هر داده‌ای»، `..` یعنی «و هر فیلدِ نام‌دارِ دیگری»، `|` یعنی «یا». و آن `if` انتهایی به *داده‌ای* که گونه حمل می‌کند نگاه می‌کند، نه فقط به اینکه کدام گونه است — برای همین `score: 9` مورد علاقه است و `score: 4` نیست.

`matches!` عمداً کم‌توان است: فقط `true` یا `false` می‌دهد و داده را به تو نمی‌دهد. ابزارِ واقعیِ باز کردنِ یک شمارشی `match` است، و آن کلِ درسِ [۱.۵.۴](../04-match-in-depth/README.fa.md) است. تا آنجا شمارشی را می‌سازی و شکلش را می‌پرسی؛ از آنجا به بعد بازش می‌کنی.

### `Option` و `Result` هم فقط شمارشی‌اند

از ۱.۱.۲ به بعد مدام چیزی مثل `Some(3)` و `None` دیده‌ای و هر بار گفته‌ایم «بعداً». الان وقتش است، چون ابزارش را داری. این تعریفِ کاملِ کتابخانه‌ی استاندارد است:

```rust
enum Option<T> {
    None,
    Some(T),
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

همین. هیچ چیزِ ویژه‌ای در زبان برایشان ساخته نشده؛ همان دو چیزی‌اند که خودت همین حالا می‌توانستی بنویسی: یک گونه‌ی واحد و یک گونه‌ی تاپلی. آن `<T>` یعنی «هر نوعی که خواستی» و اسمش **جنریک (generics)** است — کاملش در [فاز ۲](../../../phase2-intermediate/03-generics-and-traits/01-generic-functions-and-structs/README.fa.md) می‌آید؛ اینجا همین‌قدر بس است که `Option<i32>` یعنی «یا هیچ، یا یک `i32`».

و چون این‌ها گونه‌اند، می‌شود کاملشان را نوشت:

```rust
println!("{:?}", Option::Some(9_u8));
println!("{:?}", Option::<u8>::None);
println!("{:?}", Result::<u8, String>::Ok(9));
```

```text
Some(9)
None
Ok(9)
```

`Some` و `None` و `Ok` و `Err` را بدونِ پیشوند می‌نویسی فقط چون prelude آن‌ها را برایت وارد کرده — نه چون کلمه‌ی کلیدیِ زبان‌اند. و برای همین است که `.last()` روی یک وکتور `Option` می‌دهد: «یک نگاه به آخرین عنصر اگر عنصری باشد» و «شکلِ دیگری اگر نباشد»، دو گونه‌ی یک شمارشی‌اند. کار کردن با آن‌ها در [۱.۶.۱](../../06-absence-and-failure/01-option-and-null-safety/README.fa.md) و [۱.۶.۳](../../06-absence-and-failure/03-result-and-question-mark/README.fa.md) است.

### یک شمارشی چقدر جا می‌گیرد

یک مقدارِ شمارشی باید دو چیز را نگه دارد: داده‌ی گونه، و اینکه *کدام* گونه است. آن دومی یک عددِ کوچک است به نامِ **شناسه‌ی گونه (discriminant)**.

```senpai-visual
{"kind":"concept","labels":["شناسه‌ی گونه","بزرگ‌ترین گونه","لایی هم‌ترازی","size_of"]}
```

```rust
println!("Medium: {} bytes", size_of::<Medium>());
println!("Small:  {} bytes", size_of::<Small>()); // A(u8) | B(u8)
println!("Wide:   {} bytes", size_of::<Wide>()); // Tiny(u8) | Big(u64)
println!("Entry:  {} bytes", size_of::<Entry>());
```

```text
Medium:                1 bytes
Small:                 2 bytes
Wide:                  16 bytes
Entry:                 32 bytes
```

قاعده را از همین چهار عدد بخوان: **یک شمارشی به‌اندازه‌ی بزرگ‌ترین گونه‌اش است، به‌علاوه‌ی شناسه‌ی گونه، گرد شده به هم‌ترازی (alignment).**

- `Medium` داده ندارد، پس فقط شناسه می‌ماند: ۱ بایت.
- `Small` یک بایت داده دارد و یک بایت شناسه: ۲ بایت.
- `Wide` بزرگ‌ترین گونه‌اش یک `u64` است (۸ بایت، با هم‌ترازیِ ۸). شناسه یک بایت است ولی کل باید تا مضربِ ۸ گرد شود: ۱۶ بایت. `Wide::Tiny(1)` هم ۱۶ بایت می‌گیرد، چون اندازه خاصیتِ *نوع* است نه مقدار.
- `Entry` بزرگ‌ترین گونه‌اش `Dropped` است: یک `String` (۲۴ بایت) به‌علاوه‌ی یک `u32` (۴ بایت) یعنی ۲۸، و هم‌ترازیِ ۸ آن را به ۳۲ می‌رساند. شناسه در همان ۴ بایتِ لاییِ بی‌مصرف جا شد و مجانی درآمد.

پیامدِ عملی: یک `Vec<Entry>` با هزار عنصر، هزار برابرِ اندازه‌ی بزرگ‌ترین گونه جا می‌گیرد — حتی اگر همه‌شان `Planned` باشند. اگر یک گونه خیلی از بقیه بزرگ‌تر شد، همان‌جاست که `Box` به کار می‌آید ([فاز ۲](../../../phase2-intermediate/05-smart-pointers/01-box-and-heap-allocation/README.fa.md)).

### آن یکی که از قاعده پیروی نمی‌کند

```rust
println!("i32:              {} bytes", size_of::<i32>());
println!("Option<i32>:      {} bytes", size_of::<Option<i32>>());
println!("bool:             {} bytes", size_of::<bool>());
println!("Option<bool>:     {} bytes", size_of::<Option<bool>>());
println!("Box<i32>:         {} bytes", size_of::<Box<i32>>());
println!("Option<Box<i32>>: {} bytes", size_of::<Option<Box<i32>>>());
```

```text
i32:                   4 bytes
Option<i32>:           8 bytes
bool:                  1 bytes
Option<bool>:          1 bytes
Box<i32>:              8 bytes
Option<Box<i32>>:      8 bytes
```

`Option<i32>` طبقِ قاعده رفتار کرد: ۴ بایت داده، به‌علاوه‌ی شناسه، گرد شده به ۸.

دو تای دیگر نه. `Option<bool>` و `bool` هر دو یک بایت‌اند، و `Option<Box<i32>>` و `Box<i32>` هر دو هشت بایت. **پوشش مجانی درآمد.**

دلیلش این است که کامپایلر می‌داند کدام الگوهای بیتی برای آن نوع *ناممکن*‌اند و یکی‌شان را برای `None` قرض می‌گیرد. یک `bool` فقط `0` و `1` است، پس ۲۵۴ الگوی بلااستفاده دارد و `None` یکی از آن‌ها می‌شود. یک `Box` هرگز اشاره‌گرِ تهی نیست، پس `None` می‌شود همان صفر. به این کار **بهینه‌سازیِ جای‌خالی (niche optimisation)** می‌گویند، و حالتِ خاصِ اشاره‌گرش تاریخاً «بهینه‌سازیِ اشاره‌گرِ تهی» نامیده می‌شود.

نتیجه‌ای که باید با خودت ببری: در Rust ایمنی در برابرِ تهی **هزینه‌ی حافظه‌ای ندارد**. یک `Option<Box<T>>` دقیقاً به‌اندازه‌ی یک اشاره‌گرِ خامِ nullable در C جا می‌گیرد، با این تفاوت که کامپایلر نمی‌گذارد بدونِ بررسی از آن استفاده کنی.

---

## دست‌به‌کد

```sh
cargo run -p p1-05-03-enums-as-data --example 01-three-shapes
cargo run -p p1-05-03-enums-as-data --example 02-invalid-states
cargo run -p p1-05-03-enums-as-data --example 03-methods-on-enums
cargo run -p p1-05-03-enums-as-data --example 04-option-is-an-enum
cargo run -p p1-05-03-enums-as-data --example 05-what-an-enum-costs
```

بعد چهار تای خراب:

```sh
cargo run -p p1-05-03-enums-as-data --example 06-wrong-variant-shape --features broken
cargo run -p p1-05-03-enums-as-data --example 07-no-such-field --features broken
cargo run -p p1-05-03-enums-as-data --example 08-variant-is-not-a-type --features broken
cargo run -p p1-05-03-enums-as-data --example 09-cannot-compare --features broken
```

بعد این‌ها را امتحان کن:

۱. در `01-three-shapes`، یک گونه‌ی پنجم به `Entry` اضافه کن که یک `String` حمل کند، و بسازش. چیزی خراب شد؟
۲. در `05-what-an-enum-costs`، `Big(u64)` را به `Big(u32)` تغییر بده. اندازه‌ی `Wide` چه شد و چرا؟
۳. باز در `05-what-an-enum-costs`، اندازه‌ی `Option<Option<bool>>` را چاپ کن. اول حدس بزن، بعد اجرا کن.

---

## خطاهایی که خواهی دید

### `E0533` — ساختنِ گونه در شکلِ اشتباه

```text
error[E0533]: expected value, found struct variant `Entry::Rated`
  --> examples\06-wrong-variant-shape.rs:17:22
   |
17 |     println!("{:?}", Entry::Rated(9));
   |                      ^^^^^^^^^^^^ not a value
   |
help: you might have meant to create a new value of the struct
   |
17 -     println!("{:?}", Entry::Rated(9));
17 +     println!("{:?}", Entry::Rated { score: /* value */ });
   |
```

**کامپایلر به چه اعتراض دارد:** `Rated` یک گونه‌ی ساختاری است، پس `Entry::Rated` به‌تنهایی یک مقدار نیست — یک قالب است که باید با آکولاد و نامِ فیلد پُر شود. `Entry::Rated(9)` دارد آن را مثل یک تابع صدا می‌زند، و چنین تابعی وجود ندارد.

**راه‌حل:** `Entry::Rated { score: 9 }`.

**چرا این راه‌حل است:** شکلی که گونه را با آن *می‌سازی* دقیقاً همان شکلی است که در تعریف نوشته‌ای. گونه‌ی تاپلی با پرانتز، گونه‌ی ساختاری با آکولاد، گونه‌ی واحد با هیچ. اشتباهِ وارونه‌اش هم خطای خودش را دارد: `Entry::Watching { episode: 7 }` می‌شود `E0559` با پیامِ «`Entry::Watching` is a tuple variant, use the appropriate syntax».

### `E0609` — چنین فیلدی روی این نوع نیست

```text
error[E0609]: no field `score` on type `Entry`
  --> examples\07-no-such-field.rs:19:33
   |
19 |     println!("score: {}", entry.score);
   |                                 ^^^^^ unknown field
```

**کامپایلر به چه اعتراض دارد:** `entry` از نوعِ `Entry` است و `Entry` فیلد ندارد. `score` فیلدِ **یکی از گونه‌ها**ست. کامپایلر فقط از روی نوعِ متغیر نمی‌داند این مقدار کدام گونه است — و اگر `Planned` باشد، `score`ای در کار نیست که خوانده شود.

**راه‌حل:** فعلاً هیچ. این خطا دقیقاً همان دری است که درسِ [۱.۵.۴](../04-match-in-depth/README.fa.md) باز می‌کند: `match` اول می‌پرسد کدام گونه است و بعد، *داخلِ همان شاخه*، داده‌اش را در اختیارت می‌گذارد.

**چرا این راه‌حل است:** این همان معامله‌ی کلِ درسِ امروز است. مدلِ `LooseEntry` اجازه می‌داد هر لحظه `score` را بخوانی، به این قیمت که گاهی بی‌معنا بود. شمارشی خواندنِ بی‌قید را ممنوع می‌کند تا هر خواندنی معتبر باشد. و اگر همین حالا فقط می‌خواهی *بدانی* کدام گونه است، `matches!` کافی است.

### `E0573` — گونه یک نوع نیست

```text
error[E0573]: expected type, found variant `Entry::Rated`
  --> examples\08-variant-is-not-a-type.rs:14:16
   |
14 | fn show(entry: Entry::Rated) {
   |                ^^^^^^^^^^^^ not a type
   |
help: try using the variant's enum
   |
14 - fn show(entry: Entry::Rated) {
14 + fn show(entry: crate::Entry) {
   |
```

**کامپایلر به چه اعتراض دارد:** نوع، `Entry` است. `Entry::Rated` یکی از *مقدارهای* آن نوع است. جای نوع نمی‌شود مقدار گذاشت — مثل اینکه بنویسی `fn f(x: 5)`.

**راه‌حل:** `fn show(entry: Entry)`، همان‌طور که خودِ کامپایلر پیشنهاد داد.

**چرا این راه‌حل است:** این اشتباه معمولاً از یک آرزوی درست می‌آید: «تابعی می‌خواهم که فقط `Rated` قبول کند.» Rust این را نمی‌دهد و عمداً نمی‌دهد. اگر واقعاً به یک نوعِ باریک‌تر احتیاج داری، جدا بسازش — مثلاً یک `struct Rated { score: u8 }` — و گونه را طوری تعریف کن که آن را حمل کند: `Rated(Rated)`. آن‌وقت هم شمارشی را داری و هم نوعِ باریک را.

### `E0369` — مقایسه بدونِ `PartialEq`

```text
error[E0369]: binary operation `==` cannot be applied to type `Medium`
  --> examples\09-cannot-compare.rs:19:15
   |
19 |     if chosen == Medium::Manga {
   |        ------ ^^ ------------- Medium
   |        |
   |        Medium
   |
note: an implementation of `PartialEq` might be missing for `Medium`
  --> examples\09-cannot-compare.rs:8:1
   |
 8 | enum Medium {
   | ^^^^^^^^^^^ must implement `PartialEq`
help: consider annotating `Medium` with `#[derive(PartialEq)]`
   |
 8 + #[derive(PartialEq)]
 9 | enum Medium {
   |
```

**کامپایلر به چه اعتراض دارد:** `#[derive(Debug)]` فقط توانِ چاپ شدن را داد. برابری توانِ جداگانه‌ای است و نوعِ تو آن را ندارد، پس `==` روی آن معنایی ندارد.

**راه‌حل:** `#[derive(Debug, PartialEq)]`.

**چرا این راه‌حل است:** همان `derive` درسِ ۱.۲.۳، این بار روی توانایی دیگری. `PartialEq` روی یک شمارشی معنای «همان گونه، و داده‌های برابر» را می‌سازد؛ پس `Rated { score: 9 } == Rated { score: 9 }` درست است ولی `== Rated { score: 8 }` نه. اگر فقط می‌خواهی بدانی گونه‌شان یکی است و داده برایت مهم نیست، `matches!` جواب می‌دهد و اصلاً به `PartialEq` احتیاج ندارد.

---

## تمرین

### گرم‌کردن

<details>
<summary>تفاوتِ یک <code>struct</code> و یک <code>enum</code> در یک جمله؟</summary>

`struct` همه‌ی فیلدهایش را با هم دارد؛ `enum` دقیقاً یکی از گونه‌هایش است.

</details>

<details>
<summary>سه شکلِ گونه کدام‌اند و هرکدام چه‌طور ساخته می‌شوند؟</summary>

واحد (`Planned`، بدونِ چیزی)، تاپلی (`Watching(7)`، با پرانتز)، ساختاری (`Rated { score: 9 }`، با آکولاد و نامِ فیلد).

</details>

<details>
<summary>چرا <code>let x: Entry::Rated = ...</code> کامپایل نمی‌شود؟</summary>

چون `Entry::Rated` یک نوع نیست، یکی از مقدارهای نوعِ `Entry` است. `E0573`.

</details>

<details>
<summary><code>Option&lt;T&gt;</code> دقیقاً چیست؟</summary>

یک شمارشی با دو گونه: `None` که واحد است و `Some(T)` که تاپلی. هیچ چیزِ ویژه‌ای در زبان ندارد و خودت هم می‌توانستی بنویسی‌اش.

</details>

<details>
<summary>یک شمارشی با گونه‌های <code>A(u8)</code> و <code>B(u64)</code> چند بایت است؟ چرا؟</summary>

۱۶. بزرگ‌ترین گونه ۸ بایت است با هم‌ترازیِ ۸، و شناسه‌ی گونه کل را به مضربِ بعدیِ ۸ می‌رساند.

</details>

<details>
<summary>چرا <code>size_of::&lt;Option&lt;Box&lt;i32&gt;&gt;&gt;()</code> با <code>size_of::&lt;Box&lt;i32&gt;&gt;()</code> برابر است؟</summary>

چون `Box` هرگز تهی نیست، پس الگوی بیتیِ صفر بلااستفاده است و کامپایلر همان را برای `None` برمی‌دارد. بهینه‌سازیِ جای‌خالی.

</details>

### تعمیر

هر چهار مثالِ خراب را درست کن:

۱. `examples/06-wrong-variant-shape.rs` — و بعد `Entry::Watching { episode: 7 }` را هم امتحان کن تا خطای وارونه‌اش را ببینی.
۲. `examples/07-no-such-field.rs` — بدونِ `match`، طوری بازنویسی‌اش کن که برنامه کامپایل شود و چیزی معنادار چاپ کند. (سرنخ: `matches!` و `{:?}`.)
۳. `examples/08-variant-is-not-a-type.rs` — با پیشنهادِ خودِ کامپایلر.
۴. `examples/09-cannot-compare.rs` — و بعد بگو اگر `Medium` یک گونه‌ی داده‌دار داشت، `PartialEq` چه چیزهایی را مقایسه می‌کرد.

### پیاده‌سازی

سه تابع و دو متد در `src/lib.rs`:

```sh
cargo test -p p1-05-03-enums-as-data
```

هیچ‌کدام به `match` احتیاج ندارند. سه تای اول فقط گونه‌ی درست را *می‌سازند*؛ دو تای آخر با `matches!` یک پرسش می‌پرسند — یکی درباره‌ی شکلِ گونه، یکی درباره‌ی داده‌ای که گونه حمل می‌کند.

### بساز

یک شمارشی برای دامنه‌ی خودت طراحی کن: **نتیجه‌ی یک کارِ پس‌زمینه** در یک سرویس.

دستِ‌کم باید یک گونه‌ی واحد، یک گونه‌ی تاپلی و یک گونه‌ی ساختاری داشته باشد — مثلاً «در صف»، «در حالِ اجرا با درصدِ پیشرفت»، «شکست‌خورده با کد و پیام». بعد:

۱. `#[derive(Debug)]` بگذار، از هر گونه یکی بساز و چاپ کن.
۲. قبل از اجرا حدس بزن `size_of` نوعت چند است، بعد چاپش کن. اگر اشتباه حدس زدی، بگو کدام گونه اندازه را تعیین کرد.
۳. یک متدِ `is_terminal(&self) -> bool` با `matches!` بنویس که بگوید این حالت پایانی است یا نه.
۴. یک پاراگراف بنویس: اگر همین را با یک `struct` و یک فیلدِ `status: String` مدل می‌کردی، دقیقاً کدام مقدارِ بی‌معنا ساختنی می‌شد؟

### چالش (اختیاری)

**بخشِ یک.** این‌ها را حدس بزن و بعد چاپ کن: `size_of::<Option<u8>>()`، `size_of::<Option<char>>()`، `size_of::<Option<&i32>>()`، `size_of::<Option<Option<bool>>>()`. کدام‌ها جای‌خالی گرفتند و کدام‌ها نه؟

**بخشِ دو.** یک شمارشیِ بازگشتی بنویس — فهرستی که هر حلقه‌اش حلقه‌ی بعدی را حمل می‌کند:

```rust
enum Chain {
    End,
    Link(u32, Chain),
}
```

خطای `E0072` را بخوان. کامپایلر خودش راه‌حل را نام می‌برد؛ پیاده‌اش کن و توضیح بده چرا اندازه با آن تغییر متناهی می‌شود. (این تو را به [فاز ۲ — `Box` و تخصیص روی هیپ](../../../phase2-intermediate/05-smart-pointers/01-box-and-heap-allocation/README.fa.md) می‌رساند.)

**بخشِ سه.** یک شمارشیِ بدونِ داده می‌تواند عددِ شناسه‌اش را خودش انتخاب کند و به عدد تبدیل شود:

```rust
#[derive(Debug)]
enum Rank {
    Bronze = 1,
    Silver = 2,
    Gold = 3,
}
```

`Rank::Silver as i32` را چاپ کن. حالا همین کار را با `Entry` امتحان کن و خطا را بخوان — چرا این فقط برای شمارشیِ بدونِ داده ممکن است؟

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به کارت می‌آید |
|---|---|---|
| شمارشی (enum) | نوعی که دقیقاً یکی از چند شکل است | مدل کردنِ حالت |
| گونه (variant) | یکی از آن شکل‌ها | `Entry::Rated` |
| گونه‌ی واحد | گونه‌ای بدونِ داده | `Planned` |
| گونه‌ی تاپلی | داده‌ی بی‌نام، با پرانتز | `Watching(7)` |
| گونه‌ی ساختاری | داده‌ی نام‌دار، با آکولاد | `Rated { score: 9 }` |
| نوع جمعی (sum type) | مقدارهای ممکن = جمعِ گونه‌ها | چرا حالتِ نامعتبر نانوشتنی است |
| `matches!` | «این کدام گونه است؟» به‌صورتِ `bool` | پرسشِ ساده، بدونِ `match` |
| شناسه‌ی گونه (discriminant) | عددی که می‌گوید کدام گونه | چرا شمارشی یک بایتِ اضافه دارد |
| بهینه‌سازیِ جای‌خالی | استفاده از الگوی بیتیِ ناممکن برای شناسه | `Option<Box<T>>` مجانی است |
| `E0533` | ساختنِ گونه در شکلِ اشتباه | پرانتز به‌جای آکولاد |
| `E0609` | فیلدِ گونه از روی خودِ نوع خوانده نمی‌شود | دری که `match` بازش می‌کند |
| `E0573` | گونه یک نوع نیست | امضای تابع |
| `E0369` | `==` بدونِ `PartialEq` | `#[derive(Debug, PartialEq)]` |

### الان می‌دانی

- یک شمارشی دقیقاً یکی از گونه‌هایش است، و کامپایلر می‌داند کدام.
- هر گونه شکلِ خودش را دارد: واحد، تاپلی یا ساختاری — همه در یک شمارشی.
- گونه با همان شکلی ساخته می‌شود که تعریف شده، و همیشه با پیشوندِ نامِ شمارشی.
- شمارشی یک نوع است، پس `impl` و `derive` می‌گیرد.
- اگر معنای یک فیلد به فیلدِ دیگری وابسته باشد، همان‌جا یک شمارشی پنهان شده.
- `Option<T>` و `Result<T, E>` شمارشی‌های معمولی‌اند، نه جادوی زبان.
- اندازه‌ی یک شمارشی = بزرگ‌ترین گونه + شناسه، گرد شده به هم‌ترازی.
- بهینه‌سازیِ جای‌خالی گاهی شناسه را کاملاً مجانی می‌کند.

### بعداً کامل‌تر می‌بینی

- **باز کردنِ یک شمارشی و برداشتنِ داده‌اش** — [۱.۵.۴ — `match` به‌طورِ کامل](../04-match-in-depth/README.fa.md)
- **وقتی فقط یک گونه برایت مهم است** — [۱.۵.۵ — `if let`، `while let`، `let else`](../05-if-let-while-let-let-else/README.fa.md)
- **`Option` و نبودِ null** — [۱.۶.۱ — `Option` و ایمنی در برابرِ null](../../06-absence-and-failure/01-option-and-null-safety/README.fa.md)
- **`Result` و عملگرِ `?`** — [۱.۶.۳ — `Result` و `?`](../../06-absence-and-failure/03-result-and-question-mark/README.fa.md)
- **آن `<T>` که در تعریفِ `Option` دیدی** — [فاز ۲ — جنریک‌ها](../../../phase2-intermediate/03-generics-and-traits/01-generic-functions-and-structs/README.fa.md)
- **گونه‌ی بازگشتی و شمارشیِ بیش‌ازحد بزرگ** — [فاز ۲ — `Box` و تخصیص روی هیپ](../../../phase2-intermediate/05-smart-pointers/01-box-and-heap-allocation/README.fa.md)
- **الگوهای پیشرفته‌تر: نگهبان، اتصال، الگوی تودرتو** — [فاز ۲ — عمقِ تطبیقِ الگو](../../../phase2-intermediate/08-rust-toolbox/01-pattern-matching-depth/README.fa.md)

### می‌توانی توضیح بدهی؟

- تفاوتِ `enum` در Rust با `enum` در C را در یک جمله بگو.
- سه شکلِ گونه را نام ببر و برای هرکدام یک نمونه بساز.
- چرا `entry.score` کامپایل نمی‌شود، در حالی که `Entry::Rated` واقعاً یک `score` دارد؟
- `Option<T>` را از حفظ تعریف کن.
- اندازه‌ی یک شمارشی از کجا می‌آید؟
- یک جا از کدِ خودت را نام ببر که یک فیلدِ رشته‌ای دارد نقشِ شناسه‌ی گونه را بازی می‌کند.

---

## بیشتر

- [کتابِ Rust — فصلِ ۶: شمارشی‌ها](https://doc.rust-lang.org/book/ch06-00-enums.html) — همین زمین، رسمی.
- [`std::option::Option`](https://doc.rust-lang.org/std/option/enum.Option.html) — روی «Source» کلیک کن و تعریفِ واقعی‌اش را ببین. همان دو خط است.
- [مرجعِ Rust — چیدمانِ نوع‌ها](https://doc.rust-lang.org/reference/type-layout.html#enum-layout) — قاعده‌ی شناسه و جای‌خالی، به زبانِ رسمی.
- [Making Illegal States Unrepresentable](https://blog.janestreet.com/effective-ml-video/) — همان ایده‌ی «حالتِ نامعتبر را نانوشتنی کن»، از دنیای OCaml؛ جایی که این جمله از آن آمد.
