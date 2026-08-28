# ۲.۱.۲ — `HashMap` از نزدیک: `entry`، هشرها، جست‌وجو با `&str`

## در یک نگاه

بعد از این درس می‌توانی:

- توضیح بدهی `HashMap` چطور جست‌وجو و درج را به‌طورِ میانگین تقریباً آنی (O(1)) انجام می‌دهد، و چرا هیچ‌وقت نباید روی ترتیبِ پیمایشش حساب کنی.
- بینِ `.get()`، `.get_mut()` و ایندکس‌کردنِ مستقیم (`map[key]`) برای یک نگاه یا یک تغییر انتخاب کنی، و بگویی دقیقاً کدام‌یک روی کلیدِ گم‌شده پنیک می‌گیرد.
- کدی را که یک کلید را دوبار جستجو می‌کند — یک‌بار برای چک‌کردن، یک‌بار برای نوشتن — با ای‌پی‌آیِ `entry` (`or_insert`، `or_insert_with`، `and_modify`) به یک جستجوی تکی بازنویسی کنی.
- بگویی چرا کلیدهای `HashMap` باید `Hash` و `Eq` باشند، و یک `struct` خودت را برای کلید‌بودن آماده کنی.
- یک `HashMap<String, V>` را با یک لیترالِ `&str` جستجو کنی، بدونِ اینکه یک `String` تازه فقط برای همان جستجو بسازی.

**زمان:** حدود ۷۵ دقیقه · **پیش‌نیاز:** [۲.۱.۱ — نوع‌های `Vec`](../01-vec-depth/README.fa.md)

---

## چرا اهمیت دارد

تا همین‌جا، هر جا لازم بود چیزی را بر اساسِ یک شناسه پیدا کنی، یک `Vec` داشتی و یک حلقه: بگرد تا آیتمی پیدا شود که فیلدش با آنچه دنبالش هستی برابر باشد. برای ده آیتم این هیچ مشکلی ندارد. برای ده‌هزار آیتم — یک فهرستِ کاربر، یک کش، شمارشِ تکرارِ کلمه‌ها در یک متنِ بلند — هر جستجو یعنی به‌طورِ میانگین نصفِ فهرست را خط‌به‌خط چک کنی. این هزینه اسمِ رسمی دارد: **O(n)**، یعنی زمانِ جستجو با اندازه‌ی فهرست رشد می‌کند.

`HashMap<K, V>` همین مسئله را حل می‌کند: کلید را می‌دهی، مقدار را می‌گیری، بدونِ گشتنِ خطی — به‌طورِ میانگین در زمانِ ثابت (**O(1)**)، فارغ از اینکه نقشه ده تا کلید دارد یا ده میلیون‌تا. این تنها یک بهینه‌سازیِ کوچک نیست؛ نوعی است که تقریباً هر برنامه‌ی واقعیِ Rust به آن نیاز دارد — کشِ یک وب‌سرور، شمارشگرِ یک پردازشگرِ لاگ، پیکربندیِ خوانده‌شده از یک فایل، هرجایی که «این کلید را داری؟» سؤالِ اصلی است.

اگر از پایتون آمده باشی، این چیزِ جدیدی نیست: `HashMap<K, V>` همان کاری را می‌کند که `dict` در پایتون یا `Map` در جاوااسکریپت می‌کند — یک کلید می‌دهی، یک مقدار می‌گیری. ولی یک فرقِ مهم هست که همین‌جای درس رویش مکث می‌کنیم: هم `dict` پایتون (از نسخه‌ی ۳.۷ به بعد) و هم `Map` جاوااسکریپت ترتیبِ درج را به‌خاطر می‌سپارند — یک تضمینِ رسمیِ خودِ زبان. `HashMap` در Rust چنین قولی نمی‌دهد، و این درس همان‌قدر که درباره‌ی *چطور استفاده‌کردن*ش است، درباره‌ی همین *تفاوت* هم هست — چون اگر جایی از کدت، بی‌خبر، روی یک ترتیبِ ثابت حساب کند، یک روز، رویِ یک ماشینِ دیگر یا بعدِ یک ری‌استارتِ ساده، خودش را نشان می‌دهد.

---

## مفهوم

### یک نقشه از کلید به مقدار: `HashMap::new` و `.insert()`

`HashMap<K, V>` را مثلِ یک `Vec<T>` می‌سازی — فقط این‌بار دو نوع داری، نه یکی: نوعِ کلید و نوعِ مقدار.

```rust
let mut watched: HashMap<String, u32> = HashMap::new();
watched.insert("Frieren".to_string(), 12);
watched.insert("Bocchi the Rock".to_string(), 12);
println!("tracked shows: {}", watched.len());

let previous = watched.insert("Frieren".to_string(), 13);
println!("previous count for Frieren: {previous:?}");
```

```text
tracked shows: 2
previous count for Frieren: Some(12)
```

`.insert()` دو کار می‌کند: مقدار را زیرِ آن کلید می‌گذارد، و مقدارِ *قبلی‌ای* که آن‌جا بود را برمی‌گرداند — پیچیده‌شده در `Option`: `None` اگر کلید تازه بود، `Some(old)` اگر داشت جایگزینش می‌کرد. بارِ اول که `"Frieren"` را نوشتیم `None` برگشت (این‌جا چاپش نکردیم)؛ بارِ دوم `Some(12)` — دقیقاً مقدارِ قبلی.

### `.get()` و `.get_mut()` — همان `Option` که از فاز ۱ می‌شناسی

یک نگاه یا یک تغییر، بدونِ ریسکِ پنیک روی یک کلیدِ گم‌شده — این‌جا هم `Option` همان کاری را می‌کند که در [۱.۶.۱](../../../phase1-fundamentals/06-absence-and-failure/01-option-and-null-safety/README.fa.md) برای `Vec`، برش و فیلدهای اختیاری کرد: `.get()` یک `Option<&V>` می‌دهد، `.get_mut()` یک `Option<&mut V>`.

```rust
match watched.get("Frieren") {
    Some(count) => println!("Frieren: {count} episodes"),
    None => println!("Frieren: not tracked"),
}
match watched.get("Your Name") {
    Some(count) => println!("Your Name: {count} episodes"),
    None => println!("Your Name: not tracked"),
}

if let Some(count) = watched.get_mut("Frieren") {
    *count += 1;
}
println!("Frieren now: {:?}", watched.get("Frieren"));
```

```text
Frieren: 13 episodes
Your Name: not tracked
Frieren now: Some(14)
```

هیچ چیزِ تازه‌ای در خودِ `Option` نیست — فقط همان ابزار، این‌بار روی یک نوعِ جدید. `match` و `if let` هر دو دقیقاً همان‌طور کار می‌کنند که روی `Vec::get` یا `.first()`ِ یک برش کار می‌کردند.

راهِ سوم هم هست: مستقیم ایندکس بزنی، `watched["Frieren"]`. کار می‌کند — ولی برخلافِ `.get()`، اگر کلید نباشد پنیک می‌گیری، با پیامِ `no entry found for key`. تا وقتی مطمئن نیستی کلید آن‌جاست، `.get()` انتخابِ امن‌تر است.

### کلیدها باید `Hash` و `Eq` باشند

`HashMap` برای اینکه بداند یک کلید را کجا بگذارد و بعداً از کجا پیدایش کند، به دو چیز از نوعِ کلید نیاز دارد: صفتِ `Hash` (کلید را به یک عددِ هش تبدیل کند) و صفتِ `Eq` (دو کلید را با برابریِ کامل، نه تقریبی، از هم تشخیص بدهد). برای `String`، `&str`، اعدادِ صحیح و `char` این‌ها از قبل آماده‌اند. برای یک `struct` خودت، هر دو فقط یک `#[derive]` فاصله دارند:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Coord {
    row: i32,
    col: i32,
}

let mut board: HashMap<Coord, char> = HashMap::new();
board.insert(Coord { row: 1, col: 2 }, 'O');
println!("{:?}", board.get(&Coord { row: 1, col: 2 }));
```

```text
Some('O')
```

دو `Coord` که جدا از هم ساخته شده‌اند ولی فیلدهاشان یکی است، با `==` برابرند — این `PartialEq`/`Eq` است — و به همان سطلِ داخلیِ نقشه می‌روند — این `Hash` است. بدونِ هرکدام، `.get()` نمی‌تواند بفهمد کلیدی که الان دادی، همان کلیدی است که قبلاً گذاشتی.

(یک جمله درباره‌ی خودِ الگوریتمِ هش، چون سؤالت می‌شود: هشرِ پیش‌فرضِ `HashMap`، به اسمِ SipHash، طوری انتخاب شده که در برابرِ کاربرِ مخربی که عمداً کلیدهایی می‌سازد که همه با هم تصادم کنند مقاوم باشد — نه اینکه سریع‌ترینِ ممکن باشد. هشرهای سریع‌تر برای مسیرهای داغِ برنامه وجود دارند؛ آن یک تعویض است، نه یک بازنویسی، و موضوعِ همین درس نیست.)

### هیچ ترتیبِ تضمین‌شده‌ای در پیمایش نیست

این‌جا همان جایی است که اگر از `dict` پایتون یا `Map` جاوااسکریپت آمده باشی، غافلگیر می‌شوی. شش کلید را به این ترتیب می‌گذاریم: `zeta`، `alpha`، `mu`، `beta`، `quill`، `delta`. بعد دوبار پشتِ سرِ هم، در همین یک اجرا، نقشه را می‌گردیم:

```rust
let mut ranks: HashMap<&str, u32> = HashMap::new();
ranks.insert("zeta", 1);
ranks.insert("alpha", 2);
ranks.insert("mu", 3);
ranks.insert("beta", 4);
ranks.insert("quill", 5);
ranks.insert("delta", 6);

for (name, _) in &ranks {
    print!("{name} ");
}
```

اجرای اول:

```text
zeta mu delta alpha beta quill
```

دقیقاً همین برنامه را — بدونِ تغییرِ یک حرف — دوباره اجرا می‌کنیم:

```text
mu quill alpha zeta delta beta
```

دو نکته این‌جاست. اول: ترتیب اصلاً ترتیبِ درج نیست — `alpha` دومین کلیدی بود که گذاشتیم، ولی نه در اجرایِ اول اول آمد نه در دومی. دوم، و مهم‌تر: **ترتیب بینِ این دو اجرا هم فرق کرد** — همان برنامه، همان کد، همان کلیدها، دو خروجیِ کاملاً متفاوت. (اگر خودِ فایل را چند بار پشتِ سرِ هم اجرا کنی، خودت هم همین را می‌بینی.)

دلیلش تصادفی نیست، عمدی است. هر بار که برنامه اجرا می‌شود، Rust یک کلیدِ تصادفیِ تازه برای هشر انتخاب می‌کند — دقیقاً همان SipHashی که بالا اسمش آمد — تا کسی نتواند از قبل کلیدهایی بسازد که همه به یک سطل بریزند و نقشه را کند کنند (یک حمله‌ی واقعی به اسمِ *hash-flooding*). قیمتش این است که هیچ‌وقت، حتی در یک برنامه‌ی کاملاً قطعی، نمی‌توانی روی ترتیبِ پیمایشِ `HashMap` حساب کنی. اگر ترتیب برایت مهم است — نمایش به کاربر، یک تست که پاسخِ ثابت می‌خواهد، هر چیزی که قرار است یک انسان یا یک `assert_eq!` بخواندش — باید *خودت*، صریح، یک قاعده بنویسی. تابعِ `most_common` در تمرینِ همین درس دقیقاً همین را از تو می‌خواهد.

### ای‌پی‌آیِ `entry`: `or_insert` و `or_insert_with`

فرض کن می‌خواهی تکرارِ هر کلمه را در یک لیست بشماری. راهِ اولی که به ذهن می‌رسد این است: چک کن کلید هست یا نه، بعد یا مقدارش را زیاد کن یا با ۱ بسازش:

```rust
if counts.contains_key(word) {
    let count = counts.get_mut(word).unwrap();
    *count += 1;
} else {
    counts.insert(word, 1);
}
```

این کار می‌کند، ولی هر کلمه را **دوبار** جستجو می‌کند — یک‌بار برایِ `.contains_key()`، یک‌بار برای `.get_mut()` یا `.insert()`. ای‌پی‌آیِ `entry` همین کار را با یک جستجوی تکی انجام می‌دهد: `.entry(key)` جایگاهِ آن کلید را برمی‌گرداند — چه از قبل پر باشد چه خالی — و `.or_insert(default)` می‌گوید «اگر خالی است، همین مقدار را بگذار»، در هر دو حالت یک `&mut V` می‌دهد که مستقیم به همان جایگاه اشاره می‌کند:

```rust
let seen = ["fish", "cat", "fish", "dog", "fish", "cat"];
let mut counts: HashMap<&str, u32> = HashMap::new();

for word in seen {
    *counts.entry(word).or_insert(0) += 1;
}

println!("fish: {}", counts["fish"]);
println!("cat:  {}", counts["cat"]);
println!("dog:  {}", counts["dog"]);
```

```text
fish: 3
cat:  2
dog:  1
```

`*counts.entry(word).or_insert(0) += 1` یک خط است، یک جستجو، بدونِ شاخه‌بندی. اولین بارِ هر کلمه، `.or_insert(0)` یک `0` می‌گذارد و `&mut` به آن می‌دهد؛ `+= 1` آن را ۱ می‌کند. بارهای بعدی، `.or_insert(0)` کاری نمی‌کند — کلید از قبل هست — و همان `&mut` را به مقدارِ موجود می‌دهد.

```senpai-visual
{"kind":"concept","labels":["entry(key)","slot found?","or_insert: fill it","&mut V either way"]}
```

اگر مقدارِ پیش‌فرضت رایگان نیست — مثلاً باید یک `Vec` تازه بسازی — از `.or_insert_with(f)` استفاده کن: `f` فقط وقتی صدا زده می‌شود که کلید واقعاً خالی بوده باشد.

```rust
let mut first_letters: HashMap<char, Vec<&str>> = HashMap::new();
for word in seen {
    let letter = word.chars().next().unwrap();
    first_letters.entry(letter).or_insert_with(Vec::new).push(word);
}

println!("starting with 'f': {:?}", first_letters[&'f']);
```

```text
starting with 'f': ["fish", "fish", "fish"]
```

فرقش با `.or_insert(Vec::new())` ظریف ولی واقعی است: با آن نوشتار، یک `Vec` تازه ساخته می‌شود — و فوراً دور ریخته می‌شود — در *هر* بارِ صدازدن، چه کلید تازه باشد چه نه. `.or_insert_with(Vec::new)` این ساختن را فقط برایِ آن یک لحظه که واقعاً لازم است نگه می‌دارد.

### `and_modify` — وقتی خودِ به‌روزرسانی هم شرطی است

الگویِ کامل‌تر، و آن‌که بیشتر از همه به‌کارت می‌آید: `.and_modify(f)` کلوژرِ `f` را *فقط* وقتی صدا می‌زند که کلید از قبل موجود باشد؛ `.or_insert(default)` مقدارِ اولیه را برای وقتی می‌گذارد که موجود نبوده. (این کلوژر همان چیزی است که در [۱.۶.۲](../../../phase1-fundamentals/06-absence-and-failure/02-option-combinators/README.fa.md) دیدی — تابعِ درجایی که به‌عنوانِ مقدار پاس می‌دهی؛ توضیحِ کاملش، با `Fn`/`FnMut`/`FnOnce`، بعداً می‌آید.) هر دو با هم، دقیقاً الگویِ شمارشِ کلمه است:

```rust
let review = "great show great cast good story great animation good pacing";
let mut counts: HashMap<&str, u32> = HashMap::new();

for word in review.split_whitespace() {
    counts
        .entry(word)
        .and_modify(|count| *count += 1)
        .or_insert(1);
}

println!("great: {}", counts["great"]);
println!("good:  {}", counts["good"]);
```

```text
great: 3
good:  2
```

`.and_modify()` به‌تنهایی هیچ‌وقت کلیدِ تازه نمی‌سازد — چیزی برای «تغییردادن» ندارد. `.or_insert()` است که واقعاً اولین مقدار را می‌گذارد. به همین دلیل هر دو لازم‌اند، و به همین ترتیب.

### جست‌وجو با `&str` روی نقشه‌ای که کلیدش `String` است

این یکی معمولاً غافلگیرکننده است: یک `HashMap<String, V>` کلیدهایش را مالک است، ولی برای *پیدا کردن* یکی از آن‌ها مجبور نیستی خودت هم مالکِ یک `String` باشی.

```rust
let mut ratings: HashMap<String, u8> = HashMap::new();
ratings.insert(String::from("Frieren"), 10);
ratings.insert(String::from("Bocchi the Rock"), 9);

println!("Frieren: {:?}", ratings.get("Frieren"));

let name = String::from("Bocchi the Rock");
println!("{name}: {:?}", ratings.get(&name));
println!("still own it: {name}");
```

```text
Frieren: Some(10)
Bocchi the Rock: Some(9)
still own it: Bocchi the Rock
```

`"Frieren"` این‌جا یک لیترالِ `&str` است — نه `.to_string()`ای در کار است، نه یک `String`ِ تازه که فقط برای همین یک مقایسه ساخته و فوراً دور ریخته شود. `name` هم که یک `String`ِ واقعی است، فقط قرضی داده می‌شود (`&name`) و بعدش هنوز مالِ توست — همان قاعده‌ای که از [۱.۳.۱](../../../phase1-fundamentals/03-borrowing-and-references/01-shared-and-mutable-refs/README.fa.md) می‌شناسی.

این اتفاق نمی‌افتد چون یک استثنا در کدِ `HashMap` است — یک قاعده‌ی عمومی‌تر پشتش است به اسمِ صفتِ `Borrow`، که می‌گوید یک `String` و یک `&str` می‌توانند برای همین‌جور مقایسه‌ای «همان چیز» حساب شوند. این‌جا فقط اعتماد کن که کار می‌کند؛ خودِ مکانیزم — و چرا این خیلی خاص‌ترِ auto-derefِ معمولی است — مالِ [فاز ۲.۴](../../04-lifetimes-and-conversion/03-deref-asref-borrow/README.fa.md) است.

---

## دست‌به‌کد

```sh
cargo run -p p2-01-02-hashmap-in-depth --example 01-new-insert-get
cargo run -p p2-01-02-hashmap-in-depth --example 02-no-guaranteed-order
cargo run -p p2-01-02-hashmap-in-depth --example 03-entry-or-insert
cargo run -p p2-01-02-hashmap-in-depth --example 04-entry-and-modify
cargo run -p p2-01-02-hashmap-in-depth --example 05-custom-key-hash-eq
cargo run -p p2-01-02-hashmap-in-depth --example 06-str-lookup-without-allocating
```

بعد سه‌تای خراب:

```sh
cargo run -p p2-01-02-hashmap-in-depth --example 07-forgot-deref-on-entry --features broken
cargo run -p p2-01-02-hashmap-in-depth --example 08-key-missing-hash-eq --features broken
cargo run -p p2-01-02-hashmap-in-depth --example 09-double-borrow-get-then-insert --features broken
```

بعد این‌ها را امتحان کن:

۱. `02-no-guaranteed-order` را ده بار پشتِ سرِ هم اجرا کن. چند تا ترتیبِ متفاوت می‌بینی؟ آیا هیچ‌کدام دقیقاً ترتیبِ درج (`zeta, alpha, mu, beta, quill, delta`) بود؟
۲. در `03-entry-or-insert`، یک کلمه‌ی تازه (مثلاً `"bird"`) یک‌بار به آرایه‌ی `seen` اضافه کن. `counts["bird"]` بعدِ اجرا چه می‌شود؟
۳. در `05-custom-key-hash-eq`، `#[derive(...)]`ِ بالای `Coord` را طوری عوض کن که فقط `Hash` را داشته باشد (نه `Eq`، نه `PartialEq`). کامپایل کن — کدام کدِ خطا را می‌بینی، و پیامش با نسخه‌ی اصلی چه فرقی دارد؟

---

## خطاهایی که خواهی دید

### `E0368` — روی `&mut u32` نمی‌شود `+=` زد

```text
error[E0368]: binary assignment operation `+=` cannot be applied to type `&mut u32`
  --> phase2-intermediate\01-collections\02-hashmap-in-depth\examples\07-forgot-deref-on-entry.rs:16:9
   |
16 |         counts.entry(word).or_insert(0) += 1;
   |         -------------------------------^^^^^
   |         |
   |         cannot use `+=` on type `&mut u32`
   |
help: `+=` can be used on `u32` if you dereference the left-hand side
   |
16 |         *counts.entry(word).or_insert(0) += 1;
   |         +

For more information about this error, try `rustc --explain E0368`.
```

**کامپایلر به چه اعتراض دارد:** `.or_insert(0)` یک `&mut u32` برمی‌گرداند — آدرسِ جایی که مقدار زندگی می‌کند، نه خودِ عدد. `+=` روی یک ارجاع تعریف نشده؛ روی خودِ نوعِ زیرینش تعریف شده.

**راه‌حل:** پیشنهادِ خودِ کامپایلر را بردار — یک `*` جلوی کلِ عبارت:

```rust
*counts.entry(word).or_insert(0) += 1;
```

**چرا این راه‌حل است:** `*` ارجاع را باز می‌کند و به خودِ `u32`ای که آن‌طرفش است می‌رسد — همان بازکردنِ ارجاعی که [۱.۳.۱](../../../phase1-fundamentals/03-borrowing-and-references/01-shared-and-mutable-refs/README.fa.md) به‌ات یاد داد. `+= 1` بعدش دقیقاً همان جایی را که `entry` پیدا کرده بود تغییر می‌دهد — نه یک کپیِ گذرا.

### `E0599` — کلیدی که `Hash` و `Eq` ندارد

```text
error[E0599]: the method `insert` exists for struct `HashMap<Coord, char>`, but its trait bounds were not satisfied
  --> phase2-intermediate\01-collections\02-hashmap-in-depth\examples\08-key-missing-hash-eq.rs:19:11
   |
12 | struct Coord {
   | ------------ doesn't satisfy `Coord: Eq` or `Coord: Hash`
...
19 |     board.insert(Coord { row: 0, col: 0 }, 'X');
   |           ^^^^^^
   |
   = note: the following trait bounds were not satisfied:
           `Coord: Eq`
           `Coord: Hash`
help: consider annotating `Coord` with `#[derive(Eq, Hash, PartialEq)]`
   |
12 + #[derive(Eq, Hash, PartialEq)]
13 | struct Coord {
   |

For more information about this error, try `rustc --explain E0599`.
```

**کامپایلر به چه اعتراض دارد:** `.insert()` روی `HashMap<K, V>` فقط وقتی موجود است که `K: Hash + Eq` باشد — چون برای گذاشتنِ یک کلید، نقشه باید بتواند هم هشش کند هم بعداً با برابریِ کامل تشخیصش بدهد. `Coord` این‌جا فقط `#[derive(Debug)]` دارد؛ نه `Hash`، نه `Eq`. کامپایلر نمی‌گوید «این نوع بد است» — می‌گوید این متد، روی *این* `HashMap<Coord, char>`ِ به‌خصوص، اصلاً وجود ندارد، چون شرط‌هایش برآورده نشده.

**راه‌حل:** دقیقاً پیشنهادِ کامپایلر:

```rust
#[derive(Debug, Eq, Hash, PartialEq)]
struct Coord {
    row: i32,
    col: i32,
}
```

**چرا این راه‌حل است:** `derive` این سه صفت را از روی فیلدهای `Coord` می‌سازد — چون `i32` خودش `Hash` و `Eq` است، `Coord` هم می‌تواند باشد، فقط با گفتنِ اینکه بخواهی. (`Eq` بدونِ `PartialEq` معنا ندارد؛ همیشه هر دو با هم می‌آیند.)

### `E0502` — یک قرضِ خواندنی که هنوز زنده است، جلویِ یک قرضِ نوشتنی

```text
error[E0502]: cannot borrow `scores` as mutable because it is also borrowed as immutable
  --> phase2-intermediate\01-collections\02-hashmap-in-depth\examples\09-double-borrow-get-then-insert.rs:17:9
   |
16 |     if let Some(current) = scores.get("alice") {
   |                            ------ immutable borrow occurs here
17 |         scores.insert("bob".to_string(), *current);
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
18 |         println!("alice's score is still {current}");
   |                                           ------- immutable borrow later used here

For more information about this error, try `rustc --explain E0502`.
```

**کامپایلر به چه اعتراض دارد:** `current` یک `&i32` است که از `scores.get("alice")` قرض گرفته شده. آخرین استفاده‌اش همان `println!` خطِ بعد است — یعنی قرضِ خواندنی تا آن‌جا زنده می‌ماند. ولی خطِ وسط، `scores.insert(...)`، یک قرضِ نوشتنیِ کاملِ `scores` می‌خواهد. یک قرضِ خواندنیِ زنده به‌علاوه‌ی یک قرضِ نوشتنیِ هم‌زمان — دقیقاً همان قانونِ alias که [۱.۳.۲](../../../phase1-fundamentals/03-borrowing-and-references/02-borrow-checker-rules/README.fa.md) یادت داد، فقط این‌بار قرض از دلِ یک `HashMap` بیرون آمده، نه یک متغیرِ ساده.

```senpai-visual
{"kind":"borrowing","labels":["get(\"alice\") -> &i32","borrow alive until println!","insert() needs &mut scores","collision"]}
```

**راه‌حل:** مقداری را که لازم داری، *قبل* از قرضِ نوشتنی، یک‌جا بردار:

```rust
let alice_score = *scores.get("alice").unwrap();
scores.insert("bob".to_string(), alice_score);
println!("alice's score is still {alice_score}");
```

**چرا این راه‌حل است:** `alice_score` یک `i32` است — یک کپی، نه یک ارجاع — پس هیچ قرضی از `scores` باز نمی‌ماند. تا وقتی این خط تمام شود، `scores` کاملاً آزاد است و `.insert()` می‌تواند قرضِ نوشتنیِ خودش را بگیرد بدونِ برخورد با چیزی. این دقیقاً همان دلیلی است که ای‌پی‌آیِ `entry` وجود دارد: یک جستجو-و-تصمیم را در یک قدم انجام می‌دهد، بدونِ اینکه مجبور شوی دستی مقدار را جدا نگه داری.

---

## تمرین

### گرم‌کردن

<details>
<summary>این چه چاپ می‌کند؟</summary>

```rust
let mut m: HashMap<&str, i32> = HashMap::new();
m.insert("a", 1);
let old = m.insert("a", 2);
println!("{old:?}");
```

</details>

<details>
<summary>پاسخ</summary>

```text
Some(1)
```

`.insert()` همیشه مقدارِ *قبلیِ* آن کلید را برمی‌گرداند، پیچیده در `Option`. بارِ اول کلید تازه بود، ولی این‌جا فقط بارِ دوم را چاپ کردیم؛ آن‌موقع مقدارِ قبلی `1` بود.

</details>

<details>
<summary>این کامپایل می‌شود؟ اگر بله، وقتِ اجرا چه اتفاقی می‌افتد؟</summary>

```rust
let scores: HashMap<&str, i32> = HashMap::new();
println!("{}", scores["missing"]);
```

</details>

<details>
<summary>پاسخ</summary>

بله، کامپایل می‌شود — نوع‌ها درست‌اند. ولی وقتِ اجرا پنیک می‌گیری، با پیامِ `no entry found for key`. ایندکس‌کردنِ مستقیم (`[]`) فرض می‌کند کلید آن‌جاست؛ اگر مطمئن نیستی، `.get()` را می‌خواهی.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
let mut counts: HashMap<&str, u32> = HashMap::new();
counts.entry("a").or_insert(0) += 1;
```

</details>

<details>
<summary>پاسخ</summary>

نه. `.or_insert(0)` یک `&mut u32` می‌دهد، نه یک `u32`؛ `+=` باید روی خودِ عدد بیاید، بعدِ یک `*`. کدِ خطا `E0368` است — کاملِ ماجرا در «خطاهایی که خواهی دید».

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
#[derive(Debug)]
struct Tag(String);

let mut seen: HashMap<Tag, u32> = HashMap::new();
seen.insert(Tag("x".to_string()), 1);
```

</details>

<details>
<summary>پاسخ</summary>

نه. `Tag` فقط `Debug` دارد، نه `Hash` و `Eq`. `.insert()` روی `HashMap<Tag, u32>` بدونِ آن دو صفت اصلاً وجود ندارد. کدِ خطا `E0599` است.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
let mut scores: HashMap<String, i32> = HashMap::new();
scores.insert("alice".to_string(), 10);

if let Some(current) = scores.get("alice") {
    scores.insert("bob".to_string(), *current);
    println!("{current}");
}
```

</details>

<details>
<summary>پاسخ</summary>

نه. `current` یک قرضِ خواندنیِ `scores` است که تا `println!` زنده می‌ماند؛ `.insert()` وسطِ همین بازه یک قرضِ نوشتنی می‌خواهد. کدِ خطا `E0502` است.

</details>

### تعمیر

هر سه مثالِ خراب را درست کن:

۱. `examples/07-forgot-deref-on-entry.rs` — یک `*` در جایِ درست اضافه کن تا `+=` روی خودِ عدد بیاید، نه روی ارجاع.
۲. `examples/08-key-missing-hash-eq.rs` — `#[derive(...)]`ِ بالای `Coord` را طوری کامل کن که `Hash` و `Eq` (و `PartialEq`) هم داشته باشد.
۳. `examples/09-double-borrow-get-then-insert.rs` — مقدارِ لازم را قبل از `.insert()` در یک متغیرِ جدا (یک `i32` مالک، نه یک ارجاع) بردار، طوری که وقتِ صدازدنِ `.insert()` هیچ قرضی از `scores` باز نمانده باشد.

### پیاده‌سازی

پنج تابع در `src/lib.rs`:

```sh
cargo test -p p2-01-02-hashmap-in-depth
```

بدونِ `BTreeMap`، `HashSet` یا `VecDeque` — آن‌ها [۲.۱.۳](../03-btreemap-hashset-vecdeque/README.fa.md) هستند. `HashMap` و ای‌پی‌آیِ `entry` برای هر پنج‌تا کافی‌اند.

`most_common` را با دقت بخوان: مشخصاتش یک قاعده‌ی رفعِ تساوی می‌خواهد (کلیدِ الفبایی‌زودتر می‌برد) — دقیقاً به‌همین‌دلیل که «هیچ ترتیبِ تضمین‌شده‌ای در پیمایش نیست» را چند صفحه‌ی قبل خواندی. بدونِ آن قاعده، تابعت ممکن است رویِ ماشینِ خودت درست به‌نظر برسد، ولی بینِ دو اجرا جوابش عوض شود.

### بساز

یک `pub fn` بنویس که چیزی را که خودت انتخاب می‌کنی — ژانرهای یک لیستِ انیمه، کدهایِ وضعیتِ HTTP در یک لاگ، موادِ اولیه‌ی چند دستورِ آشپزی، هر چیزی — با ای‌پی‌آیِ `entry` بشمرد یا گروه‌بندی کند. فرمتِ دقیقِ ورودی و خروجی را خودت انتخاب کن و در کامنتِ مستنداتِ تابع بنویس، بعد دست‌کم دو تست برایش اضافه کن.

### چالش (اختیاری)

یک `pub fn merge_counts(a: &HashMap<String, u32>, b: &HashMap<String, u32>) -> HashMap<String, u32>` بنویس که دو نقشه‌ی شمارش را با هم ادغام کند: هر کلیدی که در هرکدام هست در خروجی باشد، و اگر کلیدی در هر دو بود، مقدارهایش با هم جمع شوند. سعی کن با یک نقشه‌ی نتیجه شروع کنی (کپیِ `a`) و فقط روی `b` با `entry` + `and_modify` + `or_insert` حلقه بزنی — نه با ساختنِ نقشه از صفر و پیمایشِ دستیِ هر دو.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| `HashMap<K, V>` | نقشه‌ی کلید به مقدار، با جست‌وجویِ میانگین O(1) | هر جا «این کلید را داری؟» سؤالِ اصلی است |
| ای‌پی‌آیِ `entry` | یک جستجو، به‌جایِ جستجو-چک-جستجو | شمارش، گروه‌بندی، به‌روزرسانیِ شرطی |
| `.or_insert(v)` | مقدارِ پیش‌فرض را مشتاقانه می‌سازد | وقتی پیش‌فرض رایگان است (یک عددِ ساده) |
| `.or_insert_with(f)` | مقدارِ پیش‌فرض را فقط اگر لازم شود می‌سازد | وقتی پیش‌فرض کار دارد (مثلِ `Vec::new`) |
| `.and_modify(f)` | `f` را فقط اگر کلید از قبل بود صدا می‌زند | «اگر بود تغییرش بده، اگر نبود بسازش» |
| صفتِ `Hash` / `Eq` | آنچه یک نوع را برای کلید‌بودن آماده می‌کند | هر `struct` یا `enum` که می‌خواهی کلیدِ `HashMap` باشد |
| هَشر (hasher) | چیزی که یک کلید را به یک عددِ هش تبدیل می‌کند | پیش‌فرض: SipHash، در برابرِ hash-flooding مقاوم |

### الان می‌دانی

- `HashMap<K, V>` جست‌وجو و درج را به‌طورِ میانگین در زمانِ ثابت انجام می‌دهد؛ هزینه‌اش این است که هیچ ترتیبِ پیمایشی تضمین نمی‌کند — نه ترتیبِ درج، نه حتی ترتیبِ ثابت بینِ دو اجرای همان برنامه.
- `.get()`/`.get_mut()` همان `Option<&V>`/`Option<&mut V>`ی را می‌دهند که از فاز ۱ می‌شناسی؛ ایندکس‌کردنِ مستقیم (`[]`) روی کلیدِ گم‌شده پنیک می‌گیرد.
- ای‌پی‌آیِ `entry` — `or_insert`، `or_insert_with`، `and_modify` — یک جستجوی دوگانه را به یکی تبدیل می‌کند؛ `.or_insert_with(f)` را فقط وقتی پیش‌فرض واقعاً کار دارد بردار.
- کلیدهای `HashMap` باید `Hash` و `Eq` باشند؛ برای یک `struct` خودت، هر دو یک `#[derive]` فاصله دارند.
- `HashMap<String, V>` را می‌شود با یک `&str` جستجو کرد، بدونِ ساختنِ یک `String` تازه فقط برای همان جستجو.

### بعداً کامل‌تر می‌بینی

- **`BTreeMap`، `HashSet`، `VecDeque`** — [۲.۱.۳](../03-btreemap-hashset-vecdeque/README.fa.md)
- **کدام مجموعه را کِی انتخاب کنی** — [۲.۱.۴ — انتخابِ یک مجموعه](../04-choosing-a-collection/README.fa.md)
- **کلوژرها، `Fn`/`FnMut`/`FnOnce` و قرض‌گیریِ محیط** — [۲.۲.۱ — کلوژرها و صفت‌های Fn](../../02-iterators-and-closures/01-closures-and-fn-traits/README.fa.md)
- **مکانیزمِ صفتِ `Borrow` — چرا `.get("literal")` روی `HashMap<String, V>` کار می‌کند** — [۲.۴.۳ — Deref، AsRef، Borrow](../../04-lifetimes-and-conversion/03-deref-asref-borrow/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا `HashMap` هیچ ترتیبِ پیمایشی تضمین نمی‌کند، و این تضمین‌نکردن دقیقاً چه مشکلی را حل می‌کند؟
- تفاوتِ `.or_insert()` و `.or_insert_with()` را با یک مثال که فرقشان واقعاً مهم است توضیح بده.
- چرا `.and_modify()` به‌تنهایی هیچ‌وقت یک کلیدِ تازه نمی‌سازد؟
- یک نوعِ خودت را تصور کن که می‌خواهی کلیدِ یک `HashMap` باشد. چه دو صفتی لازم دارد، و چرا هرکدام؟
- چرا `.get("literal")` روی یک `HashMap<String, V>` بدونِ تخصیصِ یک `String` تازه کار می‌کند؟

---

## بیشتر

- [کتابِ Rust — ذخیره‌کردنِ کلید و مقدار با HashMap](https://doc.rust-lang.org/book/ch08-03-hash-maps.html) — همین زمین، از زبانِ خودِ تیمِ Rust.
- [مستنداتِ `std::collections::HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html) — فهرستِ کاملِ متدهایش.
- [مستنداتِ `std::collections::hash_map::Entry`](https://doc.rust-lang.org/std/collections/hash_map/enum.Entry.html) — همه‌ی متدهایِ ای‌پی‌آیِ `entry`، نه فقط سه‌تایی که امروز دیدی.
- [مقاله‌ی اصلیِ SipHash](https://www.aumasson.jp/siphash/siphash.pdf) — برای وقتی کنجکاو شدی چرا این هشرِ به‌خصوص انتخاب شده.
