# ۲.۴.۴ — `Cow<'_, str>` و کپی‌هنگامِ‌نوشتن

## در یک نگاه

بعد از این درس می‌توانی:

- توضیح بدهی `Cow<'a, B>` دقیقاً چه چیزی است — یک شمارشیِ واقعی با دو حالت — و چرا معمولاً لازم نیست خودت رویش `match` بزنی.
- تابعی بنویسی که در حالتِ رایج هیچ کپی‌ای نمی‌گیرد و فقط وقتی واقعاً لازم است یک `String` تازه می‌سازد، طوری که صداکننده‌اش جز در هزینه هیچ فرقی حس نکند.
- بگویی `.to_mut()` دقیقاً کِی کپی می‌گیرد، و صادقانه بگویی `Cow` کِی این پیچیدگی را واقعاً ارزشش دارد و کِی ندارد.

**زمان:** حدود ۵۰ دقیقه · **پیش‌نیاز:**
[۲.۴.۲ — طول‌عمر در ساختارها و متدها](../02-lifetimes-in-structs-and-methods/README.fa.md)،
[۲.۴.۳ — `Deref`، `AsRef`، `Borrow`، `ToOwned`](../03-deref-asref-borrow/README.fa.md)

---

## چرا اهمیت دارد

یک تابع بنویس که یک `&str` می‌گیرد و باید «تمیزش» کند — فاصله‌های اضافه را جمع کند، یک پسوند را اضافه کند، هرچه. اکثرِ ورودی‌هایی که این تابع می‌بیند از قبل تمیزند؛ کاربرها معمولاً درست تایپ می‌کنند. حالا با یک انتخاب روبه‌رویی که در نگاهِ اول هیچ گزینه‌ی خوبی ندارد:

اگر امضایش `-> String` باشد، هر بار — حتی برای آن ورودیِ از قبل تمیز — باید یک `String` تازه بسازی، فقط برای اینکه چیزی برگردانده باشی که نوعش با ورودی یکی نیست. یک تخصیصِ کاملاً بی‌فایده، هر بار. اگر امضایش `-> &str` باشد، برای همان حالتِ رایج عالی است — رایگان، بدونِ کپی — ولی وقتی واقعاً باید چیزی را عوض کنی، دیگر نمی‌توانی: نسخه‌ی تغییریافته اصلاً دیگر همان `&str`ی که گرفته بودی نیست، جایی برای برگرداندنش نداری.

این دقیقاً همان جایی است که Rust یک راهِ سوم می‌گذارد. `Cow<'_, str>` — مخففِ **c**lone-**o**n-**w**rite (کپی‌هنگامِ‌نوشتن) — به تابع اجازه می‌دهد در حالتِ رایج همان نمایِ بدونِ‌کپی را برگرداند، و فقط در حالتی که واقعاً لازم است، یک `String` مالک بسازد. صداکننده جز در هزینه هیچ فرقی حس نمی‌کند — دقیقاً همان چیزی که در پایتون یا جنگو هیچ‌وقت مجبور نیستی رویش فکر کنی، چون جمع‌آورِ زباله همیشه یک آبجکتِ تازه برایت درست می‌کند و هزینه‌اش را از دیدت پنهان می‌کند. Rust این هزینه را پنهان نمی‌کند — و `Cow` دقیقاً ابزاری است برای اینکه تو هم مجبور نباشی همیشه بپردازیش.

این درس، ماژولِ ۲.۴ را می‌بندد. سه درسِ قبلی، هرکدام یک تکه از همین پازل را گذاشتند: [۲.۴.۱](../01-lifetime-basics-and-elision/README.fa.md) اسمِ صریحی به «یک قرض چقدر زنده می‌ماند» داد، [۲.۴.۲](../02-lifetimes-in-structs-and-methods/README.fa.md) نشان داد یک ساختار چطور اجازه دارد خودش یک ارجاع نگه دارد، و [۲.۴.۳](../03-deref-asref-borrow/README.fa.md) راهِ رفتن — ارزان — بینِ یک نمایِ قرضی و نسخه‌ی مالکش را باز کرد. امروز این سه‌تا کنارِ هم می‌نشینند تا یک چیزِ چهارم بسازند: نوعی که خودش تصمیم می‌گیرد آن تبدیل را کِی واقعاً انجام بدهد.

---

## مفهوم

### `Cow<'a, B>` دقیقاً چیست: یک شمارشیِ دو-حالته

مستنداتِ رسمیِ کتابخانه‌ی استاندارد `Cow` را با یک جمله معرفی می‌کنند: «یک اشاره‌گرِ هوشمندِ کپی‌هنگامِ‌نوشتن» (a clone-on-write smart pointer). این جمله از نظرِ رفتار درست است — به‌لطفِ `Deref`، دقیقاً مثلِ یک ارجاع به `B` رفتار می‌کند — ولی از نظرِ ساختار هیچ رازی ندارد. تعریفِ واقعی‌اش را باز کن:

```rust
pub enum Cow<'a, B>
where
    B: 'a + ToOwned + ?Sized,
{
    Borrowed(&'a B),
    Owned(<B as ToOwned>::Owned),
}
```

همین. یک شمارشیِ معمولی با دو گونه — دقیقاً همان جنس چیزی که [۱.۵.۳](../../../phase1-fundamentals/05-your-own-types/03-enums-as-data/README.fa.md) به تو یاد داد. هیچ اشاره‌گرِ خامی، هیچ trickِ پنهانی در کار نیست: یا یک ارجاعِ قرضی داری (`Borrowed`)، یا یک نسخه‌ی مالک (`Owned`). چون یک شمارشیِ واقعی است، می‌توانی رویش `match` بزنی، دقیقاً مثلِ هر شمارشیِ دیگری:

```rust
fn describe(value: &Cow<'_, str>) -> &'static str {
    match value {
        Cow::Borrowed(_) => "borrowed",
        Cow::Owned(_) => "owned",
    }
}
```

(به‌عمد `&Cow<'_, str>` گرفته، نه `&str` — clippy معمولاً برای یک پارامترِ `&Cow` پیشنهاد می‌دهد `&str` بگیری، و اکثرِ وقت‌ها همین پیشنهاد درست است. ولی اینجا کارِ تابع دقیقاً همین است که بفهمد کدام گونه است؛ یک `&str` ساده این اطلاعات را از قبل پاک کرده. برای همین امضا همین‌طور می‌ماند و بالای تابع یک `#[allow(clippy::ptr_arg)]` با یک کامنتِ توضیح می‌گذاریم — سکوت‌کردنِ یک هشدار وقتی می‌دانی چرا اشتباه می‌کند، خودش یک مهارت است.)

حالا این را بساز و صدا بزن:

```rust
let borrowed: Cow<str> = Cow::Borrowed("Matin");
let owned: Cow<str> = Cow::Owned(String::from("Matin"));

println!("borrowed: {borrowed:?} -> {}", describe(&borrowed));
println!("owned:    {owned:?} -> {}", describe(&owned));
```

```text
borrowed: "Matin" -> borrowed
owned:    "Matin" -> owned
```

یک نکته‌ی دقیق: خروجیِ `{:?}` فقط `"Matin"` است، نه `Borrowed("Matin")` یا `Owned("Matin")`. پیاده‌سازیِ `Debug` برایِ `Cow` مستقیم به مقدارِ داخلی‌اش عبور می‌کند، نه به اسمِ گونه‌اش — یعنی `{:?}` هیچ‌وقت به‌تنهایی نمی‌گوید کدام حالت داری؛ برایِ آن یا `match` می‌خواهی، یا دقیقاً همین کمکیِ `describe`.

با این حال، به‌ندرت لازم است دستی `match` بزنی، چون [۲.۴.۳](../03-deref-asref-borrow/README.fa.md) از قبل حلش کرده بود: `Cow<'a, B>` پیاده‌سازِ `Deref<Target = B>` است، پس هر متدی که رویِ `&B` کار می‌کند، مستقیم رویِ خودِ `Cow` هم کار می‌کند:

```rust
println!("borrowed.len(): {}", borrowed.len());
println!("owned.len():    {}", owned.len());
println!("borrowed == owned: {}", borrowed == owned);
```

```text
borrowed.len(): 5
owned.len():    5
borrowed == owned: true
```

`.len()` اینجا هیچ اسمی از `Cow` نمی‌داند — همان `str::len()` است، از پشتِ `Deref` رد شده. `borrowed` و `owned` هم با `==` مقایسه شدند، نه با یک متدِ مخصوصِ `Cow`؛ از دیدِ مقایسه، این دو مقدار برابرند چون هردو، از پشتِ همان `Deref`، به یک `str` یکسان می‌رسند.

### شکلِ انگیزه‌ساز: تابعی که معمولاً نیازی به تغییر ندارد

حالا مسئله‌ی «چرا اهمیت دارد» را واقعی کن: تابعی که فاصله‌های پیاپی را در یک متن به یک فاصله جمع می‌کند — یک پاکسازیِ متنیِ ساده، از همان جنسی که رویِ ورودیِ کاربر یا یک فایلِ پیکربندی زیاد می‌بینی.

```rust
fn collapse_spaces(input: &str) -> Cow<'_, str> {
    if !input.contains("  ") {
        return Cow::Borrowed(input);
    }
    let mut collapsed = String::with_capacity(input.len());
    let mut previous_was_space = false;
    for ch in input.chars() {
        if ch == ' ' && previous_was_space {
            continue;
        }
        previous_was_space = ch == ' ';
        collapsed.push(ch);
    }
    Cow::Owned(collapsed)
}
```

خطِ دوم کلِ نکته‌ی درس است: قبل از هر کاری، تابع یک بررسیِ ارزان می‌کند — آیا اصلاً دو فاصله‌ی پشتِ‌سرِهم وجود دارد؟ اگر نه، همان لحظه `input` را بدونِ هیچ کپی‌ای برمی‌گرداند. فقط وقتی واقعاً یک اجرا پیدا شود، وارد مسیرِ کندتر می‌شود و یک `String` تازه می‌سازد.

```senpai-visual
{"kind":"borrowing","labels":["&str input","two spaces in a row?","no: Cow::Borrowed, 0 copies","yes: build String","Cow::Owned, 1 copy"]}
```

حالا با یک ورودیِ تمیز و یک ورودیِ نامرتب صدایش بزن، و آدرسِ حافظه را — نه فقط محتوا را — مقایسه کن:

```rust
println!("clean out:  {clean_result:?}");
println!(
    "clean same address as input? {}",
    clean_result.as_ptr() == clean.as_ptr()
);
println!("messy out:  {messy_result:?}");
println!(
    "messy same address as input? {}",
    messy_result.as_ptr() == messy.as_ptr()
);
```

```text
clean out:  "Trigun: 26 episodes"
clean same address as input? true
messy out:  "Trigun: 26 episodes"
messy same address as input? false
```

این فقط یک ادعا نیست — یک اثبات است. برایِ ورودیِ تمیز، آدرسِ خروجی دقیقاً همان آدرسِ ورودی است: هیچ بایتی جابه‌جا نشده، هیچ حافظه‌ای تخصیص نگرفته. برایِ ورودیِ نامرتب، آدرس عوض شده — یک `String` واقعاً تازه ساخته شد. صداکننده هیچ‌کدامِ این دو حالت را در نوعِ برگشتی نمی‌بیند؛ هردو یک `Cow<'_, str>`اند و هردو، از پشتِ `Deref`، دقیقاً مثلِ یک `&str` رفتار می‌کنند. تنها جایی که فرق را می‌بینی، هزینه است.

### `.to_mut()`: جایی که کپی واقعاً اتفاق می‌افتد

تا اینجا `Cow`هایی ساختی که از اول مشخص بود کدام حالت‌اند. ولی مکانیزمِ واقعیِ «کپی‌هنگامِ‌نوشتن» یک متد است: `.to_mut()`. مستنداتِ خودش می‌گوید دقیقاً چه‌کار می‌کند: «یک ارجاعِ تغییرپذیر به نسخه‌ی مالکِ داده می‌گیرد. اگر داده از قبل مالک نبود، کلونش می‌کند.»

```senpai-visual
{"kind":"ownership","labels":["Cow::Borrowed(&str)",".to_mut() called","clone happens once","Cow::Owned(String)","later .to_mut() calls: no clone"]}
```

یک `Cow` را با `Borrowed` بساز و رد شو:

```rust
let source = String::from("Trigun");
let mut value: Cow<str> = Cow::Borrowed(&source);
println!(
    "before .to_mut(): borrowed? {}",
    matches!(value, Cow::Borrowed(_))
);

value.to_mut().push_str(" - 26 episodes");
println!(
    "after .to_mut():  borrowed? {}",
    matches!(value, Cow::Borrowed(_))
);
println!("value:  {value:?}");
println!("source: {source:?} (never touched)");
```

```text
before .to_mut(): borrowed? true
after .to_mut():  borrowed? false
value:  "Trigun - 26 episodes"
source: "Trigun" (never touched)
```

پیش از `.to_mut()`، `value` یک `Cow::Borrowed` بود — بدونِ هیچ کپی، فقط یک ارجاع به `source`. صدا زدنِ `.to_mut()` یک `&mut String` می‌دهد؛ برایِ اینکه این ارجاع را واقعاً بسازد، اول `source` را کلون کرد و آن کپی را داخلِ `value` گذاشت. بعدش `.push_str(...)` رویِ همان کپی نوشت — `source` دست‌نخورده ماند، دقیقاً همان‌طور که چاپش نشان می‌دهد.

یک نکته‌ی مهم‌تر: کلون شدن به لحظه‌ی *صدا زدنِ* `.to_mut()` بستگی دارد، نه به لحظه‌ای که واقعاً از آن `&mut` چیزی می‌نویسی. حتی اگر مقدارِ برگشتی را هیچ‌وقت استفاده نکنی، همان صدا زدن به‌تنهایی کافی است تا به `Owned` تبدیل شود — «کپی‌هنگامِ‌نوشتن» دقیق‌تر یعنی «کپی‌هنگامِ‌درخواستِ دسترسیِ‌قابلِ‌نوشتن»، نه هنگامِ نوشتنِ واقعی.

و صدا زدنِ دوباره؟ چون `value` از قبل `Owned` است، دیگر چیزی برایِ کلون‌شدن نیست:

```rust
let cloned_address = value.as_ptr();
let _ = value.to_mut(); // از قبل `Owned` است — چیزی برایِ کلون نمانده
println!(
    "a second .to_mut() moved the data? {}",
    value.as_ptr() != cloned_address
);
```

```text
a second .to_mut() moved the data? false
```

این دقیقاً همان چیزی است که اسمِ «کپی‌هنگامِ‌نوشتن» را توجیه می‌کند: کپی، دقیقاً یک‌بار، فقط سرِ همان گذارِ اولِ `Borrowed` به `Owned` اتفاق می‌افتد.

### پیوند به ۲.۴.۳: `Deref` و `ToOwned` در عمل

`Cow` هیچ مکانیزمِ تازه‌ای اختراع نکرده — دقیقاً همان دو صفتی است که [۲.۴.۳](../03-deref-asref-borrow/README.fa.md) نشانت داد، این‌بار سرِ کار. اول `Deref`: چون `Cow<'a, B>: Deref<Target = B>` است، یک تابعی که `&B` می‌خواهد، یک `&Cow<'a, B>` را هم بدونِ هیچ تغییری قبول می‌کند — همان تبدیلِ ضمنی‌ای که [۲.۴.۳](../03-deref-asref-borrow/README.fa.md) برایِ `&String` نشان داد:

```rust
fn shout(input: &str) -> String {
    format!("{}!", input.to_uppercase())
}
```

```rust
let value: Cow<str> = Cow::Borrowed("trigun");
println!("{}", shout(&value));
```

```text
TRIGUN!
```

`shout` حتی اسمِ `Cow` را نمی‌داند؛ فقط `&str` می‌بیند.

دوم `ToOwned`: تعریفِ شمارشی بالا را دوباره نگاه کن — گونه‌ی `Owned` دقیقاً `<B as ToOwned>::Owned` را نگه می‌دارد، نه خودِ `B` را. برایِ `B = str`، این یعنی `String` — همان نوعی که `.to_owned()`یِ [۲.۴.۳](../03-deref-asref-borrow/README.fa.md) تحویل می‌دهد:

```rust
let built_with_to_owned: Cow<str> = Cow::Owned("trigun".to_owned());
let built_with_string_from: Cow<str> = Cow::Owned(String::from("trigun"));
println!(
    "same value either way: {}",
    built_with_to_owned == built_with_string_from
);
```

```text
same value either way: true
```

هر دو راه به یک نوع می‌رسند، چون هر دو راهِ رسیدن به همان `String`اند. این تصادفی نیست: `ToOwned` برایِ هر نوعِ قرضی‌ای که پیاده‌اش کند، همین رابطه را می‌گذارد — `[T]` به `Vec<T>`، `Path` به `PathBuf`، `OsStr` به `OsString` — و `Cow<'_, [T]>`، `Cow<'_, Path>` هم دقیقاً به همین شکل کار می‌کنند. امروز فقط با `str`/`String` سروکار داریم، ولی مکانیزم عمومی است.

### کِی `Cow` واقعاً ارزشش را دارد، کِی نه

`Cow` رایگان نیست — یک enum با دو گونه، یک `match` (یا `Deref`) هرجا استفاده‌اش می‌کنی، و یک نوعِ برگشتی که کمی پیچیده‌تر از یک `String` ساده به‌نظر می‌رسد. این پیچیدگی وقتی می‌ارزد که حالتِ «نیازی به تغییر نیست» واقعاً رایج باشد — طوری که جلوگیری از تخصیصِ آن حالت واقعاً به چیزی برسد.

وقتی نمی‌ارزد: تابعی که همیشه تخصیص می‌گیرد، مهم نیست چه ورودی‌ای بدهی. این را ببین:

```rust
fn tag_always(input: &str) -> Cow<'_, str> {
    Cow::Owned(format!("[{input}]"))
}

fn tag_always_plain(input: &str) -> String {
    format!("[{input}]")
}
```

```rust
for input in ["Trigun", "Blame!", ""] {
    let wrapped = tag_always(input);
    println!(
        "{input:?} -> {wrapped:?} (borrowed? {})",
        matches!(wrapped, Cow::Borrowed(_))
    );
}
```

```text
"Trigun" -> "[Trigun]" (borrowed? false)
"Blame!" -> "[Blame!]" (borrowed? false)
"" -> "[]" (borrowed? false)
```

هر سه بار `false`. گذاشتنِ کروشه دورِ یک رشته هیچ‌وقت نمی‌تواند از بایت‌هایِ خودِ ورودی استفاده کند — همیشه یک تخصیصِ تازه لازم است. برگرداندنِ `Cow` اینجا برایِ صداکننده فقط یک لایه‌ی اضافه است: باید یا رویش `Deref` کند یا `.into_owned()` بزند تا یک `String` واقعی بگیرد، بدونِ اینکه در ازایش حتی یک بایت کپی صرفه‌جویی شده باشد. `tag_always_plain` دقیقاً همان کار را می‌کند، با یک قراردادِ ساده‌تر. قاعده: `Cow` را وقتی انتخاب کن که واقعاً یک مسیرِ رایج و بدونِ‌تغییر داری که ارزشِ حفظ‌کردن دارد — نه چون «شاید یک‌جا به کارت بیاید».

---

## دست‌به‌کد

```sh
cargo run -p p2-04-04-cow-and-clone-on-write --example 01-cow-is-an-enum
cargo run -p p2-04-04-cow-and-clone-on-write --example 02-collapse-spaces-fast-and-slow-path
cargo run -p p2-04-04-cow-and-clone-on-write --example 03-to-mut-triggers-the-clone
cargo run -p p2-04-04-cow-and-clone-on-write --example 04-owned-is-exactly-toowned-owned
cargo run -p p2-04-04-cow-and-clone-on-write --example 05-when-cow-is-not-worth-it
```

بعد سه‌تای خراب:

```sh
cargo run -p p2-04-04-cow-and-clone-on-write --example 06-forgot-to-wrap-in-cow-broken --features broken
cargo run -p p2-04-04-cow-and-clone-on-write --example 07-owned-wants-string-broken --features broken
cargo run -p p2-04-04-cow-and-clone-on-write --example 08-to-mut-borrow-conflict-broken --features broken
```

بعد این‌ها را امتحان کن:

۱. در `01-cow-is-an-enum.rs`، یک سومینِ `Cow::Owned(String::from("Someone Else"))` بساز و با `borrowed` مقایسه‌اش کن. حدس بزن `==` چه چیزی می‌دهد، بعد چک کن.
۲. در `02-collapse-spaces-fast-and-slow-path.rs`، یک رشته با یک تب بینِ دو کلمه اضافه کن (نه فاصله). آیا هنوز `Borrowed` برمی‌گردد؟ چرا باید همین‌طور باشد؟
۳. در `03-to-mut-triggers-the-clone.rs`، خطِ اول را از `Cow::Borrowed(&source)` به `Cow::Owned(source.clone())` عوض کن. حالا `matches!(value, Cow::Borrowed(_))` قبل از `.to_mut()` چه چیزی می‌دهد، و چرا؟

---

## خطاهایی که خواهی دید

### `E0308` — فراموش‌کردنِ بسته‌بندی در `Cow`

```text
error[E0308]: mismatched types
  --> phase2-intermediate\04-lifetimes-and-conversion\04-cow-and-clone-on-write\examples\06-forgot-to-wrap-in-cow-broken.rs:11:5
   |
10 | fn passthrough(input: &str) -> Cow<'_, str> {
   |                                ------------ expected `Cow<'_, str>` because of return type
11 |     input
   |     ^^^^^ expected `Cow<'_, str>`, found `&str`
   |
   = note:   expected enum `Cow<'_, str>`
           found reference `&str`
help: try wrapping the expression in `std::borrow::Cow::Borrowed`
   |
11 |     std::borrow::Cow::Borrowed(input)
   |     +++++++++++++++++++++++++++     +

For more information about this error, try `rustc --explain E0308`.
```

**کامپایلر به چه اعتراض دارد:** `Cow<'_, str>` یک `&str` را به‌طورِ خودکار قبول نمی‌کند — برخلافِ `Deref`، که فقط در جهتِ «از `Cow` به `&str`» کار می‌کند، هیچ تبدیلِ ضمنی‌ای در جهتِ برعکس وجود ندارد. باید صریحاً بگویی کدام گونه را می‌خواهی.

**راه‌حل:** `Cow::Borrowed` را دورش بگذار:

```rust
fn passthrough(input: &str) -> Cow<'_, str> {
    Cow::Borrowed(input)
}
```

**چرا این راه‌حل است:** `input` همان چیزی است که باید برگردد — بدونِ هیچ تغییری — پس `Borrowed` دقیقاً همان گونه‌ای است که باید. خودِ کامپایلر هم همین پیشنهاد را می‌دهد.

### `E0271` — گونه‌ی `Owned` به `String` نیاز دارد، نه `&str`

```text
error[E0271]: type mismatch resolving `<str as ToOwned>::Owned == &str`
  --> phase2-intermediate\04-lifetimes-and-conversion\04-cow-and-clone-on-write\examples\07-owned-wants-string-broken.rs:12:27
   |
12 |     let value: Cow<str> = Cow::Owned("Trigun");
   |                           ^^^^^^^^^^^^^^^^^^^^ expected `&str`, found `String`

For more information about this error, try `rustc --explain E0271`.
```

**کامپایلر به چه اعتراض دارد:** `Cow::Owned` یک مقدار از نوعِ `<B as ToOwned>::Owned` می‌خواهد، نه از نوعِ `B`. برایِ `Cow<str>`، این یعنی `String`، نه `&str` — حتی اگر `&str` هم به‌نظر «به‌اندازه‌ی کافی مالک» برسد. (برچسب‌های expected/found را برعکسِ چیزی که حدس می‌زنی بخوان: `expected` همان چیزی است که واقعاً نوشتی — `"Trigun"` از نوعِ `&str`؛ `found` همان چیزی است که این جایگاه واقعاً لازم دارد — `String`.)

**راه‌حل:** یک `String` واقعی بده:

```rust
let value: Cow<str> = Cow::Owned("Trigun".to_owned());
```

**چرا این راه‌حل است:** `.to_owned()` دقیقاً همان چیزی می‌سازد که `<str as ToOwned>::Owned` می‌خواهد — یک `String`. `String::from("Trigun")` هم همین‌قدر درست است؛ هر دو به یک نوع می‌رسند، دقیقاً همان چیزی که زیربخشِ پیوند به ۲.۴.۳ نشانت داد.

### `E0502` — نگه‌داشتنِ ارجاعِ `.to_mut()` هم‌زمان با خواندنِ `Cow`

```text
error[E0502]: cannot borrow `value` as immutable because it is also borrowed as mutable
  --> phase2-intermediate\04-lifetimes-and-conversion\04-cow-and-clone-on-write\examples\08-to-mut-borrow-conflict-broken.rs:14:24
   |
13 |     let episodes = value.to_mut();
   |                    ----- mutable borrow occurs here
14 |     println!("before: {value}");
   |                        ^^^^^ immutable borrow occurs here
15 |     episodes.push_str(" - 26 episodes");
   |     -------- mutable borrow later used here

For more information about this error, try `rustc --explain E0502`.
```

**کامپایلر به چه اعتراض دارد:** `.to_mut()` یک `&mut self` می‌خواهد، و مقداری که برمی‌گرداند این قرضِ تغییرپذیر را تا وقتی که دوباره استفاده شود زنده نگه می‌دارد — دقیقاً همان قاعده‌ی هم‌نامی‌ای که از فازِ ۱ می‌شناسی: هر تعداد قرضِ اشتراکی، یا دقیقاً یک قرضِ تغییرپذیر، هرگز هردو با هم. `episodes` هنوز زنده است (چون خطِ ۱۵ دوباره استفاده‌اش می‌کند)، پس `println!` روی خطِ ۱۴ نمی‌تواند یک قرضِ اشتراکی از `value` بگیرد.

**راه‌حل:** بگذار قرضِ `.to_mut()` قبل از خواندنِ `value` تمام شود:

```rust
let mut value: Cow<str> = Cow::Borrowed("Trigun");
value.to_mut().push_str(" - 26 episodes");
println!("after: {value}");
```

**چرا این راه‌حل است:** اینجا مقدارِ `.to_mut()` هیچ‌وقت در یک متغیر ذخیره نمی‌شود — همان لحظه که `.push_str(...)` صدا زده می‌شود، قرض تمام می‌شود. تا وقتی `println!` می‌رسد، هیچ قرضِ تغییرپذیرِ زنده‌ای نمانده که با آن تصادم کند. این همان چیزی است که [۱.۳.۳](../../../phase1-fundamentals/03-borrowing-and-references/03-borrow-scopes-and-nll/README.fa.md) «دامنه‌ی قرض» نامیدش — قرض تا *آخرین استفاده*اش زنده است، نه تا پایانِ بلوک.

---

## تمرین

### گرم‌کردن

<details>
<summary>آیا این کامپایل می‌شود؟</summary>

```rust
use std::borrow::Cow;

let value: Cow<str> = Cow::Owned("Trigun");
```

</details>

<details>
<summary>پاسخ</summary>

نه. `Cow::Owned` یک `String` می‌خواهد، نه `&str` — `"Trigun"` از نوعِ `&str` است. خطا `E0271` است، همان چیزی که در «خطاهایی که خواهی دید» دیدی.

</details>

<details>
<summary>این چه چیزی چاپ می‌کند؟</summary>

```rust
let value: Cow<str> = Cow::Borrowed("Trigun");
println!("{value:?}");
```

</details>

<details>
<summary>پاسخ</summary>

```text
"Trigun"
```

نه `Borrowed("Trigun")`. `Debug` برایِ `Cow` مستقیم به مقدارِ داخلی عبور می‌کند، نه به اسمِ گونه.

</details>

<details>
<summary>آیا این کامپایل می‌شود؟</summary>

```rust
fn shorten(input: &str) -> Cow<'_, str> {
    input
}
```

</details>

<details>
<summary>پاسخ</summary>

نه. `Cow<'_, str>` یک `&str` را خودکار قبول نمی‌کند؛ باید صریحاً `Cow::Borrowed(input)` بنویسی.

</details>

<details>
<summary>بعد از اجرایِ این، <code>before</code> و <code>after</code> چه مقداری دارند؟</summary>

```rust
let mut value: Cow<str> = Cow::Borrowed("Trigun");
let before = matches!(value, Cow::Borrowed(_));
value.to_mut();
let after = matches!(value, Cow::Borrowed(_));
```

</details>

<details>
<summary>پاسخ</summary>

`before` برابرِ `true`، `after` برابرِ `false` — حتی با اینکه مقدارِ برگشتیِ `.to_mut()` هیچ‌وقت استفاده نشد. صرفِ *صدا زدن* `.to_mut()` کافی است تا کلون اتفاق بیفتد.

</details>

### تعمیر

هر سه مثالِ خراب را درست کن:

۱. `examples/06-forgot-to-wrap-in-cow-broken.rs` را با `Cow::Borrowed(input)` درست کن.
۲. `examples/07-owned-wants-string-broken.rs` را طوری درست کن که `Cow::Owned` یک `String` واقعی بگیرد — با `.to_owned()` یا `String::from`، هرکدام را دوست داری.
۳. `examples/08-to-mut-borrow-conflict-broken.rs` را طوری بازنویسی کن که قرضِ `.to_mut()` قبل از `println!` تمام شده باشد.

### پیاده‌سازی

سه تابع در `src/lib.rs`:

```sh
cargo test -p p2-04-04-cow-and-clone-on-write
```

- `describe` — می‌گوید یک `Cow` قرضی است یا مالک.
- `collapse_spaces` — تابعِ انگیزه‌سازِ همین درس: فاصله‌های پیاپی را جمع می‌کند، فقط وقتی واقعاً لازم است تخصیص می‌گیرد.
- `ensure_exclaimed` — مطمئن می‌شود یک `Cow<str>` به `'!'` ختم می‌شود، فقط وقتی واقعاً لازم است با `.to_mut()` چیزی اضافه می‌کند.

مشخصاتِ دقیق — از جمله رفتارِ عینیِ هر حالت — در کامنتِ مستنداتِ بالایِ هر تابع است.

### بساز

یک تابعِ دیگر، مالِ خودت، به همان شکلِ `collapse_spaces` بنویس: `pub fn نام_خودت(input: &str) -> Cow<'_, str>` که یک پاکسازیِ متنیِ ساده انجام می‌دهد و فقط وقتی واقعاً لازم است تخصیص می‌گیرد — مثلاً بریدنِ فاصله‌های ابتدا/انتهایِ رشته فقط اگر واقعاً وجود داشته باشند، یا جایگزینیِ یک کاراکترِ ممنوعه فقط اگر توی متن باشد. در یک کامنتِ بالای تابع بنویس چرا فکر می‌کنی حالتِ «نیازی به تغییر نیست» برایِ این تابع واقعاً رایج است — همان استدلالی که بخشِ «کِی `Cow` ارزشش را دارد» از تو خواست.

### چالش (اختیاری)

**بخشِ یک.** خودت (بدونِ اینکه به `src/lib.rs` اضافه‌اش کنی) بنویس: `fn to_static(input: &str) -> Cow<'static, str>`. با صدای بلند توضیح بده چرا `Cow::Borrowed(input)` هیچ‌وقت نمی‌تواند خروجیِ معتبرِ این تابع باشد، مهم نیست `input` چه باشد — و همین باعث می‌شود این تابع، برخلافِ `collapse_spaces`، همیشه در گونه‌ی `Owned` تمام شود.

**بخشِ دو.** (این یکی جلوتر را نگاه می‌کند.) `ToOwned` فقط مخصوصِ `str` نیست — `[T]` هم آن را پیاده می‌کند، با `Owned = Vec<T>`. یک نسخه‌ی کوچک از همین ایده را برایِ `Cow<'_, [i32]>` بنویس: تابعی که اعدادِ تکراریِ پشتِ‌سرِهم را فقط وقتی واقعاً وجود دارند حذف می‌کند، وگرنه برشِ ورودی را بدونِ تغییر برمی‌گرداند.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| `Cow<'a, B>` | شمارشیِ دو-حالته: `Borrowed(&'a B)` یا `Owned(<B as ToOwned>::Owned)` | تابعی که معمولاً فقط می‌خواند، گاهی باید بسازد |
| `Cow::Borrowed` | حالتِ بدونِ‌کپی — فقط یک ارجاع، هیچ تخصیصی نگرفته | مسیرِ رایج و بدونِ‌تغییر |
| `Cow::Owned` | حالتِ تخصیص‌گرفته — دقیقاً همان نوعی که `ToOwned::Owned` می‌سازد | حالتی که واقعاً به یک کپی نیاز داشت |
| `.to_mut()` | یک `&mut <B as ToOwned>::Owned` می‌دهد؛ فقط اگر هنوز `Borrowed` بود، یک‌بار کلون می‌کند | مکانیزمِ واقعیِ «کپی‌هنگامِ‌نوشتن» |
| کپی‌هنگامِ‌نوشتن (clone-on-write) | تعویقِ کپی‌گرفتن تا لحظه‌ای که واقعاً کسی بخواهد بنویسد | داده‌ای که بیشترِ وقت‌ها فقط خوانده می‌شود |

### الان می‌دانی

- `Cow<'a, B>` یک شمارشیِ واقعی با دو گونه است، نه یک ترفندِ اشاره‌گرِ هوشمند؛ `Deref` فقط باعث می‌شود به‌ندرت لازم باشد خودت رویش `match` بزنی.
- گونه‌ی `Owned` دقیقاً `<B as ToOwned>::Owned` را نگه می‌دارد — برایِ `str` یعنی `String` — نه یک `B` ساده.
- `.to_mut()` مکانیزمِ واقعیِ کلون است: فقط سرِ اولین گذار از `Borrowed` به `Owned` کپی می‌گیرد، و صرفِ صدا زدنش کافی است، حتی بدونِ نوشتنِ واقعی.
- برگرداندنِ `Cow` از یک تابع به آن اجازه می‌دهد در حالتِ رایج بدونِ کپی جواب بدهد و فقط در حالتِ لازم تخصیص بگیرد، بدونِ اینکه صداکننده جز در هزینه فرقی حس کند.
- `Cow` وقتی می‌ارزد که مسیرِ بدونِ‌تغییر واقعاً رایج باشد؛ برایِ تابعی که همیشه تخصیص می‌گیرد، فقط پیچیدگیِ اضافه است.

### بعداً کامل‌تر می‌بینی

- **`Rc<str>`/`Arc<str>` برایِ مالکیتِ واقعاً اشتراکی** — `Cow` مشکلِ «شاید لازم باشد خودم بسازمش» را حل می‌کند؛ وقتی مسئله‌ی واقعی این است که چند مالک باید همان یک مقدار را، ارزان، نگه دارند، آن یک مسئله‌ی دیگر است — [۲.۶.۳ — `Rc` و `Arc`](../../06-smart-pointers/03-rc-and-arc/README.fa.md).
- **سنجشِ اینکه این صرفه‌جویی واقعاً به دردت می‌خورد یا نه** — بخشِ «کِی ارزشش را دارد» با استدلال پیش رفت، نه با اندازه‌گیری؛ برایِ واقعاً سنجیدنش به‌جایِ حدس‌زدن — [۲.۷.۵ — سنجشِ کارایی با `criterion`](../../07-project-structure-and-testing/05-benchmarking-with-criterion/README.fa.md).

### می‌توانی توضیح بدهی؟

- دو گونه‌ی `Cow<'a, B>` را اسم ببر، و بگو چرا گونه‌ی `Owned` یک `B` ساده نگه نمی‌دارد.
- چرا صدا زدنِ `.to_mut()` برایِ دومین‌بار، وقتی `Cow` از قبل `Owned` است، هیچ کلونِ تازه‌ای نمی‌سازد؟
- چرا `describe` عمداً `&Cow<'_, str>` می‌گیرد، نه `&str` — برخلافِ پیشنهادِ پیش‌فرضِ clippy؟
- مسیرِ ماژول را دنبال کن: چطور اسم‌گذاریِ طول‌عمرِ یک قرض (۲.۴.۱) قدم‌به‌قدم به این می‌رسد که بتوانی یک کپی را تا لحظه‌ی واقعاً لازم عقب بیندازی (۲.۴.۴)؟
- یک مثالِ واقعی (یا قابلِ‌قبول) از تابعی بزن که در آن `Cow` ارزشِ پیچیدگی‌اش را ندارد، و بگو چرا.

---

## بیشتر

- [`std::borrow::Cow`](https://doc.rust-lang.org/std/borrow/enum.Cow.html) — مستنداتِ رسمی، همراه با فهرستِ کاملِ متدهایش.
- [`std::borrow::ToOwned`](https://doc.rust-lang.org/std/borrow/trait.ToOwned.html) — همان صفتی که ۲.۴.۳ نشانت داد، از زبانِ خودِ مستندات؛ فهرستِ کاملِ نوع‌هایی که پیاده‌اش کرده‌اند را هم همین‌جا می‌بینی.
- [ماژولِ `std::borrow`](https://doc.rust-lang.org/std/borrow/index.html) — دیدِ کلی روی رابطه‌ی `Borrow`، `ToOwned` و `Cow` با هم.
