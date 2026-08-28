# ۲.۳.۴ — مشتق‌های استاندارد، دستی پیاده‌سازی‌شده

## در یک نگاه

بعد از این درس می‌توانی:

- توضیح بدهی `#[derive(Debug)]` دقیقاً چه کدی تولید می‌کند — با نوشتنِ همان چیز یک‌بار با `f.debug_struct(...).field(...).finish()` و یک‌بار با `write!` خام — و بگویی چرا `#[derive(Display)]`ای اصلاً وجود ندارد.
- برایِ یک نوعِ خودت، `Default`، `PartialEq`/`Eq` و `PartialOrd`/`Ord` را هم با `#[derive]` بسازی هم با دست بنویسی، و دقیقاً بگویی این دو مسیر کِی به یک نتیجه می‌رسند و کِی نه.
- توضیح بدهی چرا `f64` هیچ‌وقت `Eq` یا `Ord` نمی‌شود، و یک `Hash` دستی بنویسی که با `Eq`ات هم‌خوان بماند — وگرنه `HashMap`/`HashSet` بی‌سروصدا خراب می‌شود.

**زمان:** حدود ۱۰۰ دقیقه · **پیش‌نیاز:**
[۲.۳.۳ — `From`، `Into`، `TryFrom`، `TryInto`](../03-from-into-tryfrom/README.fa.md)

---

## چرا اهمیت دارد

سه بار همین فاز، دقیقاً همین دیوار جلوی راهت سبز شده — و هر سه بار درس همان یک جمله را گفته و رد شده.

در [۲.۱.۱](../../01-collections/01-vec-depth/README.fa.md)، `ratings.sort()` روی یک `Vec<f64>` اصلاً کامپایل نشد؛ کامپایلر گفت `f64` صفتِ `Ord` را ندارد (`E0277`). در [۲.۱.۳](../../01-collections/03-btreemap-hashset-vecdeque/README.fa.md)، همان داستان یک پله جدی‌تر: `BinaryHeap<f64>` حتی از زمینِ کامپایل بلند نشد (`E0599`) — چون یک صفِ اولویت باید بتواند هر دو عضو را مقایسه کند، و `f64` این قول را نمی‌دهد. در [۲.۲.۳](../../02-iterators-and-closures/03-consuming-and-collecting/README.fa.md)، نوبتِ `.max()` روی یک ایتریتورِ `f64` بود — باز همان `Ord`، باز همان `E0277`. و در [۲.۱.۲](../../01-collections/02-hashmap-in-depth/README.fa.md)، یک `struct Coord` که فقط `#[derive(Debug)]` داشت، روی `HashMap::insert` به یک دیوارِ خواهر خورد: `Coord: Eq` و `Coord: Hash` برآورده نشده بودند (`E0599`) — و رفعش هم شد یک `#[derive(Eq, Hash, PartialEq)]` که خودِ کامپایلر پیشنهاد داد، بدونِ اینکه واقعاً بدانی آن سه کلمه با هم چه قولی می‌دهند.

هر چهار بار، جوابِ درس یک تعویقِ صادقانه بود، نه یک توضیح: «خودِ این را ۲.۳.۴ کامل جا می‌اندازد.» امروز همان روزی است که آن تعویق تمام می‌شود — و داستانِ `NaN`ای که `Ord` را می‌شکند، این‌بار نه یک واقعیتِ تازه، بلکه یک جواب برایِ سؤالی است که چهار بار پرسیده‌ای.

یک نخِ دومِ درس هم هست، آرام‌تر ولی هرروزه‌تر. از همان [۱.۵.۱](../../../phase1-fundamentals/05-your-own-types/01-structs-and-methods/README.fa.md) به بعد، بالایِ تقریباً هر ساختاری که نوشته‌ای یک `#[derive(Debug)]` گذاشته‌ای، بدونِ اینکه یک‌بار بپرسی آن یک خط دقیقاً چه کدی پشتِ‌صحنه می‌نویسد. اگر کامپایلر آن پیاده‌سازیِ بدیهی را رایگان می‌دهد، چرا باید بلد باشی دستی بنویسی‌اش؟ چون بعضی‌هایشان اصلاً بدیهی نیستند (`Display` هیچ‌وقت `derive` نمی‌شود؛ می‌فهمی چرا)، چون گاهی رفتارِ متفاوتی از پیش‌فرضِ `derive` می‌خواهی (`Ord` بر اساسِ فقط یک فیلد، نه همه‌شان)، و چون وقتی یک `HashSet` بی‌سروصدا تکراری نگه می‌دارد، تنها راهِ فهمیدنِ چرا این است که بدانی آن `derive` واقعاً چه قولی داده بود.

---

## مفهوم

### همان `#[derive(Debug)]`ای که صدها بار زده‌ای — امروز می‌نویسیمش

یک نوعِ ساده برایِ کارِ امروز:

```rust
#[derive(Debug)]
struct Anime {
    title: String,
    episodes: u32,
    score: u8,
}

let a = Anime { title: "Frieren".to_string(), episodes: 28, score: 96 };
println!("{a:?}");
```

```text
Anime { title: "Frieren", episodes: 28, score: 96 }
```

این را صدها بار دیده‌ای. چیزی که ندیده‌ای این است که `#[derive(Debug)]` دقیقاً همین را، به‌عنوانِ یک `impl fmt::Debug`، خودش می‌نویسد. با دستِ خودت، با ابزارِ ارگونومیکی که خودِ `derive` هم زیرِ پوستش استفاده می‌کند — `f.debug_struct(نام).field(...).finish()` — همان چیز را دوباره می‌سازی:

```rust
use std::fmt;

impl fmt::Debug for Anime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Anime")
            .field("title", &self.title)
            .field("episodes", &self.episodes)
            .field("score", &self.score)
            .finish()
    }
}
```

`f: &mut fmt::Formatter` بافری است که متنِ خروجی رویش نوشته می‌شود؛ `fmt::Result` یعنی «نوشتن یا موفق شد یا نشد» — همان `Result`ی که از [۱.۶.۳](../../../phase1-fundamentals/06-absence-and-failure/03-result-and-question-mark/README.fa.md) می‌شناسی. `debug_struct("Anime")` می‌گوید «این یک ساختار به اسمِ Anime است»، هر `.field(...)` یک فیلد اضافه می‌کند، و `.finish()` قالب را می‌بندد. نتیجه، خط‌به‌خط، همانی است که `derive` تولید کرد.

اما این تنها راه نیست. می‌شود همان متن را با یک `write!` خام هم ساخت — بدونِ هیچ builderای:

```rust
impl fmt::Debug for Anime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Anime {{ title: {:?}, episodes: {:?}, score: {:?} }}",
            self.title, self.episodes, self.score
        )
    }
}
```

(دو آکولادِ پیاپی — `{{` و `}}` — یعنی «یک آکولادِ واقعی چاپ کن»، نه یک جایگزین. این‌جا لازمش داری چون `{` و `}`ِ خودِ ساختار، بدونش، مثلِ جایگزین‌هایِ دیگرِ `format!` به‌نظر می‌رسیدند.)

هر دو، برایِ `{:?}` جوابِ یکسان می‌دهند. فرق جایی خودش را نشان می‌دهد که فرمِ جایگزین را بخواهی، `{:#?}`:

```sh
cargo run -p p2-03-04-standard-derives-by-hand --example 01-debug-two-ways
```

```text
compact — derived: Derived { title: "Frieren", episodes: 28, score: 96 }
compact — builder: Builder { title: "Frieren", episodes: 28, score: 96 }
compact — raw:     Raw { title: "Frieren", episodes: 28, score: 96 }

alternate — derived:
Derived {
    title: "Frieren",
    episodes: 28,
    score: 96,
}
alternate — builder:
Builder {
    title: "Frieren",
    episodes: 28,
    score: 96,
}
alternate — raw:
Raw { title: "Frieren", episodes: 28, score: 96 }
```

نسخه‌ی `builder` زیرِ `{:#?}` خودش را تورفته و چندخطی می‌کند. نسخه‌ی `raw` نه — همان یک خطِ فشرده، حتی وقتی صریحاً فرمِ جایگزین را خواسته‌ای. `debug_struct` این پشتیبانی را رایگان می‌دهد، چون خودش می‌داند `f.alternate()` صدا زده شده یا نه؛ یک `write!` دستی این را نمی‌داند مگر خودت صریح چکش کنی. این دقیقاً همان چیزی است که builder را، برایِ یک `struct` واقعی، انتخابِ درست می‌کند — نه فقط کمتر تایپ‌کردن.

### `Display` — چرا هیچ `#[derive]`ی برایش نیست

`{:?}` سؤالِ «شکلِ برایِ برنامه‌نویس چیست؟» را می‌پرسد — سؤالی که [۱.۴.۳](../../../phase1-fundamentals/04-text-and-strings/03-building-and-transforming-strings/README.fa.md) معرفی کرد. `{}` سؤالِ دیگری می‌پرسد: «شکلِ برایِ کاربر چیست؟» و این یکی را کامپایلر هیچ‌وقت نمی‌تواند حدس بزند. برایِ `Debug`، جواب مکانیکی است — اسمِ فیلد، دو نقطه، مقدار، همیشه همین قالب، برایِ هر نوعی. برایِ `Display`، جواب یک تصمیمِ انسانی است: `Anime` را برایِ کاربر چطور نشان بدهم؟ عنوان به‌تنهایی؟ عنوان به‌علاوه‌ی امتیاز؟ به همین دلیل صفتِ `Display` **هیچ‌وقت** `derive` نمی‌شود — برایِ هر نوعی که واقعاً به آن نیاز داری، دستی می‌نویسی‌اش:

```rust
impl fmt::Display for Anime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} — {} episodes, {}/100", self.title, self.episodes, self.score)
    }
}
```

```sh
cargo run -p p2-03-04-standard-derives-by-hand --example 02-display-hand-written
```

```text
Display (for a viewer):    Frieren — 28 episodes, 96/100
Debug   (for a developer): Anime { title: "Frieren", episodes: 28, score: 96 }
```

همان مقدار، دو جمله‌ی کاملاً متفاوت — و این دقیقاً نکته است. `Debug` و `Display` دو صفتِ جدا هستند چون دو مخاطبِ جدا دارند؛ یکی همیشه از رویِ فیلدها ساخته می‌شود، دیگری همیشه از رویِ یک تصمیم.

### `Default` — یک نقطه‌ی شروعِ معقول

`Default` جوابِ سؤالِ «یک مقدارِ شروعِ معقول برایِ این نوع چیست؟» است. `#[derive(Default)]` این جواب را از رویِ فیلدها می‌سازد — به شرطی که *هر* فیلد خودش `Default` باشد (`String` خالی، `u32`/`u8` صفر):

```rust
#[derive(Debug, Default)]
struct Anime {
    title: String,
    episodes: u32,
    score: u8,
}

let blank = Anime::default();
println!("{blank:?}");
```

```text
Anime { title: "", episodes: 0, score: 0 }
```

دستی هم همین‌قدر ساده است — فقط خودت هر فیلد را صریح می‌نویسی:

```rust
impl Default for AnimeManual {
    fn default() -> Self {
        AnimeManual { title: String::new(), episodes: 0, score: 0 }
    }
}
```

جایی که `Default` واقعاً به کار می‌آید، **نحوِ به‌روزرسانیِ ساختار (struct update syntax)** است — همان `..other`ی که در [۱.۵.۱](../../../phase1-fundamentals/05-your-own-types/01-structs-and-methods/README.fa.md) دیدی، فقط این‌بار سمتِ راستش یک مقدارِ ساخته‌شده نیست، خودِ `Default::default()` است:

```rust
let started = Anime {
    title: "Bocchi the Rock!".to_string(),
    ..Default::default()
};
```

```sh
cargo run -p p2-03-04-standard-derives-by-hand --example 03-default-and-struct-update
```

```text
derived default:      Anime { title: "", episodes: 0, score: 0 }
hand-written default:  title="" episodes=0 score=0
struct-update syntax:  Anime { title: "Bocchi the Rock!", episodes: 0, score: 0 }
```

`title` را صریح گفتی؛ `episodes` و `score` را `Default::default()` پر کرد. برایِ ساختارهایی با چندین فیلد که اکثرشان معمولاً همان مقدارِ شروع را می‌خواهند، این ترکیب دقیقاً همان چیزی است که از کدنویسیِ تکراری نجاتت می‌دهد.

### `PartialEq` و `Eq` — برابریِ ساختاری، و قولِ اضافه‌ای که `Eq` می‌دهد

`#[derive(PartialEq)]` یک `==` می‌سازد که فیلد به فیلد مقایسه می‌کند — همه باید برابر باشند تا کل مقدار برابر باشد:

```rust
#[derive(Debug, PartialEq, Eq)]
struct Anime {
    title: String,
    episodes: u32,
    score: u8,
}

let a = Anime { title: "Frieren".to_string(), episodes: 28, score: 96 };
let b = Anime { title: "Frieren".to_string(), episodes: 28, score: 96 };
let c = Anime { title: "Frieren".to_string(), episodes: 28, score: 95 };
println!("a == b: {}", a == b);
println!("a == c: {}", a == c);
```

```text
a == b: true
a == c: false
```

دستی‌اش هم چیزِ عجیبی نیست — یک تابعِ `eq` که همان مقایسه را می‌نویسد:

```rust
impl PartialEq for Anime {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title && self.episodes == other.episodes && self.score == other.score
    }
}
```

حالا سؤالِ اصلی: `#[derive(PartialEq, Eq)]` بالا **دو تا** صفت اضافه کرد، نه یکی. `impl Eq for Anime {}` را نگاه کن — بدنه‌اش خالی است. `Eq` هیچ تابعِ تازه‌ای نمی‌خواهد؛ فقط یک **نشانه (marker trait)** است که یک قولِ اضافه می‌دهد: **بازتابی‌بودن (reflexivity)** — یعنی هر مقدار همیشه با خودش برابر است، `x == x`، بدونِ استثنا. `PartialEq` این قول را نمی‌دهد؛ فقط می‌گوید وقتی دو مقدار برابرند، آن برابری با قواعدِ معمولی (تقارن، تعدی) رفتار می‌کند — نه اینکه *هر* مقدار حتماً با خودش برابر باشد.

اینجاست که `f64` از قافله جا می‌ماند:

```rust
let nan = f64::NAN;
println!("f64::NAN == f64::NAN: {}", nan == nan);
println!("1.0_f64 == 1.0_f64:   {}", 1.0_f64 == 1.0_f64);
```

```sh
cargo run -p p2-03-04-standard-derives-by-hand --example 04-partialeq-eq-and-nan
```

```text
a == b (every field matches): true
a == c (score differs):       false
f64::NAN == f64::NAN:         false
1.0_f64 == 1.0_f64:           true
```

(دو خطِ اول از مقایسه‌هایِ زودتر‌ِ فایل بینِ `a`/`b`/`c` می‌آید — همان `PartialEq`ِ معمولی که بالاتر دیدی درست کار می‌کند. نکته‌ی اصلی کاملاً در دو خطِ آخر است.)

`NaN` («عدد نیست» — نتیجه‌ی چیزی مثلِ ۰٫۰ ÷ ۰٫۰) با هیچ مقداری، حتی با خودش، برابر نیست — این قاعده از استانداردِ IEEE 754 می‌آید، نه از Rust. پس `f64` بازتابی نیست، و کتابخانه‌ی استاندارد صادقانه `Eq` را برایش پیاده نمی‌کند — فقط `PartialEq`. اگر خودت بخواهی رویِ یک `struct` که یک فیلدِ `f64` دارد `#[derive(Eq)]` بزنی، کامپایلر همین را با یک خطا بهت می‌گوید؛ کاملش در «خطاهایی که خواهی دید» است.

### `PartialOrd` و `Ord` — همان داستان، یک پله بالاتر

`#[derive(PartialOrd, Ord)]` فیلدها را به همان **ترتیبِ اعلانشان** مقایسه می‌کند — دقیقاً مثلِ مقایسه‌ی تاپل‌ها: اول فیلدِ اول، فقط اگر مساوی بود سراغِ دومی:

```rust
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct AnimeByFields {
    title: String,
    episodes: u32,
    score: u8,
}

let mut by_fields = vec![
    AnimeByFields { title: "Frieren".into(), episodes: 28, score: 96 },
    AnimeByFields { title: "Bocchi".into(), episodes: 12, score: 90 },
];
by_fields.sort();
```

```text
derived Ord (title, then episodes, then score):
  AnimeByFields { title: "Bocchi", episodes: 12, score: 90 }
  AnimeByFields { title: "Frieren", episodes: 28, score: 96 }
```

`"Bocchi"` قبل از `"Frieren"` می‌آید چون `title` فیلدِ اول است — نه چون امتیازش کمتر است. اگر ترتیبِ معنادارتری می‌خواهی — مثلاً «بر اساسِ امتیاز» — خودت `cmp` را می‌نویسی:

```rust
impl Ord for AnimeByScore {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}
```

```sh
cargo run -p p2-03-04-standard-derives-by-hand --example 05-partialord-ord-and-nan
```

```text
hand-written Ord (score only):
  AnimeByScore { title: "Frieren", episodes: 28, score: 96 }
  AnimeByScore { title: "Bocchi", episodes: 12, score: 98 }
```

(`PartialOrd::partial_cmp` را هم باید بنویسی، ولی وقتی `cmp` داری، همیشه همین یک خط است: `Some(self.cmp(other))` — هیچ‌وقت چیزِ دیگری نیست.)

و حالا جوابِ آن سه دیوار. `Ord` دقیقاً همان قولِ اضافه‌ای را می‌دهد که `Eq` می‌داد، فقط یک پله بالاتر: **ترتیبِ کامل (total order)** — یعنی برایِ *هر* دو مقدار، همیشه دقیقاً یک جواب هست به «کدام کوچک‌تر است؟». `PartialOrd` این قول را نمی‌دهد؛ `partial_cmp`اش اجازه دارد `None` برگرداند — «این دو مقدار اصلاً قابلِ‌مقایسه نیستند»:

```rust
let nan = f64::NAN;
println!("1.0.partial_cmp(&NAN): {:?}", 1.0_f64.partial_cmp(&nan));
println!("NAN.partial_cmp(&NAN): {:?}", nan.partial_cmp(&nan));
```

```text
1.0.partial_cmp(&NAN): None
NAN.partial_cmp(&NAN): None
```

نه `Some(Less)`، نه `Some(Equal)`، نه `Some(Greater)` — `None`. `NaN` با `۱٫۰` هم قابلِ‌مقایسه نیست، با خودش هم. `cmp` امضایش اصلاً جایی برایِ «قابلِ‌مقایسه نیست» ندارد — `fn cmp(&self, other: &Self) -> Ordering`، بدونِ `Option`، همیشه یک جوابِ قطعی. `f64` نمی‌تواند این امضا را صادقانه پر کند، پس `Ord` برایش پیاده نمی‌شود. این همان جمله‌ای است که در [۲.۱.۱](../../01-collections/01-vec-depth/README.fa.md) پشتِ `ratings.sort()`، در [۲.۱.۳](../../01-collections/03-btreemap-hashset-vecdeque/README.fa.md) پشتِ `BinaryHeap<f64>`، و در [۲.۲.۳](../../02-iterators-and-closures/03-consuming-and-collecting/README.fa.md) پشتِ `.max()` ایستاده بود. حالا دیگر یک قانونِ حفظی نیست؛ یک نتیجه است که خودت از رویِ امضایِ `cmp` می‌توانی استنتاجش کنی.

### `Hash` — چرا کلیدهایِ `HashMap`/`HashSet` باید `Hash` باشند

[۲.۱.۲](../../01-collections/02-hashmap-in-depth/README.fa.md) گفت کلیدهایِ `HashMap` باید `Hash` و `Eq` باشند، بدونِ اینکه بگوید `Hash` دقیقاً چطور کار می‌کند. جوابش: `Hash` یک مقدار را به‌ترتیب، فیلد به فیلد، به یک شیءِ `Hasher` تغذیه می‌کند — همان شیءای که در آخر یک عددِ هش بیرون می‌دهد. `#[derive(Hash)]` همین را خودکار می‌نویسد:

```rust
#[derive(Debug, PartialEq, Eq, Hash)]
struct AnimeDerived {
    title: String,
    episodes: u32,
}
```

دستی‌اش هم فقط چند صدا زدنِ `.hash(state)` است، به‌ترتیب:

```rust
impl Hash for AnimeManual {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.title.hash(state);
        self.episodes.hash(state);
    }
}
```

```sh
cargo run -p p2-03-04-standard-derives-by-hand --example 06-hash-derive-and-hand
```

```text
derived Hash — inserting the same value again returns: false
set size: 1
hand-written Hash — inserting the same value again returns: false
set size: 1
```

هر دو یکسان رفتار می‌کنند: `HashSet::insert` وقتی مقداری از قبل هست، `false` برمی‌گرداند و اندازه عوض نمی‌شود — دقیقاً همان قراردادی که [۲.۱.۲](../../01-collections/02-hashmap-in-depth/README.fa.md) برایِ `.insert()` نشانت داد.

و `f64`؟ نه فقط `Ord` ندارد — `Hash` هم ندارد. دلیلش را می‌شود با چشمِ خودت دید:

```rust
let z = 0.0_f64;
let neg_z = -0.0_f64;
println!("0.0 == -0.0:      {}", z == neg_z);
println!("0.0.to_bits():    {}", z.to_bits());
println!("(-0.0).to_bits(): {}", neg_z.to_bits());
```

```text
0.0 == -0.0:      true
0.0.to_bits():    0
(-0.0).to_bits(): 9223372036854775808
```

`0.0` و `-0.0` با `==` برابرند، ولی الگویِ بیتیِ‌شان کاملاً متفاوت است. یک `Hash` ساده‌لوح که مستقیم رویِ بیت‌ها کار کند، به این دو مقدارِ *برابر* دو هشِ *متفاوت* می‌داد — دقیقاً همان چیزی که یک پاراگراف دیگر می‌بینی که چرا فاجعه است.

```senpai-visual
{"kind":"concept","labels":["insert(value)","hash(value) → شماره‌ی سطل","داخلِ سطل: Eq مقایسه می‌کند","برابر → جایگزین می‌شود، نابرابر → افزوده می‌شود"]}
```

این دقیقاً همان مسیری است که یک `HashMap`/`HashSet` هر بار طی می‌کند: اول `hash(value)` می‌گوید کدام سطل، بعد — فقط داخلِ همان سطل — `Eq` مقایسه می‌کند تا ببیند این دقیقاً همان مقداری است که قبلاً آنجا بود یا نه. حالا قاعده‌ای که این مسیر رویش تکیه دارد را می‌شود دقیق نوشت: **دو مقدارِ برابر (بر اساسِ `Eq`) باید هشِ برابر هم بدهند.** اگر ندهند، مرحله‌ی «داخلِ سطل: `Eq` مقایسه می‌کند» هیچ‌وقت اصلاً اجرا نمی‌شود — چون دو مقدار از همان مرحله‌ی اول، به دو سطلِ متفاوت فرستاده شده‌اند.

این را با یک `Hash` عمداً ناهماهنگ می‌شود دید، نه فقط باورش کرد:

```rust
impl PartialEq for Anime {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title // فقط عنوان؛ episodes_watched مهم نیست
    }
}
impl Hash for Anime {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.title.hash(state);
        self.episodes_watched.hash(state); // باگ: eq() اصلاً به این نگاه نمی‌کند
    }
}
```

```sh
cargo run -p p2-03-04-standard-derives-by-hand --example 07-hash-eq-inconsistency-trap
```

```text
== says these are the same show: true
.contains() finds it:            false
set length after the "duplicate" insert: 2
  Anime { title: "Frieren", episodes_watched: 5 }
  Anime { title: "Frieren", episodes_watched: 12 }
```

`==` می‌گوید این دو مقدار همان یک نمایش‌اند — درست هم می‌گوید، چون `eq()` فقط `title` را چک می‌کند. ولی `.contains()` پیدایش نمی‌کند، و `.insert()` دومی را هم قبول می‌کند، چون هش‌شان — که `episodes_watched` را هم قاطی کرده — متفاوت است، پس هیچ‌وقت حتی به مرحله‌ی مقایسه‌یِ `Eq` نمی‌رسند. نه خطایی، نه پنیکی، نه هشداری — فقط یک `HashSet` با ۲ عضو که باید ۱ تا می‌بود.

**قاعده‌ای که باید محکم بچسبی:** هرچه `Eq`ات نادیده می‌گیرد، `Hash`ات هم باید نادیده بگیرد — نه بیشتر، نه کمتر. کامپایلر این را چک نمی‌کند؛ نمی‌تواند، چون تصمیمِ اینکه «کدام فیلدها به هویت مربوط‌اند» فقط دستِ توست.

### دیدنشان با هم — یک نوع که چندتایش را دارد

حالا که می‌دانی هرکدام دقیقاً چه قولی می‌دهد، چیدنِ چندتایشان کنارِ هم روی یک `derive` دیگر ترسناک نیست:

```rust
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
struct Anime {
    title: String,
    episodes: u32,
    score: u8,
}

impl Ord for Anime {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score) // معنادارتر از ترتیبِ فیلدها
    }
}
```

پنج تا `derive` شدند، به‌علاوه‌ی یک `Ord` دستی — دقیقاً چون معنایِ «کدام یکی بزرگ‌تر است» برایِ یک انیمه، امتیاز است، نه اینکه اسمش الفبایی کجا می‌افتد. حالا همین یک نوع را در دو ساختارِ دادهٔ کاملاً متفاوت به‌کار می‌بریم:

```sh
cargo run -p p2-03-04-standard-derives-by-hand --example 08-all-together
```

```text
watch next: Some(Anime { title: "Frieren", episodes: 28, score: 96 })
then:       Some(Anime { title: "Bocchi the Rock!", episodes: 12, score: 90 })
duplicate insert returned: false
library size:              1
Default::default():        Anime { title: "", episodes: 0, score: 0 }
```

`BinaryHeap<Anime>` از `Ord` دستی‌ات استفاده می‌کند تا همیشه بالاترین امتیاز را اول بدهد. `HashSet<Anime>` از `Hash`/`Eq`ِ مشتق‌شده استفاده می‌کند تا یک نسخه‌ی تکراری را رد کند. `Default` یک نقطه‌ی شروعِ خالی می‌دهد. سه صفتِ کاملاً جدا، سه کاری که هرکدام فقط از خودشان برمی‌آید، رویِ یک نوع.

یک نکته‌ی ظریف، برایِ وقتی سراغِ `BTreeSet`/`BTreeMap` هم رفتی: آن دو، برایِ تشخیصِ «این دو مقدار یکی‌اند؟»، فقط به `Ord` نگاه می‌کنند، نه به `Eq`. اگر `Ord`ات — مثلِ همین بالا — فقط رویِ `score` قضاوت کند، دو انیمه‌ی *متفاوت* با امتیازِ *یکسان* را در یک `BTreeSet` یکی حساب می‌کند، حتی اگر `==` بگوید نابرابرند. همان بیماری‌ای که در `Hash` دیدی، این‌بار در لباسِ `Ord`.

---

## دست‌به‌کد

```sh
cargo run -p p2-03-04-standard-derives-by-hand --example 01-debug-two-ways
cargo run -p p2-03-04-standard-derives-by-hand --example 02-display-hand-written
cargo run -p p2-03-04-standard-derives-by-hand --example 03-default-and-struct-update
cargo run -p p2-03-04-standard-derives-by-hand --example 04-partialeq-eq-and-nan
cargo run -p p2-03-04-standard-derives-by-hand --example 05-partialord-ord-and-nan
cargo run -p p2-03-04-standard-derives-by-hand --example 06-hash-derive-and-hand
cargo run -p p2-03-04-standard-derives-by-hand --example 07-hash-eq-inconsistency-trap
cargo run -p p2-03-04-standard-derives-by-hand --example 08-all-together
```

بعد دوتای خراب:

```sh
cargo run -p p2-03-04-standard-derives-by-hand --example 09-derive-eq-needs-field-eq --features broken
cargo run -p p2-03-04-standard-derives-by-hand --example 10-hashset-needs-hash-and-eq --features broken
```

بعد این‌ها را امتحان کن:

۱. در `01-debug-two-ways`، یک فیلدِ چهارم به هر سه `struct` اضافه کن. خروجیِ نسخه‌ی `builder` زیرِ `{:#?}` خودش را با فیلدِ تازه هماهنگ می‌کند؛ نسخه‌ی `raw` چطور؟
۲. در `05-partialord-ord-and-nan`، ترتیبِ فیلدهایِ `AnimeByFields` را عوض کن (`episodes` را اول بگذار). ترتیبِ مرتب‌شده عوض می‌شود؟ چرا دقیقاً همین انتظار را داشتی؟
۳. در `07-hash-eq-inconsistency-trap`، خطِ `self.episodes_watched.hash(state);` را حذف کن. `set length after the "duplicate" insert` این‌بار چند می‌شود؟

---

## خطاهایی که خواهی دید

### هیچ خطایی نیست — `Hash`ای که با `Eq` هم‌خوان نیست، `HashSet` را بی‌سروصدا می‌شکند

کدِ کاملش همان `07-hash-eq-inconsistency-trap` بالاست. **کامپایلر به چه اعتراض دارد:** به هیچ‌چیز. `impl PartialEq` و `impl Hash` هر دو به‌تنهایی معتبرند؛ هیچ قاعده‌ای در زبان نمی‌گوید این دو باید دربارهٔ یک فیلد توافق داشته باشند — این توافق فقط یک قرارداد است، نه چیزی که `rustc` بتواند چکش کند. برنامه کامپایل می‌شود، اجرا می‌شود، و بدونِ هیچ خطا یا پنیکی تمام می‌شود؛ فقط جوابش غلط است: `set length after the "duplicate" insert` باید ۱ می‌بود، شد ۲.

**راه‌حل:** `Hash` را دقیقاً همان فیلدهایی بده که `Eq` نگاه می‌کند — نه یکی بیشتر:

```rust
impl Hash for Anime {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.title.hash(state); // فقط همینی که eq() هم چک می‌کند
    }
}
```

**چرا این راه‌حل است:** حالا دو مقداری که `eq()` برابر می‌داند، به یک هش هم می‌رسند، پس به یک سطل هم می‌روند، و `Eq` بالاخره فرصتِ مقایسه‌کردن پیدا می‌کند. **این نوع خطا خطرناک‌ترین نوعِ خطاست:** هیچ کدِ قرمزی، هیچ پنیکی، هیچ چیزی که تست‌های سطحی بگیرندش — فقط یک `HashSet` که چیزی را که باید یکی می‌شمرد، دوتا می‌شمرد.

### `E0277` — `#[derive(Eq)]` روی یک `struct` با فیلدِ `f64`

```text
error[E0277]: the trait bound `f64: Eq` is not satisfied
   --> phase2-intermediate\03-traits-and-generics\04-standard-derives-by-hand\examples\09-derive-eq-needs-field-eq.rs:12:5
    |
 10 | #[derive(Debug, PartialEq, Eq)]
    |                            -- in this derive macro expansion
 11 | struct Rating {
 12 |     value: f64,
    |     ^^^^^^^^^^ the trait `Eq` is not implemented for `f64`
    |
    = help: the following other types implement trait `Eq`:
              i128
              i16
              i32
              i64
              i8
              isize
              u128
              u16
            and 4 others
note: required by a bound in `std::cmp::AssertParamIsEq`
   --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\cmp.rs:380:31
    |
380 | pub struct AssertParamIsEq<T: Eq + PointeeSized> {
    |                               ^^ required by this bound in `AssertParamIsEq`

For more information about this error, try `rustc --explain E0277`.
```

**کامپایلر به چه اعتراض دارد:** `#[derive(Eq)]` فقط وقتی معتبر است که *هر* فیلدِ `struct` خودش `Eq` باشد — چون نمی‌شود قولِ بازتابی‌بودن داد وقتی یکی از فیلدهایت (اینجا `value: f64`) خودش این قول را نمی‌دهد. این همان واقعیتِ `NaN`ای است که در «مفهوم» دیدی، فقط این‌بار پشتِ `f64: Ord`ِ خامِ [۲.۱.۱](../../01-collections/01-vec-depth/README.fa.md) و [۲.۱.۳](../../01-collections/03-btreemap-hashset-vecdeque/README.fa.md) نیست — پشتِ یک `struct` از خودِ توست.

**راه‌حل:** یا `Eq` را نخواه (فقط `PartialEq` کافی است اگر واقعاً به قولِ بازتابی‌بودن نیازی نداری)، یا نوعِ فیلد را عوض کن — مثلاً امتیاز را ضربدرِ ده کن و به‌عنوانِ عددِ صحیح نگه دار، همان ترفندی که [۲.۱.۱](../../01-collections/01-vec-depth/README.fa.md) برایِ `.sort()` استفاده کرد.

**چرا این راه‌حل است:** `#[derive(PartialEq)]` به‌تنهایی هیچ قولی دربارهٔ بازتابی‌بودن نمی‌دهد، پس هیچ فیلدی مجبور نیست `Eq` باشد — دقیقاً همان‌طور که خودِ `f64` رفتار می‌کند. و یک عددِ صحیح، برخلافِ `f64`، همیشه با خودش برابر است؛ `Eq` را رایگان می‌گیرد.

### `E0599` — `HashSet<Anime>` بدونِ `Hash` و `Eq`

```text
error[E0599]: the method `insert` exists for struct `HashSet<Anime>`, but its trait bounds were not satisfied
  --> phase2-intermediate\03-traits-and-generics\04-standard-derives-by-hand\examples\10-hashset-needs-hash-and-eq.rs:19:10
   |
12 | struct Anime {
   | ------------ doesn't satisfy `Anime: Eq` or `Anime: Hash`
...
19 |     seen.insert(Anime {
   |     -----^^^^^^
   |
   = note: the following trait bounds were not satisfied:
           `Anime: Eq`
           `Anime: Hash`
help: consider annotating `Anime` with `#[derive(Eq, Hash, PartialEq)]`
   |
12 + #[derive(Eq, Hash, PartialEq)]
13 | struct Anime {
   |

For more information about this error, try `rustc --explain E0599`.
```

**کامپایلر به چه اعتراض دارد:** این همان دیواری است که در [۲.۱.۲](../../01-collections/02-hashmap-in-depth/README.fa.md) پشتِ `HashMap::insert` دیدی، این‌بار پشتِ یک `HashSet`. برایِ اینکه `.insert()` بداند کجا بگذارد و بعداً چطور تشخیص بدهد این مقدار قبلاً بوده یا نه، به هر دو صفت نیاز دارد — `Hash` برایِ سطل، `Eq` برایِ مقایسهٔ داخلِ سطل. `Anime` اینجا فقط `#[derive(Debug)]` دارد؛ هیچ‌کدام از آن دو را ندارد.

**راه‌حل:** دقیقاً پیشنهادِ کامپایلر:

```rust
#[derive(Debug, Eq, Hash, PartialEq)]
struct Anime {
    title: String,
    episodes: u32,
}
```

**چرا این راه‌حل است:** `String` و `u32` هر دو خودشان `Hash` و `Eq` هستند، پس `derive` می‌تواند هر دو صفت را از رویِ فیلدها بسازد — دقیقاً همان مکانیزمی که در «مفهوم» با دستِ خودت هم نوشتی‌اش.

---

## تمرین

### گرم‌کردن

<details>
<summary>یک <code>struct</code> با <code>#[derive(Debug)]</code> و فیلدهایِ <code>id: u32</code> و <code>name: String</code> داری. برایِ <code>Item { id: 7, name: "x".to_string() }</code>، <code>format!("{item:?}")</code> دقیقاً چه رشته‌ای می‌دهد؟</summary>

فکرت را قبل از دیدنِ پاسخ بنویس.

</details>

<details>
<summary>پاسخ</summary>

```text
Item { id: 7, name: "x" }
```

اسمِ نوع، بعد آکولاد، بعد هر فیلد به شکلِ `اسم: مقدار` جدا با کاما — دقیقاً همان چیزی که `f.debug_struct("Item").field(...)...` می‌ساخت.

</details>

<details>
<summary><code>#[derive(Debug, PartialEq, Eq)] struct Meters(f64);</code> کامپایل می‌شود؟</summary>

فکرت را بنویس — چه چیزی رویِ `f64` قبلاً دیدی؟

</details>

<details>
<summary>پاسخ</summary>

نه. `f64` صفتِ `Eq` را ندارد (چون `NaN != NaN` بازتابی‌بودن را می‌شکند)، و `#[derive(Eq)]` نیاز دارد *هر* فیلد خودش `Eq` باشد. کدِ خطا `E0277` است.

</details>

<details>
<summary><code>f64::NAN.partial_cmp(&f64::NAN)</code> چه چیزی برمی‌گرداند؟</summary>

فکرت را بنویس.

</details>

<details>
<summary>پاسخ</summary>

`None` — نه `Some(Equal)`. `NaN` حتی با خودش هم قابلِ‌مقایسه نیست؛ `partial_cmp` برایِ همین «قابلِ‌مقایسه نیست» یک `Option` برمی‌گرداند.

</details>

<details>
<summary>یک <code>struct</code> با <code>#[derive(PartialOrd, Ord, ...)]</code> و فیلدهایِ <code>(a: u32, b: u32)</code> به همین ترتیب داری. برایِ دو مقدار که فقط در <code>b</code> فرق دارند — یکی <code>b: 1</code>، دیگری <code>b: 9</code> — کدام موقعِ <code>.sort()</code> اول می‌آید؟</summary>

فکرت را بنویس — `derive` فیلدها را به چه ترتیبی مقایسه می‌کند؟

</details>

<details>
<summary>پاسخ</summary>

آن‌که `b: 1` دارد. چون `a` در هر دو یکسان است، مقایسه به فیلدِ دوم، `b`، می‌رسد، و مقایسه‌ی فیلدها همیشه به ترتیبِ اعلانشان است.

</details>

<details>
<summary>یک نوع <code>impl PartialEq</code> دارد که فقط فیلدِ <code>x</code> را چک می‌کند، ولی <code>#[derive(Hash)]</code> رویش هر دو فیلدِ <code>x</code> و <code>y</code> را هش می‌کند. این کامپایل می‌شود؟</summary>

فکرت را بنویس — چیزی هست که کامپایلر این‌جا بتواند چک کند؟

</details>

<details>
<summary>پاسخ</summary>

بله، کامپایل می‌شود — هیچ خطایی نیست. مشکل زمانِ اجرا خودش را نشان می‌دهد: دو مقداری که `==` برابر می‌داند ممکن است هشِ متفاوت بگیرند، و یک `HashSet`/`HashMap` که از این نوع کلید می‌سازد، بی‌سروصدا شروع می‌کند به ندیدنِ تکراری‌ها.

</details>

### تعمیر

هر دو مثالِ خراب را درست کن:

۱. `examples/09-derive-eq-needs-field-eq.rs` را طوری درست کن که کامپایل شود — بدونِ اینکه معنایِ `value` را از دست بدهی (راهنمایی: نوعش را عوض کن، نه اینکه `Eq` را کلاً بردار مگر واقعاً به آن نیاز نداشته باشی).
۲. `examples/10-hashset-needs-hash-and-eq.rs` را طوری درست کن که کامپایل شود — دقیقاً پیشنهادِ خودِ کامپایلر.

### پیاده‌سازی

یک `struct Track` در `src/lib.rs`، و شش صفت — یکی‌یکی، هرکدام یک `todo!()`:

```sh
cargo test -p p2-03-04-standard-derives-by-hand
```

`impl Eq for Track {}` و `impl PartialOrd for Track` از قبل نوشته شده‌اند — هر دو، وقتی `PartialEq`/`Ord` واقعی داری، همیشه همان یک خط‌اند. کامنتِ مستنداتِ هر تابع دقیقاً می‌گوید انتظارِ خروجی چیست؛ چیزی را حدس نزن، مخصوصاً قاعده‌ی «`Hash`ات دقیقاً همان فیلدهایی را ببیند که `Eq`ات می‌بیند».

### بساز

یک `struct` کوچکِ خودت بساز (دو یا سه فیلد، هر دامنه‌ای که دوست داری) و دست‌کم **سه‌تا** از شش صفتِ امروز را دستی — نه با `#[derive]` — رویش پیاده کن؛ دست‌کم یکی از آن سه‌تا باید `Ord` یا `Hash` باشد. در کامنتِ مستناتِ `struct` بنویس کدام سه‌تا را انتخاب کردی و چرا — و برایِ کدام‌ها یک `#[derive]` ساده کافی *نبود*.

### چالش (اختیاری)

[۲.۱.۳](../../01-collections/03-btreemap-hashset-vecdeque/README.fa.md) در چالشِ اختیاری‌اش نشانت داد چطور با یک `BinaryHeap<Reverse<(u32, String)>>` که هیچ‌وقت بیشتر از `k` عضو ندارد، `k` تای برتر را بدونِ مرتب‌کردنِ کاملِ لیست پیدا کنی. همان تکنیک را این‌بار رویِ `Track`ات (یا `struct`ی که در «بساز» نوشتی) پیاده کن: یک تابع بنویس که `k` تا کوتاه‌ترین آهنگ را برمی‌گرداند، با یک `BinaryHeap` که هیچ‌وقت از `k` عضو بیشتر نمی‌شود — بدونِ اینکه یک‌بار کلِ لیست را کامل مرتب کنی.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| `f.debug_struct(...)` | ابزارِ ارگونومیکِ ساختِ خروجیِ `Debug`؛ `{:#?}` را رایگان پشتیبانی می‌کند | `impl Debug` دستی برایِ یک `struct` واقعی |
| `Default` | یک مقدارِ شروعِ معقول؛ `Default::default()` | نحوِ به‌روزرسانیِ ساختار (`..Default::default()`) |
| بازتابی‌بودن (reflexivity) | هر مقدار همیشه با خودش برابر است، `x == x` | چرا `f64` فقط `PartialEq` دارد، نه `Eq` |
| ترتیبِ کامل (total order) | برایِ هر دو مقدار همیشه دقیقاً یک جوابِ «کدام کوچک‌تر است؟» هست | چرا `f64` فقط `PartialOrd` دارد، نه `Ord` |
| `NaN` | «عدد نیست»؛ با هیچ مقداری، حتی خودش، قابلِ‌مقایسه یا برابر نیست | دلیلِ ریشه‌ایِ نبودِ `Eq`/`Ord`/`Hash` روی `f64` |
| صفتِ `Hash` | مقدار را فیلد به فیلد به یک `Hasher` تغذیه می‌کند | هر نوعی که می‌خواهد کلیدِ `HashMap`/`HashSet` باشد |
| هم‌خوانیِ `Hash` با `Eq` | دو مقدارِ برابر (بر اساسِ `Eq`) باید هشِ برابر هم بدهند | نقضش را کامپایلر نمی‌بیند؛ `HashSet` بی‌سروصدا می‌شکند |

### الان می‌دانی

- `#[derive(Debug)]` دقیقاً همان چیزی است که `f.debug_struct(...).field(...).finish()` هم می‌سازد؛ یک `write!` خام همان خروجیِ فشرده را می‌دهد ولی `{:#?}` را پشتیبانی نمی‌کند.
- `Display` هیچ‌وقت `derive` نمی‌شود، چون شکلِ «برایِ کاربر» یک تصمیمِ انسانی است، نه چیزی که از رویِ فیلدها مکانیکی دربیاید.
- `Default` یک مقدارِ شروعِ معقول می‌دهد؛ `#[derive(Default)]` نیاز دارد هر فیلد خودش `Default` باشد، و نحوِ به‌روزرسانیِ ساختار رویش تکیه می‌کند.
- `Eq` روی `PartialEq` فقط یک قولِ اضافه می‌گذارد — بازتابی‌بودن — و `Ord` روی `PartialOrd` همان قول را یک پله بالاتر می‌برد — ترتیبِ کامل. `f64` به‌خاطرِ `NaN` هیچ‌کدام را نمی‌تواند صادقانه بدهد.
- `Hash` مقدار را فیلد به فیلد به یک `Hasher` می‌دهد؛ باید دقیقاً همان فیلدهایی را ببیند که `Eq`ات می‌بیند — نه بیشتر، نه کمتر — وگرنه `HashMap`/`HashSet` بدونِ هیچ خطایی، بی‌سروصدا جواب‌هایِ غلط می‌دهد.
- `f64` حتی `Hash` هم ندارد: `0.0 == -0.0` است ولی الگویِ بیتیِ‌شان فرق دارد — دقیقاً همان بیماری‌ای که یک `Hash` ناهماهنگ با `Eq` می‌سازد.

### بعداً کامل‌تر می‌بینی

- **پیاده‌سازیِ یک صفت رویِ نوعی که خودت صاحبش نیستی، و قاعده‌ی یتیم** — [۲.۳.۶ — ابرصفت‌ها، پیاده‌سازیِ فراگیر و قاعده‌ی یتیم](../06-supertraits-blanket-impls-orphan-rule/README.fa.md)
- **چیزی که رویِ چند نوعِ متفاوت یکسان کار کند، بدونِ دانستنِ نوعِ دقیق در زمانِ کامپایل** — [۲.۳.۷ — ارسالِ ایستا در برابرِ پویا](../07-static-vs-dynamic-dispatch/README.fa.md)
- **`Display` روی نوع‌هایِ خطا، برایِ پیام‌هایِ خطایِ خودت** — [۲.۵.۱ — نوع‌هایِ خطایِ سفارشی](../../05-error-handling/01-custom-error-types/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا یک `impl Debug` دستی با `write!` خام، زیرِ `{:#?}` همان چیزی را می‌دهد که زیرِ `{:?}` می‌داد، ولی نسخه‌ی `debug_struct` نه؟
- چرا هیچ‌وقت `#[derive(Display)]`ای وجود ندارد؟
- `Eq` دقیقاً چه قولِ اضافه‌ای نسبت به `PartialEq` می‌دهد، و `f64` چرا نمی‌تواند بدهدش؟
- `Ord` دقیقاً چه قولِ اضافه‌ای نسبت به `PartialOrd` می‌دهد؟
- چرا خودِ `f64` هیچ‌وقت `Hash` هم نیست — نه فقط `Ord`؟
- اگر `Hash`ات با `Eq`ات هم‌خوان نباشد، دقیقاً کجایِ کارِ یک `HashSet` می‌شکند، و چرا هیچ پنیکی نمی‌گیری؟

---

## بیشتر

- [کتابِ Rust — ضمیمه‌ی C: صفت‌هایِ قابلِ‌مشتق‌شدن](https://doc.rust-lang.org/book/appendix-03-derivable-traits.html) — فهرستِ کاملِ صفت‌هایِ استاندارد که `#[derive]` می‌شناسد، از زبانِ خودِ تیمِ Rust.
- [مستنداتِ `std::fmt`](https://doc.rust-lang.org/std/fmt/index.html) — همه‌ی ماکروهایِ فرمت و امضایِ دقیقِ `Debug`/`Display`.
- [مستنداتِ `std::hash::Hash`](https://doc.rust-lang.org/std/hash/trait.Hash.html) — از جمله همان جمله‌ای که صریح می‌گوید: «`Eq` و `Hash` باید هم‌خوان بمانند.»
- [مستنداتِ `f64::total_cmp`](https://doc.rust-lang.org/std/primitive.f64.html#method.total_cmp) — همان ابزاری که [۲.۱.۱](../../01-collections/01-vec-depth/README.fa.md) برایِ دور زدنِ همین مشکل به‌کار برد.
