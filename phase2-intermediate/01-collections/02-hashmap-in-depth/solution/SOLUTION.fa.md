# راه‌حل — ۲.۱.۲ `HashMap` از نزدیک

```rust
pub fn get_or_zero(counts: &HashMap<String, u32>, key: &str) -> u32 {
    counts.get(key).copied().unwrap_or(0)
}

pub fn word_counts(text: &str) -> HashMap<String, u32> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for word in text.split_whitespace() {
        counts
            .entry(word.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }
    counts
}

pub fn group_by_first_letter(words: &[&str]) -> HashMap<char, Vec<String>> {
    let mut groups: HashMap<char, Vec<String>> = HashMap::new();
    for word in words {
        if let Some(letter) = word.chars().next() {
            groups
                .entry(letter)
                .or_insert_with(Vec::new)
                .push(word.to_string());
        }
    }
    groups
}

pub fn bump_or_init(counts: &mut HashMap<String, i32>, key: &str, amount: i32) {
    counts
        .entry(key.to_string())
        .and_modify(|value| *value += amount)
        .or_insert(amount);
}

pub fn most_common(counts: &HashMap<String, u32>) -> Option<(String, u32)> {
    let mut best: Option<(&String, &u32)> = None;
    for (key, value) in counts {
        let replace = match best {
            None => true,
            Some((best_key, best_value)) => {
                value > best_value || (value == best_value && key < best_key)
            }
        };
        if replace {
            best = Some((key, value));
        }
    }
    best.map(|(key, value)| (key.clone(), *value))
}
```

هر پنج تابع فقط با `HashMap` و ای‌پی‌آیِ `entry` نوشته شده‌اند — بدونِ `BTreeMap`، `HashSet` یا `VecDeque`. آن‌ها [۲.۱.۳](../../03-btreemap-hashset-vecdeque/README.fa.md) هستند.

## `get_or_zero` — یک `Option` که همین امروز می‌شناسی‌اش

```rust
counts.get(key).copied().unwrap_or(0)
```

`.get(key)` یک `Option<&u32>` می‌دهد — دقیقاً همان چیزی که فاز ۱ بارها نشانت داد. `.copied()` آن `Option<&u32>` را به `Option<u32>` تبدیل می‌کند (چون `u32` خودش `Copy` است)، و `.unwrap_or(0)` همان ترکیب‌گرِ [۱.۶.۲](../../../../phase1-fundamentals/06-absence-and-failure/02-option-combinators/README.fa.md) است، اینجا روی نتیجه‌ی `HashMap::get` به‌کار رفته. چیزِ تازه‌ای در این تابع نیست؛ فقط ثابت می‌کند `.get()` روی `HashMap` هیچ فرقی با `.get()` روی `Vec` یا `.first()` روی یک برش ندارد: هر سه‌شان `Option` می‌دهند، و هر سه‌شان با همان ابزارها باز می‌شوند.

## `word_counts` — الگویِ `and_modify` + `or_insert`

```rust
for word in text.split_whitespace() {
    counts
        .entry(word.to_string())
        .and_modify(|count| *count += 1)
        .or_insert(1);
}
```

همان ترکیبِ مثالِ ۰۴ است، این‌بار روی متنِ واقعی به‌جایِ یک لیستِ ثابت. `.entry(word.to_string())` یک جستجوی تکی است. اگر کلید از قبل باشد، `.and_modify(...)` مقدارش را یکی زیاد می‌کند؛ اگر نباشد، `.or_insert(1)` با مقدارِ اولیه‌ی ۱ می‌سازدش. هیچ‌وقت هر دو بازو با هم اجرا نمی‌شوند — دقیقاً همان چیزی که اسم‌شان می‌گوید.

نکته‌ی ظریف: `word` که از `.split_whitespace()` می‌آید نوعش `&str` است — یک برش از خودِ `text`. ولی کلیدِ نقشه باید `String` باشد، چون نقشه باید مالکِ کلیدهایش باشد و `text` ممکن است بعد از پایانِ تابع از بین برود. `word.to_string()` دقیقاً همین را حل می‌کند: یک کپیِ مالک از برش می‌سازد.

## `group_by_first_letter` — `or_insert_with` برای یک مقدارِ پیش‌فرضِ واقعاً گران

```rust
if let Some(letter) = word.chars().next() {
    groups
        .entry(letter)
        .or_insert_with(Vec::new)
        .push(word.to_string());
}
```

`word.chars().next()` یک `Option<char>` می‌دهد — `None` فقط وقتی `word` رشته‌ی خالی است، و `if let` دقیقاً همان یک مورد را رد می‌کند بدونِ اینکه چیزی بترکد. `.or_insert_with(Vec::new)` — نه `.or_insert(Vec::new())` — اینجا مهم است: با نسخه‌ی دوم، هر بار که این خط اجرا می‌شد — چه کلید تازه بود چه نبود — یک `Vec` جدید ساخته می‌شد، فقط برای اینکه در حالتِ «کلید از قبل بود» فوراً دور ریخته شود. با `n` کلمه و `k` کلیدِ متفاوت، این یعنی `n` بار تخصیص به‌جایِ `k` بار.

ترتیب حفظ می‌شود چون حلقه روی `words` به‌ترتیبِ ورودی پیش می‌رود و هر بار فقط `.push` می‌کند — هیچ‌جا چیزی مرتب یا برعکس نمی‌شود.

## `bump_or_init` — همان الگو، این‌بار با تغییرِ درجا

```rust
counts
    .entry(key.to_string())
    .and_modify(|value| *value += amount)
    .or_insert(amount);
```

فرقش با `word_counts` این است که خروجی ندارد — پارامترِ `counts` یک `&mut HashMap<...>` است و تابع مستقیم روی همان نقشه کار می‌کند. `amount` می‌تواند منفی هم باشد (`i32` است، نه `u32`)، پس این دقیقاً «شمردن» نیست — «تنظیمِ یک مقدار» است، با همان یک الگوی `entry`.

## `most_common` — چرا این تابع اصلاً به یک قاعده‌ی رفعِ تساوی نیاز داشت

```rust
let mut best: Option<(&String, &u32)> = None;
for (key, value) in counts {
    let replace = match best {
        None => true,
        Some((best_key, best_value)) => {
            value > best_value || (value == best_value && key < best_key)
        }
    };
    if replace {
        best = Some((key, value));
    }
}
best.map(|(key, value)| (key.clone(), *value))
```

اینجا همان درسِ اصلیِ همین فصل را عملاً پیاده کردی: **پیمایشِ `for (key, value) in counts` هیچ ترتیبِ تضمین‌شده‌ای ندارد.** اگر فقط اولین مقداری که به آن برمی‌خوردی را «برنده» اعلام می‌کردی، جوابِ `most_common` برای یک نقشه‌ی با تساوی می‌توانست بینِ دو بار اجرای همین برنامه فرق کند — دقیقاً همان چیزی که مثالِ ۰۲ (`02-no-guaranteed-order`) نشانت داد. به‌جایش، شرطِ `replace` یک قاعده‌ی صریح دارد: مقدارِ بزرگ‌تر همیشه می‌برد؛ در تساویِ مقدار، کلیدی که الفبایی زودتر است می‌برد. این قاعده هیچ ربطی به ترتیبِ پیمایش ندارد، پس جواب همیشه یکی است — هر بار که اجرا شود، روی هر ماشینی.

`best` روی ارجاع‌ها (`&String`, `&u32`) کار می‌کند تا از کلون‌کردن در هر قدمِ حلقه فرار کند؛ فقط در همان انتها، یک‌بار، `.map(...)` مقدارِ نهایی را مالکانه می‌کند (`key.clone()`، `*value`).

## این درس واقعاً درباره‌ی چه بود

- ای‌پی‌آیِ `entry` یک جستجو را به یک جستجو تبدیل می‌کند — چه با `.or_insert()`، چه با `.or_insert_with()`، چه با `.and_modify().or_insert()`.
- `.or_insert_with(f)` را وقتی مقدارِ پیش‌فرض واقعاً کاری برای انجام‌دادن دارد (مثلِ `Vec::new`) به‌جایِ `.or_insert(...)` بردار — وگرنه آن کار را بی‌جهت روی هر جستجو تکرار می‌کنی.
- کلیدِ `HashMap` باید مالکِ خودش باشد وقتی نقشه از یک تابع بیرون می‌رود؛ `.to_string()` روی یک `&str` گذرا دقیقاً همین را می‌دهد.
- پیمایشِ `HashMap` ترتیبِ تضمین‌شده ندارد — پس هر جا جوابِ درست به «کدام‌یک اول» بستگی دارد، خودت باید یک قاعده‌ی صریح بنویسی، نه اینکه به ترتیبِ پیمایش تکیه کنی.
