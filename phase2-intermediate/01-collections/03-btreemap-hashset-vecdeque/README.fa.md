# ۲.۱.۳ — `BTreeMap`، `HashSet`، `VecDeque`، `BinaryHeap`

## در یک نگاه

بعد از این درس می‌توانی:

- بینِ `BTreeMap` و `HashMap` برای یک تکه کدِ واقعی انتخاب کنی، و انتخابت را با یک معاوضه‌ی مشخصِ پیچیدگی توجیه کنی — نه با حدس.
- `.range()` را روی `BTreeMap`، متدهای جبرِ مجموعه را روی `HashSet`، هر دو سرِ `VecDeque` را، و `pop()` را روی `BinaryHeap` دقیقاً برای همان کاری به‌کار ببری که هرکدام برایش ساخته شده‌اند.
- یک هیپِ بیشینه را با `Reverse<T>` به هیپِ کمینه تبدیل کنی، و چهار خطایی را که این چهار مجموعه بیشتر از همه به‌جانت می‌اندازند، خودت بخوانی و رفع کنی.

**زمان:** حدود ۹۰ دقیقه · **پیش‌نیاز:**
[۲.۱.۲ — `HashMap` از نزدیک: `entry`، هشرها، جست‌وجو با `&str`](../02-hashmap-in-depth/README.fa.md)

---

## چرا اهمیت دارد

تا اینجا دو شکل از داده داری: `Vec` (یک دنباله) و `HashMap` (یک جست‌وجوی کلید-به-مقدار، بی‌ترتیب و میانگین `O(1)`). این دو برای خیلی از مسئله‌ها کافی‌اند — ولی نه برای همه‌شان، و جاهایی که کافی نیستند، کمبودشان خودش را به‌شکلِ خطای کامپایلر نشان نمی‌دهد؛ به‌شکلِ یک باگِ خاموش نشان می‌دهد.

سه تا مثالِ مشخص: اگر یک گزارش یا یک تست باید هر بار دقیقاً همان ترتیب را نشان بدهد، `HashMap` این را به‌ات قول نمی‌دهد — دیدی که [۲.۱.۲](../02-hashmap-in-depth/README.fa.md) همین را گفت. اگر داری یک صف پیاده می‌کنی که هم باید چیزی به تهش اضافه شود و هم گاهی چیزی از اولش، `Vec` یکی از این دو کار را با هزینه‌ی `O(n)` انجام می‌دهد — بدونِ اینکه هیچ خطایی به‌ات بدهد، فقط کندتر می‌شود، هرچه صف بزرگ‌تر باشد. و اگر مدام باید بپرسی «الان مهم‌ترین کار کدام است؟» — یک صفِ پشتیبانی، یک زمان‌بندِ کار — مرتب کردنِ کلِ لیست هر بار که یک آیتمِ تازه می‌رسد، اسراف است.

این درس چهار ابزار می‌دهد که دقیقاً همین سه مسئله را حل می‌کنند — هرکدام با یک معاوضه‌ی مشخص، نه رایگان. انتخابِ درست بینِ آن‌ها یک مهارت است که این درس یادت می‌دهد، و درسِ بعدی ([۲.۱.۴](../04-choosing-a-collection/README.fa.md)) آن را به یک جدولِ کاملِ تصمیم‌گیری می‌رساند.

---

## مفهوم

### `BTreeMap<K, V>` — همان `HashMap`، فقط مرتب

`BTreeMap` تقریباً همان API را دارد که `HashMap` دارد — `.insert()`، `.get()`، حتی همان entry API که در [۲.۱.۲](../02-hashmap-in-depth/README.fa.md) یاد گرفتی، دقیقاً به همان شکل کار می‌کند — با یک فرق: کلیدهایش را همیشه مرتب نگه می‌دارد. یعنی پیمایشِ یک `BTreeMap` همیشه ورودی‌ها را بر اساسِ کلید، به ترتیبِ صعودی، برمی‌گرداند. همان سه‌تا insert را یک بار در `HashMap` بگذار، یک بار در `BTreeMap`:

```rust
use std::collections::{BTreeMap, HashMap};

let mut counts: HashMap<String, u32> = HashMap::new();
counts.insert("naruto".to_string(), 4);
counts.insert("bleach".to_string(), 2);
counts.insert("frieren".to_string(), 9);

let mut sorted: BTreeMap<String, u32> = BTreeMap::new();
for (title, count) in counts {
    sorted.insert(title, count);
}
for (title, count) in &sorted {
    println!("{title}: {count}");
}
```

```text
bleach: 2
frieren: 9
naruto: 4
```

توجه کن ترتیبِ درجِ داده‌ها در `counts` هیچ اهمیتی نداشت — چه اول `naruto` را می‌گذاشتی چه آخر، پیمایشِ `sorted` همیشه همین سه خط را همین‌طوری می‌دهد. و entry API که یاد گرفتی، همین‌جا هم کار می‌کند:

```rust
*sorted.entry("bocchi".to_string()).or_insert(0) += 1;
*sorted.entry("bleach".to_string()).or_insert(0) += 1;
for (title, count) in &sorted {
    println!("{title}: {count}");
}
```

```text
bleach: 3
bocchi: 1
frieren: 9
naruto: 4
```

این قطعیت رایگان به دست نمی‌آید. `HashMap` یک جست‌وجو را با هش کردنِ کلید مستقیم به یک بازه می‌رسد — میانگین `O(1)`. `BTreeMap` یک درخت را طبقه‌به‌طبقه طی می‌کند — `O(log n)`. برای چند صد یا چند هزار عنصر این فرق را عملاً حس نمی‌کنی؛ ولی هنوز یک فرقِ واقعی است، و به همین دلیل قانونِ سرانگشتی این است: پیش‌فرض برو سراغِ `HashMap`، و فقط وقتی *صریحاً* به پیمایشِ مرتب نیاز داری — یک گزارش، یک تست، یا یک کوئریِ بازه‌ای (که همین حالا می‌بینی) — سراغِ `BTreeMap` برو.

### `.range()` — کوئری بازه‌ای، رایگان از مرتب بودن

چون `BTreeMap` کلیدهایش را مرتب نگه می‌دارد، یک سؤال که `HashMap` هرگز جوابِ ارزانی برایش ندارد را رایگان جواب می‌دهد: «همه‌ی چیزهایی که کلیدشان بینِ X و Y است، به من بده.» با `HashMap` مجبوری هر عنصر را تک‌تک چک کنی — `O(n)`. با `BTreeMap`، متدِ `.range()` مستقیم می‌رود سراغِ همان بخشِ درخت:

```rust
let mut releases: BTreeMap<u32, &str> = BTreeMap::new();
releases.insert(2013, "attack-on-titan");
releases.insert(2023, "frieren");
releases.insert(2019, "demon-slayer");
releases.insert(2022, "bocchi");
releases.insert(2001, "spirited-away");

for (year, title) in releases.range(2019..=2023) {
    println!("{year}: {title}");
}
```

```text
2019: demon-slayer
2022: bocchi
2023: frieren
```

`.range()` همان نحوِ بازه‌ای را می‌گیرد که از برش‌ها می‌شناسی — `2019..=2023` یعنی هر دو سر شامل می‌شوند، `..2019` یعنی «هر چیزِ کمتر از ۲۰۱۹»، و به همین شکل. یک نکته‌ی مهم: خودت باید مطمئن شوی که سرِ شروع از سرِ پایان بزرگ‌تر نیست — در «خطاهایی که خواهی دید» می‌بینی دقیقاً چه اتفاقی می‌افتد اگر این را رعایت نکنی.

### `HashSet<T>` و `BTreeSet<T>` — عضویت، بدون مقدار

`HashSet<T>` از نظرِ مفهومی یک `HashMap<T, ()>` است: فقط می‌پرسد «این مقدار هست یا نه؟»، بدونِ اینکه هیچ داده‌ی اضافه‌ای کنارش نگه دارد.

```rust
use std::collections::HashSet;

let mut watched: HashSet<&str> = HashSet::new();
watched.insert("frieren");
watched.insert("bocchi");
println!("watched frieren? {}", watched.contains("frieren"));
println!("watched naruto?  {}", watched.contains("naruto"));
```

```text
watched frieren? true
watched naruto?  false
```

اگر تا حالا از `set` تو پایتون استفاده کرده باشی، این کاملاً آشناست — با یک فرق: `HashSet` تو Rust هیچ اندیس یا موقعیتی ندارد. `set[0]` تو پایتون هم بی‌معنی است، ولی پایتون فقط `TypeError` می‌دهد؛ کامپایلرِ Rust حتی نمی‌گذارد چنین کدی ساخته شود — در «خطاهایی که خواهی دید» می‌بینی.

و همان رابطه‌ای که بینِ `HashMap` و `BTreeMap` بود، بینِ `HashSet` و `BTreeSet` هم هست: همان عملیاتِ عضویت، این‌بار همیشه مرتب.

```rust
use std::collections::BTreeSet;

let sorted_genres: BTreeSet<&str> = BTreeSet::from(["isekai", "action", "comedy", "drama"]);
for genre in &sorted_genres {
    println!("{genre}");
}
```

```text
action
comedy
drama
isekai
```

### جبرِ مجموعه‌ها: اشتراک، اجتماع، تفاضل

جایی که `HashSet` واقعاً می‌درخشد، همان جبرِ مجموعه‌هایی است که از ریاضیِ دبیرستان یا از `set` پایتون می‌شناسی:

| پایتون | Rust |
|---|---|
| `a & b` | `a.intersection(&b)` |
| `a \| b` | `a.union(&b)` |
| `a - b` | `a.difference(&b)` |
| `a ^ b` | `a.symmetric_difference(&b)` |

تنها جایی که تشبیه می‌شکند: تو پایتون `a & b` یک `set` جدید می‌دهد. تو Rust، هر چهارتای این متدها یک *پیمایشگر (iterator)* از ارجاع‌های قرض‌گرفته‌شده می‌دهند — نه یک `HashSet` تازه. اگر فقط می‌خواهی نتیجه را ببینی یا رویش حلقه بزنی، همین کافی است:

```rust
let action: HashSet<&str> = HashSet::from(["frieren", "bleach", "naruto"]);
let comedy: HashSet<&str> = HashSet::from(["bocchi", "bleach"]);

for title in action.intersection(&comedy) {
    println!("{title}");
}
```

```text
bleach
```

`.union()`، `.difference()` و `.symmetric_difference()` هم دقیقاً به همین شکل کار می‌کنند — هرکدام یک پیمایشگر می‌دهند که می‌توانی رویش حلقه بزنی. اگر بخواهی از نتیجه یک `HashSet`ِ مالکِ تازه بسازی، باید خودت یک `HashSet` خالی بسازی و هر عضو را (کلون‌شده، چون فقط ارجاعش را داری) داخلش بگذاری — دقیقاً همان الگویی که در «تمرین» پیاده می‌کنی.

### `VecDeque<T>` — بافرِ حلقه‌ای، سریع از هر دو سر

از [۲.۱.۱](../01-vec-depth/README.fa.md) یادت هست: `Vec` تهش سریع است — `O(1)` برای `.push()` و `.pop()` — ولی اولش کند: `.insert(0, x)` یا `.remove(0)` مجبورند هر عنصرِ دیگر را یک خانه جابه‌جا کنند، `O(n)`. دلیلش این است که `Vec` یک بلوکِ پیوسته از حافظه است؛ خالی‌کردنِ جا در ابتدایش یعنی هل دادنِ همه‌چیزِ بعدی.

`VecDeque<T>` («صفِ دو-سَر»، double-ended queue) این مشکل را با یک ساختارِ داده‌ای متفاوت حل می‌کند: یک **بافرِ حلقه‌ای (ring buffer)**. به‌جایِ اینکه همیشه از خانه‌ی صفر شروع شود، دو اشاره‌گر — یکی برای جلو، یکی برای عقب — دارد که می‌توانند هر جای بافر باشند و وقتی به ته بافر رسیدند، از اول همان بافر ادامه می‌دهند. نتیجه: افزودن یا برداشتن از *هر دو* سر، `O(1)`.

```senpai-visual
{"kind":"queue","labels":["push_front — O(1)","بافر حلقه‌ای","push_back — O(1)","جلو","عقب"]}
```

```rust
use std::collections::VecDeque;

let mut queue: VecDeque<&str> = VecDeque::new();
queue.push_back("Frieren");
queue.push_back("Bocchi");
queue.push_front("Bleach");
println!("{queue:?}");

let next = queue.pop_front();
println!("{next:?}");
```

```text
["Bleach", "Frieren", "Bocchi"]
Some("Bleach")
```

اگر پایتون‌کار باشی، این دقیقاً `collections.deque` است — همان ایده، همان `.append()`/`.appendleft()`/`.pop()`/`.popleft()`، فقط با اسم‌های دیگر. قانونِ سرانگشتی: اگر فقط از یک سر کار داری، `Vec` کافی و ساده‌تر است؛ اگر باید از هر دو سر — یک صف، یک پنجره‌ی لغزان (sliding window)، یک تاریخچه‌ی «واگرد کن» — `VecDeque` را انتخاب کن.

### `BinaryHeap<T>` — صفِ اولویت‌دار

`BinaryHeap<T>` یک سؤالِ کاملاً متفاوت جواب می‌دهد: نه «مرتب کن»، نه «عضو است یا نه»، بلکه «الان مهم‌ترین چی است؟». هر بار که `.pop()` را صدا می‌زنی، همیشه *بیشینه‌ی* فعلی را برمی‌گرداند — نه به ترتیبِ ورود، نه مرتب‌شده — در `O(log n)`. اگر فقط یک‌بار به بیشینه نیاز داری، `.iter().max()` رویِ یک `Vec` هم کار می‌کند؛ فرقِ `BinaryHeap` وقتی معنا پیدا می‌کند که مدام آیتم‌های تازه اضافه می‌کنی و مدام بیشینه‌ی فعلی را برمی‌داری — بدونِ اینکه هر بار کلِ لیست را از نو مرتب کنی.

```rust
use std::collections::BinaryHeap;

let mut ratings: BinaryHeap<u32> = BinaryHeap::new();
ratings.push(7);
ratings.push(2);
ratings.push(9);
ratings.push(4);

println!("{:?}", ratings.pop());
println!("{:?}", ratings.pop());
```

```text
Some(9)
Some(7)
```

اسمش «هیپ» است، همان کلمه‌ای که در [۱.۲.۱](../../../phase1-fundamentals/02-ownership-and-memory/01-stack-and-heap/README.fa.md) دیدی — ولی اینجا معنایش فرق دارد: آنجا یک ناحیه‌ی حافظه بود، اینجا شکلِ یک ساختارِ داده است (یک درختِ دودویی کامل، ذخیره‌شده داخلِ یک آرایه). دو معنایِ کاملاً جدا از یک کلمه؛ خودِ Rust هم همین دوگانگی را دارد.

جایی که `BinaryHeap` واقعاً به‌کار می‌آید: یک صفِ اولویت‌دار (priority queue). تاپل‌ها به‌ترتیبِ واژه‌نامه‌ای مقایسه می‌شوند — اول عنصرِ اول، و فقط اگر مساوی بود، سراغِ دومی می‌رود — و همین برایِ ساختنِ یک صفِ اولویت‌دار از جفت‌های `(اولویت, عنوان)` کافی است، بدونِ نوشتنِ حتی یک خط کدِ اضافه:

```rust
let mut up_next: BinaryHeap<(u32, &str)> = BinaryHeap::new();
up_next.push((2, "Bocchi"));
up_next.push((5, "Frieren"));
up_next.push((1, "Bleach"));

while let Some((priority, title)) = up_next.pop() {
    println!("priority {priority}: {title}");
}
```

```text
priority 5: Frieren
priority 2: Bocchi
priority 1: Bleach
```

اگر پایتون‌کار باشی، ماژولِ `heapq` را می‌شناسی — همان ایده، با یک تفاوتِ مهم که همین‌جا بگویمش: `heapq` پیش‌فرضش هیپِ کمینه است؛ `BinaryHeap` تو Rust پیش‌فرضش هیپِ بیشینه است. اگر این یکی یادت برود، منطقِ برنامه‌ات بی‌سروصدا برعکس می‌شود — بدونِ هیچ خطایی.

### `Reverse<T>` — همان هیپ، جهتِ برعکس

برای وقتی که واقعاً هیپِ کمینه لازم داری، مجبور نیستی خودت منطقِ مقایسه بنویسی. `std::cmp::Reverse` یک بسته‌بندیِ تک‌فیلدی است که فقط نتیجه‌ی مقایسه را برعکس می‌کند — کوچک‌تر می‌شود بزرگ‌تر، از نگاهِ `BinaryHeap`:

```rust
use std::cmp::Reverse;

let mut low_first: BinaryHeap<Reverse<u32>> = BinaryHeap::new();
low_first.push(Reverse(7));
low_first.push(Reverse(2));
low_first.push(Reverse(9));

println!("{:?}", low_first.pop());
```

```text
Some(Reverse(2))
```

همان `BinaryHeap`، همان `.push()`/`.pop()`، هیچ کدِ تازه‌ای برای «کمینه» نوشته نشد — فقط نوعِ داخلش عوض شد. این دقیقاً همان چیزی است که در «چالش» به کارت می‌آید.

---

## دست‌به‌کد

پنج مثالِ سالم:

```sh
cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 01-btreemap-same-api-sorted
cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 02-btreemap-range-queries
cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 03-hashset-and-set-algebra
cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 04-vecdeque-both-ends
cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 05-binaryheap-max-and-min
```

بعد چهارتای خراب:

```sh
cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 06-btreemap-range-start-after-end --features broken
cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 07-hashset-no-indexing --features broken
cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 08-vec-has-no-pop-front --features broken
cargo run -p p2-01-03-btreemap-hashset-vecdeque --example 09-binaryheap-needs-ord --features broken
```

بعد این‌ها را امتحان کن:

۱. در `01-btreemap-same-api-sorted`، یک عنوانِ چهارم به `counts` (نسخه‌ی `HashMap`) اضافه کن که با حرفِ «a» شروع شود. کجای خروجیِ `sorted` ظاهر می‌شود؟
۲. در `03-hashset-and-set-algebra`، `comedy` را طوری عوض کن که هیچ عضوِ مشترکی با `action` نداشته باشد. خروجیِ `.intersection()` چه می‌شود؟
۳. در `05-binaryheap-max-and-min`، یک عددِ پنجم به `ratings` اضافه کن که از همه بزرگ‌تر باشد، درست قبلِ اولین `.pop()`. آیا `.peek()` که بالاتر چاپ شده بود هنوز درست است؟

---

## خطاهایی که خواهی دید

### پنیکِ `.range()` — وقتی ابتدا از انتها بزرگ‌تر است

```text
thread 'main' (25344) panicked at C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\alloc\src\collections\btree\search.rs:121:21:
range start is greater than range end in BTreeMap
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(عددِ داخلِ پرانتز جلوی `'main'` شناسه‌ی نخ است و هر بار عوض می‌شود؛ بقیه‌ی پیام ثابت است.)

**کامپایلر به چه اعتراض دارد:** این حتی خطای کامپایلر نیست — کد کامپایل می‌شود، چون `.range()` روی هر بازه‌ای از نوعِ درست تایپ‌چک می‌شود. مشکل زمانِ اجرا خودش را نشان می‌دهد: `.range()` بازه‌ای که می‌نویسی را همان‌طور که نوشته‌ای می‌گیرد، خودش برایت مرتبش نمی‌کند. اگر سرِ شروع از سرِ پایان بزرگ‌تر باشد — مثلاً جابه‌جا نوشتنِ «۲۰۲۳ تا ۲۰۱۹» به‌جایِ «۲۰۱۹ تا ۲۰۲۳» — نتیجه پنیک است.

**راه‌حل:** قبل از صدا زدنِ `.range()`، خودت چک کن که شروع از پایان بزرگ‌تر نیست:

```rust
let (start, end) = (2023, 2019);
if start > end {
    println!("no results — start is after end");
} else {
    for (year, title) in releases.range(start..=end) {
        println!("{year}: {title}");
    }
}
```

```text
no results — start is after end
```

**چرا این راه‌حل است:** همان نگهبانی که در [۱.۶.۱](../../../phase1-fundamentals/06-absence-and-failure/01-option-and-null-safety/README.fa.md) دیدی — مورد استثنایی را اول رفعِ زحمت کن، بقیه‌ی کد مجبور نیست به آن فکر کند. `.range()` این نگهبان را خودش نمی‌گذارد، چون نمی‌داند «بازه‌ی خالی» برایت یعنی چه — شاید بخواهی پنیک بگیری، شاید بخواهی یک لیستِ خالی، شاید بخواهی جهتِ بازه را خودت برعکس کنی. تصمیم با توست، و باید قبل از `.range()` گرفته شود.

### `E0608` — نمی‌شود یک `HashSet` را اندیس زد

```text
error[E0608]: cannot index into a value of type `HashSet<&str>`
  --> phase2-intermediate\01-collections\03-btreemap-hashset-vecdeque\examples\07-hashset-no-indexing.rs:13:23
   |
13 |     let first = genres[0];
   |                       ^^^

For more information about this error, try `rustc --explain E0608`.
```

**کامپایلر به چه اعتراض دارد:** `[0]` یعنی «چیزی که در موقعیتِ صفر است»، و این فقط برای چیزهایی معنی دارد که موقعیت دارند — یک `Vec`، یک آرایه، یک برش. `HashSet` یک دنباله نیست؛ عضوهایش موقعیت ندارند، فقط هستند یا نیستند. Rust برایِ `HashSet` هیچ پیاده‌سازیِ اندیس‌گذاری ننوشته، چون چیزی برایِ نوشتن نبود.

**راه‌حل:** سؤالی که واقعاً می‌خواهی بپرسی معمولاً «این عضو هست؟» است، نه «عضوِ صفرم چیست؟»:

```rust
println!("{}", genres.contains("comedy"));
```

```text
true
```

**چرا این راه‌حل است:** `.contains()` دقیقاً همان سؤالی است که یک مجموعه می‌تواند جواب بدهد، و همان چیزی است که تقریباً همیشه از یک `HashSet` واقعاً می‌خواهی. اگر واقعاً — به‌ندرت — فقط یک عضوِ دلخواه (هر کدام) لازم داری، `.iter().next()` این کار را می‌کند؛ ولی کدام عضو را می‌دهد، نامشخص است، چون `HashSet` هیچ ترتیبی تضمین نمی‌کند — دقیقاً همان قاعده‌ای که [۲.۱.۲](../02-hashmap-in-depth/README.fa.md) برایِ `HashMap` گفت.

### `E0599` — `Vec` هیچ‌وقت `pop_front` نداشته

```text
error[E0599]: no method named `pop_front` found for struct `Vec<&str>` in the current scope
  --> phase2-intermediate\01-collections\03-btreemap-hashset-vecdeque\examples\08-vec-has-no-pop-front.rs:12:22
   |
12 |     let next = queue.pop_front();
   |                      ^^^^^^^^^ method not found in `Vec<&str>`

For more information about this error, try `rustc --explain E0599`.
```

**کامپایلر به چه اعتراض دارد:** `Vec` هیچ متدی به اسمِ `pop_front` تعریف نکرده. `.remove(0)` هست — ولی `O(n)` است، چون باید هر عنصرِ باقی‌مانده را یک خانه جابه‌جا کند. `pop_front` اصلاً وجود ندارد، چون معنایی که می‌خواهی — بردار از جلو، در `O(1)` — برایِ ساختارِ داخلیِ `Vec` (یک بلوکِ پیوسته که همیشه از خانه‌ی صفر شروع می‌شود) قابلِ ساخت نیست.

**راه‌حل:** نوع را عوض کن، نه اسمِ متد را:

```rust
let mut queue: VecDeque<&str> = vec!["Frieren", "Bocchi"].into();
let next = queue.pop_front();
println!("{next:?}");
```

```text
Some("Frieren")
```

**چرا این راه‌حل است:** این دقیقاً همان دیواری است که `VecDeque` وجود دارد تا بردارَدش. `.into()` یک `Vec` را به یک `VecDeque` تبدیل می‌کند (عناصر را به بافرِ حلقه‌ایِ تازه منتقل می‌کند)، و از همان لحظه `pop_front` — و `push_front` — هر دو در `O(1)` در دسترس‌اند.

### `E0599` — `BinaryHeap<f64>` هیچ‌وقت راه نمی‌افتد

```text
error[E0599]: the method `push` exists for struct `BinaryHeap<f64>`, but its trait bounds were not satisfied
  --> phase2-intermediate\01-collections\03-btreemap-hashset-vecdeque\examples\09-binaryheap-needs-ord.rs:15:13
   |
15 |     ratings.push(8.5);
   |             ^^^^ method cannot be called on `BinaryHeap<f64>` due to unsatisfied trait bounds
   |
   = note: the following trait bounds were not satisfied:
           `f64: Ord`

For more information about this error, try `rustc --explain E0599`.
```

**کامپایلر به چه اعتراض دارد:** `BinaryHeap<T>` برای پیدا کردنِ بیشینه باید بتواند هر دو عضو را با هم مقایسه کند، پس `T` باید `Ord` باشد — یعنی برای *هر* جفت مقدار، جوابِ «کدام بزرگ‌تر است؟» را قطعی بدهد. `f64` این قول را نمی‌تواند بدهد، چون `NaN` با هیچ عددی — حتی با خودش — قابلِ‌مقایسه نیست. به همین دلیل `f64` فقط `PartialOrd` دارد (مقایسه‌ای که ممکن است جواب نداشته باشد)، نه `Ord`، و `BinaryHeap<f64>` هیچ‌وقت از زمینِ کامپایل بلند نمی‌شود.

**راه‌حل:** یک نوعِ کاملاً مرتب استفاده کن — مثلاً امتیاز را ضربدرِ ۱۰ کن و به‌عنوانِ عدد صحیح نگه دار:

```rust
let mut ratings: BinaryHeap<u32> = BinaryHeap::new();
ratings.push(85); // امتیاز ضربدرِ ۱۰، تا عددِ صحیح بماند
println!("{:?}", ratings.peek());
```

```text
Some(85)
```

**چرا این راه‌حل است:** `u32` هر دو مقدار را همیشه، بدونِ استثنا، می‌تواند مقایسه کند — دقیقاً همان چیزی که `BinaryHeap` نیاز دارد. این تنها راه هم نیست (نوع‌های اعشاریِ کاملاً-مرتب هم به‌عنوانِ crateهای بیرونی وجود دارند)، ولی ساده‌ترین است و هیچ ابزارِ تازه‌ای لازم ندارد.

---

## تمرین

### گرم‌کردن

<details>
<summary>این چه چاپ می‌کند؟</summary>

```rust
use std::collections::BTreeMap;

let mut map: BTreeMap<&str, u32> = BTreeMap::new();
map.insert("naruto", 4);
map.insert("bleach", 2);
map.insert("frieren", 9);

for (title, count) in &map {
    println!("{title}: {count}");
}
```

</details>

<details>
<summary>پاسخ</summary>

```text
bleach: 2
frieren: 9
naruto: 4
```

ترتیبِ درج مهم نبود؛ `BTreeMap` همیشه بر اساسِ کلید، الفبایی، پیمایش می‌شود.

</details>

<details>
<summary>این چه چاپ می‌کند؟</summary>

```rust
use std::collections::VecDeque;

let mut q: VecDeque<i32> = VecDeque::new();
q.push_back(1);
q.push_front(2);
q.push_back(3);
q.push_front(4);
println!("{q:?}");
```

</details>

<details>
<summary>پاسخ</summary>

```text
[4, 2, 1, 3]
```

هر `push_front` به اولِ چیزی که تا آن لحظه ساخته شده اضافه می‌کند، هر `push_back` به آخرش. دنبال کن: `[1]` → `[2, 1]` → `[2, 1, 3]` → `[4, 2, 1, 3]`.

</details>

<details>
<summary>این چه چاپ می‌کند؟</summary>

```rust
use std::collections::BinaryHeap;

let mut heap: BinaryHeap<i32> = BinaryHeap::new();
heap.push(3);
heap.push(8);
heap.push(1);
println!("{:?}", heap.pop());
```

</details>

<details>
<summary>پاسخ</summary>

```text
Some(8)
```

`.pop()` همیشه بیشینه‌ی فعلی را می‌دهد، نه آخرین چیزی که `push` شد و نه چیزی که اول `push` شد.

</details>

<details>
<summary><code>a.intersection(&b)</code> روی دو تا <code>HashSet&lt;String&gt;</code> چه نوعی برمی‌گرداند — یک <code>HashSet</code>ِ تازه؟</summary>

نه. یک پیمایشگر (iterator) از ارجاع‌های قرض‌گرفته‌شده از `a` برمی‌گرداند، نه یک مجموعه‌ی تازه. اگر یک `HashSet<String>`ِ مالک لازم داری، باید خودت یکی خالی بسازی و هر عضو را (کلون‌شده) داخلش بگذاری.

</details>

<details>
<summary>آیا <code>BinaryHeap&lt;f64&gt;</code> کامپایل می‌شود؟</summary>

نه. `BinaryHeap<T>` نیاز دارد `T` نوعی باشد که `Ord` را پیاده کند — یعنی هر دو مقدار همیشه قابلِ‌مقایسه باشند. `f64` به‌خاطرِ `NaN` فقط `PartialOrd` را پیاده می‌کند، نه `Ord`، پس `BinaryHeap<f64>` اصلاً کامپایل نمی‌شود.

</details>

### تعمیر

هر چهار مثالِ خراب را درست کن:

۱. `examples/06-btreemap-range-start-after-end.rs` را طوری درست کن که دیگر پنیک نگیرد — یک نگهبان بگذار که قبل از `.range()` چک کند شروع از پایان بزرگ‌تر نباشد.
۲. `examples/07-hashset-no-indexing.rs` را طوری درست کن که کامپایل شود — با `.contains()` به‌جایِ `[0]`.
۳. `examples/08-vec-has-no-pop-front.rs` را طوری درست کن که کامپایل شود — با تغییرِ نوعِ `queue` از `Vec` به `VecDeque`.
۴. `examples/09-binaryheap-needs-ord.rs` را طوری درست کن که کامپایل شود — بدونِ اینکه معنایِ «امتیاز» را از دست بدهی (نکته: نگهش دار، فقط نوعش را عوض کن).

### پیاده‌سازی

شش قطعه در `src/lib.rs` — دو تابع برایِ `BTreeMap`، دو تابع برایِ `HashSet`، و دو ساختار برایِ `VecDeque` و `BinaryHeap`:

```sh
cargo test -p p2-01-03-btreemap-hashset-vecdeque
```

هیچ‌جا به `.collect()` نیاز نداری — یک `for` ساده که داخلِ یک مجموعه‌ی تازه می‌ریزد، همیشه کافی است. برای `shows_in_year_range` با دقت بخوان: مشخصاتش دقیقاً می‌گوید وقتی `start` از `end` بزرگ‌تر باشد چه باید برگردانی — همان نگهبانی که در «خطاهایی که خواهی دید» دیدی، این‌بار خودت می‌نویسی‌اش.

### بساز

`WatchQueue` را گسترش بده (یا یک ساختارِ کوچکِ تازه بساز) طوری که `enqueue` اگر عنوانی از قبل در صف منتظر باشد، بی‌سروصدا هیچ کاری نکند — بدونِ اینکه هر بار کلِ صف را خطی بگردی. راهنمایی: یک `HashSet<String>` کنارِ `VecDeque<String>` نگه دار تا چک کردنِ «آیا از قبل هست؟» به‌جایِ `O(n)`، `O(1)` بشود. شکلِ دقیقِ API با خودت — این تمرین باز است، تستی برایش نیست.

### چالش (اختیاری)

این یکی از نیازِ این درس فراتر می‌رود — فقط برای وقتی که کنجکاوی چطور از `BinaryHeap` برایِ چیزی جدی‌تر از یک مثال استفاده می‌شود.

فرض کن یک `BTreeMap<u32, String>` بزرگ از سال به عنوان داری و فقط `k` تای *جدیدترین* عنوان را می‌خواهی — بدونِ اینکه کلِ چیز را مرتب کنی. یک `BinaryHeap<Reverse<(u32, String)>>` نگه دار که هیچ‌وقت بیشتر از `k` عضو نداشته باشد: هر عضوِ تازه را اضافه کن، و اگر اندازه از `k` رد شد، بیشینه‌ی این هیپِ کمینه را — یعنی *کوچک‌ترین* سالِ باقی‌مانده — بردار و دور بریز. آخرِ کار، همان `k` تا جدیدترین در دستت است، بدونِ اینکه دیگران را حتی یک بار مرتب کرده باشی.

(اگر همین تمرین را با یک آداپتورِ Iterator انجام می‌دادی، چند خط کوتاه‌تر می‌شد — دقیقاً همان چیزی که [۲.۲](../../02-iterators-and-closures/README.fa.md) یادت می‌دهد. اینجا هنوز نه؛ فقط با حلقه و `if` بنویسش.)

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| `BTreeMap<K, V>` | مثلِ `HashMap`، ولی کلیدها همیشه مرتب‌اند — `O(log n)` به‌جایِ میانگینِ `O(1)` | گزارش، تست، هر جا ترتیبِ پیمایش باید قطعی باشد |
| `.range()` | همه‌ی ورودی‌های یک `BTreeMap` که کلیدشان در یک بازه است | کوئریِ «بینِ X و Y» بدونِ گشتنِ کلِ مجموعه |
| `HashSet<T>` / `BTreeSet<T>` | عضویت بدون مقدار — مثلِ `HashMap<T, ()>`؛ نسخه‌ی دوم همیشه مرتب | چک کردنِ سریعِ «این هست؟»، بدونِ داده‌ی اضافه |
| اشتراک / اجتماع / تفاضل / تفاضلِ متقارن | `.intersection()`، `.union()`، `.difference()`، `.symmetric_difference()` — هرکدام یک پیمایشگر می‌دهند | مقایسه‌ی دو مجموعه بدونِ حلقه‌ی دستی |
| `VecDeque<T>` | بافرِ حلقه‌ای؛ `O(1)` در هر دو سر، برخلافِ `Vec` که فقط تهش سریع است | صف، تاریخچه‌ی واگرد، پنجره‌ی لغزان |
| صفِ اولویت‌دار (priority queue) | ساختاری که همیشه «مهم‌ترینِ فعلی» را بدونِ مرتب‌سازیِ کامل می‌دهد | `BinaryHeap` دقیقاً همین است |
| `BinaryHeap<T>` | `.pop()` همیشه بیشینه را می‌دهد، در `O(log n)`؛ پیمایشِ مستقیمش مرتب نیست | زمان‌بندی، صفِ پشتیبانی، «الان چی؟» |
| `Reverse<T>` | بسته‌بندیِ نتیجه‌ی مقایسه را برعکس می‌کند | تبدیلِ همان `BinaryHeap` به هیپِ کمینه |

### الان می‌دانی

- `BTreeMap` همان API را دارد که `HashMap` دارد — entry API هم — با معاوضه‌ی `O(log n)` در برابرِ پیمایشِ همیشه‌مرتب.
- `.range()` یک کوئریِ بازه‌ای را مستقیم روی بخشِ مربوطه‌ی درخت اجرا می‌کند؛ خودت باید مطمئن شوی سرِ شروع از سرِ پایان بزرگ‌تر نیست.
- `HashSet<T>` عضویت بدون مقدار است، و متدهایِ جبرش پیمایشگر می‌دهند، نه یک `HashSet` تازه.
- `Vec` تهش `O(1)` است، اولش `O(n)`؛ `VecDeque` هر دو سر را `O(1)` می‌دهد، چون بافرِ حلقه‌ای است نه بلوکِ پیوسته.
- `BinaryHeap::pop()` همیشه بیشینه‌ی فعلی را می‌دهد، در `O(log n)`، بدونِ نیاز به مرتب‌سازیِ کامل؛ نیازمندِ نوعی است که `Ord` را پیاده کند، نه فقط `PartialOrd`.
- `Reverse<T>` همان `BinaryHeap` را به هیپِ کمینه تبدیل می‌کند، بدونِ هیچ منطقِ تازه‌ای.
- انتخابِ اشتباهِ مجموعه، خطای کامپایلر نمی‌دهد — یک باگِ خاموشِ کارایی یا ترتیب می‌دهد.

### بعداً کامل‌تر می‌بینی

- **جدولِ کاملِ تصمیم‌گیری بینِ همه‌ی مجموعه‌ها** — [۲.۱.۴ — انتخابِ مجموعه](../04-choosing-a-collection/README.fa.md)
- **`.collect()` و آداپتورهایِ Iterator، برایِ ساختِ یک `HashSet`/`BTreeMap` در یک خط** — [۲.۲ — Iteratorها و کلوژرها](../../02-iterators-and-closures/README.fa.md)
- **پیاده‌سازیِ `Ord`/`PartialOrd`/`Hash` روی نوع‌هایِ خودت، تا بتوانی آن‌ها را هم داخلِ `BinaryHeap` یا `HashSet` بگذاری** — [۲.۳.۴ — دِرایوهایِ استاندارد، دستی](../../03-traits-and-generics/04-standard-derives-by-hand/README.fa.md)
- **نوشتنِ تابعی که رویِ هر نوعِ `Ord` کار کند، نه فقط `u32`** — [۲.۳.۲ — توابع و ساختارهایِ جنریک](../../03-traits-and-generics/02-generic-functions-and-structs/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا `BTreeMap` کندتر از `HashMap` است، و دقیقاً چه چیزی در ازایِ آن کندی می‌گیری؟
- `.range()` چه سؤالی جواب می‌دهد که `HashMap` اصلاً نمی‌تواند رایگان جوابش بدهد؟
- چرا `a.intersection(&b)` یک `HashSet` تازه نمی‌دهد، و اگر یکی لازم داشتی چه کار می‌کنی؟
- چرا `.remove(0)` روی یک `Vec` کند است، و `VecDeque` دقیقاً چطور همین کار را ارزان می‌کند؟
- تفاوتِ `heapq` پایتون با `BinaryHeap` در Rust روی «کدام سر پیش‌فرض است» چیست؟
- چرا `BinaryHeap<f64>` کامپایل نمی‌شود، ولی `BinaryHeap<u32>` می‌شود؟

---

## بیشتر

- [مستنداتِ `BTreeMap`](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html) و [مستنداتِ `HashSet`](https://doc.rust-lang.org/std/collections/struct.HashSet.html) — فهرستِ کاملِ متدهایشان.
- [مستنداتِ `VecDeque`](https://doc.rust-lang.org/std/collections/struct.VecDeque.html) و [مستنداتِ `BinaryHeap`](https://doc.rust-lang.org/std/collections/struct.BinaryHeap.html) — همین‌طور.
- [کتابِ Rust — مجموعه‌های رایج](https://doc.rust-lang.org/book/ch08-00-common-collections.html) — همین چهارتا و رفقایشان، از زبانِ خودِ تیمِ Rust.
- [مستنداتِ `std::cmp::Reverse`](https://doc.rust-lang.org/std/cmp/struct.Reverse.html) — کوچک، ولی همه‌جا از `sort_by_key` تا `BinaryHeap` به کارت می‌آید.
