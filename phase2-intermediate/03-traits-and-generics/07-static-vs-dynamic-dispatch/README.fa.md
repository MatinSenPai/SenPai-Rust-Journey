# ۲.۳.۷ — ارسالِ ایستا در برابرِ پویا، و ایمنیِ شیء

## در یک نگاه

بعد از این درس می‌توانی:

- `impl Trait` تویِ جایگاهِ پارامتر (فقط یک نوشتارِ دیگر برایِ همان کراندِ جنریک) را از `impl Trait` تویِ جایگاهِ خروجی (ابزاری واقعاً متفاوت که یک نوعِ ملموس را پنهان می‌کند) از هم تشخیص بدهی، و بگویی چرا هیچ‌کدام‌شان هرگز به vtable نیاز ندارد.
- برایِ یک امضایِ واقعی، بینِ یک پارامترِ جنریک و `dyn Trait` انتخاب کنی، و در یک جمله بگویی چه چیزی به‌دست می‌آوری و چه چیزی از دست می‌دهی.
- یک خطایِ «is not dyn compatible» را بخوانی، دقیقاً بگویی کدام متد مقصر است، و رفعش کنی.

**زمان:** حدود ۸۰ دقیقه · **پیش‌نیاز:**
[۲.۲.۱ — کلوژرها، `Fn`/`FnMut`/`FnOnce` و `move`](../../02-iterators-and-closures/01-closures-and-fn-traits/README.fa.md)،
[۲.۲.۴ — پیاده‌سازیِ `Iterator` و `IntoIterator` برای نوعِ خودت](../../02-iterators-and-closures/04-implementing-iterator/README.fa.md)،
[۲.۳.۱ — تعریف و پیاده‌سازیِ صفت‌ها](../01-defining-and-implementing-traits/README.fa.md)،
[۲.۳.۲ — توابع و ساختارهای جنریک، کران‌ها، `where`](../02-generic-functions-and-structs/README.fa.md)

---

## چرا اهمیت دارد

از وقتی صفت‌ها را شناختی، هر بار `item.summary()` یا هر متدِ دیگرِ یک صفت را صدا زدی، یک چیز هرگز نگفتی: کامپایلر این فراخوانی را دقیقاً کِی تصمیم می‌گیرد — هنگامِ کامپایل، یا هنگامِ اجرا؟ تا همین‌جا هیچ‌وقت اهمیتی نداشت، چون همیشه یک نوعِ ملموسِ مشخص را صدا می‌زدی. امروز اهمیت پیدا می‌کند، از دو راه:

اول، [۲.۲.۱](../../02-iterators-and-closures/01-closures-and-fn-traits/README.fa.md) یک وعده به تو داد و نگفت کِی برمی‌گردد سراغش. آن‌جا `make_adder` یک `impl Fn(i32) -> i32` برگرداند، و درس یک نکته گذاشت برایِ بعد: «`impl Trait` تویِ خروجی فقط یک نوعِ ملموس را قول می‌دهد؛ وقتی نوعِ واقعی بسته به شرط فرق می‌کند، به `Box<dyn Fn(i32) -> i32>` نیاز داری — چیزی که تویِ درسِ اشیایِ trait کاملش را می‌بینی.» آن درسِ اشیایِ trait همین درس است. امروز آن وعده را کامل ادا می‌کنیم.

دوم، به‌زودی به کدی می‌رسی که باید چند نوعِ *متفاوت* را — نه یک نوعِ ثابت — تویِ یک متغیرِ واحد نگه دارد: یک صف از هندلرهایِ متفاوت، یک لیستِ افزونه، یک مجموعه از چیزهایی که فقط در این مشترک‌اند که همه یک صفت را پیاده کرده‌اند. جنریک‌ها اینجا کم می‌آورند — نه چون خراب‌اند، بلکه چون از اساس برایِ کارِ دیگری ساخته شده‌اند. و وقتی سراغِ `dyn Trait` بروی تا این مشکل را حل کنی، گاهی به یک دیوار می‌خوری: کامپایلر می‌گوید صفتت «is not dyn compatible» و یک پاراگراف توضیح می‌دهد که اولین بار خواندنش گیج‌کننده است. این درس آن پاراگراف را برایت قابلِ‌فهم می‌کند.

این درس، بستنِ ماژولِ ۲.۳ است. هر شش درسِ قبلی — صفت‌ها، جنریک‌ها، `From`/`TryFrom`، مشتق‌های استاندارد، نوع‌های وابسته، ابرصفت‌ها — همه در همین‌جا کنارِ هم می‌نشینند: سؤالِ نهایی این نیست که «چطور یک صفت بنویسم؟»، بلکه این است که «وقتی صفتم را دارم، کامپایلر دقیقاً چطور از آن استفاده می‌کند؟»

---

## مفهوم

### یادآوری: جنریک‌ها تک‌ریخت می‌شوند

[۲.۳.۲](../02-generic-functions-and-structs/README.fa.md) قبلاً این را نشانت داد: یک تابعِ جنریک، برایِ هر نوعِ ملموسی که واقعاً به‌کارش می‌بری، یک کپیِ کامپایل‌شده‌ی جداگانه می‌گیرد — **تک‌ریختی‌سازی (monomorphization)**. امروز فقط یک صفت و دو نوع لازم داریم تا این را دوباره ببینیم، این‌بار با یک اثباتِ ملموس:

```rust
trait Summarize {
    fn summary(&self) -> String;
}
// AnimeSeries و MangaVolume هرکدام summary() خودشان را دارند —
// همان کاری که از ۲.۳.۱ بلدی.
```

یک تابعِ جنریک، دقیقاً به همان شکلی که [۲.۳.۲](../02-generic-functions-and-structs/README.fa.md) یاد داد:

```rust
fn announce<T: Summarize>(item: &T) -> String {
    format!("now: {}", item.summary())
}

println!("{}", announce(&anime));
println!("{}", announce(&manga));
```

```text
now: Trigun - 26 episodes
now: Blame! - 10 chapters
```

حالا اثبات: اگر کامپایلر واقعاً دو تابعِ جداگانه ساخته باشد، آن دو تا در دو نشانیِ متفاوت زندگی می‌کنند. آدرسِ خودِ تابع را، برایِ هرکدام از این دو نمونه‌سازی، بگیر و مقایسه کن:

```rust
let anime_fn = announce::<AnimeSeries> as usize;
let manga_fn = announce::<MangaVolume> as usize;
println!("same compiled function? {}", anime_fn == manga_fn);
```

```text
same compiled function? false
```

دو نشانیِ متفاوت. `announce::<AnimeSeries>` و `announce::<MangaVolume>` دو تابعِ واقعاً جدا از هم‌اند — نه یک تابعِ عمومی که هنگامِ اجرا تصمیم بگیرد. کامپایلر تویِ همان محلِ فراخوانی می‌داند دقیقاً کدام `summary` را صدا می‌زند، و اغلب می‌تواند آن فراخوانی را همان‌جا inline کند — بدونِ هیچ هزینه‌ی اضافه‌ای. این را **ارسالِ ایستا (static dispatch)** می‌نامند: «ایستا» یعنی همه‌چیز، هنگامِ کامپایل، ثابت و مشخص شده.

```senpai-visual
{"kind":"concept","labels":["generic call site","T = AnimeSeries","T = MangaVolume","two compiled functions"]}
```

### ارسالِ پویا: یک تابع، چند نوع

حالا همان کار را با یک نوشتارِ متفاوت انجام بده — به‌جایِ `T: Summarize`، بنویس `dyn Summarize`:

```rust
fn announce_dyn(item: &dyn Summarize) -> String {
    format!("now: {}", item.summary())
}

println!("{}", announce_dyn(&anime));
println!("{}", announce_dyn(&manga));
```

```text
now: Trigun - 26 episodes
now: Blame! - 10 chapters
```

همان خروجی — ولی این‌بار امضایِ تابع اصلاً `<T>` ندارد. چیزی برایِ نمونه‌سازی وجود ندارد، پس چیزی هم برایِ تک‌ریخت‌شدن نیست: `announce_dyn` **یک** تابعِ کامپایل‌شده است، همان‌طور که هست، و همان یک تابع هر دو نوع را بالا جواب داد.

این یعنی چه؟ `&dyn Summarize` یک **شیءِ صفتی (trait object)** است: ارجاعی به مقدارِ *یک* نوعِ ملموس که `Summarize` را پیاده کرده، در حالی که آن نوعِ ملموس پاک شده — کامپایلر فقط می‌داند «این، `Summarize` را پیاده می‌کند»، نه اینکه دقیقاً کدام ساختار است. برایِ اینکه فراخوانیِ `.summary()` هنوز درست کار کند، Rust کنارِ آن ارجاع یک چیزِ دوم هم نگه می‌دارد: یک **جدولِ متدهایِ مجازی (vtable)** — جدولِ کوچکی از اشاره‌گرهایِ تابع، یکی برایِ هر متدِ صفت، که به پیاده‌سازیِ همان نوعِ ملموسی اشاره می‌کند که این مقدارِ خاص واقعاً دارد.

این «یک ارجاع، به‌اضافه‌یِ یک اشاره‌گرِ دوم» را از قبل می‌شناسی: [۱.۳.۴](../../../phase1-fundamentals/03-borrowing-and-references/04-slices/README.fa.md) به آن گفت **اشاره‌گرِ چاق (fat pointer)** — یک برش هم دقیقاً همین است، فقط کلمه‌ی دومش طول است، نه vtable. `&dyn Summarize` هم یک اشاره‌گرِ چاق است: یک کلمه به دادهی واقعی، یک کلمه به vtable.

صدا زدنِ `.summary()` رویِ یک `dyn Summarize` یعنی: هنگامِ اجرا، اشاره‌گرِ vtable را از کنارِ داده بخوان، اشاره‌گرِ تابعِ `summary` را تویِ همان جدول پیدا کن، و بپر بهش. یک پرشِ اضافه — کوچک، ولی واقعی — در مقایسه با فراخوانیِ مستقیمی که کامپایلر از قبل می‌دانست کجاست. این را **ارسالِ پویا (dynamic dispatch)** می‌نامند.

```senpai-visual
{"kind":"concept","labels":["call through &dyn Trait","read the vtable pointer","find summary in the table","jump to the real function"]}
```

### ناهمگونی: کاری که یک جنریک هرگز نمی‌تواند

پاک‌شدنِ نوع، که بالا هزینه به‌نظر رسید، دقیقاً همان چیزی است که یک قابلیتِ تازه به‌ات می‌دهد. یک `Box<dyn Summarize>` را — یک `AnimeSeries` جعبه‌شده و یک `MangaVolume` جعبه‌شده — تویِ یک `Vec` بگذار:

```rust
let lineup: Vec<Box<dyn Summarize>> = vec![
    Box::new(anime),
    Box::new(manga),
];
for item in &lineup {
    println!("{}", item.summary());
}
```

```text
Trigun - 26 episodes
Blame! - 10 chapters
```

یک `Vec`، یک نوعِ عنصر (`Box<dyn Summarize>`)، دو ساختارِ واقعاً متفاوت زیرِ آن. این کاری است که هیچ `Vec<T>`ای با یک `T`ِ جنریکِ ثابت هرگز نمی‌تواند بکند — `T` فقط یک‌بار انتخاب می‌شود، برایِ کلِ `Vec`، و اینجا به دو تا نیاز داریم.

اینجا `Box` عمداً همین‌قدر گفته می‌شود که لازم است، نه بیشتر: یک جعبه‌یِ رویِ هیپ که هر چیزِ داخلش را به یک اشاره‌گرِ هم‌اندازه تبدیل می‌کند، همین. `AnimeSeries` و `MangaVolume` اندازه‌های متفاوتی دارند؛ `Box<dyn Summarize>` برایِ هردو یک اندازه‌ی ثابت است (یک اشاره‌گرِ چاق)، پس `Vec` می‌تواند کنارِ هم ردیفشان کند. داستانِ کاملِ `Box` — تخصیصِ هیپ، مالکیت، کِی به‌تنهایی سراغش بروی — مالِ [۲.۶.۱](../../06-smart-pointers/01-box-and-heap-allocation/README.fa.md) است؛ امروز فقط همین یک نقشش را لازم داریم.

بدونِ `Box`، این ایده اصلاً کامپایل نمی‌شود — دلیلش را در «خطاهایی که خواهی دید» می‌بینی.

### `impl Trait` در جایگاهِ پارامتر: فقط نوشتارِ دیگری برایِ همان کراند

[۲.۲.۱](../../02-iterators-and-closures/01-closures-and-fn-traits/README.fa.md) دقیقاً همین حرکت را، رویِ کلوژرها، زد: `apply_twice<F: Fn(i32) -> i32>(f: F, x: i32)` و `apply_twice_v2(f: impl Fn(i32) -> i32, x: i32)` — یک امضایِ کاملاً یکسان، با دو نوشتار. همان حرکت، رویِ `Summarize` هم کار می‌کند:

```rust
fn describe<T: Summarize>(item: &T) -> String {
    item.summary()
}
fn describe_v2(item: &impl Summarize) -> String {
    item.summary()
}
```

`describe` و `describe_v2` را با `&AnimeSeries` صدا بزن — هردو یک نسخه‌ی تک‌ریخت‌شده با `T = AnimeSeries` می‌گیرند، هردو ارسالِ ایستا هستند، و کامپایلر همان یک کد را از هردو تولید می‌کند. `impl Trait` تویِ جایگاهِ پارامتر **دقیقاً** کراندِ جنریک است، فقط با یک نوشتارِ کوتاه‌تر که نامِ `T` را حذف می‌کند. هیچ چیزِ تازه‌ای اینجا نیست — فقط تأییدی بر چیزی که [۲.۲.۱](../../02-iterators-and-closures/01-closures-and-fn-traits/README.fa.md) از قبل نشانت داد.

### `impl Trait` در جایگاهِ خروجی: پنهان‌کردنِ نوع

جایگاهِ خروجی داستانِ دیگری است. [۲.۲.۱](../../02-iterators-and-closures/01-closures-and-fn-traits/README.fa.md) `make_adder` را دقیقاً برایِ همین نوشت:

```rust
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}
```

نوعِ برگشتیِ واقعیِ یک کلوژر را هیچ‌وقت نمی‌توانی بنویسی — هیچ اسمی ندارد. اینجا جنریک کمکی نمی‌کند، چون نوعِ برگشتی را صداکننده انتخاب نمی‌کند؛ خودِ تابع باید تصمیم بگیرد. `impl Trait` تویِ خروجی همین را می‌گوید: «یک نوعِ ملموس، بی‌آنکه بگویم دقیقاً کدام.» و مهم‌ترین نکته: این هنوز **ارسالِ ایستا** است — فقط یک نوعِ ملموس وجود دارد، کامپایلر آن را می‌شناسد، فقط صداکننده اجازه ندارد اسمش را بداند.

همین ایده، رویِ چیزی که از قبل می‌شناسی — پیمایشگرِ Fibonacci از [۲.۲.۴](../../02-iterators-and-closures/04-implementing-iterator/README.fa.md):

```rust
fn fibonacci() -> impl Iterator<Item = u64> {
    Fibonacci { current: 0, next: 1 }
}

let first_eight: Vec<u64> = fibonacci().take(8).collect();
println!("{first_eight:?}");
```

```text
[0, 1, 1, 2, 3, 5, 8, 13]
```

نوعِ برگشتی می‌گوید «یک `Iterator` از `u64`»، نه «`Fibonacci`». صداکننده هیچ‌وقت نمی‌فهمد نوعِ واقعی چیست — و لازم هم نیست بفهمد؛ همه‌ی چیزی که [۲.۲.۴](../../02-iterators-and-closures/04-implementing-iterator/README.fa.md) رویِ `Fibonacci` رایگان به‌ات داد (`.take()`، `.filter()`، `.collect()`) همین‌جا هم رایگان کار می‌کند، چون هرچه که هست، واقعاً `Iterator` است. تنها چیزی که عوض شده، این است که دیگر مجبور نیستی نامِ `Fibonacci` را در امضایِ عمومیِ تابعت بنویسی — سودمند وقتی آن نوع پیاده‌سازیِ داخلی‌ای است که نمی‌خواهی متعهدش بمانی.

### وقتی `impl Trait` کم می‌آورد

[۲.۲.۱](../../02-iterators-and-closures/01-closures-and-fn-traits/README.fa.md) دقیقاً همین‌جا یک نکته برایِ بعد گذاشت: «`impl Trait` تویِ خروجی فقط یک نوعِ ملموس را قول می‌دهد؛ وقتی نوعِ واقعی بسته به شرط فرق می‌کند، به `Box<dyn Fn(i32) -> i32>` نیاز داری.» ببین دقیقاً کجا می‌شکند. دو کلوژر، یک شرط:

```rust
fn make_adjuster(bonus: bool, n: i32) -> impl Fn(i32) -> i32 {
    if bonus {
        move |x| x + n
    } else {
        move |x| x * n
    }
}
```

```text
error[E0308]: `if` and `else` have incompatible types
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\10-branch-mismatch-broken.rs:14:9
   |
11 | /     if bonus {
12 | |         move |x| x + n
   | |         --------------
   | |         |
   | |         the expected closure
   | |         expected because of this
13 | |     } else {
14 | |         move |x| x * n
   | |         ^^^^^^^^^^^^^^ expected closure, found a different closure
15 | |     }
   | |_____- `if` and `else` have incompatible types
   |
   = note: expected closure `{closure@phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\10-branch-mismatch-broken.rs:12:9: 12:17}`
              found closure `{closure@phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\10-branch-mismatch-broken.rs:14:9: 14:17}`
   = note: no two closures, even if identical, have the same type
   = help: consider boxing your closure and/or using it as a trait object
help: if you change the return type to expect trait objects, box the returned expressions
   |
12 ~         Box::new(move |x| x + n)
13 |     } else {
14 ~         Box::new(move |x| x * n)
   |
```

پیامِ کامپایلر دقیق است: «هیچ دو کلوژری، حتی اگر عینِ هم باشند، یک نوع ندارند» — هر کلوژر، نوعِ خودش را دارد که هیچ‌جا نوشته نمی‌شود، و `impl Fn(i32) -> i32` فقط می‌تواند **یکی** از آن‌ها را نام ببرد، نه «یکی از این دو، بسته به شرط.» راهِ حل دقیقاً همانی است که [۲.۲.۱](../../02-iterators-and-closures/01-closures-and-fn-traits/README.fa.md) قول داده بود — `Box<dyn Fn(i32) -> i32>`:

```rust
fn make_adjuster(bonus: bool, n: i32) -> Box<dyn Fn(i32) -> i32> {
    if bonus {
        Box::new(move |x| x + n)
    } else {
        Box::new(move |x| x * n)
    }
}

println!("{}", make_adjuster(true, 5)(1));
println!("{}", make_adjuster(false, 5)(1));
```

```text
6
5
```

این همان ارسالِ پویایِ بخشِ قبل است، فقط رویِ `Fn` به‌جایِ `Summarize`: هر دو شاخه حالا یک نوعِ مشترک برمی‌گردانند — `Box<dyn Fn(i32) -> i32>` — پس فرقی نمی‌کند کدام کلوژرِ واقعی زیرش باشد. قاعده‌ی کلی: `impl Trait` وقتی جواب می‌دهد که واقعاً یک نوعِ ملموس در کار باشد؛ همین که نوعِ واقعی به شرط، حلقه یا ورودی بستگی پیدا کرد، به `dyn Trait` نیاز داری.

### ایمنیِ شیء: چرا نه هر صفتی `dyn` می‌شود

تا اینجا هر صفتی که دیدیم راحت `dyn` شد. این همیشه درست نیست. یک صفت را در نظر بگیر که یک نسخه‌ی تازه از خودِ نوع برمی‌گرداند — کاملاً معمولی به‌نظر می‌رسد:

```rust
trait Spinoff {
    fn spinoff(&self) -> Self;
}
```

حالا یک تابع که `&dyn Spinoff` می‌گیرد:

```text
error[E0038]: the trait `Spinoff` is not dyn compatible
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\08-not-object-safe-self-by-value-broken.rs:26:21
   |
26 | fn announce(_item: &dyn Spinoff) {}
   |                     ^^^^^^^^^^^ `Spinoff` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\08-not-object-safe-self-by-value-broken.rs:11:26
   |
10 | trait Spinoff {
   |       ------- this trait is not dyn compatible...
11 |     fn spinoff(&self) -> Self;
   |                          ^^^^ ...because method `spinoff` references the `Self` type in its return type
   = help: consider moving `spinoff` to another trait
   = help: only type `AnimeSeries` implements `Spinoff`; consider using it directly instead.
```

**ایمنیِ شیء (object safety)** همین است: شرطی که یک صفت باید داشته باشد تا بتواند `dyn Trait` شود. کامپایلرِ خودت، تویِ همین پیام، اسمِ تازه‌تری هم به‌اش می‌دهد — «dyn compatible» — همان ایده، فقط نامِ رسمی‌ترش تویِ نسخه‌های جدیدِ Rust. هر دو اسم را می‌بینی؛ هر دو یک چیزند.

دلیلش رجوع می‌کند به همان vtable. یک `dyn Spinoff` نوعِ واقعیِ زیرش را پاک کرده — کامپایلر فقط می‌داند «این، `Spinoff` را پیاده می‌کند»، نه اینکه چقدر جا می‌گیرد. `spinoff(&self) -> Self` از vtable می‌خواهد یک مقدار را **با مقدار** برگرداند — یعنی جایی برایش رزرو کند — ولی هیچ‌کس نمی‌داند آن نوعِ پاک‌شده چقدر بزرگ است. یک slot تویِ یک جدول نمی‌تواند اندازه‌ای را قول بدهد که خودش نمی‌داند.

این تنها راهِ شکستن نیست. یک متدِ جنریک هم دقیقاً همین مشکل را، از زاویه‌ی دیگری، می‌سازد:

```rust
trait Rated {
    fn rating_as<T: From<u8>>(&self) -> T;
}
```

```text
error[E0038]: the trait `Rated` is not dyn compatible
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\09-not-object-safe-generic-method-broken.rs:24:21
   |
24 | fn announce(_item: &dyn Rated) {}
   |                     ^^^^^^^^^ `Rated` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\09-not-object-safe-generic-method-broken.rs:11:8
   |
10 | trait Rated {
   |       ----- this trait is not dyn compatible...
11 |     fn rating_as<T: From<u8>>(&self) -> T;
   |        ^^^^^^^^^ ...because method `rating_as` has generic type parameters
   = help: consider moving `rating_as` to another trait
   = help: only type `AnimeSeries` implements `Rated`; consider using it directly instead.
```

یک vtable **یک** جدولِ ثابت است، ساخته‌شده یک‌بار، پیش از آنکه هیچ صداکننده‌ای `T` را انتخاب کرده باشد. یک متدِ جنریک به یک slot جداگانه برایِ هر `T`ِ ممکن نیاز دارد — تعدادی که از پیش معلوم نیست، شاید بی‌نهایت. جدولی که نمی‌داند چند ردیف دارد، جدول نیست.

این دو، طبقِ گفته‌ی این درس، شایع‌ترین دلیلِ واقعیِ شکستنِ ایمنیِ شیء‌اند — نه فهرستِ کاملِ قانون‌ها. اگر واقعاً به یکی از این دو متد نیاز داری ولی می‌خواهی بقیه‌ی صفت همچنان `dyn`‌پذیر بماند، «چالش» زیر یک راهِ فرار واقعی نشانت می‌دهد.

```senpai-visual
{"kind":"concept","labels":["dyn Trait call","method returns Self by value","erased Self has no size","not dyn compatible"]}
```

### جدولِ تصمیم: جنریک یا `dyn`؟

همه‌چیزِ بالا، یک‌جا:

| محور | جنریک / `impl Trait` برنده است وقتی... | `dyn Trait` برنده است وقتی... |
|---|---|---|
| شکلِ مجموعه | همه‌ی عنصرها واقعاً یک نوعِ ثابت‌اند | به یک ترکیبِ واقعی از چند نوع، تویِ یک متغیر یا کالکشن، نیاز داری |
| سرعت | فراخوانی داغ است و باید inline شود | یک پرشِ vtable کنارِ بقیه‌ی کار، ناچیز است |
| حجمِ باینری | تعدادِ کمی نوعِ ملموس واقعاً به‌کار می‌رود | نوع‌های ملموسِ زیادی هر کدام یک کپیِ کاملِ مجزا می‌ساختند |
| خودِ صفت | همیشه در دسترس است | فقط اگر صفت ایمنِ شیء باشد — نه متدِ جنریک، نه بازگشتِ `Self` با مقدار |

یک قاعده‌یِ کوتاه‌تر، برایِ وقتی جدول یادت رفت: **پیش‌فرض، جنریک.** فقط وقتی که واقعاً به ناهمگونی نیاز داری — یا اندازه‌ی باینری واقعاً مشکل شده — سراغِ `dyn Trait` برو. فازِ ۳ همین معامله را دوباره نشانت می‌دهد: روترِ `axum` و رجیستریِ هندلرهایش دقیقاً به همین دلیل به `dyn Trait` تکیه می‌کنند — یک روتر باید تعدادِ زیادی نوعِ *متفاوت* از هندلر را تویِ یک کالکشنِ واحد نگه دارد، دقیقاً همان مسئله‌ی «ناهمگونی» که بالا دیدی.

---

## دست‌به‌کد

```sh
cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 01-static-two-compiled-functions
cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 02-dynamic-one-function-many-types
cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 03-heterogeneous-vec-box-dyn
cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 04-fibonacci-hidden-behind-impl-trait
cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 05-branch-fixed-with-box-dyn-fn
```

بعد پنج‌تای خراب:

```sh
cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 06-vec-dyn-no-box-broken --features broken
cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 07-missing-dyn-keyword-broken --features broken
cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 08-not-object-safe-self-by-value-broken --features broken
cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 09-not-object-safe-generic-method-broken --features broken
cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 10-branch-mismatch-broken --features broken
```

بعد این‌ها را امتحان کن:

۱. در `01-static-two-compiled-functions.rs`، یک سومین نوع بساز (مثلاً `LightNovel`)، `Summarize` را برایش پیاده کن، و آدرسِ `announce::<LightNovel>` را هم به مقایسه اضافه کن. هر سه آدرس با هم فرق دارند؟
۲. در `03-heterogeneous-vec-box-dyn.rs`، به‌جایِ `for item in &lineup`، طولِ همه‌ی `summary()`ها را با `.iter().map(...).sum()` جمع بزن — همان چیزی که `total_summary_length_dyn` تویِ `src/lib.rs` از تو می‌خواهد.
۳. در `05-branch-fixed-with-box-dyn-fn.rs`، یک شاخه‌ی سوم اضافه کن (مثلاً یک کلوژرِ تفریق‌کننده) و ببین آیا با یک `if`/`else if`/`else` هنوز `Box<dyn Fn(i32) -> i32>` کافی است — چرا باید باشد؟

---

## خطاهایی که خواهی دید

### `E0782` — یک صفتِ لختِ بدونِ `dyn`

```text
error[E0782]: expected a type, found a trait
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\07-missing-dyn-keyword-broken.rs:23:20
   |
23 | fn announce(item: &Summarize) -> String {
   |                    ^^^^^^^^^
   |
help: use a new generic type parameter, constrained by `Summarize`
   |
23 - fn announce(item: &Summarize) -> String {
23 + fn announce<T: Summarize>(item: &T) -> String {
   |
help: you can also use an opaque type, but users won't be able to specify the type parameter when calling the `fn`, having to rely exclusively on type inference
   |
23 | fn announce(item: &impl Summarize) -> String {
   |                    ++++
help: alternatively, use a trait object to accept any type that implements `Summarize`, accessing its methods at runtime using dynamic dispatch
   |
23 | fn announce(item: &dyn Summarize) -> String {
   |                    +++

For more information about this error, try `rustc --explain E0782`.
```

**کامپایلر به چه اعتراض دارد:** `&Summarize` انگار می‌گوید «ارجاعی به خودِ صفت» — ولی صفت یک نوع نیست، قراردادی برایِ نوع‌هاست. از نسخه‌ی ۲۰۲۱ به بعد، Rust مجبورت می‌کند بگویی دقیقاً چه می‌خواهی: یک `T`ِ جنریک، یک `impl Summarize`، یا یک `dyn Summarize`.

**راه‌حل:** یکی از سه پیشنهادِ خودِ کامپایلر را انتخاب کن؛ برایِ ارسالِ پویا، `&dyn Summarize`.

**چرا این راه‌حل است:** این خطا خودش خلاصه‌ی کل درس است — همان سه گزینه‌ای که تویِ «مفهوم» دیدی، این‌بار از زبانِ خودِ کامپایلر. هیچ‌کدام «درست‌تر» نیست؛ انتخاب به این بستگی دارد که یک نوعِ ثابت کافی است یا به ناهمگونی نیاز داری.

### `E0277` — یک `Vec<dyn Summarize>` اندازه‌ی معلوم ندارد

```text
error[E0277]: the size for values of type `(dyn Summarize + 'static)` cannot be known at compilation time
   --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\06-vec-dyn-no-box-broken.rs:14:29
    |
 14 | fn total_summaries(_lineup: &Vec<dyn Summarize>) {}
    |                             ^^^^^^^^^^^^^^^^^^^ doesn't have a size known at compile-time
    |
    = help: the trait `Sized` is not implemented for `(dyn Summarize + 'static)`
note: required by an implicit `Sized` bound in `Vec`
   --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\alloc\src\vec\mod.rs:438:16
    |
438 | pub struct Vec<T, #[unstable(feature = "allocator_api", issue = "32838")] A: Allocator = Global> {
    |                ^ required by the implicit `Sized` requirement on this type parameter in `Vec`

For more information about this error, try `rustc --explain E0277`.
```

**کامپایلر به چه اعتراض دارد:** آرایه‌ی زیربناییِ یک `Vec` عنصرهایش را پشتِ‌سرِهم، به‌صورتِ خطی، ذخیره می‌کند — و برایِ این کار باید از قبل بداند هر عنصر دقیقاً چند بایت است. `dyn Summarize` به‌تنهایی این را نمی‌گوید: `AnimeSeries` و `MangaVolume` اندازه‌های متفاوتی دارند، و «هر نوعی که `Summarize` را پیاده کند» می‌تواند هر اندازه‌ای باشد.

**راه‌حل:** نوعِ عنصر را `Box<dyn Summarize>` کن، نه `dyn Summarize` لخت:

```rust
fn total_summaries(lineup: &Vec<Box<dyn Summarize>>) -> usize {
    lineup.len()
}
```

**چرا این راه‌حل است:** یک `Box`، فارغ از اینکه چه چیزی را تویِ خودش جا داده، همیشه یک اندازه‌ی ثابت دارد — یک اشاره‌گر، به یک جایگاهِ رویِ هیپ. `Box<dyn Summarize>` از دیدِ `Vec` یک نوعِ عنصرِ کاملاً معمولی و باندازه است؛ فقط نمی‌داند سرِ دیگرِ هر اشاره‌گر دقیقاً چیست. به همین دلیل است که `dyn Trait` تقریباً همیشه پشتِ یک اشاره‌گر ظاهر می‌شود (`Box<dyn T>`، `&dyn T`، `Rc<dyn T>`)، نه لخت.

### `E0308` — `if`/`else` با `impl Trait` هم‌نوع نمی‌مانند

```text
error[E0308]: `if` and `else` have incompatible types
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\10-branch-mismatch-broken.rs:14:9
   |
11 | /     if bonus {
12 | |         move |x| x + n
   | |         --------------
   | |         |
   | |         the expected closure
   | |         expected because of this
13 | |     } else {
14 | |         move |x| x * n
   | |         ^^^^^^^^^^^^^^ expected closure, found a different closure
15 | |     }
   | |_____- `if` and `else` have incompatible types
   |
   = note: expected closure `{closure@phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\10-branch-mismatch-broken.rs:12:9: 12:17}`
              found closure `{closure@phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\10-branch-mismatch-broken.rs:14:9: 14:17}`
   = note: no two closures, even if identical, have the same type
   = help: consider boxing your closure and/or using it as a trait object
help: you could change the return type to be a boxed trait object
   |
10 - fn make_adjuster(bonus: bool, n: i32) -> impl Fn(i32) -> i32 {
10 + fn make_adjuster(bonus: bool, n: i32) -> Box<dyn Fn(i32) -> i32> {
   |
help: if you change the return type to expect trait objects, box the returned expressions
   |
12 ~         Box::new(move |x| x + n)
13 |     } else {
14 ~         Box::new(move |x| x * n)
   |

For more information about this error, try `rustc --explain E0308`.
```

**کامپایلر به چه اعتراض دارد:** دو کلوژر، حتی با یک امضایِ کاملاً یکسان، دو نوعِ کاملاً متفاوت‌اند — هرکدام نوعِ بی‌نامِ خودش را دارد. `impl Fn(i32) -> i32` قول می‌دهد «یک نوعِ ملموسِ ثابت»، ولی این تابع بسته به `bonus` دو نوعِ متفاوت برمی‌گرداند.

**راه‌حل:** خروجی را `Box<dyn Fn(i32) -> i32>` کن و هر دو کلوژر را جعبه کن — دقیقاً پیشنهادِ خودِ کامپایلر.

**چرا این راه‌حل است:** `Box<dyn Fn(i32) -> i32>` یک نوعِ **واحد** است که هر دو کلوژر می‌توانند به آن تبدیل شوند (پاک‌شدنِ نوع، دقیقاً مثلِ `Box<dyn Summarize>`). حالا هر دو شاخه‌ی `if` واقعاً یک نوع برمی‌گردانند، و تصمیمِ اینکه کدام کلوژر بود، به‌جایِ کامپایل، به زمانِ اجرا موکول می‌شود.

### `E0038` — یک صفت که `Self` را با مقدار برمی‌گرداند، «dyn compatible» نیست

```text
error[E0038]: the trait `Spinoff` is not dyn compatible
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\08-not-object-safe-self-by-value-broken.rs:26:21
   |
26 | fn announce(_item: &dyn Spinoff) {}
   |                     ^^^^^^^^^^^ `Spinoff` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\08-not-object-safe-self-by-value-broken.rs:11:26
   |
10 | trait Spinoff {
   |       ------- this trait is not dyn compatible...
11 |     fn spinoff(&self) -> Self;
   |                          ^^^^ ...because method `spinoff` references the `Self` type in its return type
   = help: consider moving `spinoff` to another trait
   = help: only type `AnimeSeries` implements `Spinoff`; consider using it directly instead.

For more information about this error, try `rustc --explain E0038`.
```

**کامپایلر به چه اعتراض دارد:** پیامِ خودش دقیق است — `spinoff` نوعِ `Self` را تویِ خروجی‌اش استفاده کرده. پشتِ `dyn Spinoff`، `Self` پاک شده و اندازه‌اش معلوم نیست؛ یک vtable نمی‌تواند قول بدهد مقداری با اندازه‌ی نامعلوم برمی‌گرداند.

**راه‌حل:** یا این متد را از صفتی که قرار است `dyn` شود بیرون ببر (پیشنهادِ خودِ کامپایلر)، یا یک نوعِ ملموس یا جعبه‌شده به‌جایِ `Self` برگردان — مثلاً `Box<dyn Spinoff>`.

**چرا این راه‌حل است:** مشکل خودِ ایده‌ی «یک نسخه‌ی تازه از خودم بساز» نیست — مشکل این است که «خودم» پشتِ `dyn` دیگر یک اندازه‌ی مشخص نیست. یک `Box<dyn Spinoff>` برمی‌گردانی، مسئله حل می‌شود، چون `Box` — درست مثلِ بخشِ «ناهمگونی» — همیشه اندازه‌ای ثابت دارد.

### `E0038` — یک متدِ جنریک هم «dyn compatible» را می‌شکند

```text
error[E0038]: the trait `Rated` is not dyn compatible
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\09-not-object-safe-generic-method-broken.rs:24:21
   |
24 | fn announce(_item: &dyn Rated) {}
   |                     ^^^^^^^^^ `Rated` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>
  --> phase2-intermediate\03-traits-and-generics\07-static-vs-dynamic-dispatch\examples\09-not-object-safe-generic-method-broken.rs:11:8
   |
10 | trait Rated {
   |       ----- this trait is not dyn compatible...
11 |     fn rating_as<T: From<u8>>(&self) -> T;
   |        ^^^^^^^^^ ...because method `rating_as` has generic type parameters
   = help: consider moving `rating_as` to another trait
   = help: only type `AnimeSeries` implements `Rated`; consider using it directly instead.

For more information about this error, try `rustc --explain E0038`.
```

**کامپایلر به چه اعتراض دارد:** این‌بار دلیل «references the Self type» نیست — «has generic type parameters» است. `rating_as::<T>` برایِ هر `T`ِ ممکن به یک ردیفِ جداگانه تویِ vtable نیاز دارد، ولی vtable یک‌بار، پیش از آنکه هیچ صداکننده‌ای `T` را انتخاب کرده باشد، ساخته می‌شود.

**راه‌حل:** این متد را از صفت بیرون ببر، یا آن را غیرِ جنریک کن (یک نوعِ برگشتیِ ثابت به‌جایِ `T`).

**چرا این راه‌حل است:** یک vtable نمی‌تواند تعدادِ نامعلومی slot داشته باشد. تنها راهِ اینکه `Rated` هنوز `dyn`‌پذیر بماند، این است که هر متدش دقیقاً یک امضایِ ثابت داشته باشد — نه یک خانواده‌ی نامتناهی از امضاها، یکی به‌ازایِ هر `T`.

---

## تمرین

### گرم‌کردن

<details>
<summary>یک تابعِ جنریک <code>f&lt;T: Trait&gt;</code> را دوبار، با دو نوعِ ملموسِ متفاوت صدا می‌زنی. کامپایلر چند نسخه از <code>f</code> می‌سازد؟</summary>

دو نسخه — یکی برایِ هر `T`. این همان تک‌ریختی‌سازی است: هر نمونه‌سازیِ جنریک، کپیِ کامپایل‌شده‌ی خودش را می‌گیرد.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
trait Summarize {
    fn summary(&self) -> String;
}
fn take_it(x: &Summarize) -> String {
    x.summary()
}
```

</details>

<details>
<summary>پاسخ</summary>

نه — `E0782`. `&Summarize` یک ارجاع به خودِ صفت نیست، بلکه باید بگویی چه می‌خواهی: `&dyn Summarize` برایِ ارسالِ پویا، یا `&impl Summarize` (یا یک `T` جنریک) برایِ ارسالِ ایستا.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
trait Summarize {
    fn summary(&self) -> String;
}
let items: Vec<dyn Summarize> = Vec::new();
```

</details>

<details>
<summary>پاسخ</summary>

نه — `E0277`. `dyn Summarize` اندازه‌ی ثابتی ندارد، و `Vec` باید اندازه‌ی هر عنصر را از قبل بداند. باید `Vec<Box<dyn Summarize>>` باشد.

</details>

<details>
<summary>یک صفت این متد را دارد: <code>fn make(&self) -> Self;</code>. آیا این صفت می‌تواند <code>dyn Trait</code> شود؟ چرا؟</summary>

نه. `make` نوعِ `Self` را با مقدار برمی‌گرداند؛ پشتِ `dyn`، `Self` پاک شده و اندازه‌اش معلوم نیست، پس یک vtable نمی‌تواند برایش جا رزرو کند. این دقیقاً همان چیزی است که خطایِ `E0038` رویِ `Spinoff` نشانت داد.

</details>

<details>
<summary>این چه چاپ می‌کند؟</summary>

```rust
fn seq() -> impl Iterator<Item = u64> {
    Fibonacci { current: 0, next: 1 }
}
println!("{:?}", seq().take(3).collect::<Vec<_>>());
```

</details>

<details>
<summary>پاسخ</summary>

```text
[0, 1, 1]
```

سه مقدارِ اولِ همان دنباله‌ای که [۲.۲.۴](../../02-iterators-and-closures/04-implementing-iterator/README.fa.md) ساخت — `impl Trait` تویِ خروجی فقط اسمِ نوع را پنهان می‌کند، رفتارش را عوض نمی‌کند.

</details>

### تعمیر

هر پنج مثالِ خراب را درست کن — نه با یک ترفندِ نحوی، بلکه با تغییرِ تصمیم:

۱. `examples/07-missing-dyn-keyword-broken.rs` را با اضافه‌کردنِ `dyn` درست کن (یکی از سه پیشنهادِ کامپایلر را انتخاب کن و بگو چرا همین یکی).
۲. `examples/06-vec-dyn-no-box-broken.rs` را با تغییرِ نوعِ عنصر به `Box<dyn Summarize>` درست کن.
۳. `examples/10-branch-mismatch-broken.rs` را با تغییرِ نوعِ برگشتی به `Box<dyn Fn(i32) -> i32>` و جعبه‌کردنِ هر دو کلوژر درست کن.
۴. `examples/08-not-object-safe-self-by-value-broken.rs` را طوری درست کن که `Spinoff` بدونِ `Self` با مقدار همچنان معنایِ خودش را داشته باشد — یا متد را جابه‌جا کن، یا برگشتی‌اش را عوض کن.
۵. `examples/09-not-object-safe-generic-method-broken.rs` را طوری درست کن که `Rated` دیگر متدِ جنریک نداشته باشد.

### پیاده‌سازی

چهار تابع در `src/lib.rs`، هرکدام یک تکه از این درس:

```sh
cargo test -p p2-03-07-static-vs-dynamic-dispatch
```

- `total_summary_length_generic` — ارسالِ ایستا، رویِ یک `T` ثابت.
- `total_summary_length_dyn` — همان حساب، رویِ اشیایِ صفتی.
- `lineup` — یک `Vec<Box<dyn Summarize>>` دو عضوی بساز.
- `fibonacci` — یک `Fibonacci` تازه پشتِ `impl Iterator<Item = u64>` برگردان.

مشخصاتِ دقیق — از جمله فرمتِ عینیِ خروجی هرکدام — در کامنتِ مستنداتِ بالایِ هر تابع است.

### بساز

یک صفتِ تازه، مالِ خودت، با دست‌کم دو نوعِ متفاوت که پیاده‌اش می‌کنند — هرچیزی که دوست داری (یک صفتِ `Playable` برایِ چند نوعِ رسانه، یک صفتِ `Notify` برایِ چند نوعِ پیام، هرچه). یک تابعِ جنریک **و** یک تابعِ مبتنی‌بر `dyn` برایش بنویس، هردو رویِ همان دو نوع کار کنند. در یک کامنت بنویس کدام‌یک را واقعاً تویِ کدِ واقعی انتخاب می‌کردی و چرا — و دقیقاً چه چیزی را با انتخابِ دیگر از دست می‌دادی.

### چالش (اختیاری)

صفتِ `Spinoff` از «خطاهایی که خواهی دید» را دوباره باز کن. این‌بار، به‌جایِ جابه‌جا کردنِ `spinoff` به یک صفتِ دیگر، رویِ همان متد بنویس `fn spinoff(&self) -> Self where Self: Sized;` — یک کراندِ اضافه، فقط رویِ همین یک متد. کامپایلش کن، و `&dyn Spinoff` را دوباره امتحان کن. آیا این‌بار کامپایل می‌شود؟ کدام متدها هنوز از پشتِ `dyn Spinoff` صدازدنی‌اند، و کدام نه؟ در یک یا دو جمله بگو `where Self: Sized` رویِ یک متدِ تکی دقیقاً چه قولی به کامپایلر می‌دهد که به کلِ صفت اجازه می‌دهد بازهم `dyn`‌پذیر بماند.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| ارسالِ ایستا (static dispatch) | تصمیمِ اینکه هر فراخوانی کدام تابع را هدف می‌گیرد، تماماً هنگامِ کامپایل | جنریک‌ها، `impl Trait` تویِ پارامتر یا خروجی |
| ارسالِ پویا (dynamic dispatch) | همان تصمیم، هنگامِ اجرا، از رویِ یک vtable | `&dyn Trait`، `Box<dyn Trait>` |
| شیءِ صفتی (trait object) | مقدارِ نوعی که یک صفت را پیاده کرده، با نوعِ ملموسش پاک‌شده | هر جا به ناهمگونی نیاز داری |
| جدولِ متدهایِ مجازی (vtable) | جدولِ اشاره‌گرهایِ تابع، کنارِ داده‌یِ شیءِ صفتی | چیزی که ارسالِ پویا هر فراخوانی از آن عبور می‌کند |
| ایمنیِ شیء (object safety / dyn compatibility) | شرطی که یک صفت باید داشته باشد تا `dyn Trait` شود | صفتی که متدِ جنریک یا بازگشتِ `Self` با مقدار دارد، این شرط را می‌شکند |
| `impl Trait` | تویِ پارامتر: کراندِ جنریک با نوشتارِ دیگر. تویِ خروجی: پنهان‌کردنِ یک نوعِ ملموسِ ثابت | هر دو، ارسالِ ایستا |

### الان می‌دانی

- یک تابعِ جنریک تک‌ریخت می‌شود — یک کپیِ کامپایل‌شده برایِ هر نوعِ ملموس؛ یک تابعِ مبتنی‌بر `dyn Trait` فقط یک کپی دارد و هنگامِ اجرا، از رویِ vtable، تصمیم می‌گیرد کدام پیاده‌سازی را صدا بزند.
- `Box<dyn Trait>` تویِ یک `Vec` کاری می‌کند که هیچ `Vec<T>`ای با `T`ِ ثابت نمی‌تواند: چند نوعِ ملموسِ واقعاً متفاوت را تویِ یک کالکشن نگه می‌دارد.
- `impl Trait` تویِ پارامتر دقیقاً همان کراندِ جنریک است؛ تویِ خروجی، ابزارِ متفاوتی است که یک نوعِ ملموسِ ثابت را پنهان می‌کند — و همین که آن نوع بسته به شرط عوض شود، دیگر کافی نیست.
- ایمنیِ شیء (یا «dyn compatibility»، اسمِ رسمی‌ترش تویِ خروجیِ کامپایلر) شرطی است که یک صفت باید داشته باشد تا `dyn Trait` شود؛ متدِ جنریک و بازگشتِ `Self` با مقدار، دو دلیلِ شایعِ شکستنِ آن‌اند.
- پیش‌فرض، جنریک؛ سراغِ `dyn Trait` برو وقتی به یک کالکشنِ واقعاً ناهمگون نیاز داری، یا حجمِ باینری از تک‌ریختی‌سازیِ زیاد نگرانت کرده.

### بعداً کامل‌تر می‌بینی

- **`Box<T>` و تخصیصِ هیپ، کامل** — این درس فقط از `Box` به‌عنوانِ ابزاری برایِ باندازه‌کردنِ یک شیءِ صفتی استفاده کرد؛ داستانِ کاملش — [۲.۶.۱ — `Box` و تخصیصِ هیپ](../../06-smart-pointers/01-box-and-heap-allocation/README.fa.md).
- **`Rc`/`Arc` برایِ مالکیتِ اشتراکی** — قدمِ بعدی، وقتی حتی یک مالکِ تنها (مثلِ `Box<dyn Trait>`) هم کافی نیست — [۲.۶.۳ — انواع `Rc` و `Arc`](../../06-smart-pointers/03-rc-and-arc/README.fa.md).
- **کراندِ `Send`/`Sync` رویِ اشیایِ صفتی** — کراندِ اضافه‌ای که یک `dyn Trait` برایِ عبور از مرزِ یک ریسه لازم دارد — [۲.۸.۴ — `Send` و `Sync`](../../08-concurrency/04-send-and-sync/README.fa.md).
- **متدهایِ `async` تویِ یک صفت** — چرا `async fn` تویِ صفت، سال‌ها با همین دیوارِ ایمنیِ شیء دست‌وپنجه نرم می‌کرد، و امروز چطور حلش می‌کنی — [۲.۹.۴ — صفت‌های async و `spawn_blocking`](../../09-async-in-practice/04-async-traits-and-blocking/README.fa.md).

### می‌توانی توضیح بدهی؟

- چرا یک تابعِ جنریک، برایِ هر نوعِ ملموس، یک کپیِ جداگانه می‌گیرد، ولی یک تابعِ مبتنی‌بر `&dyn Trait` فقط یک کپی دارد؟
- چرا `Vec<dyn Summarize>` کامپایل نمی‌شود ولی `Vec<Box<dyn Summarize>>` می‌شود؟
- چرا `impl Trait` تویِ پارامتر «فقط یک نوشتارِ دیگر» است، ولی تویِ خروجی یک ابزارِ واقعاً متفاوت؟
- بازگشتِ `Self` با مقدار و یک متدِ جنریک، هر دو ایمنیِ شیء را می‌شکنند — ولی از دو راهِ متفاوت. هرکدام را با کلماتِ خودت توضیح بده.
- برایِ سناریویِ «بساز»، توضیح بده چرا هم جنریک و هم `dyn Trait` رویِ همان دو نوع کار می‌کردند، ولی فقط یکی‌شان را انتخاب کردی؟

---

## بیشتر

- [فصلِ ۱۸.۲ کتابِ Rust — Using Trait Objects](https://doc.rust-lang.org/book/ch18-02-trait-objects.html) — همین موضوع، از زبانِ خودِ تیمِ Rust.
- [مرجعِ Rust — Dyn compatibility](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility) — فهرستِ کاملِ قانون‌هایِ ایمنیِ شیء؛ این درس فقط دو موردِ شایع‌ترینش را نشان داد.
- [`std::boxed::Box`](https://doc.rust-lang.org/std/boxed/struct.Box.html) — مستنداتِ رسمیِ همان ابزاری که امروز برایِ باندازه‌کردنِ اشیایِ صفتی به‌کار بردی.
- [RFC ۱۵۲۲ — Static and Dynamic Dispatch](https://rust-lang.github.io/rfcs/1522-conservative-impl-trait.html) — پیشنهادِ اصلیِ `impl Trait`، برایِ کسی که دوست دارد ریشه‌اش را ببیند.
