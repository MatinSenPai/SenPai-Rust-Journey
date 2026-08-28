# ۲.۳.۱ — تعریف و پیاده‌سازیِ صفت‌ها

## در یک نگاه

بعد از این درس می‌توانی:

- بگویی یک صفت (trait) دقیقاً چیست — قراردادی از امضایِ متد، بدونِ هیچ فیلدی، نه یک کلاسِ پایه — و کجا این تشبیه با ABC یا Protocol در پایتون می‌شکند.
- یک صفتِ خودت را با یک متدِ اجباری و یک متدِ پیش‌فرض تعریف کنی، آن را برایِ دو نوعِ کاملاً نامرتبط پیاده‌سازی کنی — یکی با پذیرفتنِ پیش‌فرض، یکی با بازنویسی‌اش — و توضیح بدهی چرا بعدش یک فراخوانیِ یکسان رویِ هر دو کار می‌کند.
- بگویی چرا فراخوانیِ متدِ یک صفت به importشدنِ خودِ صفت نیاز دارد، حتی وقتی نوع کاملاً در دسترس است، و خطاهایِ `E0046` و `E0599` را خودت بخوانی و رفع کنی.

**زمان:** حدود ۴۵ دقیقه · **پیش‌نیاز:** [۲.۲.۴ — پیاده‌سازیِ Iterator و IntoIterator برایِ نوعِ خودت](../../02-iterators-and-closures/04-implementing-iterator/README.fa.md)

---

## چرا اهمیت دارد

کلمه‌ی «صفت (trait)» برایت غریبه نیست. تویِ [۲.۲.۱](../../02-iterators-and-closures/01-closures-and-fn-traits/README.fa.md) گفتیم هر کلوژری، بسته به بدنه‌اش، یکی از سه صفت را پیاده‌سازی می‌کند — `Fn`، `FnMut`، `FnOnce` — و همان‌جا نوشتیم که «جنریک‌ها و impl Trait را ماژولِ ۲.۳ کامل یاد می‌دهد». و تویِ [۲.۲.۴](../../02-iterators-and-closures/04-implementing-iterator/README.fa.md) دقیقاً همین نحو را با دستِ خودت نوشتی:

```rust
impl Iterator for Fibonacci {
    type Item = u64;
    // fn next(&mut self) -> Option<u64> { ... } — همان چیزی که در ۲.۲.۴ نوشتی
}
```

آن یک **پیاده‌سازیِ صفت** بود. فقط خودِ صفتش — `Iterator` — را کتابخانه‌ی استاندارد از قبل برایت تعریف کرده بود؛ کارِ تو فقط پُرکردنِ قراردادی بود که از قبل نوشته شده بود.

امروز چیزی که فرق دارد این‌جاست: خودت آن قرارداد را می‌نویسی، نه فقط پُرش می‌کنی. و این چیزِ کمی نیست. بدونِ آن، هر بار که یک قابلیتِ مشترک بینِ چند نوعِ خودت لازم داری — همه باید بتوانند «خلاصه‌شان» را بدهند، همه باید بتوانند اعتبارسنجی شوند، هرچه — یا باید کدت را قفلِ یک ساختارِ خاص کنی، یا اسمِ دقیقِ تابعِ جداگانه‌یِ هر نوع را جداگانه به‌خاطر بسپاری. صفتِ خودت‌نوشته یک اسمِ رسمی به آن قرارداد می‌دهد، و از این به بعد، کامپایلر نگهبانِ آن قرارداد است، نه حافظه‌ی تو.

---

## مفهوم

### صفت چیست: یک قرارداد، نه یک پایه‌ی کلاس

یک صفت (trait) مجموعه‌ای از امضایِ متد است — نامِ متد، پارامترهایش، نوعِ برگشتی‌اش — بدونِ هیچ فیلدی. هیچ داده‌ای داخلِ یک trait زندگی نمی‌کند؛ فقط رفتار. این دقیقاً همان چیزی است که آن را از یک کلاسِ پایه در زبان‌هایِ شیءگرا جدا می‌کند: یک کلاسِ پایه معمولاً هم داده به ارث می‌گذارد هم رفتار؛ یک trait فقط قولِ رفتار می‌دهد، و هر نوعی که آن قول را قبول کند، داده‌هایِ خودش را با خودش می‌آورد.

نزدیک‌ترین چیزی که احتمالاً تویِ پایتون دیده‌ای، یک کلاسِ ABC (`abc.ABC` به‌همراهِ دکوراتورِ `@abstractmethod`) یا یک `Protocol` است:

```python
class Summarize(ABC):
    @abstractmethod
    def title(self) -> str: ...
```

ایده شبیه است — قراردادِ رفتاری‌ای که چند کلاسِ مختلف می‌توانند قبولش کنند. تفاوتِ اصلی *کِی* بررسی می‌شود. پایتون فقط زمانی که بخواهی از کلاسِ ناقص یک نمونه بسازی می‌فهمد چیزی جا افتاده — یا با یک `Protocol`ِ ساده، شاید هیچ‌وقت نفهمد، تا همان خطی که واقعاً متدِ غایب را صدا می‌زند و برنامه می‌ترکد. Rust هر بلاکِ پیاده‌سازیِ یک صفت را در زمانِ کامپایل بررسی می‌کند — پیش از آنکه هیچ کدی اجرا شود. این تفاوت را «خطاهایی که خواهی دید» با یک مثالِ واقعی نشانت می‌دهد.

### تعریفِ یک صفت: امضایِ اجباری و پیاده‌سازیِ پیش‌فرض

بیا یکی واقعی بنویسیم — قراردادی برایِ «هر چیزی که بتواند خودش را معرفی کند»:

```rust
trait Summarize {
    fn title(&self) -> String;

    fn summary(&self) -> String {
        format!("{} (no summary available)", self.title())
    }
}
```

دو متد، دو شکلِ کاملاً متفاوت. تعریفِ `title` هیچ بدنه‌ای ندارد — فقط یک امضا که با `;` تمام می‌شود. این یک **متدِ اجباری** است: هر نوعی که `Summarize` را پیاده‌سازی کند، باید نسخه‌ی خودش را برایِ آن بنویسد، وگرنه کامپایل نمی‌شود. تعریفِ `summary` اما یک بدنه دارد — این یک **متدِ پیش‌فرض** است: هر پیاده‌کننده‌ای آن را دقیقاً همان‌طور که نوشته شده، رایگان می‌گیرد، مگر خودش تصمیم بگیرد بازنویسی‌اش کند.

نکته‌ای که ارزشِ توقف دارد: بدنه‌ی پیش‌فرضِ `summary` دارد متدِ `title` را صدا می‌زند — متدی که همین‌جا، تویِ همین تعریف، هیچ بدنه‌ای ندارد و هیچ نوعِ ملموسی هم هنوز پشتش نیست. این کامپایل می‌شود چون خودِ trait تضمین می‌کند: هر نوعی که اینجا قرار بگیرد، حتماً یک `title` دارد. کامپایلر برایِ اینکه بداند این فراخوانی ایمن است، نیازی ندارد بداند نوعِ ملموس دقیقاً چیست؛ همین‌که بداند هر چه هست `Summarize` را پیاده‌سازی کرده، کافی است.

### پیاده‌سازی برایِ اولین نوع

خودِ trait تنها یک قرارداد است — تا وقتی کسی پیاده‌سازی‌اش نکند، هیچ کدی اجرا نمی‌شود. با نحوِ `impl Trait for Type` پُرش می‌کنی:

```rust
struct AnimeSeries {
    title: String,
    episodes: u32,
}

impl Summarize for AnimeSeries {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn summary(&self) -> String {
        format!("{} — {} episodes", self.title(), self.episodes)
    }
}
```

```rust
let death_note = AnimeSeries {
    title: "Death Note".to_string(),
    episodes: 37,
};
println!("{}", death_note.title());
println!("{}", death_note.summary());
```

```text
Death Note
Death Note — 37 episodes
```

`AnimeSeries` هم `title` (اجباری) را داد، هم `summary` را — با نسخه‌ی خودش، نه با پیش‌فرض. این یک انتخابِ واقعی بود: می‌توانست `summary` را کلاً ننویسد و پیش‌فرض را بگیرد. اینجا ترجیح داد خودش بنویسدش، چون می‌خواست تعدادِ قسمت‌ها را هم نشان بدهد — چیزی که پیش‌فرض از آن خبر ندارد.

### همان صفت، برایِ نوعی کاملاً نامرتبط

اینجاست که این سرمایه‌گذاری جواب می‌دهد. `MangaVolume` هیچ ربطی به `AnimeSeries` ندارد — نه ساختارِ پایه‌ی مشترک، نه ارث‌بری، هیچ. تنها چیزی که این دو نوع را به هم وصل می‌کند این است که هر دو قولِ `Summarize` را داده‌اند:

```rust
struct MangaVolume {
    title: String,
}

impl Summarize for MangaVolume {
    fn title(&self) -> String {
        self.title.clone()
    }

    // بدونِ بازنویسیِ summary — عمدی. MangaVolume کاملاً به
    // پیاده‌سازیِ پیش‌فرضِ Summarize تکیه می‌کند.
}
```

```rust
let berserk = MangaVolume {
    title: "Berserk Vol. 1".to_string(),
};
println!("{}", death_note.summary());
println!("{}", berserk.summary());
```

```text
Death Note — 37 episodes
Berserk Vol. 1 (no summary available)
```

همین یک فراخوانی، دوبار، رویِ دو نوعِ کاملاً بی‌ربط. فراخوانیِ `summary` رویِ `death_note` نسخه‌ی خودِ `AnimeSeries` را اجرا می‌کند؛ همان فراخوانی رویِ `berserk` — که هیچ‌وقت خودش ننوشتش — نسخه‌ی پیش‌فرضِ trait را اجرا می‌کند، که خودش متدِ `title` را صدا می‌زند و این‌بار نسخه‌ی `MangaVolume` جواب می‌دهد. هیچ‌کدام از این دو نوع نمی‌دانند دیگری وجود دارد. فقط هر دو یک قرارداد را قبول کردند.

```senpai-visual
{"kind":"concept","labels":["فراخوانیِ summary روی berserk","impl هیچ بازنویسی‌ای ندارد","بدنه‌ی پیش‌فرضِ trait اجرا می‌شود","متدِ title از داخلِ خودش صدا زده می‌شود","پیاده‌سازیِ MangaVolume اجرا می‌شود","رشته‌ی نهایی برمی‌گردد"]}
```

### صفت باید در دامنه باشد

یک نکته‌ی آخر، و این یکی معمولاً همه را غافلگیر می‌کند: صدازدنِ یک متدِ صفت رویِ یک مقدار، به importشدنِ خودِ صفت نیاز دارد — حتی وقتی نوع کاملاً `pub` و در دسترس است. `Summarize` را داخلِ یک ماژول بگذار:

```rust
mod catalog {
    pub trait Summarize {
        fn summary(&self) -> String;
    }

    pub struct AnimeSeries {
        pub title: String,
    }

    impl Summarize for AnimeSeries {
        fn summary(&self) -> String {
            self.title.clone()
        }
    }
}
```

```rust
use catalog::Summarize;

fn main() {
    let death_note = catalog::AnimeSeries {
        title: "Death Note".to_string(),
    };
    println!("{}", death_note.summary());
}
```

```text
Death Note
```

خطِ `use catalog::Summarize;` را بردار — `AnimeSeries` هنوز کاملاً `pub` است، هنوز می‌توانی بسازیش، ولی فراخوانیِ `summary` رویِ آن دیگر کامپایل نمی‌شود. کامپایلر می‌داند این متد یک‌جایی وجود دارد؛ فقط اجازه نمی‌دهد صدایش بزنی مگر خودِ trait را هم بیاوری. نسخه‌ی خرابش، با خطایِ کاملش، تویِ «خطاهایی که خواهی دید» است.

---

## دست‌به‌کد

```sh
cargo run -p p2-03-01-defining-and-implementing-traits --example 01-declaring-and-implementing-a-trait
cargo run -p p2-03-01-defining-and-implementing-traits --example 02-same-trait-two-unrelated-types
cargo run -p p2-03-01-defining-and-implementing-traits --example 03-trait-must-be-in-scope
```

بعد دوتایِ خراب:

```sh
cargo run -p p2-03-01-defining-and-implementing-traits --example 04-missing-required-method --features broken
cargo run -p p2-03-01-defining-and-implementing-traits --example 05-trait-not-in-scope --features broken
```

بعد این‌ها را امتحان کن:

۱. تویِ `02-same-trait-two-unrelated-types`، یک خطِ دیگر اضافه کن که نتیجه‌ی فراخوانیِ `title` رویِ `berserk` را چاپ کند. کامپایل می‌شود؟ چه چاپ می‌کند؟
۲. تویِ `03-trait-must-be-in-scope`، خطِ `use catalog::Summarize;` را کامنت کن و پیشِ خودت پیش‌بینی کن چه خطایی می‌گیری — بعد با `examples/05-trait-not-in-scope.rs` مقایسه کن.
۳. تویِ `01-declaring-and-implementing-a-trait`، کلِ بازنویسیِ `summary` را از بلاکِ `impl Summarize for AnimeSeries` پاک کن. پیشِ خودت حدس بزن فراخوانیِ `summary` رویِ `death_note` حالا چه چاپ می‌کند، بعد اجرا کن و ببین درست حدس زدی یا نه.

---

## خطاهایی که خواهی دید

### `E0046` — نه همه‌ی موادِ صفت پیاده‌سازی شده‌اند

```text
error[E0046]: not all trait items implemented, missing: `title`
  --> phase2-intermediate\03-traits-and-generics\01-defining-and-implementing-traits\examples\04-missing-required-method.rs:17:1
   |
 6 |     fn title(&self) -> String;
   |     -------------------------- `title` from trait
...
17 | impl Summarize for LightNovel {}
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `title` in implementation

For more information about this error, try `rustc --explain E0046`.
```

**کامپایلر به چه اعتراض دارد:** بلاکِ `impl Summarize for LightNovel` باز است — بدونِ هیچ متدی داخلش. تعریفِ `title` تویِ trait هیچ بدنه‌ای نداشت، پس اجباری بود؛ `LightNovel` قولش را نداد.

**راه‌حل:** متدِ جاافتاده را بنویس:

```rust
impl Summarize for LightNovel {
    fn title(&self) -> String {
        self.title.clone()
    }
}
```

**چرا این راه‌حل است:** بازنویسیِ `summary` (که پیش‌فرض دارد) اختیاری بود و هست؛ ولی `title` (که بدنه ندارد) هیچ‌وقت اختیاری نبوده. کامپایلر دقیقاً همان چیزی را می‌خواهد که trait از اول قول داده بود: یک `title` برایِ هر پیاده‌کننده.

### `E0599` — متد پیدا نشد، چون صفتش در دامنه نیست

```text
error[E0599]: no method named `summary` found for struct `AnimeSeries` in the current scope
  --> phase2-intermediate\03-traits-and-generics\01-defining-and-implementing-traits\examples\05-trait-not-in-scope.rs:25:31
   |
 7 |         fn summary(&self) -> String;
   |            ------- the method is available for `AnimeSeries` here
...
10 |     pub struct AnimeSeries {
   |     ---------------------- method `summary` not found for this struct
...
25 |     println!("{}", death_note.summary());
   |                               ^^^^^^^ method not found in `AnimeSeries`
   |
   = help: items from traits can only be used if the trait is in scope
help: trait `Summarize` which provides `summary` is implemented but not in scope; perhaps you want to import it
   |
 5 + use crate::catalog::Summarize;
   |

For more information about this error, try `rustc --explain E0599`.
```

**کامپایلر به چه اعتراض دارد:** `AnimeSeries` کاملاً `pub` است، و کامپایلر خودش می‌گوید «این متد برایِ `AnimeSeries` هست» — ولی چون `Summarize` هیچ‌جا importنشده، اجازه نمی‌دهد صدایش بزنی. حتی دقیقاً می‌گوید کدام `use` را کم داری.

**راه‌حل:** پیشنهادِ خودِ کامپایلر را اضافه کن:

```rust
use catalog::Summarize;
```

**چرا این راه‌حل است:** وجودِ متد کافی نیست؛ باید صفتی که آن متد رویش تعریف شده هم در دامنه باشد. این قاعده حتی وقتی نوع را از یک ماژولِ دیگر بگیری هم صدق می‌کند — و همان چیزی است که پیامِ کامپایلر مستقیم به‌ات می‌گوید.

---

## تمرین

### گرم‌کردن

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
trait Greet {
    fn hello(&self) -> String;
}
```

</details>

<details>
<summary>پاسخ</summary>

بله. یک trait به‌تنهایی هیچ پیاده‌کننده‌ای لازم ندارد. اینجا هیچ نوعی هنوز قولِ `Greet` را نداده — و این کاملاً بی‌اشکال است؛ Rust چیزی را مجبور نمی‌کند این قرارداد را قبول کند.

</details>

<details>
<summary>این چه چاپ می‌کند؟</summary>

```rust
trait Greet {
    fn name(&self) -> String;
    fn hello(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}

struct Robot;

impl Greet for Robot {
    fn name(&self) -> String {
        "R2".to_string()
    }
}

println!("{}", Robot.hello());
```

</details>

<details>
<summary>پاسخ</summary>

```text
Hello, R2!
```

`Robot` هیچ‌وقت `hello` را خودش ننوشت، پس پیش‌فرض اجرا می‌شود — که متدِ `name` را صدا می‌زند، و آن یکی نسخه‌ی `Robot` است.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
trait Greet {
    fn name(&self) -> String;
    fn hello(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}

struct Cat;

impl Greet for Cat {
    fn hello(&self) -> String {
        "meow".to_string()
    }
}
```

</details>

<details>
<summary>پاسخ</summary>

نه. `Cat` تصمیم گرفت `hello` (که پیش‌فرض دارد) را بازنویسی کند — این کاملاً مجاز است. ولی هیچ‌جا `name` (که اجباری است) را ننوشت، و بازنویسیِ متدِ اختیاری، متدِ اجباری را جایگزین نمی‌کند. خطایی شبیهِ همان چیزی که در «خطاهایی که خواهی دید» دیدی می‌گیری.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
mod shapes {
    pub trait Area {
        fn area(&self) -> f64;
    }

    pub struct Square {
        pub side: f64,
    }

    impl Area for Square {
        fn area(&self) -> f64 {
            self.side * self.side
        }
    }
}

fn main() {
    let sq = shapes::Square { side: 3.0 };
    println!("{}", sq.area());
}
```

</details>

<details>
<summary>پاسخ</summary>

نه. `Square` کاملاً `pub` است و می‌توانی بسازیش، ولی `Area` هیچ‌جا importنشده — نه با `use shapes::Area;` و نه با هیچ چیزِ دیگری.

</details>

### تعمیر

هر دو مثالِ خراب را درست کن:

۱. `examples/04-missing-required-method.rs` را طوری درست کن که کامپایل شود — با نوشتنِ متدِ `title` که جا افتاده، نه با پاک‌کردنِ چیزِ دیگری.
۲. `examples/05-trait-not-in-scope.rs` را با یک `use` درست کن. بعد امتحان کن: اگر همان `use` را بیرونِ `fn main` ولی همچنان تویِ همان فایل بگذاری، هنوز کار می‌کند؟

### پیاده‌سازی

چهار بخش تویِ `src/lib.rs`:

```sh
cargo test -p p2-03-01-defining-and-implementing-traits
```

خودِ `trait Summarize` از قبل کامل نوشته شده — دقیقاً همان چیزی که تویِ «مفهوم» دیدی. کارِ تو پرکردنِ بلاک‌هایِ `impl` است: `AnimeSeries`، `MangaVolume`، یک نوعِ تازه به اسمِ `GameTitle`، و یک تابعِ ساده‌ی `shelf_summary`. فرمتِ دقیقِ هر `summary`ِ بازنویسی‌شده، بالایِ همان متد، تویِ کامنتِ مستندات نوشته شده — چیزی حدس نزن.

### بساز

یک صفتِ **تازه** بساز — نه `Summarize` — با دستِ‌کم یک متدِ اجباری و یک متدِ پیش‌فرض. برایِ دستِ‌کم یکی از نوع‌هایی که همین الان تویِ `src/lib.rs` داری (یا یک نوعِ کاملاً تازه‌ی خودت) پیاده‌سازی‌اش کن، و دستِ‌کم یک تست برایش بنویس.

ایده، اگر لازمش داری: یک `Rateable` با یک متدِ اجباری که یک عدد بین ۰ تا ۵ برمی‌گرداند، و یک متدِ پیش‌فرض که آن عدد را به رشته‌ای از `★` تبدیل کند.

### چالش (اختیاری)

یک ساختارِ تازه بساز، `Playlist`، با یک فیلدِ `pub items: Vec<AnimeSeries>`. `Summarize` را برایش پیاده‌سازی کن: `title` چیزی مثلِ «Playlist (تعدادِ items)» برمی‌گرداند، و `summary` خلاصه‌یِ هر آیتم را — با فراخوانیِ `summary` رویِ تک‌تکِ آن‌ها — کنارِ هم می‌گذارد؛ جداکننده هرچه خودت تصمیم بگیری، فقط تویِ کامنتِ مستنداتِ تابع بنویسش.

(این بخش جلوتر را نگاه می‌کند.) این فقط برایِ `Vec<AnimeSeries>` کار می‌کند — نه `Vec<MangaVolume>`، نه ترکیبی از هر دو. یک تابعِ واحد که برایِ هر برشی از هر نوعی که `Summarize` را دارد کار کند، دقیقاً همان چیزی است که جنریک‌ها ([۲.۳.۲](../02-generic-functions-and-structs/README.fa.md)) حل می‌کنند.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| صفت (trait) | قراردادی از امضایِ متد؛ بدونِ داده | هر جا چند نوعِ نامرتبط باید یک رفتار را به اشتراک بگذارند |
| متدِ اجباری | متدی تویِ trait بدونِ بدنه؛ هر پیاده‌کننده باید خودش بنویسدش | `title` تویِ این درس |
| متدِ پیش‌فرض | متدی تویِ trait با بدنه؛ رایگان، مگر بازنویسی شود | `summary` تویِ این درس |
| `impl Trait for Type` | نحوی که قرارداد را برایِ یک نوعِ ملموس پُر می‌کند | هر پیاده‌سازی تویِ این درس |
| importکردنِ صفت | آوردنِ trait به دامنه با `use`، تا متدهایش صدازدنی شوند | هر جا نوع از یک ماژولِ دیگر بیاید |

### الان می‌دانی

- یک trait فقط امضایِ متد است — بدونِ فیلد، بدونِ داده — و این همان چیزی است که آن را از یک کلاسِ پایه جدا می‌کند.
- متدی بدونِ بدنه اجباری است؛ متدی با بدنه پیش‌فرض است و اختیاری برایِ بازنویسی.
- بدنه‌ی پیش‌فرض می‌تواند متدهایِ اجباری‌ای را صدا بزند که هنوز هیچ نوعِ ملموسی پشتشان نیست — چون خودِ trait تضمینش کرده.
- دو نوعِ کاملاً نامرتبط، با پیاده‌سازیِ یک trait مشترک، بعدش با یک فراخوانیِ یکسان جواب می‌دهند — بدونِ ارث‌بری، بدونِ فیلدِ مشترک.
- کامپایلر هر بلاکِ پیاده‌سازیِ یک صفت را در زمانِ کامپایل بررسی می‌کند؛ یک متدِ اجباریِ جاافتاده، ساختِ برنامه را همان‌جا متوقف می‌کند.
- صدازدنِ متدِ یک trait به importشدنِ خودِ trait نیاز دارد — حتی وقتی نوع کاملاً `pub` و در دسترس است.

### بعداً کامل‌تر می‌بینی

- **توابع و ساختارهایِ جنریک، کراندشده به traitی که خودت نوشتی** — [۲.۳.۲ — توابع و ساختارهایِ جنریک](../02-generic-functions-and-structs/README.fa.md)
- **نوع‌هایِ وابسته — `Iterator`ِ ۲.۲.۴ از قبل یکی داشت، `type Item`** — [۲.۳.۵ — نوع‌هایِ وابسته](../05-associated-types/README.fa.md)
- **مشتق‌گرفتنِ خودکارِ traitهایِ استاندارد، به‌جایِ نوشتنِ دستیِ هر پیاده‌سازی** — [۲.۳.۴ — مشتق‌هایِ استاندارد، دستی پیاده‌سازی‌شده](../04-standard-derives-by-hand/README.fa.md)
- **شیءهایِ trait، و اینکه کامپایلر دقیقاً چطور تصمیم می‌گیرد کدام پیاده‌سازی اجرا شود** — [۲.۳.۷ — ارسالِ ایستا در برابرِ پویا](../07-static-vs-dynamic-dispatch/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا یک trait نمی‌تواند فیلد داشته باشد، مثلِ یک struct؟
- فرقِ متدِ اجباری و متدِ پیش‌فرض چیست، و کامپایلر از کجا می‌فهمد کدام‌یک را برایِ یک پیاده‌کننده‌یِ مشخص اجرا کند؟
- چرا بدنه‌ی پیش‌فرضِ `summary` کامپایل می‌شود، با اینکه متدِ `title` را صدا می‌زند و آن متد هیچ بدنه‌ای تویِ trait ندارد؟
- چرا `AnimeSeries` به فراخوانیِ `summary` جواب می‌دهد، با اینکه هیچ چیزِ مشترکی با `MangaVolume` ندارد؟
- چرا فراخوانیِ `summary` تویِ مثالِ «صفت باید در دامنه باشد» شکست خورد، با اینکه `AnimeSeries` کاملاً `pub` بود؟
- این را با یک ABC یا `Protocol` در پایتون مقایسه کن — دقیقاً کجا این تشبیه می‌شکند؟

---

## بیشتر

- [کتابِ Rust — Traits: تعریفِ رفتارِ مشترک](https://doc.rust-lang.org/book/ch10-02-traits.html) — همین زمین، رسمی و کامل.
- [مرجعِ Rust — Traits](https://doc.rust-lang.org/reference/items/traits.html) — تعریفِ دقیقِ نحو و قواعد.
- [Rust by Example — Traits](https://doc.rust-lang.org/rust-by-example/trait.html) — مثال‌هایِ بیشتر، کوتاه‌تر.
- [`rustc --explain E0046`](https://doc.rust-lang.org/error_codes/E0046.html) و [`rustc --explain E0599`](https://doc.rust-lang.org/error_codes/E0599.html) — توضیحِ رسمیِ هر دو خطایی که امروز دیدی.
