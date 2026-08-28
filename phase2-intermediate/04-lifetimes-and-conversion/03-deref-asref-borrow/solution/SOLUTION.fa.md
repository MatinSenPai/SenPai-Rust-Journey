# راه‌حل — ۲.۴.۳ `Deref`، `AsRef`، `Borrow`، `ToOwned`

```rust
use std::borrow::Borrow;
use std::ops::{Deref, DerefMut};

pub struct Playlist(pub Vec<String>);

impl Deref for Playlist {
    type Target = Vec<String>;
    fn deref(&self) -> &Vec<String> {
        &self.0
    }
}

impl DerefMut for Playlist {
    fn deref_mut(&mut self) -> &mut Vec<String> {
        &mut self.0
    }
}

pub struct DisplayName(pub String);

impl AsRef<str> for DisplayName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Handle(pub String);

impl Borrow<str> for Handle {
    fn borrow(&self) -> &str {
        &self.0
    }
}

pub fn longest_word_owned(text: &str) -> String {
    text.split_whitespace()
        .max_by_key(|word| word.chars().count())
        .unwrap_or("")
        .to_owned()
}
```

## `Playlist` — هم `Deref` هم `DerefMut`، هردو همان یک فیلد را برمی‌گردانند

```rust
impl Deref for Playlist {
    type Target = Vec<String>;
    fn deref(&self) -> &Vec<String> {
        &self.0
    }
}
impl DerefMut for Playlist {
    fn deref_mut(&mut self) -> &mut Vec<String> {
        &mut self.0
    }
}
```

هر دو متد کمترین کاری را می‌کنند که ممکن است: یک ارجاع به فیلدی که از قبل آن‌جاست برمی‌گردانند. `deref` یک قرضِ اشتراکی از `self` می‌گیرد و `&self.0` را پس می‌دهد؛ `deref_mut` یک قرضِ تغییرپذیر می‌گیرد و `&mut self.0` را. هیچ‌چیز کپی، ساخته یا تبدیل نمی‌شود — این پوششگر واقعاً فقط یک اسمِ دیگر برایِ همان `Vec<String>` داخلش است. دقیقاً همین است که باعث می‌شود `list.len()`، `list.push(...)` و `list[1]` کامپایل شوند بدونِ اینکه `Playlist` حتی یکی از این متدها را خودش تعریف کرده باشد: جست‌وجویِ متد اول رویِ `Playlist` می‌گردد، پیدا نمی‌کند، بعد از رویِ `Deref`/`DerefMut` به `Vec<String>` می‌رود و همان‌جا پیدایش می‌کند.

## `DisplayName` — یک خط، چون قراردادِ `AsRef<str>` دقیقاً همانِ `deref` است

```rust
impl AsRef<str> for DisplayName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
```

همان شکلِ `deref`ِ بالا، این‌بار برایِ صفتِ دیگری: یک ارجاع به فیلد برمی‌گردانی، دقیقاً با نوعی که خودِ صفت قول داده (`&str`، نه `&Vec<String>` یا `&String`ی نپیچیده — این‌جا `&self.0` از قبل `&str` است چون فیلد خودش `String` است و امضایِ تابع `&str` می‌خواهد). از این به بعد، هر تابعی که `fn f(name: impl AsRef<str>)` نوشته شده، یک `DisplayName` را هم رایگان قبول می‌کند، کنارِ هر `&str` و `String`ی که از قبل قبول می‌کرد.

## `Handle` — `Borrow<str>` باید با همان خطِ `derive` بالایِ سرش هم‌خوان بماند

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Handle(pub String);

impl Borrow<str> for Handle {
    fn borrow(&self) -> &str {
        &self.0
    }
}
```

`derive`، `Hash` و `Eq` مالِ `Handle` را از رویِ همان یک فیلدش می‌سازد — با هش‌کردن و مقایسه‌ی خودِ `String`، با `Hash`/`Eq`ِ خودِ `String`. `borrow()` هم دقیقاً همان فیلد را، بدونِ هیچ تغییری، برمی‌گرداند — پس هشِ `Handle` و هشِ کلیدِ جستجو، هر دو از رویِ همان بایت‌ها ساخته می‌شوند، و قرارداد برقرار می‌ماند. این همان نسخه‌ی خسته‌کننده و *درستِ* چیزی است که `examples/05-borrow-contract-violation.rs` عمداً شکسته بود: هیچ‌جایِ این کد، رشته را یک‌جور برایِ هش‌کردن و یک‌جورِ دیگر برایِ مقایسه‌کردن تبدیل نمی‌کند.

تستِ `handle_looks_up_by_str_in_hashmap` همان چیزی است که این همه دقت برایش بود: یک `Handle` بگذار، با یک لیترالِ `&str` ساده پیدایش کن.

## `longest_word_owned` — یک `.max_by_key()` به‌علاوه‌ی یک `.to_owned()`

```rust
pub fn longest_word_owned(text: &str) -> String {
    text.split_whitespace()
        .max_by_key(|word| word.chars().count())
        .unwrap_or("")
        .to_owned()
}
```

`.split_whitespace()` کلمه‌هایِ `text` را به‌شکلِ برش‌هایِ قرضیِ `&str` می‌دهد، همه قرض‌گرفته از خودِ `text`. `.max_by_key(|word| word.chars().count())` آن‌که بیشترین کاراکتر را دارد انتخاب می‌کند — و سرِ تساوی، `Iterator::max_by_key` **آخرین** بیشینه‌ای را که می‌بیند نگه می‌دارد، دقیقاً به همین دلیل `"cat dog owl bee"` (چهار کلمه‌ی سه‌کاراکتری) به‌جایِ `"cat"`، `"bee"` را برمی‌گرداند. `.unwrap_or("")` حالتِ بی‌کلمه را بدونِ یک `match` جدا مدیریت می‌کند. آن `&str`ی که از همه‌ی این‌ها بیرون می‌آید، هنوز قرضی از `text` است — نمی‌تواند از خودِ ورودی بیشتر عمر کند — پس آن `.to_owned()`ِ آخر است که واقعاً همان `String`ی را می‌سازد که امضا قول داده: یک کپیِ مستقل، که هرچقدر صداکننده بخواهد نگهش دارد، سالم می‌ماند.

## این درس واقعاً درباره‌ی چه بود

- `Deref`/`DerefMut` همان چیزی‌اند که بازکردنِ خودکار و تبدیلِ ضمنیِ ارجاع واقعاً صدا می‌زنند — نه جادو، یک صفت با یک متد هرکدام (`deref`، `deref_mut`)، که `Box`، `String`، `Vec`، و حالا `Playlist` هرکدام یک‌بار پیاده‌اش کرده‌اند.
- رویِ یک newtypeِ به‌این‌سادگی، `AsRef<T>` و `Deref` شبیهِ هم به‌نظر می‌رسند، ولی به دو سؤالِ متفاوت جواب می‌دهند: `Deref` می‌گوید «همه‌جا با من مثلِ همان چیزی که پیچیده‌ام رفتار کن»؛ `AsRef<T>` می‌گوید «بگذار یک تابع، عمداً، سرِ یک محلِ فراخوانیِ به‌خصوص، یک نمایِ ارزانِ `&T` بخواهد.»
- امضایِ `Borrow<T>` عیناً همانِ `AsRef<T>` است — فرق فقط یک قول است که کامپایلر هیچ‌وقت خودش چکش نمی‌کند: `Hash`، `Eq` و `Ord` بینِ فرمِ قرضی و فرمِ مالک باید هم‌خوان بمانند. `Handle` این قول را با هرگز-تبدیل-نکردنِ رشته نگه می‌دارد؛ `CiKey`ِ مثال‌ها عمداً همین قول را شکست، و کلیدی که ثابت‌شدنی وجود داشت (`len() == ۱`) با هیچ املایی از `.get()` پیدا نشد.
- `ToOwned` وجود دارد چون `Clone` نمی‌توانست: `str::clone(&self) -> Self` مجبور بود یک `str` بی‌اندازه را با مقدار برگرداند، که اصلاً کامپایل نمی‌شود. `ToOwned::to_owned(&self) -> Self::Owned` می‌گذارد نوعِ مالک، نوعِ کاملاً دیگری باشد — `String` برایِ `str`، `Vec<T>` برایِ `[T]`.
