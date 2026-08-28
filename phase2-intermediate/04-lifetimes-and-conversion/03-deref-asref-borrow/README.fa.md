# ۲.۴.۳ — `Deref`، `AsRef`، `Borrow`، `ToOwned`

## در یک نگاه

بعد از این درس می‌توانی:

- صفتِ `Deref` و `DerefMut` را برایِ یک نوعِ پوششگرِ (newtype) خودت پیاده کنی، طوری که آن نوع هم سرِ فراخوانیِ متد و هم با عملگرِ `*` مثلِ نوعِ داخلی‌اش رفتار کند — و بگویی `Box`، `String` و `Vec` هم دقیقاً رویِ همین مکانیزم سوارند.
- یک تابع بنویسی که به‌جایِ یک نوعِ ثابت `impl AsRef<str>` می‌گیرد، و توضیح بدهی چرا صدازدنش با یک `&str`، یک `String`، یا (برایِ `AsRef<Path>`) یک `PathBuf`، هیچ هزینه‌ی اضافه‌ای برایِ صداکننده ندارد.
- بگویی چرا `Borrow<T>` قراردادی سخت‌گیرتر از `AsRef<T>` است، و دقیقاً بگویی اگر یک پیاده‌سازیِ `Borrow` با `Hash`/`Eq`ِ خودش هم‌خوان نباشد، چه چیزی تویِ یک `HashMap` خراب می‌شود.
- یک `&str` قرضی را با `ToOwned`، نه `Clone`، به یک `String` مالک تبدیل کنی، و بگویی چرا اصلاً `str` به `ToOwned` نیاز دارد.

**زمان:** حدود ۹۰ دقیقه · **پیش‌نیاز:**
[۲.۴.۲ — طول‌عمر در ساختارها و متدها](../02-lifetimes-in-structs-and-methods/README.fa.md)،
[۲.۳.۱ — تعریف و پیاده‌سازیِ صفت‌ها](../../03-traits-and-generics/01-defining-and-implementing-traits/README.fa.md)،
[۲.۳.۶ — ابرصفت‌ها و پیاده‌سازیِ فراگیر](../../03-traits-and-generics/06-supertraits-blanket-impls-orphan-rule/README.fa.md)،
[۲.۱.۲ — `HashMap` از نزدیک](../../01-collections/02-hashmap-in-depth/README.fa.md)

---

## چرا اهمیت دارد

از فازِ ۱ به این‌طرف، بارها یک تابع نوشته‌ای که `&str` می‌گیرد و بدونِ فکرکردن یک `&String` بهش داده‌ای — و کار کرده. واژه‌نامه‌ی این دوره از همان روزها یک اصطلاح برایِ این دارد: **تبدیلِ ضمنیِ ارجاع (Deref coercion)**، همان چیزی که کامپایلر سرِ فراخوانی `&String` را به `&str` تبدیل می‌کند. تا امروز فقط یک واقعیتِ پذیرفته‌شده بود. امروز می‌فهمی دقیقاً چه صفتی پشتِ آن است.

[۲.۱.۲](../../01-collections/02-hashmap-in-depth/README.fa.md) یک قدم جلوتر رفت: یک `HashMap<String, V>` ساختی، و بعد با یک لیترالِ `&str` ساده `.get()`ش کردی — بدونِ اینکه یک `String` تازه فقط برایِ همان جستجو بسازی. آن درس گفت این اتفاق یک استثنایِ خاصِ کدِ `HashMap` نیست؛ یک قاعده‌ی عمومی‌تر پشتش است، به اسمِ صفتِ `Borrow`، و از تو خواست فقط اعتماد کنی که کار می‌کند تا برسی به «فازِ ۲.۴» — همین‌جا. این درس دقیقاً همان وعده را ادا می‌کند: هم مکانیزمِ `Borrow` را نشانت می‌دهد، هم می‌گوید چرا این خیلی خاص‌ترِ auto-derefِ معمولی است.

این‌ها، به‌علاوه‌ی هر متدی که رویِ یک `Box<T>` یا یک `Vec<T>` صدا زده‌ای و هیچ‌وقت نپرسیدی «این متد مالِ کدام نوع است؟»، همه یک منبعِ مشترک دارند: یک مشتِ صفتِ کوچک تویِ کتابخانه‌ی استاندارد که هرکدام یک کارِ بسیار محدود انجام می‌دهند، ولی رویِ همه‌جایِ Rust سوارند. امروز چهار تا از این صفت‌ها را از دو زاویه می‌بینی: هم می‌فهمی چرا کاری که ماه‌هاست ازش استفاده می‌کنی کار می‌کند، هم یاد می‌گیری همان ارگونومی را برایِ نوع‌هایِ خودت بسازی.

---

## مفهوم

### `Deref`: این‌طوری یک نوع مثلِ اشاره‌گر رفتار می‌کند

یک پوششگرِ (newtype) ساده دورِ یک `Vec<String>`:

```rust
use std::ops::Deref;

struct Watchlist(Vec<String>);

impl Deref for Watchlist {
    type Target = Vec<String>;

    fn deref(&self) -> &Vec<String> {
        &self.0
    }
}
```

بدونِ این `impl`، `Watchlist` هیچ متدی ندارد — نه `.len()`، نه ایندکس‌کردن، هیچ‌چیز؛ `struct Watchlist(Vec<String>);` تنها یک تاپلِ یک‌فیلدی است، هرچند فیلدش یک `Vec` باشد. صفتِ `Deref` (the `Deref` trait) دقیقاً یک متد می‌خواهد: `deref(&self) -> &Self::Target`، جایی که `Target` می‌گوید «وقتی کسی از میانِ من رد شود، به چه نوعی می‌رسد؟» همین یک `impl`، همه‌چیز را عوض می‌کند:

```rust
let list = Watchlist(vec!["Frieren".to_string(), "Bocchi the Rock".to_string()]);

println!("count: {}", list.len());
println!("first: {}", list[0]);
println!("via *: {}", (*list).len());
```

```text
count: 2
first: Frieren
via *: 2
```

سه خط، سه شکلِ متفاوت از همین مکانیزم. `list.len()` کار می‌کند چون جست‌وجویِ متد اول رویِ `Watchlist` می‌گردد، `len` پیدا نمی‌کند، بعد کامپایلر خودش `*list` را امتحان می‌کند — یعنی `Deref::deref(&list)` را صدا می‌زند — و آن‌جا، رویِ `Vec<String>`، `len` را پیدا می‌کند. این را **بازکردنِ خودکارِ ارجاع (auto-deref)** می‌نامند، همان اصطلاحی که از قبل تویِ واژه‌نامه بود؛ حالا می‌دانی دقیقاً چه چیزی صدا می‌زند. `list[0]` هم همین راه را می‌رود، چون ایندکس‌کردن هم یک صفت است (`Index`) که `Watchlist` ندارد ولی `Vec<String>` دارد. و `(*list)` همان بازکردنِ ارجاعِ (dereference) صریح است — همان `*`ی که از [۱.۳.۱](../../../phase1-fundamentals/03-borrowing-and-references/01-shared-and-mutable-refs/README.fa.md) می‌شناسی، فقط این‌بار رویِ یک نوعِ خودت.

### `DerefMut`: همان مکانیزم، برایِ نوشتن

`Deref` فقط یک `&Target` می‌دهد — کافی برایِ خواندن، ولی نه برایِ چیزی مثلِ `.push()` که به `&mut self` نیاز دارد. صفتِ `DerefMut` (the `DerefMut` trait) دقیقاً همین را اضافه می‌کند، با یک متدِ دیگر:

```rust
impl DerefMut for Watchlist {
    fn deref_mut(&mut self) -> &mut Vec<String> {
        &mut self.0
    }
}
```

خودِ تعریفش، `trait DerefMut: Deref`، یک **ابرصفت (supertrait)** است — همان الگویی که در [۲.۳.۶](../../03-traits-and-generics/06-supertraits-blanket-impls-orphan-rule/README.fa.md) دیدی: نمی‌توانی `DerefMut` را بدونِ `Deref` پیاده کنی، چون `DerefMut` به همان `Target` که `Deref` تعریفش کرده تکیه دارد. حالا `Watchlist` می‌تواند نوشته هم بشود:

```rust
let mut list = Watchlist(vec!["Frieren".to_string()]);
list.push("Trigun".to_string());
println!("after push: {}", list.len());

let inner: &Vec<String> = &list;
println!("coerced: {inner:?}");
```

```text
after push: 2
coerced: ["Frieren", "Trigun"]
```

`list.push(...)` دقیقاً همان مسیرِ `list.len()` را می‌رود، فقط این‌بار کامپایلر به یک `&mut` نیاز دارد و آن را از `DerefMut::deref_mut` می‌گیرد. خطِ آخر یک نکته‌ی دیگر نشان می‌دهد: `&list` (یک `&Watchlist`) را مستقیم به جایی که یک `&Vec<String>` می‌خواهد داده‌ایم — بدونِ `*`، بدونِ فراخوانیِ صریحِ چیزی. این همان **تبدیلِ ضمنیِ ارجاع (Deref coercion)** است، این‌بار رویِ یک نوعِ خودت.

```senpai-visual
{"kind":"concept","labels":["list.push(x)","no push on Watchlist","auto-deref via DerefMut","push on Vec<String>"]}
```

### مکانیزمِ پشتِ چیزی که از فازِ ۱ می‌شناسی

آن `&String -> &str`ی که تویِ فازِ ۱ بارها دیدی، هیچ‌وقت جادو نبود — کتابخانه‌ی استاندارد یک‌بار `impl Deref<Target = str> for String` را نوشته، دقیقاً همان‌طور که بالا برایِ `Watchlist` نوشتی، و تو از همان روز اول از رویِ آن یک `impl` سواری گرفته‌ای:

```rust
fn print_str(s: &str) {
    println!("str: {s}");
}

fn print_slice(items: &[String]) {
    println!("slice len: {}", items.len());
}
```

```rust
let owned = String::from("hello");
print_str(&owned);

let list = Watchlist(vec!["Frieren".to_string(), "Bocchi the Rock".to_string()]);
print_slice(&list);
```

```text
str: hello
slice len: 2
```

خطِ دوم جالب‌تر است: `print_slice` یک `&[String]` می‌خواهد، ولی `&list` یک `&Watchlist` است. کامپایلر **دو** قدم بازکردنِ خودکار برمی‌دارد، نه یک قدم: اول `Watchlist -> Vec<String>` (همان `impl`ی که خودت نوشتی)، بعد `Vec<String> -> [String]` (یک `impl Deref` دیگر، این‌بار خودِ `Vec<T>` نوشته‌اش، که تویِ کتابخانه‌ی استاندارد است). تبدیلِ ضمنیِ ارجاع زنجیره‌ای است — هرقدر `impl Deref` پشتِ سرِ هم لازم باشد، همان‌قدر قدم برمی‌دارد.

```senpai-visual
{"kind":"concept","labels":["&Watchlist","hop 1: Deref to &Vec<String>","hop 2: Deref to &[String]","print_slice(&[String])"]}
```

`Box<T>` هم دقیقاً همین صفت را دارد — برایِ همین یک `Box<String>` مثلِ خودِ `String` رفتار می‌کند، بدونِ اینکه هیچ متدی رویش صریحاً نوشته باشی. داستانِ کاملِ `Box`، تخصیصِ هیپ و مالکیتش، مالِ [۲.۶.۱](../../06-smart-pointers/01-box-and-heap-allocation/README.fa.md) است؛ همین‌قدر کافی است بدانی: `Box`، `String` و `Vec`، هر سه، ارگونومیِ «مثلِ نوعِ داخلی‌ام رفتار کن» را از همین یک صفت می‌گیرند، نه از سه مکانیزمِ جدا.

### `AsRef<T>`: یک نمایِ ارزان به `&T`

`Deref` می‌گوید «من همه‌جا مثلِ `Target`م رفتار می‌کنم» — یک قرارداد سراسری برایِ خودِ نوع. گاهی چیزِ کوچک‌تری می‌خواهی: یک تابعِ به‌خصوص که فقط یک بار، فقط همان‌جا، بگوید «هرچیزی که ارزان به یک `&str` قابلِ‌دیدن‌شدن است، بده». این دقیقاً قراردادِ نمایِ ارزان (`AsRef<T>`) است — یک صفتِ جداگانه، با همین یک متد:

```rust
fn shout(name: impl AsRef<str>) -> String {
    format!("{}!", name.as_ref().to_uppercase())
}
```

همان‌طور که [۲.۳.۷](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.fa.md) نشانت داد، `impl AsRef<str>` تویِ جایگاهِ پارامتر دقیقاً یک کراندِ جنریک است، فقط با نوشتارِ کوتاه‌تر — می‌توانستی به‌جایش `fn shout<T: AsRef<str>>(name: T) -> String` بنویسی. یک بدنه، سه شکلِ فراخوانی:

```rust
println!("{}", shout("frieren"));

let owned = String::from("bocchi");
println!("{}", shout(&owned));
println!("still own it: {owned}");
println!("{}", shout(owned));
```

```text
FRIEREN!
BOCCHI!
still own it: bocchi
BOCCHI!
```

یک لیترالِ `&str`، یک `&String` قرضی، یک `String` مالک که مستقیم منتقل شده — هر سه، بدونِ هیچ تخصیصِ اضافه‌ای، از پسِ همان یک `.as_ref()` برمی‌آیند؛ `str` و `String` هر دو از قبل `AsRef<str>` را پیاده کرده‌اند. دقیقاً به همین دلیل است که این‌قدر توابعِ کتابخانه‌ی استاندارد به‌جایِ یک نوعِ ثابت، `impl AsRef<Path>` می‌گیرند — مثلِ `std::fs::read_to_string`. همان یک بدنه‌ی تابع، بدونِ هیچ تغییری، با یک `&str`، یک `String`، یا یک `PathBuf` کار می‌کند، چون هر سه‌شان `AsRef<Path>` را پیاده کرده‌اند — دقیقاً همان الگویی که بالا با `shout` دیدی، فقط رویِ `Path` به‌جایِ `str`.

### `Borrow<T>`: هم‌شکلِ `AsRef`، ولی قراردادی سخت‌گیرتر

این همان جایی است که [۲.۱.۲](../../01-collections/02-hashmap-in-depth/README.fa.md) وعده داد برمی‌گردد: قراردادِ قرضِ سخت‌گیر (`Borrow<T>`). تعریفِ خودِ `Borrow<T>` تقریباً کلمه‌به‌کلمه شبیهِ `AsRef<T>` است:

```rust
trait AsRef<T: ?Sized> {
    fn as_ref(&self) -> &T;
}

trait Borrow<T: ?Sized> {
    fn borrow(&self) -> &T;
}
```

فرق نه تویِ امضا، تویِ **قرارداد**ی است که کامپایلر هیچ‌وقت خودش چک نمی‌کند: مستنداتِ خودِ `Borrow` می‌گوید `Hash`، `Eq` و `Ord` باید بینِ فرمِ قرضی و فرمِ مالک هم‌خوان بمانند — `x.borrow() == y.borrow()` باید همان جوابی را بدهد که `x == y` می‌دهد. این دقیقاً همان صفتی است که `HashMap::get` رویش سوار است — امضایِ واقعی‌اش این‌طوری است: `fn get<Q>(&self, k: &Q) -> Option<&V> where K: Borrow<Q>, Q: Hash + Eq + ?Sized`. می‌توانی خودت یک نسخه‌ی کوچک از همین امضا بنویسی:

```rust
fn find_by_ref<'a, K, Q, V>(map: &'a HashMap<K, V>, key: &Q) -> Option<&'a V>
where
    K: Borrow<Q> + Hash + Eq,
    Q: Hash + Eq + ?Sized,
{
    map.get(key)
}
```

و همان `ratings`ی که [۲.۱.۲](../../01-collections/02-hashmap-in-depth/README.fa.md) ساخته بود، دوباره:

```rust
let mut ratings: HashMap<String, u8> = HashMap::new();
ratings.insert(String::from("Frieren"), 10);
ratings.insert(String::from("Bocchi the Rock"), 9);

println!("{:?}", find_by_ref(&ratings, "Frieren"));
```

```text
Some(10)
```

`String: Borrow<str>` است — کتابخانه‌ی استاندارد این `impl` را نوشته، دقیقاً چون یک `String` و یک `&str` که یک متن یکسان دارند، `Hash` و `Eq`ِ یکسانی هم می‌دهند. برایِ همین `find_by_ref(&ratings, "Frieren")` کامپایل می‌شود و کارِ درست را می‌کند: کلید را با یک لیترالِ `&str`، بدونِ ساختنِ یک `String` تازه، پیدا می‌کند.

حالا سؤالِ واقعی: چرا این قرارداد این‌قدر سخت‌گیرانه است؟ این نوع را ببین — یک کلید که ادعا می‌کند به‌بزرگی‌کوچکیِ حروف حساس نیست:

```rust
struct CiKey(String);

impl PartialEq for CiKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_lowercase() == other.0.to_lowercase()
    }
}
impl Eq for CiKey {}
```

```rust
impl Hash for CiKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_lowercase().hash(state);
    }
}

impl Borrow<str> for CiKey {
    fn borrow(&self) -> &str {
        &self.0
    }
}
```

`Hash` و `Eq`، هر دو، رشته را قبل از مقایسه کوچک می‌کنند — تا این‌جا منطقی است. ولی `borrow()` رشته‌ی **اصلی**، بدونِ کوچک‌کردن، برمی‌گرداند. یک ناهم‌خوانیِ عمدی. اجرا کن:

```rust
let mut ratings: HashMap<CiKey, u8> = HashMap::new();
ratings.insert(CiKey("Frieren".to_string()), 10);

println!("len: {}", ratings.len());
println!("get(\"Frieren\"): {:?}", ratings.get("Frieren"));
println!("get(\"frieren\"): {:?}", ratings.get("frieren"));
```

```text
len: 1
get("Frieren"): None
get("frieren"): None
```

کلید قطعاً آن‌جاست — `len()` می‌گوید ۱. ولی هیچ املایی، نه دقیقاً همان `"Frieren"`، نه `"frieren"`ِ کوچک، پیدایش نمی‌کند. دلیلش: هنگامِ `.insert()`، `HashMap` هش را با `CiKey::hash` می‌سازد — که رشته را کوچک می‌کند و `"frieren"` را هش می‌کند. هنگامِ `.get("Frieren")`، `HashMap` هش را با `str::hash` می‌سازد — چون نوعِ جستجو این‌بار `str` است، نه `CiKey` — و این هش رویِ `"Frieren"`ِ دست‌نخورده حساب می‌شود؛ عددی کاملاً متفاوت، پس حتی به سطلِ درست هم نمی‌رسد. `.get("frieren")` به همان سطل می‌رسد (چون این‌بار بایت‌های هش‌شده با آنچه هنگامِ درج هش شد یکی است)، ولی بعد یک مقایسه‌ی تساویِ ساده‌ی `str` می‌آید: آیا رشته‌ی ذخیره‌شده (`"Frieren"`، همان که `borrow()` بدونِ کوچک‌کردن پس داد) با کلیدِ جستجو (`"frieren"`) برابر است؟ نه — این مقایسه دیگر از `CiKey::eq`ِ کوچک‌کننده استفاده نمی‌کند، از `str::eq`ِ معمولی استفاده می‌کند. نتیجه: یک کلیدی که با معیارِ خودِ `Eq`ش قطعاً آن‌جاست، از پشتِ `.get()`، با هیچ املایی، پیدا نمی‌شود.

```senpai-visual
{"kind":"concept","labels":["insert hashes via CiKey (lowercase)","get(Frieren) hashes via str (exact)","different bucket","key unreachable"]}
```

این دقیقاً همان مثالی است که خودِ مستنداتِ `Borrow` هم می‌زند — یک کلیدِ به‌بزرگی‌کوچکیِ‌حروف‌نامعتنی که نباید `Borrow<str>` پیاده کند، دقیقاً به همین دلیل؛ باید به‌جایش سراغِ `AsRef<str>` برود، که چنین قولی نمی‌دهد. `Handle` تویِ تمرینِ همین درس دقیقاً نسخه‌ی درستِ همین ایده را از تو می‌خواهد.

### `ToOwned`: تعمیمِ `Clone` برایِ وقتی نوعِ مالک فرق می‌کند

`str` یک **نوعِ بی‌اندازه (unsized type)** است — همان اصطلاحی که از [۱.۴.۱](../../../phase1-fundamentals/04-text-and-strings/01-string-vs-str/README.fa.md) می‌شناسی: هیچ‌وقت اندازه‌اش هنگامِ کامپایل معلوم نیست، پس هیچ‌وقت نمی‌توانی مستقیم نگهش داری، فقط پشتِ یک ارجاع یا یک `Box`. این یعنی `Clone::clone(&self) -> Self` برایِ `str` اصلاً معنا ندارد — مجبور می‌شد یک `str` را **با مقدار** برگرداند، و برگرداندنِ چیزی بی‌اندازه با مقدار کامپایل نمی‌شود. برایِ همین `str: Clone` نیست، و اگر عادتِ فازِ ۱ را رویِ یک `&str` امتحان کنی، یک تله می‌بینی:

```rust
let borrowed: &str = "Frieren";
let still_borrowed: &str = borrowed.clone();
println!("still borrowed: {still_borrowed}");
```

خودِ کامپایلر، همین‌جا، یک هشدار می‌دهد:

```text
warning: call to `.clone()` on a reference in this situation does nothing
  --> phase2-intermediate\04-lifetimes-and-conversion\03-deref-asref-borrow\examples\06-toowned-str-and-slice.rs:10:40
   |
10 |     let still_borrowed: &str = borrowed.clone();
   |                                        ^^^^^^^^ help: remove this redundant call
   |
   = note: the type `str` does not implement `Clone`, so calling `clone` on `&str` copies the reference, which does not do anything and can be removed
   = note: `#[warn(noop_method_call)]` on by default
```

کامپایل می‌شود، اجرا هم می‌شود، و `still_borrowed` هنوز `&str` است — چون `&str` خودش `Copy` است، پس `.clone()` فقط خودِ ارجاع را کپی می‌کند، نه محتوایش را. هیچ `String`ی ساخته نشد؛ فقط یک آدرسِ دیگر به همان متن.

ابزارِ درست، تعمیمِ `Clone` (صفتِ `ToOwned`) است:

```rust
trait Clone {
    fn clone(&self) -> Self;
}

trait ToOwned {
    type Owned: Borrow<Self>;
    fn to_owned(&self) -> Self::Owned;
}
```

فرقِ اصلی همین دو خط است: `Clone::clone` مجبور است دقیقاً همان نوع را برگرداند (`Self`)؛ `ToOwned::to_owned` یک نوعِ وابسته‌ی (associated type) جدا دارد، `Owned`، که لازم نیست `Self` باشد — و حتی خودِ تعریفش می‌گوید `Owned: Borrow<Self>`، یعنی هر نوعِ مالکی که انتخاب می‌کنی، باید بتواند دوباره به `Self` قرض داده شود. برایِ `str`، آن `Owned` می‌شود `String` — دقیقاً همان `impl Borrow<str> for String`ی که تویِ بخشِ قبل استفاده کردی، این‌بار به‌عنوانِ بخشی از قراردادِ خودِ `ToOwned`:

```rust
let owned: String = borrowed.to_owned();
println!("owned: {owned}");

let numbers: &[i32] = &[1, 2, 3];
let owned_numbers: Vec<i32> = numbers.to_owned();
println!("owned_numbers: {owned_numbers:?}");
```

```text
owned: Frieren
owned_numbers: [1, 2, 3]
```

همان الگو برایِ `[T]` هم کار می‌کند — یک برشِ بی‌اندازه‌ی دیگر، با `Vec<T>` به‌عنوانِ `Owned`ش. و برایِ نوع‌هایی که واقعاً `Clone` دارند (اکثرِ نوع‌هایی که تا امروز نوشته‌ای)، نگرانِ `ToOwned` نبوده‌ای چون یک پیاده‌سازیِ فراگیر (blanket impl) — همان الگویی که در [۲.۳.۶](../../03-traits-and-generics/06-supertraits-blanket-impls-orphan-rule/README.fa.md) دیدی — از قبل هرچه `Clone` دارد را هم `ToOwned` می‌کند: `impl<T: Clone> ToOwned for T { type Owned = T; ... }`. `str` استثنا است، نه قاعده — دقیقاً چون `Clone` رویش تعریف‌نشدنی است.

### تصمیم: `Deref`، `AsRef` یا `Borrow`؟

| صفت | وقتی به‌عنوانِ کراندِ یک پارامتر می‌خواهی | وقتی برایِ نوعِ خودت پیاده می‌کنی |
|---|---|---|
| `Deref`/`DerefMut` | تقریباً هیچ‌وقت مستقیم به‌عنوانِ کراند نمی‌نویسی‌اش | نوعت واقعاً **همان** چیزی است که پوشانده — یک اشاره‌گرِ هوشمند، یک پوششگرِ شفاف |
| `AsRef<T>` | یک بدنه‌ی تابع که `&str`/`&Path`/... را از هرچه صداکننده بدهد قبول کند، بدونِ کپی | نوعت می‌تواند ارزان یک `&T` نشان بدهد، برایِ صداکننده‌ای که صریح همان را می‌خواهد |
| `Borrow<T>` | خودت به‌ندرت می‌نویسی‌اش؛ `HashMap`/`HashSet`/`BTreeMap` پشتِ صحنه ازش استفاده می‌کنند | نوعت قرار است کلیدِ یک مجموعه باشد **و** بتوانی قول بدهی `Hash`/`Eq`/`Ord`ش با فرمِ قرضی هم‌خوان می‌ماند |

یک قاعده‌یِ کوتاه‌تر: اگر فقط می‌خواهی یک تابع، سرِ یک محلِ فراخوانیِ به‌خصوص، انعطاف‌پذیر باشد، `AsRef<T>` بگیر. اگر نوعِ خودت را داری می‌سازی و او واقعاً *همان* چیزی است که می‌پوشاند، `Deref` را پیاده کن. `Borrow<T>` را فقط وقتی پیاده کن که آن قولِ سخت‌گیرِ `Hash`/`Eq`/`Ord` را واقعاً می‌توانی نگه‌داری — وگرنه، دقیقاً مثلِ مثالِ `CiKey` بالا، `AsRef<T>` انتخابِ امن‌تر است.

---

## دست‌به‌کد

```sh
cargo run -p p2-04-03-deref-asref-borrow --example 01-deref-and-derefmut
cargo run -p p2-04-03-deref-asref-borrow --example 02-deref-coercion-chain
cargo run -p p2-04-03-deref-asref-borrow --example 03-asref-generic-param
cargo run -p p2-04-03-deref-asref-borrow --example 04-borrow-hashmap-payoff
cargo run -p p2-04-03-deref-asref-borrow --example 05-borrow-contract-violation
cargo run -p p2-04-03-deref-asref-borrow --example 06-toowned-str-and-slice
```

بعد چهارتایِ خراب:

```sh
cargo run -p p2-04-03-deref-asref-borrow --example 07-derefmut-missing-broken --features broken
cargo run -p p2-04-03-deref-asref-borrow --example 08-asref-wrong-return-type-broken --features broken
cargo run -p p2-04-03-deref-asref-borrow --example 09-hashmap-get-wrong-query-broken --features broken
cargo run -p p2-04-03-deref-asref-borrow --example 10-toowned-unsized-return-broken --features broken
```

بعد این‌ها را امتحان کن:

۱. تویِ `05-borrow-contract-violation.rs`، `borrow()` را طوری عوض کن که آن هم رشته را کوچک کند: `fn borrow(&self) -> &str { &self.0.to_lowercase() }`. کامپایل می‌شود؟ اگر نه، پیامِ کامپایلر را بخوان — چرا حتی این ساده‌ترین راهِ ممکن هم شکست می‌خورد؟ (راهنمایی: `.to_lowercase()` خودش یک `String` تازه می‌سازد، که فقط تا پایانِ همان فراخوانیِ `borrow()` زنده است.)
۲. تویِ `06-toowned-str-and-slice.rs`، به‌جایِ `numbers.to_owned()`، `numbers.clone()` را امتحان کن — و کوته‌نویسیِ نوعِ `: Vec<i32>` را هم بردار، وگرنه ناهم‌خوانیِ بینِ چیزی که `.clone()` واقعاً برمی‌گرداند (`&[i32]`) و چیزی که به کامپایلر گفته‌ای انتظار داشته باشد، هشدار را پشتِ یک خطایِ سخت پنهان می‌کند. بدونِ آن کوته‌نویسی، کامپایل می‌شود؟ چه هشداری می‌بینی، و چرا همان هشدارِ `&str` است؟
۳. تویِ `04-borrow-hashmap-payoff.rs`، `find_by_ref(&ratings, &5)` را صدا بزن (یک `&i32` به‌جایِ `&str`). کدام خطا را می‌بینی، و آیا دقیقاً همان چیزی است که در «خطاهایی که خواهی دید» می‌آید؟

---

## خطاهایی که خواهی دید

### `E0596` — `DerefMut` گمشده

```text
error[E0596]: cannot borrow data in dereference of `Watchlist` as mutable
  --> phase2-intermediate\04-lifetimes-and-conversion\03-deref-asref-borrow\examples\07-derefmut-missing-broken.rs:24:5
   |
24 |     list.push("Trigun".to_string());
   |     ^^^^ cannot borrow as mutable
   |
   = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Watchlist`

For more information about this error, try `rustc --explain E0596`.
```

**کامپایلر به چه اعتراض دارد:** `Watchlist` فقط `Deref` دارد، نه `DerefMut`. `.push()` به `&mut Vec<String>` نیاز دارد؛ بازکردنِ خودکار می‌تواند یک `&Vec<String>` (فقط‌خواندنی) از `Deref::deref` بگیرد، ولی هیچ راهی به یک نسخه‌ی تغییرپذیر ندارد، چون آن قدم فقط رویِ `DerefMut` تعریف شده. خودِ پیامِ کامپایلر این را دقیق می‌گوید.

**راه‌حل:** `DerefMut` را هم پیاده کن:

```rust
impl DerefMut for Watchlist {
    fn deref_mut(&mut self) -> &mut Vec<String> {
        &mut self.0
    }
}
```

**چرا این راه‌حل است:** حالا بازکردنِ خودکار یک قدمِ تغییرپذیر هم دارد که بردارد؛ `.push()` از همان مسیرِ `.len()` می‌رود، فقط این‌بار با یک `&mut` که `deref_mut` می‌دهد، نه یک `&` که `deref` می‌دهد.

### `E0308` — `AsRef<str>` که نوعِ اشتباه برمی‌گرداند

```text
error[E0308]: mismatched types
  --> phase2-intermediate\04-lifetimes-and-conversion\03-deref-asref-borrow\examples\08-asref-wrong-return-type-broken.rs:13:9
   |
12 |     fn as_ref(&self) -> &str {
   |                         ---- expected `&str` because of return type
13 |         &self.0
   |         ^^^^^^^ expected `&str`, found `&Vec<String>`
   |
   = note: expected reference `&str`
              found reference `&Vec<String>`

For more information about this error, try `rustc --explain E0308`.
```

**کامپایلر به چه اعتراض دارد:** `AsRef<str>` قول می‌دهد `as_ref` یک `&str` برمی‌گرداند. `Watchlist` یک `Vec<String>` می‌پوشاند، و `&self.0` یک `&Vec<String>` است — نه یک `&str`، و هیچ تبدیلِ ضمنی‌ای بینِ این دو نیست.

**راه‌حل:** نوعِ هدف را با چیزی که `Watchlist` واقعاً دارد جور کن:

```rust
impl AsRef<[String]> for Watchlist {
    fn as_ref(&self) -> &[String] {
        &self.0
    }
}
```

**چرا این راه‌حل است:** `Watchlist` یک `Vec<String>` است، نه یک رشته — هیچ `&str`ِ معناداری از دلش درنمی‌آید بدونِ ساختنِ یک رشته‌ی تازه (که دیگر «ارزان» نیست، و `AsRef` قرارش بود ارزان باشد). `AsRef<[String]>` هدفی است که واقعاً با محتوایِ نوع جور است — و همان چیزی است که `Deref` هم رایگان به‌ات می‌داد.

### `E0277` — کلیدی که `HashMap::get` نمی‌تواند به آن `Borrow` کند

```text
error[E0277]: the trait bound `String: Borrow<{integer}>` is not satisfied
    --> phase2-intermediate\04-lifetimes-and-conversion\03-deref-asref-borrow\examples\09-hashmap-get-wrong-query-broken.rs:14:34
     |
  14 |     println!("{:?}", ratings.get(&5));
     |                              --- ^^ the trait `Borrow<{integer}>` is not implemented for `String`
     |                              |
     |                              required by a bound introduced by this call
     |
help: the trait `Borrow<{integer}>` is not implemented for `String`
      but trait `Borrow<str>` is implemented for it
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\alloc\src\str.rs:229:1
     |
 229 | impl Borrow<str> for String {
     | ^^^^^^^^^^^^^^^^^^^^^^^^^^^
     = help: for that trait implementation, expected `str`, found `{integer}`
note: required by a bound in `HashMap::<K, V, S, A>::get`
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\std\src\collections\hash\map.rs:1037:12
     |
1035 |     pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
     |            --- required by a bound in this associated function
1036 |     where
1037 |         K: Borrow<Q>,
     |            ^^^^^^^^^ required by this bound in `HashMap::<K, V, S, A>::get`

For more information about this error, try `rustc --explain E0277`.
```

**کامپایلر به چه اعتراض دارد:** `HashMap<String, u8>::get<Q>` فقط وقتی موجود است که `String: Borrow<Q>` باشد. کتابخانه‌ی استاندارد این را فقط برایِ `Q = str` (و بدیهی‌اش، `Q = String`) نوشته — نه برایِ یک عدد. `&5` یعنی `Q` یک نوعِ صحیح است، و `String: Borrow<{integer}>` هیچ‌وقت نوشته نشده. پیامِ کامپایلر حتی محلِ دقیقِ همان `impl Borrow<str> for String` را تویِ خودِ کتابخانه‌ی استاندارد نشانت می‌دهد.

**راه‌حل:** با نوعی جستجو کن که `String` واقعاً می‌تواند به آن `Borrow` کند:

```rust
let ratings: HashMap<String, u8> = HashMap::new();
println!("{:?}", ratings.get("5"));
```

**چرا این راه‌حل است:** `"5"` این‌بار یک `&str` است، نه یک عدد؛ `String: Borrow<str>` برقرار است، پس `Q = str` کراندِ `K: Borrow<Q>` را برآورده می‌کند و کامپایل می‌شود (خودِ نقشه خالی است، پس نتیجه `None` می‌شود — ولی حالا این یک جوابِ منطقی است، نه یک خطایِ نوع).

### `E0277` — یک نوعِ برگشتی که `Sized` نیست

```text
error[E0277]: the size for values of type `str` cannot be known at compilation time
 --> phase2-intermediate\04-lifetimes-and-conversion\03-deref-asref-borrow\examples\10-toowned-unsized-return-broken.rs:9:22
  |
9 | fn widen(s: &str) -> str {
  |                      ^^^ doesn't have a size known at compile-time
  |
  = help: the trait `Sized` is not implemented for `str`
  = note: the return type of a function must have a statically known size

For more information about this error, try `rustc --explain E0277`.
```

**کامپایلر به چه اعتراض دارد:** نوعِ برگشتیِ یک تابع باید اندازه‌اش هنگامِ کامپایل معلوم باشد — همان قانونی که `str::clone(&self) -> Self` را هم غیرِممکن می‌کند. `s.to_owned()` این‌جا یک `String` می‌سازد، ولی امضا قول داده `str` برگرداند؛ حتی اگر بدنه چیزِ درستی بسازد، امضایِ خودش از همان ابتدا نامعتبر است.

**راه‌حل:** نوعِ برگشتی را همان چیزی کن که `ToOwned` واقعاً می‌سازد:

```rust
fn widen(s: &str) -> String {
    s.to_owned()
}
```

**چرا این راه‌حل است:** `String` اندازه‌اش هنگامِ کامپایل معلوم است (یک اشاره‌گر، یک طول، یک ظرفیت — سه عدد، رویِ پشته). این دقیقاً همان چیزی است که `ToOwned::Owned` برایِ `str` قرار است باشد؛ امضا حالا با چیزی که `to_owned()` واقعاً تحویل می‌دهد یکی است.

---

## تمرین

### گرم‌کردن

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
fn takes_str(s: &str) {
    println!("{s}");
}

let owned = String::from("hi");
takes_str(&owned);
```

</details>

<details>
<summary>پاسخ</summary>

بله. این همان تبدیلِ ضمنیِ ارجاعی است که از فازِ ۱ می‌شناسی — حالا می‌دانی پشتش `impl Deref<Target = str> for String` است، نوشته‌شده توسطِ کتابخانه‌ی استاندارد، نه توسطِ تو.

</details>

<details>
<summary>اگر <code>Watchlist</code> فقط <code>Deref</code> داشته باشد (نه <code>DerefMut</code>)، این کامپایل می‌شود؟</summary>

```rust
let mut list = Watchlist(vec!["a".to_string()]);
list.push("b".to_string());
```

</details>

<details>
<summary>پاسخ</summary>

نه. `.push()` به `&mut Vec<String>` نیاز دارد، و بدونِ `DerefMut`، بازکردنِ خودکار هیچ راهی به یک نسخه‌ی تغییرپذیر ندارد. کدِ خطا `E0596` است.

</details>

<details>
<summary>این چه چاپ می‌کند؟</summary>

```rust
let s: &str = "hi";
let c = s.clone();
println!("{}", std::mem::size_of_val(&c) == std::mem::size_of_val(&s));
```

</details>

<details>
<summary>پاسخ</summary>

```text
true
```

`c` و `s` هر دو یک `&str`ند، با دقیقاً یک اندازه — چون `.clone()` رویِ یک `&str` فقط خودِ ارجاع را کپی می‌کند، نه محتوایی که به آن اشاره می‌کند. کامپایلر رویِ همین خط یک هشدارِ `noop_method_call` هم می‌دهد.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
let ratings: HashMap<String, u8> = HashMap::new();
println!("{:?}", ratings.get(&5));
```

</details>

<details>
<summary>پاسخ</summary>

نه. `String: Borrow<i32>` هیچ‌جا نوشته نشده — کتابخانه‌ی استاندارد فقط `Borrow<str>` را برایِ `String` می‌دهد. کدِ خطا `E0277` است.

</details>

<details>
<summary>یک نوعِ خودت به‌بزرگی‌کوچکیِ حروف بی‌اعتنا است — <code>Hash</code> و <code>Eq</code>ش رشته را کوچک می‌کنند. برایِ اینکه با یک <code>&str</code> هم قابلِ‌جستجو باشد، کدام صفت را پیاده کنی: <code>Borrow&lt;str&gt;</code> یا <code>AsRef&lt;str&gt;</code>؟</summary>

</details>

<details>
<summary>پاسخ</summary>

`AsRef<str>`. `Borrow<str>` قول می‌دهد `Hash`/`Eq`ِ فرمِ قرضی (رشته‌یِ دست‌نخورده) با فرمِ مالک (کوچک‌شده) یکی بماند — و این‌جا یکی نمی‌ماند. `AsRef<str>` چنین قولی نمی‌دهد، پس دروغ نمی‌گویی.

</details>

### تعمیر

هر چهار مثالِ خراب را درست کن:

۱. `examples/07-derefmut-missing-broken.rs` — `DerefMut` را برایِ `Watchlist` پیاده کن.
۲. `examples/08-asref-wrong-return-type-broken.rs` — هدفِ `AsRef` را از `str` به چیزی عوض کن که `Watchlist` واقعاً دارد.
۳. `examples/09-hashmap-get-wrong-query-broken.rs` — `.get()` را با یک کلیدی صدا بزن که `String` واقعاً می‌تواند به آن `Borrow` کند.
۴. `examples/10-toowned-unsized-return-broken.rs` — نوعِ برگشتیِ `widen` را به چیزی عوض کن که `Sized` باشد.

### پیاده‌سازی

چهار تکه در `src/lib.rs`، هرکدام یک صفت:

```sh
cargo test -p p2-04-03-deref-asref-borrow
```

- `Playlist` — `Deref` و `DerefMut` را پیاده کن.
- `DisplayName` — `AsRef<str>` را پیاده کن.
- `Handle` — `Borrow<str>` را طوری پیاده کن که با `Hash`/`Eq`ِ از قبل `derive`‌شده‌اش هم‌خوان بماند.
- `longest_word_owned` — مشخصاتش را با دقت بخوان؛ قاعده‌ی رفعِ تساوی (آخرین کلمه‌ی هم‌طول برنده است) دقیقاً همان چیزی است که `Iterator::max_by_key` خودش، سرِ تساوی، انجام می‌دهد.

### بساز

یک نوعِ پوششگرِ (newtype) تازه، مالِ خودت، دورِ یک نوعِ ساده (یک `String`، یک `Vec<T>`، هرچه) بساز. یکی از این چهار صفت را برایش پیاده کن — هرکدام که برایِ آن نوع منطقی‌تر است — و در یک کامنت بنویس چرا همان یکی را انتخاب کردی، نه یکیِ دیگر را. دست‌کم دو تست اضافه کن.

### چالش (اختیاری)

این درس نشانت داد یک مقدار می‌تواند «قرضی» باشد یا «مالک»، و `ToOwned` پلِ بینِ این دوست. درسِ بعدی، [۲.۴.۴](../04-cow-and-clone-on-write/README.fa.md)، یک نوع می‌سازد که بینِ این دو، سرِ اجرا، خودش تصمیم می‌گیرد. بدونِ استفاده از آن نوع — فقط با چیزی که همین امروز یاد گرفتی — یک `enum` بساز با دو گونه، یکی برایِ حالتِ قرضی (یک فیلدِ `&'a str`) و یکی برایِ حالتِ مالک (یک فیلدِ `String`)، به‌علاوه‌ی یک متد رویِ آن `enum` که، فارغ از این‌که کدام گونه است، یک `&str` برمی‌گرداند. اگر این را ساختی، عملاً شکلِ `Cow` را با دستِ خودت ساخته‌ای.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| صفتِ `Deref` / `DerefMut` | `*` و بازکردنِ خودکار را ممکن می‌کنند؛ `deref(&self) -> &Target`، و `DerefMut: Deref` نسخه‌ی `&mut`ش را اضافه می‌کند | نوعِ پوششگری که باید مثلِ نوعِ داخلی‌اش رفتار کند |
| تبدیلِ ضمنیِ ارجاعِ زنجیره‌ای | چند `impl Deref` پشتِ سرِ هم، در یک محلِ فراخوانی | `&Watchlist -> &Vec<String> -> &[String]`، `Box<T> -> T` |
| `AsRef<T>` | صفتی برایِ «ارزان به `&T` قابلِ‌دیدن‌شدن»؛ `fn as_ref(&self) -> &T` | پارامترِ تابعی که باید `&str`/`String`/`PathBuf`/... را یکسان قبول کند |
| `Borrow<T>` | هم‌شکلِ `AsRef`، با قرارداد: `Hash`/`Eq`/`Ord` بینِ فرمِ قرضی و مالک باید یکی بمانند | کلیدهایِ `HashMap`/`HashSet`/`BTreeMap` که باید با فرمِ قرضی هم جستجو شوند |
| `ToOwned` | تعمیمِ `Clone` برایِ نوع‌هایِ بی‌اندازه؛ `to_owned(&self) -> Self::Owned`، `Owned` لازم نیست `Self` باشد | `str -> String`، `[T] -> Vec<T>` |

### الان می‌دانی

- `Deref`/`DerefMut` یک متدِ `deref`/`deref_mut` می‌خواهند و همه‌ی بازکردنِ خودکار و تبدیلِ ضمنیِ ارجاع را ممکن می‌کنند — همان مکانیزمی که `Box`، `String` و `Vec` ازش استفاده می‌کنند، و تبدیل می‌تواند چند `impl` را زنجیره‌ای پشتِ سرِ هم رد کند.
- `AsRef<T>` یک نمایِ ارزانِ `&T` است که یک تابع را می‌گذارد چند شکلِ متفاوتِ ورودی را با یک بدنه قبول کند، بدونِ هیچ تخصیصِ اضافه‌ای.
- `Borrow<T>` هم‌شکلِ `AsRef<T>` است، ولی یک قول اضافه می‌دهد که کامپایلر چکش نمی‌کند: `Hash`/`Eq`/`Ord` بینِ فرمِ قرضی و مالک باید هم‌خوان بمانند؛ نقضش کلیدی می‌سازد که وجود دارد ولی پیدا نمی‌شود.
- `ToOwned` جایی به‌کار می‌آید که `Clone` نمی‌تواند — نوع‌هایِ بی‌اندازه مثلِ `str` — چون نوعِ مالکش (`Owned`) لازم نیست خودِ `Self` باشد.
- سرِ انتخاب: `AsRef<T>` را برایِ انعطافِ یک پارامتر بگیر، `Deref` را برایِ نوعی پیاده کن که واقعاً همان چیزی است که می‌پوشاند، و `Borrow<T>` را فقط وقتی که قولِ سخت‌گیرش را واقعاً می‌توانی نگه داری.

### بعداً کامل‌تر می‌بینی

- **`Box<T>` و تخصیصِ هیپ، کامل** — [۲.۶.۱ — `Box` و تخصیصِ هیپ](../../06-smart-pointers/01-box-and-heap-allocation/README.fa.md)
- **`Cow<'_, str>`: انتخابِ بینِ قرضی و مالک، سرِ اجرا** — [۲.۴.۴](../04-cow-and-clone-on-write/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا `list.len()` کامپایل می‌شود وقتی `Watchlist` هیچ متدِ `len`ی ندارد؟
- تبدیلِ ضمنیِ ارجاع را طوری توضیح بده که «زنجیره‌ای‌بودن»ش هم تویِ توضیح باشد.
- `AsRef<T>` و `Deref` رویِ یک پوششگرِ ساده شبیهِ هم به‌نظر می‌رسند. با کلماتِ خودت بگو چه سؤالِ متفاوتی هرکدام جواب می‌دهند.
- چرا یک پیاده‌سازیِ `Borrow` که با `Hash`/`Eq`ِ خودش هم‌خوان نیست، کامپایل می‌شود ولی سرِ اجرا خراب است؟
- چرا `str: Clone` نیست، و `ToOwned` دقیقاً چه چیزِ اضافه‌ای رویِ `Clone` دارد که این مشکل را حل می‌کند؟

---

## بیشتر

- [مستنداتِ `std::ops::Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html) — از جمله بخشِ «Deref coercion» که همین زنجیره‌ای‌بودن را رسمی توضیح می‌دهد.
- [مستنداتِ `std::convert::AsRef`](https://doc.rust-lang.org/std/convert/trait.AsRef.html)
- [مستنداتِ `std::borrow::Borrow`](https://doc.rust-lang.org/std/borrow/trait.Borrow.html) — همان‌جا که مثالِ کلیدِ به‌بزرگی‌کوچکیِ‌حروف‌نامعتنی، تقریباً عینِ `CiKey` بالا، آمده.
- [مستنداتِ `std::borrow::ToOwned`](https://doc.rust-lang.org/std/borrow/trait.ToOwned.html)
- [کتابِ Rust — `Deref` و `Box<T>`](https://doc.rust-lang.org/book/ch15-02-deref.html)
