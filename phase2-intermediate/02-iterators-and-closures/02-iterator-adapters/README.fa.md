# ۲.۲.۲ — ترکیب‌گرهای ایتریتور

## در یک نگاه

بعد از این درس می‌توانی:

- توضیح بدهی که `Iterator` فقط یک متدِ اجباری دارد — `next(&mut self)` — و هر ترکیب‌گری که در این درس می‌بینی، از `map` تا `flat_map`، فقط دور همین یک متد پیچیده شده.
- برای یک نیازِ مشخص، ترکیب‌گرِ درست را انتخاب کنی: تبدیل با `map`، فیلتر با `filter` یا `filter_map`، محدودسازی با `take`/`skip` (یا نسخه‌های `while`دارشان)، یا ترکیبِ چند پیمایشگر با `enumerate`/`zip`/`chain`.
- بگویی چرا ساختنِ یک زنجیره‌ی ترکیب‌گر به‌تنهایی هیچ کاری انجام نمی‌دهد، و دقیقاً چه چیزی — یک `for`، یا خودِ `fold` — آن زنجیره را واقعاً به حرکت درمی‌آورد.

**زمان:** حدود ۸۰ دقیقه · **پیش‌نیاز:** [۲.۲.۱ — کلوژرها و صفت‌های `Fn`](../01-closures-and-fn-traits/README.fa.md)، و به‌طورِ خاص [۱.۱.۶ — نوع‌های `Vec` و `String`](../../../phase1-fundamentals/01-foundations/06-vec-and-string-basics/README.fa.md)

---

## چرا اهمیت دارد

تا همین‌جا، هر بار خواسته‌ای یک لیست را تبدیل کنی، فیلتر کنی یا از رویش چیزی جمع بزنی، دستت به یک حلقه‌ی دستی رفته:

- [۱.۱.۵](../../../phase1-fundamentals/01-foundations/05-control-flow/README.fa.md) یک حلقه‌ی دستی نوشت که تک‌تکِ عناصر را می‌گشت و اندیسِ اولین منفی را نگه می‌داشت.
- [۲.۱.۱](../../01-collections/01-vec-depth/README.fa.md) به‌ات `.retain(|entry| !entry.watched)` را نشان داد — یک کلوژر که برایِ هر عنصر می‌گوید «بماند یا برود» — بدونِ اینکه هنوز اسمش را «فیلتر کردن» بگذارد.
- [۲.۱.۲](../../01-collections/02-hashmap-in-depth/README.fa.md) رویِ جفت‌های یک `HashMap` با `for (k, v) in map.iter()` گشت.
- و [۲.۲.۱](../01-closures-and-fn-traits/README.fa.md) — درسِ درست قبل از همین یکی — واژگانِ `|x| ...` را به‌ات داد که از این‌جا به بعد در هر خط از این درس می‌بینی.

این درس آن حلقه‌های پراکنده را یک زبانِ واحد می‌کند. اگر تا حالا پایتون کار کرده باشی، جایی که آن‌جا سراغِ یک list comprehension می‌رفتی —

```python
uppercased = [title.upper() for title in titles if title.completed]
```

— این‌جا سراغِ یک **زنجیره‌ی ترکیب‌گر** می‌روی:

```rust
titles.iter().filter(|t| t.completed).map(|t| t.title.to_uppercase())
```

شباهت واقعی است: هر دو، یک تبدیل و یک شرط را روی یک لیست توصیف می‌کنند. ولی یک فرقِ اساسی هست، و همین درس دقیقاً حولِ همان فرق می‌چرخد: خطِ پایتون بالا، همان لحظه‌ای که نوشته می‌شود، با ولع اجرا می‌شود. خطِ Rust بالا — همان‌طور که تا آخرِ درس می‌بینی — **هیچ کاری نمی‌کند**، تا وقتی چیزی واقعاً مصرفش کند. این یکی از معدود جاهایی است که پلِ پایتون درست کارِ خودش را می‌کند و درست همان‌جا هم می‌شکند.

---

## مفهوم

### فقط یک متدِ اجباری: `next(&mut self)`

هر چیزی که در Rust قابلِ پیمایش باشد، در ته‌ِ ته‌اش فقط یک قرارداد را برآورده می‌کند: یک متد که هر بار صدایش بزنی، یا آیتمِ بعدی را با یک `Some` پس می‌دهد، یا با یک `None` می‌گوید «دیگر آیتمی نیست». به این قرارداد **Iterator** می‌گویند، و این یک متد — به‌شکلِ خلاصه، `next(&mut self) -> Option<Item>` — تنها چیزی است که باید پیاده‌سازی شود. بقیه‌ی هر چیزی که در این درس می‌بینی، از `map` تا `fold`، فقط لایه‌ای است روی همین یک متد.

بیا خودت با دست صدایش بزنی:

```rust
let scores = vec![10, 20, 30];
let mut by_hand = scores.iter();

println!("{:?}", by_hand.next());
println!("{:?}", by_hand.next());
println!("{:?}", by_hand.next());
println!("{:?}", by_hand.next());
```

```text
Some(10)
Some(20)
Some(30)
None
```

نکته را جدی بگیر: `by_hand` باید `mut` باشد. هر صدا زدنِ `next()` موقعیتِ داخلیِ پیمایشگر را جلو می‌برد — یعنی خودِ پیمایشگر را تغییر می‌دهد — و امضایش هم دقیقاً همین را می‌گوید: `&mut self`. اگر `mut` را جا بیندازی، دقیقاً همین‌جا اولین خطای این درس را می‌گیری؛ کاملش در «خطاهایی که خواهی دید» است.

یک `for` هیچ جادویی ندارد؛ دقیقاً همین حلقه است، فقط خودکار:

```rust
let mut manual = scores.iter();
loop {
    match manual.next() {
        Some(value) => println!("{value}"),
        None => break,
    }
}
```

```text
10
20
30
```

```senpai-visual
{"kind":"concept","labels":["v.iter()","next()","Some(item)","next()","None"]}
```

از این‌جا به بعد، هر بار که می‌بینی یک ترکیب‌گر «یک آیتم را می‌گیرد و کاری می‌کند»، همان یک متدِ بالا در پسِ‌زمینه دارد صدا زده می‌شود.

### تبدیل: `map`

`.map(closure)` یک پیمایشگرِ تازه پس می‌دهد که هر آیتم را از دلِ کلوژر رد می‌کند — نوعِ ورودی و خروجی هم لازم نیست یکی باشد:

```rust
let titles = vec!["frieren".to_string(), "bocchi the rock!".to_string()];

for title in titles.iter().map(|t| t.to_uppercase()) {
    println!("{title}");
}
```

```text
FRIEREN
BOCCHI THE ROCK!
```

از `&String` رفتیم به `String` بزرگ‌شده. `.map()` رویِ نوع هیچ اصراری ندارد — همان‌قدر راحت می‌توانست از `&String` به `usize` برود:

```rust
for length in titles.iter().map(|t| t.len()) {
    println!("length: {length}");
}
```

```text
length: 7
length: 16
```

### فیلتر: `filter` و `filter_map`

`.filter(predicate)` فقط آیتم‌هایی را نگه می‌دارد که کلوژرش `true` جواب بدهد:

```rust
for show in watchlist.iter().filter(|s| s.completed) {
    println!("completed: {}", show.title);
}
```

```text
completed: Frieren
completed: Mushoku Tensei
```

حالا این حالت را ببین: یک لیست از رشته‌های تایپ‌شده‌ی کاربر که قرار است به عددِ قسمت تبدیل شوند — بعضی‌هایشان اصلاً عدد نیستند. اگر فقط `.map()` بزنی، به‌جای هر آیتم یک `Result` می‌گیری، پر یا خالی:

```rust
let typed = vec!["12", "twelve", "24", "", "37"];
for parsed in typed.iter().map(|t| t.parse::<u32>()) {
    println!("{parsed:?}");
}
```

```text
Ok(12)
Err(ParseIntError { kind: InvalidDigit })
Ok(24)
Err(ParseIntError { kind: Empty })
Ok(37)
```

`.filter_map(closure)` همان کار را می‌کند — کلوژرش هم یک `Option` پس می‌دهد — با یک فرق: هر `None` بی‌سروصدا از توالی می‌افتد، و آنچه می‌ماند دیگر پیچیده در `Some` نیست:

```rust
for n in typed.iter().filter_map(|t| t.parse::<u32>().ok()) {
    println!("{n}");
}
```

```text
12
24
37
```

یعنی `.filter_map()` یک `.map()` و یک `.filter()` را در یک عبور واحد انجام می‌دهد — نه دو عبورِ جدا.

### محدودسازی: `take`/`take_while` و `skip`/`skip_while`

`.take(n)` و `.skip(n)` با شمارش کار می‌کنند — اولین/باقیِ `n` تا:

```rust
let ratings = vec![9, 8, 9, 4, 7, 2];
for r in ratings.iter().take(3) {
    println!("take(3): {r}");
}
```

```text
take(3): 9
take(3): 8
take(3): 9
```

`.take_while(predicate)` و `.skip_while(predicate)` به‌جایِ شمارش، با یک شرط کار می‌کنند — و این‌جا نکته‌ی تنبلی (laziness) را با چشمِ خودت می‌بینی. کلوژرِ زیر هر آیتمی را که می‌بیند اعلام می‌کند:

```rust
let announced = ratings.iter().take_while(|r| {
    println!("  checking {r}");
    **r >= 8
});
for r in announced {
    println!("kept: {r}");
}
```

```text
  checking 9
kept: 9
  checking 8
kept: 8
  checking 9
kept: 9
  checking 4
```

بعد از `4` (که شرط را رد می‌کند) هیچ `checking`ِ دیگری نمی‌بینی — نه برایِ `7`، نه برایِ `2`. `.take_while()` همان لحظه که شرط رد شود کلِ زنجیره را کوتاه می‌کند (short-circuit)؛ حتی زحمتِ نگاه کردن به باقیِ لیست را هم به خودش نمی‌دهد. این با `.filter()`، که همیشه تا آخرِ لیست را می‌بیند، اساساً فرق دارد. `.skip_while()` دقیقاً تصویرِ آینه‌ای‌اش است: تا وقتی شرط برقرار است رد می‌شود، از اولین شکست به بعد همه‌چیز را نگه می‌دارد، و دیگر شرط را چک نمی‌کند:

```rust
for r in ratings.iter().skip_while(|r| **r >= 8) {
    println!("skip_while(>= 8): {r}");
}
```

```text
skip_while(>= 8): 4
skip_while(>= 8): 7
skip_while(>= 8): 2
```

### ترکیبِ چند پیمایشگر: `enumerate`، `zip`، `chain`

`.enumerate()` هر آیتم را با موقعیتش (از ۰) جفت می‌کند:

```rust
for (i, title) in titles.iter().enumerate() {
    println!("{i} -> {title}");
}
```

```text
0 -> Frieren
1 -> Bocchi the Rock!
2 -> Mushoku Tensei
```

`.zip(other)` دو پیمایشگر را جفت‌جفت با هم پیش می‌برد — و همین که یکی از دو طرف تمام شود، کلِ جفت‌کردن متوقف می‌شود، حتی اگر طرفِ دیگر هنوز آیتم داشته باشد:

```rust
let ratings = vec![9, 8]; // titles سه‌تاست، ratings فقط دوتا
for (title, r) in titles.iter().zip(ratings.iter()) {
    println!("{title} rated {r}");
}
```

```text
Frieren rated 9
Bocchi the Rock! rated 8
```

«Mushoku Tensei» هیچ‌وقت ظاهر نمی‌شود — نه خطایی، نه پنیکی، فقط بی‌سروصدا کنار گذاشته می‌شود چون جفتش نبود. `.chain(other)` کاملاً کارِ متفاوتی می‌کند: هیچ‌چیزی را جفت نمی‌کند، فقط وقتی اولی تمام شد، دومی را از سرِ نو شروع می‌کند — یک پیمایشِ پشتِ‌سرِهم، بدونِ ساختنِ لیستِ تازه‌ای که هر دو را نگه دارد:

```rust
let already_watched = vec!["Frieren", "AOT"];
let plan_to_watch = vec!["Bocchi the Rock!", "Chainsaw Man"];
for title in already_watched.iter().chain(plan_to_watch.iter()) {
    println!("{title}");
}
```

```text
Frieren
AOT
Bocchi the Rock!
Chainsaw Man
```

### معکوس‌سازی: `rev` — و چرا هرکسی این حق را ندارد

`.rev()` همان پیمایش را از عقب به جلو انجام می‌دهد. ولی فقط رویِ پیمایشگرهایی کار می‌کند که هم می‌دانند از جلو کجاست، هم می‌دانند از عقب کجاست — یعنی علاوه بر `next()`، متدِ `next_back()` را هم دارند. به این توانایی **`DoubleEndedIterator`** می‌گویند، و همه‌ی پیمایشگرها آن را ندارند. رویِ یک برش، هر دو سر مشخص است:

```rust
let recently_added = vec!["Frieren", "Bocchi the Rock!", "Mushoku Tensei"];
let mut both_ends = recently_added.iter();
println!("{:?}", both_ends.next());
println!("{:?}", both_ends.next_back());
println!("{:?}", both_ends.next());
```

```text
Some("Frieren")
Some("Mushoku Tensei")
Some("Bocchi the Rock!")
```

دقیقاً «دوسر» یعنی همین: از یک طرف می‌کشی، از طرفِ دیگر هم می‌کشی، و در وسط به هم می‌رسند. `.rev()` فقط همین ایده را به‌شکلِ یک ترکیب‌گر بسته‌بندی می‌کند:

```rust
for title in recently_added.iter().rev() {
    println!("{title}");
}
```

```text
Mushoku Tensei
Bocchi the Rock!
Frieren
```

اما یک `HashMap` نه «جلو» دارد نه «عقب» — حتی ترتیبِ پیمایشش هم تضمین‌شده نیست، چه برسد به دو سرش. پیمایشگرش `DoubleEndedIterator` نیست، و `.rev()` رویش اصلاً کامپایل نمی‌شود؛ کاملش در «خطاهایی که خواهی دید».

### `fold` — ابزارِ همه‌کاره

هر ترکیب‌گرِ بالا یک پیمایشگرِ تازه پس می‌داد. `.fold(initial, |acc, item| ...)` فرق دارد: یک مقدارِ شروع می‌گیرد، آن مقدار را از دلِ هر آیتم رد می‌کند، و در پایان همان یک مقدار را — نه یک پیمایشگر — پس می‌دهد:

```rust
let episodes = vec![28, 12, 24];
let total = episodes.iter().fold(0, |acc, n| acc + n);
println!("{total}");
```

```text
64
```

این دقیقاً معادلِ همان حلقه‌ی دستی‌ای است که خودت می‌نوشتی:

```rust
let mut total = 0;
for n in episodes.iter() {
    total += n;
}
println!("{total}");
```

```text
64
```

اگر هر بخشِ این درس را کنار بگذاری، `.fold()` همان یکی است که باید بماند: هر ترکیب‌گر، هر متدِ مصرف‌کننده‌ای که بعداً می‌بینی، در تهِ‌تهش دارد همین کار را می‌کند — یک مقدار نگه می‌دارد، آن را با هر آیتم به‌روزرسانی می‌کند. `.fold()` فقط همان الگو را، صریح و به‌دستِ خودت، در یک متد گذاشته. و آن مقدار مجبور نیست عدد باشد — می‌تواند یک `Vec` باشد که خودت می‌سازی:

```rust
let titles = vec!["frieren", "bocchi the rock!"];
let shouted: Vec<String> = titles.iter().fold(Vec::new(), |mut acc, t| {
    acc.push(t.to_uppercase());
    acc
});
println!("{shouted:?}");
```

```text
["FRIEREN", "BOCCHI THE ROCK!"]
```

این دقیقاً همان چیزی است که در «تمرین» به‌اش نیاز داری — چون `.collect()` هنوز نداری (آن یکی [۲.۲.۳](../03-consuming-and-collecting/README.fa.md) است)، `.fold()` و یک `for` معمولی، تنها دو راهی‌اند که امروز یک `Vec` را از دلِ یک زنجیره بیرون می‌کشی.

### تخت کردن: `flat_map`

فرض کن هر آیتم، خودش یک لیستِ کوچک است. اگر با `.map()` بروی سراغش، نتیجه یک پیمایشگرِ تو-در-تو می‌شود — یک `Vec` داخلِ هر آیتم:

```rust
for genres in watchlist.iter().map(|s| &s.genres) {
    println!("{genres:?}");
}
```

```text
["fantasy", "adventure"]
["comedy", "music"]
```

هنوز به تک‌تکِ ژانرها نرسیده‌ای — فقط لیستِ لیست‌ها را داری. `.flat_map(closure)` دقیقاً همین دو قدم — مپ‌کردن، و بعد تخت‌کردنِ نتیجه — را در یک ترکیب‌گر انجام می‌دهد:

```rust
for genre in watchlist.iter().flat_map(|s| s.genres.iter()) {
    println!("{genre}");
}
```

```text
fantasy
adventure
comedy
music
```

یک جریانِ تختِ واحد، بدونِ اینکه دیگر معلوم باشد کدام ژانر مالِ کدام سریال بود — و مثلِ هر ترکیب‌گرِ دیگر، از همین‌جا هم می‌شود ادامه‌اش داد: `.flat_map(...).filter(...)`، دقیقاً همان‌قدر طبیعی که تا این‌جا دیدی.

### نکته‌ای که باید محکم بچسبی: هیچ‌کدام زودتر اجرا نمی‌شود

هر ترکیب‌گری که امروز دیدی — `map`، `filter`، `take`، `enumerate`، `rev`، `flat_map` — فقط یک پیمایشگرِ تازه می‌سازد که کارش را *توصیف* می‌کند. هیچ‌کدام، به‌تنهایی، حتی یک آیتم را لمس نمی‌کند:

```rust
println!("building the pipeline...");
let pipeline = shows
    .iter()
    .map(|t| {
        println!("  map saw:    {t}");
        t.to_uppercase()
    })
    .filter(|t| {
        println!("  filter saw: {t}");
        t.len() > 8
    });
println!("pipeline built — nothing printed above.");
```

```text
building the pipeline...
pipeline built — nothing printed above.
```

دقیقاً همان چیزی که ادعا کردیم: بینِ آن دو خط، نه `map saw`ای چاپ شد، نه `filter saw`ای. فقط وقتی یک `for` (یا یک متدِ مصرف‌کننده، مثلِ همان `fold` بالا) واقعاً `next()` را صدا می‌زند، خط‌لوله به حرکت درمی‌آید — و آن‌وقت هم، یک آیتم در یک لحظه، نه همه‌چیز یک‌جا:

```rust
for title in pipeline {
    println!("  got:        {title}");
}
```

```text
  map saw:    Frieren
  filter saw: FRIEREN
  map saw:    Bocchi the Rock!
  filter saw: BOCCHI THE ROCK!
  got:        BOCCHI THE ROCK!
  map saw:    Mushoku Tensei
  filter saw: MUSHOKU TENSEI
  got:        MUSHOKU TENSEI
```

خودِ کامپایلر هم همین را می‌گوید. همان زنجیره را به‌شکلِ یک عبارتِ تنها بنویس — بدونِ `let`، بدونِ چیزی که نتیجه‌اش را بگیرد — این هشدار را می‌گیری:

```rust
shows.iter().map(|t| t.to_uppercase()).filter(|t| t.len() > 8);
```

```text
warning: unused `Filter` that must be used
  = note: iterators are lazy and do nothing unless consumed
```

Rust اسمِ *بیرونی‌ترین* ترکیب‌گر را می‌آورد — `Filter` آخرین چیزی است که اعمال شده، `Map` را داخلِ خودش پیچیده، پس هشدار درباره‌ی همان نوع است. (اگر آن را به یک `let` که هیچ‌وقت نمی‌خوانی‌اش بایند کنی، مثلِ همان `pipeline` بالا، هشدارِ دیگر و ساده‌تری می‌گیری — `unused variable` — چون آن مقدار از نظرِ فنی *استفاده شده*، فقط بعداً خوانده نشده.)

```senpai-visual
{"kind":"concept","labels":["map()","filter()","پیمایشگر، بدون اجرا","for → next()","خروجی"]}
```

این تنبلی فقط یک رفتارِ عجیب نیست — یک تصمیمِ آگاهانه برایِ کاراییِ برنامه است: چون هیچ‌چیزی زودتر از موعد اجرا نمی‌شود، Rust مجبور نیست بینِ هر قدم یک `Vec`ِ میانی بسازد؛ کلِ زنجیره، هر چقدر هم طولانی، در یک عبورِ واحد و عنصر‌به‌عنصر اجرا می‌شود. چرا این برایِ کارایی مهم است — و چقدر — موضوعِ [۲.۲.۵](../05-laziness-and-performance/README.fa.md) است؛ همین یک‌پاراگراف برایِ الان کافی است.

---

## دست‌به‌کد

```sh
cargo run -p p2-02-02-iterator-adapters --example 01-next-is-the-whole-trait
cargo run -p p2-02-02-iterator-adapters --example 02-map
cargo run -p p2-02-02-iterator-adapters --example 03-filter-and-filter-map
cargo run -p p2-02-02-iterator-adapters --example 04-take-skip-and-while-variants
cargo run -p p2-02-02-iterator-adapters --example 05-enumerate-zip-chain
cargo run -p p2-02-02-iterator-adapters --example 06-rev
cargo run -p p2-02-02-iterator-adapters --example 07-fold
cargo run -p p2-02-02-iterator-adapters --example 08-flat-map
cargo run -p p2-02-02-iterator-adapters --example 09-the-pipeline-is-just-a-plan
```

بعد دوتای خراب:

```sh
cargo run -p p2-02-02-iterator-adapters --example 10-next-needs-mut --features broken
cargo run -p p2-02-02-iterator-adapters --example 11-rev-needs-double-ended --features broken
```

بعد این‌ها را امتحان کن:

۱. در `04-take-skip-and-while-variants`، لیستِ `ratings` را طوری عوض کن که اولین آیتم زیرِ ۸ باشد. `take_while` چند تا چاپ می‌کند؟
۲. در `05-enumerate-zip-chain`، به `ratings` یک آیتمِ سوم اضافه کن تا طولش با `titles` برابر شود. «Mushoku Tensei» حالا در خروجیِ `zip` ظاهر می‌شود؟
۳. در `09-the-pipeline-is-just-a-plan`، ترتیبِ `.map()` و `.filter()` را عوض کن. ترتیبِ خط‌های `map saw`/`filter saw` در خروجی چطور عوض می‌شود؟

---

## خطاهایی که خواهی دید

### `E0596` — نمی‌شود روی یک پیمایشگرِ تغییرناپذیر `next` صدا زد

```text
error[E0596]: cannot borrow `it` as mutable, as it is not declared as mutable
  --> phase2-intermediate\02-iterators-and-closures\02-iterator-adapters\examples\10-next-needs-mut.rs:12:22
   |
12 |     println!("{:?}", it.next());
   |                      ^^ cannot borrow as mutable
   |
help: consider changing this to be mutable
   |
11 |     let mut it = scores.iter();
   |         +++

For more information about this error, try `rustc --explain E0596`.
```

**کامپایلر به چه اعتراض دارد:** امضایِ `next` را دوباره بخوان — `fn next(&mut self) -> Option<Self::Item>`. هر صدا زدنش موقعیتِ داخلیِ پیمایشگر را جلو می‌برد، یعنی خودِ `it` را تغییر می‌دهد. `it` این‌جا با `let` ساده تعریف شده، نه `let mut` — یعنی تغییرناپذیر است، و کامپایلر اجازه نمی‌دهد رویِ یک متغیرِ تغییرناپذیر متدی صدا بزنی که به `&mut self` نیاز دارد.

**راه‌حل:** همان چیزی که پیشنهادِ کامپایلر هم هست:

```rust
let mut it = scores.iter();
println!("{:?}", it.next());
```

**چرا این راه‌حل است:** `it` حالا واقعاً می‌تواند تغییر کند، پس `&mut self`ای که `next` می‌خواهد را می‌شود از رویش گرفت. نکته‌ی جالب: یک `for` هیچ‌وقت این خطا را نمی‌دهد، چون خودش، پشتِ‌پرده، پیمایشگری که می‌سازد را به‌طور خودکار `mut` می‌گیرد — تو این را فقط وقتی می‌بینی که خودت با دست `next()` را صدا می‌زنی.

### `E0277` — `.rev()` روی پیمایشگری که `DoubleEndedIterator` نیست

```text
error[E0277]: the trait bound `std::collections::hash_map::Iter<'_, &str, u32>: DoubleEndedIterator` is not satisfied
    --> phase2-intermediate\02-iterators-and-closures\02-iterator-adapters\examples\11-rev-needs-double-ended.rs:16:35
     |
  16 |     let _reversed = scores.iter().rev();
     |                                   ^^^ the trait `DoubleEndedIterator` is not implemented for `std::collections::hash_map::Iter<'_, &str, u32>`
     |
note: required by a bound in `rev`
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\iter\traits\iterator.rs:3445:23
     |
3443 |     fn rev(self) -> Rev<Self>
     |        --- required by a bound in this associated function
3444 |     where
3445 |         Self: Sized + DoubleEndedIterator,
     |                       ^^^^^^^^^^^^^^^^^^^ required by this bound in `Iterator::rev`

For more information about this error, try `rustc --explain E0277`.
```

**کامپایلر به چه اعتراض دارد:** خودِ متدِ `rev` این شرط را رویِ خودش گذاشته — `Self: Sized + DoubleEndedIterator` — و `scores.iter()` این‌جا از نوعِ `std::collections::hash_map::Iter` است، که این صفت را پیاده‌سازی نکرده. دلیلش فنی نیست، منطقی است: `HashMap` اصلاً ترتیبِ پیمایشش را تضمین نمی‌کند، پس نه «جلو»یی دارد نه «عقب»ی که بشود از آن `.rev()` گرفت.

**راه‌حل:** اگر واقعاً به یک ترتیبِ قابلِ‌معکوس‌کردن نیاز داری، اول کلیدها یا مقدارها را در یک `Vec` جمع کن (با یک `for` معمولی، بدونِ `.collect()`)، و رویِ همان `Vec` بگرد:

```rust
let mut titles: Vec<&str> = Vec::new();
for title in scores.keys() {
    titles.push(title);
}
for title in titles.iter().rev() {
    println!("{title}");
}
```

**چرا این راه‌حل است:** یک `Vec` یک ترتیبِ واقعی و پایدار دارد — همانی که خودت باهاش پرش کردی — پس پیمایشگرش هم `DoubleEndedIterator` هست و `.rev()` رویش معنا دارد. این ترفند را همیشه به‌کار نبر: اگر فقط داری با `for` رویِ یک `HashMap` می‌گردی و ترتیب برایت مهم نیست، همان `.iter()` ساده کافی است.

---

## تمرین

### گرم‌کردن

<details>
<summary>یک <code>Vec&lt;i32&gt;</code> با سه آیتم داری و دوبار پشتِ‌سرهم <code>.iter().next()</code> را جدا صدا می‌زنی (هر بار رویِ یک پیمایشگرِ تازه). خروجیِ هر کدام چیست؟</summary>

فکرت را قبل از خواندنِ پاسخ بنویس.

</details>

<details>
<summary>پاسخ</summary>

هر دو `Some(<اولین آیتم>)` می‌دهند — چون هر بار یک `.iter()`ِ **تازه** ساخته‌ای، نه اینکه رویِ همان یکیِ قبلی ادامه داده باشی.

</details>

<details>
<summary><code>let v = vec![1, 2, 3]; let it = v.iter(); it.next();</code> کامپایل می‌شود؟</summary>

فکرت را قبل از خواندنِ «خطاهایی که خواهی دید» بنویس.

</details>

<details>
<summary>پاسخ</summary>

نه — `E0596`. `next(&mut self)` به یک پیمایشگرِ `mut` نیاز دارد، و `it` این‌جا با `let` ساده تعریف شده.

</details>

<details>
<summary><code>vec![1, 2, 8, 3, 4].iter().take_while(|n| **n &lt; 5)</code> چند تا آیتم برمی‌گرداند؟</summary>

فکرت را بنویس — و یادت باشد `take_while` چطور کار می‌کند.

</details>

<details>
<summary>پاسخ</summary>

فقط دوتا: `1` و `2`. به محضِ رسیدن به `8` (که شرط را رد می‌کند)، `take_while` متوقف می‌شود — حتی به `3` و `4`، که خودشان زیرِ ۵ بودند، هم نگاه نمی‌کند.

</details>

<details>
<summary><code>vec![1, 2, 3].iter().zip(vec!["a", "b"].iter())</code> چند جفت تولید می‌کند؟</summary>

فکرت را بنویس.

</details>

<details>
<summary>پاسخ</summary>

فقط دو جفت: `(1, "a")` و `(2, "b")`. `.zip()` به محضِ اینکه کوتاه‌ترین طرف تمام شود متوقف می‌شود؛ `3` هیچ‌وقت جفت نمی‌شود.

</details>

<details>
<summary>یک <code>HashMap&lt;&amp;str, u32&gt;</code> داری. <code>map.iter().rev()</code> کامپایل می‌شود؟</summary>

فکرت را قبل از خواندنِ «خطاهایی که خواهی دید» بنویس.

</details>

<details>
<summary>پاسخ</summary>

نه — `E0277`. پیمایشگرِ `HashMap` هیچ ترتیبِ تضمین‌شده‌ای ندارد، پس `DoubleEndedIterator` نیست و `.rev()` رویش تعریف نمی‌شود.

</details>

<details>
<summary><code>vec![1, 2, 3, 4].iter().fold(1, |acc, n| acc * n)</code> چه چیزی برمی‌گرداند؟</summary>

با دست حساب کن.

</details>

<details>
<summary>پاسخ</summary>

```text
24
```

شروع از `acc = 1`: `1×1=1`، بعد `1×2=2`، بعد `2×3=6`، بعد `6×4=24` — ضربِ تجمعیِ همه‌ی آیتم‌ها.

</details>

### تعمیر

هر دو مثالِ خراب را درست کن:

۱. `examples/10-next-needs-mut.rs` را طوری درست کن که کامپایل شود — فقط با تغییرِ نحوه‌ی تعریفِ `it`، نه با نوشتنِ چیزِ دیگری.
۲. `examples/11-rev-needs-double-ended.rs` را طوری درست کن که واقعاً بتوانی جفت‌های `HashMap` را برعکس چاپ کنی — با جمع‌کردنِ آن‌ها در یک `Vec` (با یک `for` معمولی) پیش از `.rev()` زدن.

### پیاده‌سازی

پنج تابع در `src/lib.rs`، رویِ یک `struct Show { title: String, episodes: u32, completed: bool }`:

```sh
cargo test -p p2-02-02-iterator-adapters
```

بدونِ `.collect()` — آن یکی [۲.۲.۳](../03-consuming-and-collecting/README.fa.md) است. برایِ ساختنِ هر `Vec`، از یک `for` معمولی یا از `.fold()` استفاده کن — دقیقاً همان دو راهی که در «مفهوم» دیدی. کامنتِ مستنداتِ هر تابع دقیقاً می‌گوید چه چیزی برمی‌گرداند؛ چیزی را حدس نزن.

### بساز

یک `pub fn watchlist_summary(shows: &[Show]) -> String` بنویس که یک خلاصه‌ی یک‌خطی از یک واچ‌لیست بسازد — چند تا شو هست، چندتاش `completed` است، مجموعِ `episodes`اش چقدر — با فرمتی که خودت انتخاب می‌کنی و در کامنتِ مستناتش دقیقاً می‌نویسی.

بعد یک تابعِ دوم بنویس، `pub fn titles_and_ratings(titles: &[String], ratings: &[u32]) -> Vec<String>`، که هر عنوان را با امتیازِ نظیرش جفت می‌کند (مثلاً به‌شکلِ `"Frieren - 9"`) — با `.zip()`، نه با اندیس‌گذاریِ دستی. اگر طول‌ها فرق داشتند، در کامنتِ مستناتش بنویس دقیقاً چند آیتم در خروجی می‌ماند و چرا.

### چالش (اختیاری)

یک `pub fn newest_and_oldest(shows: &[Show], n: usize) -> Vec<String>` بنویس که `n` تای اول (جدیدترین‌ها، به همان ترتیبِ اصلی) را با `n` تای آخر (قدیمی‌ترین‌ها — این‌ها هم به ترتیبِ اصلیِ خودشان، نه معکوس) پشتِ‌سرِهم می‌گذارد — فقط با `.take()`، `.rev()` و `.chain()`، بدونِ هیچ اندیس‌گذاریِ دستی. (سرنخ: «`n` تایِ آخر، ولی به ترتیبِ اصلی» دقیقاً همان چیزی است که یک `.rev()`، یک `.take(n)` و یک `.rev()`ِ دیگر، پشتِ‌سرِهم، به‌ات می‌دهند.) اگر `n` از نصفِ لیست بزرگ‌تر بود، چه تصمیمی می‌گیری؟ در کامنتِ مستناتت بنویس.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| `next(&mut self)` | تنها متدِ اجباریِ `Iterator`؛ آیتمِ بعدی، یا `None` | همه‌چیزِ این درس رویِ همین بنا شده |
| ترکیب‌گر ایتریتور (iterator adapter) | متدی که یک پیمایشگرِ تازه برمی‌گرداند و خودش کاری نمی‌کند | `.map()`، `.filter()`، `.take()`، … |
| `.map()` | تبدیلِ هر آیتم، یک‌به‌یک | تغییرِ نوع یا مقدار |
| `.filter()` / `.filter_map()` | نگه‌داشتنِ آیتم‌های تأییدشده / مپ + فیلتر در یک عبور | حذفِ ناخواسته‌ها، یا پارسِ چیزی که ممکن است شکست بخورد |
| `.take()`/`.take_while()`، `.skip()`/`.skip_while()` | محدودسازیِ خط‌لوله، با شمارش یا با شرط | گرفتنِ فقط اولین‌ها، یا رد‌کردنِ اولین‌ها |
| `.enumerate()`/`.zip()`/`.chain()` | جفت‌کردن با موقعیت، جفت‌کردنِ دو پیمایشگر، پشتِ‌سرِهم‌کردنِ دوتا | رتبه‌بندی، ترکیبِ دو لیستِ موازی، الحاقِ دو لیست |
| `.rev()` | پیمایش از آخر به اول؛ نیازمندِ `DoubleEndedIterator` | فقط وقتی پیمایشگر «آخر» را می‌شناسد |
| `.fold()` | ابزارِ همه‌کاره‌ی رسیدن به یک مقدار | جمع‌زدن، ساختنِ `Vec` با دست، هر ترکیبِ دیگر |
| `.flat_map()` | مپ + تخت‌کردنِ نتیجه، در یک قدم | تبدیلِ هر آیتم به چند آیتم |
| تنبلی (laziness) | یک ترکیب‌گر به‌تنهایی هیچ کاری نمی‌کند | تا وقتی چیزی — `for` یا یک متدِ مصرف‌کننده — مصرفش کند |

### الان می‌دانی

- `Iterator` فقط یک متدِ اجباری دارد — `next(&mut self) -> Option<Item>` — و هر ترکیب‌گر و هر `for`، در ته‌ِ ته‌اش، همین یک متد را پیاپی صدا می‌زند.
- `.map()` تبدیل می‌کند؛ `.filter()` نگه می‌دارد یا دور می‌ریزد؛ `.filter_map()` هر دو کار را در یک عبور انجام می‌دهد و `None`ها را بی‌سروصدا می‌اندازد.
- `.take`/`.skip` با شمارش کار می‌کنند، `.take_while`/`.skip_while` با شرط — و نسخه‌ی `while`دار به محضِ رسیدن به جوابِ منفی، کوتاه‌سازی (short-circuit) می‌کند.
- `.enumerate()` موقعیت اضافه می‌کند، `.zip()` رویِ کوتاه‌ترین طرف متوقف می‌شود، `.chain()` دوتا را پشتِ‌سرِهم می‌گذارد بدونِ ساختنِ لیستِ تازه.
- `.rev()` فقط رویِ `DoubleEndedIterator`ها کار می‌کند — پیمایشگری که هم `next()` دارد هم `next_back()`؛ `HashMap` این را ندارد.
- `.fold()` هر ترکیبِ ممکن را به یک مقدار می‌رساند، شاملِ ساختنِ یک `Vec` با دست.
- هیچ‌کدام از این‌ها زودتر از موعد اجرا نمی‌شود — یک ترکیب‌گر فقط یک خط‌لوله را توصیف می‌کند؛ فقط چیزی که واقعاً `next()` را صدا بزند (یک `for`، یا یک متدِ مصرف‌کننده) آن را به حرکت درمی‌آورد.

### بعداً کامل‌تر می‌بینی

- **`.collect()` و خانواده‌ی متدهایِ مصرف‌کننده — `.sum()`، `.count()`، `.for_each()`، و `Result<Vec<_>, E>`** — [۲.۲.۳ — مصرف و جمع‌آوری](../03-consuming-and-collecting/README.fa.md)
- **پیاده‌سازیِ `Iterator` و `IntoIterator` برایِ نوعِ خودت** — [۲.۲.۴](../04-implementing-iterator/README.fa.md)
- **چرا تنبلی برایِ کارایی مهم است، و دقیقاً چقدر** — [۲.۲.۵ — تنبلی و کاراییِ ایتریتورها](../05-laziness-and-performance/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا `Iterator` را با «یک متدِ اجباری» توصیف می‌کنیم، وقتی این‌همه متدِ دیگر (`.map()`، `.filter()`، `.fold()`، …) رویش هست؟
- `.filter_map()` دقیقاً چه کاری می‌کند که `.map()` به‌تنهایی نمی‌تواند؟
- چرا `.take_while()` می‌تواند رویِ باقیِ یک لیستِ بلند، حتی نگاه هم نیندازد، ولی `.filter()` نه؟
- `.zip()` با دو پیمایشگرِ هم‌طول نیست چه می‌کند؟
- چرا `.rev()` رویِ یک `Vec` کار می‌کند ولی رویِ یک `HashMap` نه؟
- اگر یک زنجیره‌ی `.map().filter()` بسازی و هیچ‌وقت مصرفش نکنی، دقیقاً چه اتفاقی می‌افتد؟

---

## بیشتر

- [مستنداتِ `std::iter::Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html) — فهرستِ کاملِ متدهایش؛ امروز فقط بخشی از این لیست را دیدی.
- [کتابِ Rust — پردازشِ یک سری از آیتم‌ها با پیمایشگرها](https://doc.rust-lang.org/book/ch13-02-iterators.html) — همین زمین، رسمی، با جزئیاتِ بیشتر دربارهٔ نحوهٔ تعریفِ صفت.
- [مستنداتِ `DoubleEndedIterator`](https://doc.rust-lang.org/std/iter/trait.DoubleEndedIterator.html) — همان صفتی که `.rev()` رویش تکیه دارد.
- [crate با نامِ `itertools`](https://docs.rs/itertools) — وقتی روزی این ترکیب‌گرهایِ استاندارد کم آمدند، این crate ده‌ها ترکیب‌گرِ دیگر اضافه می‌کند؛ خارج از این درس است، فقط بدان که هست.
