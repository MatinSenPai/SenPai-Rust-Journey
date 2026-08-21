# ۱.۶.۱ — `Option` و ایمنی در برابرِ نبودن

## در یک نگاه

بعد از این درس می‌توانی:

- بگویی وقتی یک `Option<T>` را طوری به کار ببری که انگار قطعاً مقدار دارد چه اتفاقی می‌افتد — و چرا Rust این را سرِ کامپایل می‌گیرد، جایی که C، جاوا و پایتون می‌گذارند رد شود تا بعداً کرش کند.
- بینِ `match`، `if let`، `.unwrap()` و `.expect("...")` برای یک `Option<T>`ِ مشخص یکی را انتخاب کنی، و انتخابت را با صدای بلند توجیه کنی.
- `Option<&T>` و `&Option<T>` را از روی شکل‌شان از هم تشخیص بدهی، و برای رفتن از دومی به اولی سراغِ `.as_ref()` بروی — بدونِ کلون.
- یک `size_of::<Option<Box<T>>>()` را کنارِ `size_of::<Box<T>>()` بگذاری و دقیقاً بگویی چرا برابرند.

**زمان:** حدود ۵۵ دقیقه · **پیش‌نیاز:** [۱.۵.۵ — `if let`، `while let`، `let else`](../../05-your-own-types/05-if-let-while-let-let-else/README.fa.md)

---

## چرا اهمیت دارد

از فاز ۰ تا همین‌جا، هر بار یک تابعِ استاندارد چیزی به تو داده که دورش یک `Some` یا `None` پیچیده شده، و هر بار درس همان لحظه گفته «توضیحِ کاملش را بعداً می‌بینی»:

- [۱.۱.۲](../../01-foundations/02-scalar-types-and-overflow/README.fa.md) — `checked_add` وقتی جواب جا نمی‌شد `None` برگرداند.
- [۱.۱.۳](../../01-foundations/03-compound-types-and-destructuring/README.fa.md) و [۱.۱.۶](../../01-foundations/06-vec-and-string-basics/README.fa.md) — `.get()` روی آرایه و روی `Vec`، به‌جای پنیک کردن، `Option` برگرداند.
- همان [۱.۱.۶](../../01-foundations/06-vec-and-string-basics/README.fa.md) — `.pop()` و `.chars().nth()` هم همین کار را کردند.
- [۱.۳.۴](../../03-borrowing-and-references/04-slices/README.fa.md) — `.first()` روی یک برش، همان بسته‌بندی.
- حتی بخشِ خطاهایِ [۱.۲.۳](../../02-ownership-and-memory/03-clone-and-copy/README.fa.md) به‌اش خورد: کلون کردنِ `lines.first()` خودِ بسته را کلون کرد، نه رشته‌ی داخلش را.
- و [۱.۱.۵](../../01-foundations/05-control-flow/README.fa.md) با یک اعتراف تمام شد: `index_of_first_negative` طولِ آرایه را برای «پیدا نشد» برمی‌گرداند — یک عددِ معمولی که صداکننده می‌توانست بی‌خبر با آن ایندکس بزند و پنیک بگیرد — و گفت این درس درستش می‌کند.

وقتش رسیده. این درس همه‌ی آن قرض‌ها را یک‌جا صاف می‌کند.

اسمِ رسمیِ مشکلی که `Option` حلش می‌کند را یک نفرِ مشخص گذاشته: تونی هوار (Tony Hoare)، کسی که در سالِ ۱۹۶۵ در زبانِ ALGOL W «ارجاعِ تهی» (null reference) را اختراع کرد. سال‌ها بعد، در یک سخنرانی در QCon لندن (۲۰۰۹)، درباره‌اش گفت:

> "I call it my billion dollar mistake. It was the invention of the null reference in 1965. ... I couldn't resist the temptation to put in a null reference, simply because it was so easy to implement. This has led to innumerable errors, vulnerabilities, and system crashes, which have probably caused a billion dollars of pain and damage in the last forty years."
>
> — Tony Hoare, QCon London 2009

یعنی: هر متغیر از هر نوعی — یک `User`، یک عدد، یک اتصالِ دیتابیس — می‌توانست بی‌خبر `null` باشد، و زبان هیچ فرقی بینِ «این حتماً یک `User` است» و «این شاید هیچ نباشد» نمی‌گذاشت. کامپایلر نمی‌توانست کمک کند، چون از نظرِ نوع هر دو یکی بودند. نتیجه یک کلاسِ کاملِ باگ شد — `NullPointerException` در جاوا، `'NoneType' object has no attribute` در پایتون، `Cannot read property of null` در جاوااسکریپت — که تنها راهِ پیدا کردنش تست کردن یا شانس بود، نه چیزی که کامپایلر از قبل به تو نشان بدهد.

Rust جوابش را در خودِ نوع می‌گذارد. اگر تابعی ممکن است چیزی پیدا نکند، امضایش این را می‌گوید، نه چیزِ دیگری:

```rust
fn find_user(id: u32) -> Option<String>
```

نه `-> String` با یک `null` پنهان درونش. `Option<String>` نوعی است که با `String` فرق دارد — همان‌قدر که `i32` با `bool` فرق دارد — و همین یک جمله کلِ درس است: **چون نوع فرق دارد، کامپایلر نمی‌گذارد `None` را جای یک مقدار به کار ببری، مگر اول با احتمالِ نبودنش روبه‌رو شوی.** بدونِ استثنا، بدونِ فراموشیِ یک چک، بدونِ اینکه شش ماه بعد در پروداکشن بفهمی.

---

## مفهوم

### `Option<T>` — یک شمارشیِ معمولی با دو حالت

```rust
let recorded: Option<u32> = Some(25);
let missing: Option<u32> = None;

println!("recorded: {recorded:?}");
println!("missing:  {missing:?}");
```

```text
recorded: Some(25)
missing:  None
```

هیچ چیزِ جدیدی در ماشین‌آلاتش نیست. `Option<T>` دقیقاً همان چیزی است که [۱.۵.۳](../../05-your-own-types/03-enums-as-data/README.fa.md) رو کرد: یک شمارشی (enum) معمولی، با دو حالت — یکی مقدار حمل می‌کند، یکی هیچ‌چیز:

```rust
enum Option<T> {
    Some(T),
    None,
}
```

همین. کتابخانه‌ی استاندارد چیزِ جادویی‌ای اضافه نکرده؛ فقط از همان ابزارهایی که [۱.۵](../../05-your-own-types/README.fa.md) به تو داد استفاده کرده. و چون یک شمارشیِ معمولی است، بدونِ باز کردنش هم می‌توانی از آن بپرسی کدام حالت است:

```rust
println!("recorded.is_some(): {}", recorded.is_some());
println!("missing.is_none():  {}", missing.is_none());
```

```text
recorded.is_some(): true
missing.is_none():  true
```

### `None` نمی‌تواند بی‌خبر جا خودش را باز کند

این را بنویس — تابعی که ممکن است چیزی پیدا نکند:

```rust
fn find_user(id: u32) -> Option<String> {
    if id == 1 {
        Some(String::from("Matin"))
    } else {
        None
    }
}
```

حالا سعی کن آن `Option<String>` را جوری استفاده کنی که انگار قطعاً یک `String` است — مثلاً بدهی‌اش به تابعی که `String` می‌خواهد:

```rust
fn greet(name: String) -> String {
    format!("hello, {name}")
}
```

این دقیقاً همان باگی است که ارجاعِ تهی در بیشترِ زبان‌ها ممکنش می‌کند: یک جست‌وجو که شاید چیزی پیدا نکرده باشد، مستقیم به کدی داده می‌شود که فرض کرده پیدا کرده. جاوا، #C و پایتون این را کامپایل می‌کنند و اولین باری که `id` جور در نیاید، کرش می‌کنند. Rust حتی نمی‌گذارد ساخته شود — خطای کاملش را در «خطاهایی که خواهی دید» می‌بینی.

**نکته را دقیق بگیر:** مشکل این نیست که `find_user` ممکن است چیزی پیدا نکند — همیشه چنین توابعی هستند. مشکل این بود که در زبان‌های دیگر، این احتمال جایی در امضای تابع نوشته نمی‌شد. اینجا نوشته می‌شود، و کامپایلر آن را می‌خواند.

### بیرون کشیدنِ مقدار: `match` و `if let`

تنها راهِ رسیدن به چیزِ داخلِ یک `Option<T>` این است که هر دو حالت را جواب بدهی — همان چیزی که [۱.۵.۴](../../05-your-own-types/04-match-in-depth/README.fa.md) و [۱.۵.۵](../../05-your-own-types/05-if-let-while-let-let-else/README.fa.md) از قبل به تو دادند، فقط حالا رویِ `Option` پیاده‌اش می‌کنی:

```rust
fn describe(rating: Option<u8>) -> String {
    match rating {
        Some(score) => format!("rated {score}/10"),
        None => "not rated yet".to_string(),
    }
}
```

```rust
println!("{}", describe(Some(9)));
println!("{}", describe(None));
```

```text
rated 9/10
not rated yet
```

اگر فقط یکی از دو حالت کاری برایت دارد، `if let` همان `match` است بدونِ بازوی «هیچ کاری نکن»:

```rust
let maybe_name: Option<&str> = Some("Matin");
if let Some(name) = maybe_name {
    println!("if let:   hello, {name}");
}
```

```text
if let:   hello, Matin
```

```senpai-visual
{"kind":"concept","labels":["fn call","Some(v)","None","match","use T safely"]}
```

و همان معامله‌ای که [۱.۵.۵](../../05-your-own-types/05-if-let-while-let-let-else/README.fa.md) گفت عمدی نگهش دار: `match` تضمین می‌کند هر دو حالت را دیده‌ای؛ `if let` این تضمین را می‌دهد از دست. برای «چیزی برای نمایش هست یا نه» این معامله‌ی خوبی است — بازوی دیگر واقعاً «هیچ کاری نکن» است.

### `.unwrap()` و `.expect("...")` — و کِی هرکدام قابلِ دفاع است

هر دو مقدارِ داخلِ یک `Some` را بیرون می‌کشند و روی `None` **پنیک** (panic) می‌کنند — برنامه فوراً متوقف می‌شود. نه غلط‌اند و نه همیشه درست؛ دقیقاً همان وقتی درست‌اند که تو چیزی می‌بینی که تایپ‌چکر نمی‌بیند:

```rust
let mut scores = Vec::new();
scores.push(10);
scores.push(20);
scores.push(30);
let last = scores.last().expect("scores just had three items pushed");
println!("expect on a Vec you just filled: {last}");
```

```text
expect on a Vec you just filled: 30
```

`.last()` نوعش `Option<&T>` است چون یک `Vec` ممکن است خالی باشد — ولی این یکی همین بالا سه‌تا `push` خورده. `.expect("...")` آن اثباتِ چشمی را می‌نویسد، برای خواننده‌ی بعدی (که خودِ تو هم می‌تواند باشد، شش ماه دیگر).

وقتی اشتباه از آب دربیاید، این دو خیلی چیزِ متفاوتی به‌ات می‌گویند. `.unwrap()` فقط این را می‌گوید:

```text
called `Option::unwrap()` on a `None` value
```

ولی `.expect("user 7 should exist in the seed data")` این را:

```text
user 7 should exist in the seed data
```

اولی می‌گوید *چه چیزی* شکست خورد. دومی می‌گوید *چرا فکر می‌کردی* نمی‌تواند شکست بخورد. پیام‌های کاملشان — با فایل و خط — در «خطاهایی که خواهی دید» است.

> **قاعده:** `.unwrap()`/`.expect()` برای آن `None`ای‌اند که *نوع* اجازه‌اش می‌دهد ولی *کدِ اطرافش* ردش می‌کند. و اگر ردش می‌کنی، در پیامِ `.expect()` بنویس چرا — همان‌طور که بالا نوشتیم.

### `Option<&T>` در برابرِ `&Option<T>`، و `.as_ref()`

این دو از یک جنس ساخته شده‌اند ولی یک چیز نیستند. اولی «یک `Option` که اگر چیزی داشته باشد، یک ارجاع است»؛ دومی «یک ارجاع به کلِ جعبه، `Some` یا `None`ش هر چه باشد».

```rust
fn describe_ref(nickname: &Option<String>) -> String {
    match nickname {
        Some(name) => format!("&Option<T>:    Some(\"{name}\")"),
        None => "&Option<T>:    None".to_string(),
    }
}
```

```rust
let known = Some(String::from("Matin"));
println!("{}", describe_ref(&known));
```

```text
&Option<T>:    Some("Matin")
```

حالا نوشتارِ دیگر. `.as_ref()` آن `Option<String>`ی که مالکشی را به یک `Option<&String>` تبدیل می‌کند که فقط نگاهش می‌کنی — بدونِ اینکه از دستت درش بیاورد:

```rust
fn describe_inner(nickname: Option<&String>) -> String {
    match nickname {
        Some(name) => format!("Option<&T>:    Some(\"{name}\")"),
        None => "Option<&T>:    None".to_string(),
    }
}
```

```rust
println!("{}", describe_inner(known.as_ref()));
println!("still have `known`: {known:?}");
```

```text
Option<&T>:    Some("Matin")
still have `known`: Some("Matin")
```

`known` بعدش هم همچنان مالِ خودت است. اگر به‌جایِ `.as_ref()` مستقیم `match known { ... }` می‌نوشتی، `known` **حرکت** می‌کرد — دقیقاً همان چیزی که در «خطاهایی که خواهی دید» می‌بینی.

و راهِ برگشت هم هست: `.cloned()` یک `Option<&T>` را به یک `Option<T>`ِ مالک تبدیل می‌کند — همان `.cloned()`ی که [۱.۲.۳](../../02-ownership-and-memory/03-clone-and-copy/README.fa.md) وعده‌اش را داده بود، این‌بار روی چیزی که داخلِ `Option` است:

```rust
let words = vec![String::from("hello"), String::from("world")];
let borrowed: Option<&String> = words.first();
let owned: Option<String> = borrowed.cloned();
println!("borrowed: {borrowed:?}");
println!("owned:    {owned:?}");
```

```text
borrowed: Some("hello")
owned:    Some("hello")
```

### `Option` در یک فیلدِ ساختار یعنی «اختیاری»

وقتی می‌خواهی بگویی یک فیلد شاید مقدار نداشته باشد، به‌جایِ رشته‌ی خالی یا عددِ ۰-به‌جایِ-هیچ، نوعش را `Option` می‌گذاری:

```rust
struct Profile {
    nickname: Option<String>,
}

impl Profile {
    fn nickname_len(&self) -> Option<usize> {
        match self.nickname.as_ref() {
            Some(name) => Some(name.len()),
            None => None,
        }
    }
}
```

`self.nickname.as_ref()` همان تله را اینجا هم دور می‌زند: بدونِ آن، `match self.nickname { ... }` می‌خواست فیلد را از داخلِ `self` بیرون بکشد — و از یک ارجاع (`&self`) نمی‌شود چیزی را مالکانه بیرون کشید.

```rust
let matin = Profile {
    nickname: Some(String::from("Matin")),
};
let anon = Profile { nickname: None };

println!("matin.nickname_len(): {:?}", matin.nickname_len());
println!("anon.nickname_len():  {:?}", anon.nickname_len());
```

```text
matin.nickname_len(): Some(5)
anon.nickname_len():  None
```

هیچ رشته‌ی سنتینلی مثلِ `""` برای «نگذاشته‌ام» نیست، هیچ قراردادِ دستی‌ای که یادت برود. خودِ نوع می‌گوید فیلد اختیاری است.

### بهینه‌سازیِ اشاره‌گرِ تهی (null-pointer optimisation)

معمولاً `Option<T>` به‌اندازه‌ی خودِ `T` بعلاوه‌ی یک برچسب («این `Some` است یا `None`؟») جا می‌گیرد:

```rust
use std::mem::size_of;

println!("size_of::<i32>():           {}", size_of::<i32>());
println!("size_of::<Option<i32>>():   {}", size_of::<Option<i32>>());
```

```text
size_of::<i32>():           4
size_of::<Option<i32>>():   8
```

هر بایتِ ممکن برای `i32` یک عددِ واقعی است، پس جایی برای «هیچ» رایگان نمانده — `Option` باید فضای تازه بگیرد. ولی برای یک اشاره‌گرِ واقعی، داستان فرق دارد: یک `&T` یا `Box<T>`ِ معتبر هرگز همه-بیت-صفر نیست، پس `None` می‌تواند همان الگوی بیتی را قرض بگیرد، مجانی:

```rust
println!("size_of::<Box<i32>>():         {}", size_of::<Box<i32>>());
println!("size_of::<Option<Box<i32>>>(): {}", size_of::<Option<Box<i32>>>());
```

```text
size_of::<Box<i32>>():         8
size_of::<Option<Box<i32>>>(): 8
```

(`Box<T>` یک اشاره‌گرِ هیپ است، مالکِ چیزی که به آن اشاره می‌کند — دقیقاً همین یک خط امروز لازم داری؛ درسِ کاملش [فاز ۲](../../../phase2-intermediate/05-smart-pointers/01-box-and-heap-allocation/README.fa.md) است.)

این ترفند فقط مخصوصِ اشاره‌گر نیست — هر نوعی که چند الگوی بیتیِ استفاده‌نشده داشته باشد همین تخفیف را می‌گیرد. `bool` فقط دو تا از ۲۵۶ الگوی یک بایت را مصرف می‌کند:

```rust
println!("size_of::<bool>():          {}", size_of::<bool>());
println!("size_of::<Option<bool>>():  {}", size_of::<Option<bool>>());
```

```text
size_of::<bool>():          1
size_of::<Option<bool>>():  1
```

**`Option<Box<T>>` هیچ بایتِ اضافه‌ای نسبت به `Box<T>` نمی‌گیرد. `Option<i32>` می‌گیرد.** این محاسبه‌ی خودت بود، نه یک قولِ مستندات.

---

## دست‌به‌کد

```sh
cargo run -p p1-06-01-option-and-null-safety --example 01-some-and-none
cargo run -p p1-06-01-option-and-null-safety --example 02-match-and-if-let
cargo run -p p1-06-01-option-and-null-safety --example 03-defensible-unwrap-and-expect
cargo run -p p1-06-01-option-and-null-safety --example 04-option-ref-and-struct-field
cargo run -p p1-06-01-option-and-null-safety --example 05-null-pointer-optimization
```

بعد چهارتای خراب:

```sh
cargo run -p p1-06-01-option-and-null-safety --example 06-cannot-use-option-directly --features broken
cargo run -p p1-06-01-option-and-null-safety --example 07-unwrap-on-none --features broken
cargo run -p p1-06-01-option-and-null-safety --example 08-expect-on-none --features broken
cargo run -p p1-06-01-option-and-null-safety --example 09-matching-by-value-moves-it --features broken
```

بعد این‌ها را امتحان کن:

۱. در `01-some-and-none`، یک `Option<bool>` بساز، `Some(false)` و `None`. مقایسه‌شان با `==` چه می‌دهد؟
۲. در `04-option-ref-and-struct-field`، `matin.nickname_len()` را سه بار پشتِ سرِ هم صدا بزن. آیا هنوز کامپایل می‌شود؟ چرا این سؤال برای `describe(nickname)` در فایلِ ۰۹ جوابش «نه» است؟
۳. در `05-null-pointer-optimization`، `size_of::<Option<char>>()` را هم چاپ کن و با `size_of::<char>()` مقایسه‌اش کن.

---

## خطاهایی که خواهی دید

### `E0308` — جست‌وجویی که شاید شکست بخورد، به کدی داده شده که فرض کرده نخورده

```text
error[E0308]: mismatched types
  --> phase1-fundamentals\06-absence-and-failure\01-option-and-null-safety\examples\06-cannot-use-option-directly.rs:24:26
   |
24 |     println!("{}", greet(name));
   |                    ----- ^^^^ expected `String`, found `Option<String>`
   |                    |
   |                    arguments to this function are incorrect
   |
   = note: expected struct `String`
                found enum `Option<String>`
note: function defined here
  --> phase1-fundamentals\06-absence-and-failure\01-option-and-null-safety\examples\06-cannot-use-option-directly.rs:18:4
   |
18 | fn greet(name: String) -> String {
   |    ^^^^^ ------------
help: consider using `Option::expect` to unwrap the `Option<String>` value, panicking if the value is an `Option::None`
   |
24 |     println!("{}", greet(name.expect("REASON")));
   |                              +++++++++++++++++
```

**کامپایلر به چه اعتراض دارد:** `find_user` یک `Option<String>` برمی‌گرداند، نه یک `String`. `greet` یک `String` می‌خواهد. این دو نوعِ متفاوت‌اند، و کامپایلر آن‌ها را با هم قاطی نمی‌کند — نه حتی برای «راحتی».

**راه‌حل:** اول با احتمالِ `None` روبه‌رو شو، بعد جواب را بده:

```rust
match find_user(7) {
    Some(name) => println!("{}", greet(name)),
    None => println!("no such user"),
}
```

**چرا این راه‌حل است:** پیشنهادِ کمکِ خودِ کامپایلر (`.expect("REASON")`) یک راهِ دیگر است، ولی صادقانه‌تر این است که اسمش را عوض کنی: این یک `panic!` عمدی است، نه رفعِ خطا. اگر واقعاً نبودِ کاربر باید برنامه را متوقف کند، `.expect()` با پیامی که بگوید *چرا* این باید همیشه پیدا شود، بنویس. اگر نه، `match` یا `if let` است که باید بنویسی.

### پنیکِ `.unwrap()` — روی یک مقدارِ `None`

```text
thread 'main' (34052) panicked at phase1-fundamentals\06-absence-and-failure\01-option-and-null-safety\examples\07-unwrap-on-none.rs:16:29:
called `Option::unwrap()` on a `None` value
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**کامپایلر به چه اعتراض دارد:** این حتی خطای کامپایلر نیست — برنامه ساخته شد و اجرا شد و درست همان‌جایی که `find_user(7)` واقعاً `None` برگرداند، پنیک گرفت. `.unwrap()` روی هر `Option<T>` تایپ‌چک می‌شود؛ کامپایلر نمی‌تواند از قبل بداند نتیجه `Some` است یا `None`.

**راه‌حل:** با `match` یا `if let` جوابِ هر دو حالت را بده، یا اگر واقعاً باور داری این `None` نمی‌تواند اتفاق بیفتد، `.expect("...")` بنویس و بگو چرا.

**چرا این راه‌حل است:** پیام فقط می‌گوید *چه چیزی* شکست خورد — `Option::unwrap()` روی `None`. نمی‌گوید کدام مقداری خالی از آب درآمد یا چرا کدِ اطراف انتظار داشت پر باشد. برای آن، `.expect()` را می‌خواهی — بعدی.

### پنیکِ `.expect("...")` — و چه چیزِ اضافه‌ای می‌گوید

```text
thread 'main' (15160) panicked at phase1-fundamentals\06-absence-and-failure\01-option-and-null-safety\examples\08-expect-on-none.rs:17:29:
user 7 should exist in the seed data
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**کامپایلر به چه اعتراض دارد:** همان چیز — یک `None` که `.expect()` رویش صدا زده شده. ولی پیامِ پنیک این‌بار دقیقاً همان متنی است که به `.expect()` دادی، نه یک جمله‌ی عمومی.

**راه‌حل:** همان راه‌حلِ بالا — یک `match` که واقعاً هر دو حالت را جواب بدهد.

**چرا این راه‌حل است:** این پیام را با پیامِ `07-unwrap-on-none` مقایسه کن. یکی می‌گوید «یک `unwrap()` شکست خورد، جایی». دیگری می‌گوید «کاربرِ ۷ باید در داده‌ی seed باشد» — یعنی دقیقاً کدام فرض نقض شده. **این تمامِ دلیلِ وجودِ `.expect()` است:** جایی که واقعاً باور داری `None` نمی‌آید، پیامِ پنیک را جایی بنویس که خواننده لازمش دارد — درونِ خودِ کد، نه در یک کامنت که کسی نمی‌خواندش.

### `E0382` — تطبیق‌دادنِ یک `Option`ِ مالک، آن را حرکت می‌دهد

```text
error[E0382]: use of moved value: `nickname`
  --> phase1-fundamentals\06-absence-and-failure\01-option-and-null-safety\examples\09-matching-by-value-moves-it.rs:15:29
   |
13 |     let nickname = Some(String::from("Matin"));
   |         -------- move occurs because `nickname` has type `Option<String>`, which does not implement the `Copy` trait
14 |     println!("{}", describe(nickname));
   |                             -------- value moved here
15 |     println!("{}", describe(nickname));
   |                             ^^^^^^^^ value used here after move
   |
note: consider changing this parameter type in function `describe` to borrow instead if owning the value isn't necessary
  --> phase1-fundamentals\06-absence-and-failure\01-option-and-null-safety\examples\09-matching-by-value-moves-it.rs:5:23
   |
 5 | fn describe(nickname: Option<String>) -> String {
   |    --------           ^^^^^^^^^^^^^^ this parameter takes ownership of the value
   |    |
   |    in this function
help: consider cloning the value if the performance cost is acceptable
   |
14 |     println!("{}", describe(nickname.clone()));
   |                                     ++++++++
```

**کامپایلر به چه اعتراض دارد:** `describe` مالکیتِ `Option<String>` را می‌گیرد (`nickname: Option<String>`, نه `&Option<String>`). صدا زدنش اولین بار `nickname` را حرکت می‌دهد. `Option<String>`، مثلِ خودِ `String`، `Copy` نیست — پس بارِ دوم چیزی برای صدا زدن نمانده.

**راه‌حل:** امضای `describe` را به یک ارجاع عوض کن:

```rust
fn describe(nickname: &Option<String>) -> String {
    match nickname {
        Some(name) => format!("hello, {name}"),
        None => "hello, stranger".to_string(),
    }
}
```

**چرا این راه‌حل است:** `describe` هیچ‌وقت نمی‌خواست مالکِ `nickname` بشود — فقط می‌خواست نگاهش کند. این دقیقاً همان تمایزِ `Option<&T>` در برابرِ `&Option<T>` است که در بخشِ مفهوم دیدی. پیشنهادِ دومِ کامپایلر — `.clone()` — هم کار می‌کند، ولی هر بار یک بافرِ تازه می‌گیرد فقط برای اینکه بلافاصله آزاد شود؛ [۱.۲.۳](../../02-ownership-and-memory/03-clone-and-copy/README.fa.md) به‌ات یاد داد این را بشناسی: وقتی قرض گرفتن کافی است، کلون گرفتن اشتباه است.

---

## تمرین

### گرم‌کردن

<details>
<summary>این چه چاپ می‌کند؟</summary>

```rust
let x: Option<i32> = Some(0);
match x {
    Some(0) => println!("zero"),
    Some(_) => println!("something else"),
    None => println!("nothing"),
}
```

</details>

<details>
<summary>پاسخ</summary>

```text
zero
```

`Some(0)` با بازوی اولِ `match` جور در می‌آید. `0` یک مقدارِ کاملاً معتبر داخلِ `Some` است — کاملاً چیزِ دیگری از `None`.

</details>

<details>
<summary>چرا نمی‌شود یک <code>Option&lt;u32&gt;</code> را مستقیم به تابعی داد که <code>u32</code> می‌خواهد؟</summary>

چون `Option<u32>` و `u32` دو نوعِ متفاوت‌اند — دقیقاً همان‌قدر متفاوت که `bool` و `i32`. کامپایلر بین‌شان تبدیلِ خودکار انجام نمی‌دهد؛ باید اول با `None` روبه‌رو شوی.

</details>

<details>
<summary>پیامِ پنیکِ <code>.unwrap()</code> چه چیزی به‌ات نمی‌گوید که پیامِ <code>.expect("...")</code> می‌گوید؟</summary>

`.unwrap()` فقط می‌گوید یک `None` باز شده. `.expect("...")` می‌گوید *چرا* نویسنده فکر می‌کرد این نمی‌تواند `None` باشد — همان متنی که خودت نوشتی.

</details>

<details>
<summary><code>Option&lt;&amp;T&gt;</code> همان <code>&amp;Option&lt;T&gt;</code> است؟</summary>

نه. اولی یک `Option` است که اگر چیزی داشته باشد، آن چیز یک ارجاع است. دومی یک ارجاع به کلِ `Option`، هر حالتی که باشد. `.as_ref()` راهِ رفتن از دومی به اولی است.

</details>

<details>
<summary>چرا <code>size_of::&lt;Option&lt;Box&lt;i32&gt;&gt;&gt;()</code> با <code>size_of::&lt;Box&lt;i32&gt;&gt;()</code> برابر است، ولی <code>size_of::&lt;Option&lt;i32&gt;&gt;()</code> از <code>size_of::&lt;i32&gt;()</code> بزرگ‌تر؟</summary>

یک اشاره‌گرِ معتبر هرگز همه-بیت-صفر نیست، پس `None` می‌تواند همان الگو را مجانی قرض بگیرد. `i32` هر الگوی بیتی‌اش را برای یک عددِ واقعی مصرف می‌کند، پس جایی برای `None` نمانده و `Option` باید فضای تازه بگیرد.

</details>

### تعمیر

هر چهار مثالِ خراب را درست کن:

۱. `examples/06-cannot-use-option-directly.rs` را طوری درست کن که کامپایل شود — با یک `match` یا `if let` که واقعاً هر دو حالت را جواب بدهد، نه با `.expect()` روی صداکردنِ `find_user`.
۲. `examples/07-unwrap-on-none.rs` و `examples/08-expect-on-none.rs` را طوری درست کن که دیگر پنیک نگیرند — پیامی معقول برای هر دو حالت چاپ کن.
۳. `examples/09-matching-by-value-moves-it.rs` را **دو** جور درست کن: یک بار با عوض کردنِ امضای `describe` به `&Option<String>`، یک بار با `.clone()` کردنِ `nickname` قبل از صدای دوم. کدام‌شان در یک برنامه‌ی واقعی بهتر است، و چرا؟

### پیاده‌سازی

پنج تابع در `src/lib.rs`:

```sh
cargo test -p p1-06-01-option-and-null-safety
```

بدونِ `.map()`، `.and_then()`، `.unwrap_or()` یا `.ok_or()` — آن‌ها [۱.۶.۲](../02-option-combinators/README.fa.md) هستند. `match` و `if let` برای هر پنج‌تا کافی‌اند.

`index_of_first_negative` را با دقت بخوان. این همان تابعِ [۱.۱.۵](../../01-foundations/05-control-flow/README.fa.md) است، همان که با `readings.len()` برای «پیدا نشد» جواب می‌داد. امضایش را به `-> Option<usize>` عوض کن و ببین صداکننده‌اش چه چیزی از دست می‌دهد که دیگر نمی‌تواند اشتباه بکند.

### بساز

یک `pub fn record_summary(records: &[Option<u32>]) -> String` بنویس که یک خلاصه‌ی یک‌خطی از یک لیستِ نمره‌ی اختیاری تولید کند — چند تا شناخته‌شده‌اند (`Some`)، چند تا نه (`None`) — با فرمتی که خودت انتخاب می‌کنی و در کامنتِ مستنداتِ تابع می‌نویسی.

بعد یک نسخه‌ی دوم بنویس که بالاترین مقدارِ شناخته‌شده را هم گزارش کند — با `match` و یک حلقه، نه با ترکیب‌گر (آن‌ها بعداً می‌آیند).

### چالش (اختیاری)

**بخشِ یک.** حدس بزن، بعد با `size_of` بررسی کن: از این سه، `size_of::<Option<Option<i32>>>()`، `size_of::<Option<Option<bool>>>()` و `size_of::<Option<Option<&i32>>>()`، کدام‌ها دقیقاً به‌اندازه‌ی نسخه‌ی تک‌لایه‌شان‌اند و کدام یکی بزرگ‌تر است؟ (سرنخ: یکی‌شان بزرگ‌تر است. آن یکی از قبل، برای لایه‌ی اول، تنها الگوی بیتیِ اضافه‌اش را خرج کرده بود.)

**بخشِ دو.** (این یکی جلوتر را نگاه می‌کند.) در مستنداتِ استاندارد `Option::flatten` را جست‌وجو کن. حدس بزن روی `Some(Some(5))` و `Some(None::<i32>)` چه برمی‌گرداند — بعد در [۱.۶.۲](../02-option-combinators/README.fa.md) جوابت را بررسی کن.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به کارت می‌آید |
|---|---|---|
| `Option<T>` | شمارشیِ دو-حالته: `Some(T)` یا `None` | هر جا مقدار ممکن است نباشد |
| `Some(x)` / `None` | دو سازنده‌ی `Option` | ساختن و تطبیق |
| `.is_some()` / `.is_none()` | پرسیدنِ حالت، بدونِ باز کردن | چک‌های سریع |
| `.unwrap()` | مقدار را بیرون می‌کشد، یا پنیکِ عمومی | وقتی خودت `Some` را ساخته‌ای |
| `.expect("...")` | مثلِ `.unwrap()`، با پیامِ پنیکِ خودت | وقتی یک قرارداد را باور داری |
| `.as_ref()` | `Option<T>` → `Option<&T>`، بدونِ حرکت | نگاه کردن بدونِ گرفتنِ مالکیت |
| `.cloned()` | `Option<&T>` → `Option<T>`، با کلون | راهِ برگشت از `.as_ref()` |
| بهینه‌سازیِ اشاره‌گرِ تهی | `Option<Box<T>>` هم‌اندازه‌ی `Box<T>` | وقتی `T` خودش یک اشاره‌گر است |

### الان می‌دانی

- `Option<T>` یک شمارشیِ معمولی است با دو حالت — `Some(T)` و `None` — نه یک استثنا و نه یک ارجاعِ تهی.
- چون `Option<T>` نوعش با `T` فرق دارد، کامپایلر نمی‌گذارد یکی را جایِ دیگری استفاده کنی.
- `match` هر دو حالت را مجبورت می‌کند جواب بدهی؛ `if let` این تضمین را می‌دهد از دست، عمداً.
- `.unwrap()` و `.expect("...")` هر دو روی `None` پنیک می‌کنند؛ فرق‌شان در چیزی است که پیامِ پنیک به‌ات می‌گوید.
- `Option<&T>` (یک نگاه، شاید) با `&Option<T>` (ارجاع به کلِ جعبه) یک چیز نیست؛ `.as_ref()` راهِ رفتن از دومی به اولی است.
- `Option<T>` در یک فیلد یعنی «این ممکن است نباشد» — بدونِ سنتینل، بدونِ قراردادِ دستی.
- `Option<Box<T>>`، `Option<&T>` و `Option<bool>` هیچ بایتِ اضافه‌ای نسبت به نسخه‌ی بدونِ `Option`شان نمی‌گیرند؛ `Option<i32>` می‌گیرد.

### بعداً کامل‌تر می‌بینی

- **ترکیب‌گرهای `Option` — `.map()`، `.and_then()`، `.unwrap_or()`، `.filter()`، `.ok_or()`** — [۱.۶.۲](../02-option-combinators/README.fa.md)
- **`Result` و جفتِ شکست‌دارِ `Option`** — [۱.۶.۳](../03-result-and-question-mark/README.fa.md)
- **کِی پنیک کردن درست است و کِی نیست** — [۱.۶.۴](../04-panic-vs-result/README.fa.md)
- **`Box<T>` و چرا یک اشاره‌گرِ هیپ هرگز تهی نیست** — [فاز ۲ — `Box` و تخصیصِ هیپ](../../../phase2-intermediate/05-smart-pointers/01-box-and-heap-allocation/README.fa.md)
- **`Deref`، `AsRef` و خانواده‌ی کاملِ `.as_ref()`** — [فاز ۲ — سطح متوسط](../../../phase2-intermediate/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا کامپایلر نمی‌گذارد یک `Option<u32>` را جایِ `u32` استفاده کنی؟
- `.unwrap()` کِی قابلِ دفاع است؟ یک مثالِ واقعی بزن که خودت پیشتر دیده‌ای (نه از این درس).
- تفاوتِ پیامِ پنیکِ `.unwrap()` و `.expect("...")` چیست؟
- `Option<&T>` و `&Option<T>` را با یک مثال از هم جدا کن.
- `.as_ref()` دقیقاً چه چیزی را عوض می‌کند که یک `match`ِ مستقیم عوضش می‌کرد؟
- چرا `Option<Box<T>>` هم‌اندازه‌ی `Box<T>` است ولی `Option<i32>` نه؟

---

## بیشتر

- [کتابِ Rust — شمارشی‌ها و `Option`](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html#the-option-enum-and-its-advantages-over-null-values) — همین زمین، رسمی.
- [سخنرانیِ تونی هوار در QCon London 2009](https://www.infoq.com/presentations/Null-References-The-Billion-Dollar-Mistake-Tony-Hoare/) — روایتِ خودش از «اشتباهِ میلیارد‌دلاری».
- [`std::option::Option`](https://doc.rust-lang.org/std/option/enum.Option.html) — فهرستِ کاملِ متدهایش؛ همین امروز نگاهی بینداز، حتی به آن‌هایی که هنوز درسشان نرسیده.
- [The Rustonomicon — layout optimizations](https://doc.rust-lang.org/nomicon/repr-rust.html) — جزئیاتِ فنی‌ترِ بهینه‌سازیِ اشاره‌گرِ تهی، برای وقتی کنجکاو شدی چقدر عمیق می‌رود.
