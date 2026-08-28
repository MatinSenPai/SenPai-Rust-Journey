# راه‌حل — ۲.۳.۲ توابع و ساختارهای جنریک، کران‌ها، `where`

```rust
pub fn smallest<T: PartialOrd>(list: &[T]) -> &T {
    let mut smallest = &list[0];
    for item in list {
        if item < smallest {
            smallest = item;
        }
    }
    smallest
}

pub struct Pair<T> {
    first: T,
    second: T,
}

impl<T> Pair<T> {
    pub fn new(first: T, second: T) -> Self {
        Pair { first, second }
    }

    pub fn first(&self) -> &T {
        &self.first
    }

    pub fn second(&self) -> &T {
        &self.second
    }

    pub fn larger(&self) -> &T
    where
        T: PartialOrd,
    {
        if self.second > self.first {
            &self.second
        } else {
            &self.first
        }
    }
}

pub fn matches_count<T, F: Fn(&T) -> bool>(items: &[T], predicate: F) -> usize {
    let mut count = 0;
    for item in items {
        if predicate(item) {
            count += 1;
        }
    }
    count
}

pub fn label_and_duplicate<T: std::fmt::Display + Clone>(item: T) -> (String, T) {
    let label = format!("label: {item}");
    (label, item.clone())
}
```

هر چهار امضا دقیقاً همان چیزی‌اند که در «مفهوم» دیدی — فقط این‌بار کدِ داخلشان هم نوشته شده.

## `smallest` — همان الگوی `largest`، فقط برعکس

```rust
let mut smallest = &list[0];
for item in list {
    if item < smallest {
        smallest = item;
    }
}
smallest
```

بدنه کلمه‌به‌کلمه همان `largest`ای است که در «مفهوم» و در `examples/01-largest.rs` دیدی — تنها فرق، `<` به‌جایِ `>`. کرانِ `T: PartialOrd` هم دقیقاً به همان دلیل لازم است: بدونِ آن، کامپایلر معنیِ `item < smallest` را نمی‌داند، و کامپایل نمی‌شود.

مثلِ `largest`، این تابع هم رویِ اولین عنصر (`&list[0]`) پنیک می‌کند اگر `list` خالی باشد — دقیقاً همان‌طور که در مستنداتِ تابع نوشته شده بود.

## `Pair<T>` — یک ساختار بدونِ کران، بعلاوه‌ی یک متد با کرانِ خودش

بلوکِ `impl<T> Pair<T>` هیچ کرانی ندارد — `new`، `first` و `second` رویِ *هر* `T`ای کار می‌کنند، حتی نوعی که هیچ صفتی پیاده نکرده باشد. اما `larger` به یک قولِ اضافه نیاز دارد: باید بتواند `self.second` و `self.first` را با `>` مقایسه کند. به‌جایِ اینکه این قول را رویِ کلِ بلوکِ `impl` بگذاریم — که آن‌وقت `new`، `first` و `second` هم بی‌خودی مجبور می‌شدند `T: PartialOrd` داشته باشند — رویِ خودِ `larger` گذاشته شده، با یک `where` که بعد از امضا می‌آید:

```rust
pub fn larger(&self) -> &T
where
    T: PartialOrd,
{
    if self.second > self.first {
        &self.second
    } else {
        &self.first
    }
}
```

`pair_works_with_non_numeric_types` دقیقاً همین را تست می‌کند: یک `Pair<String>` می‌سازد و `.larger()` رویش صدا می‌زند — کار می‌کند چون `String` هم `PartialOrd` را پیاده کرده، نه چون `Pair` خودش از قبل به آن نیاز داشت.

قاعده‌ی تساوی هم عمدی است: `if self.second > self.first` — یعنی فقط وقتی دومی *واقعاً* از اولی بزرگ‌تر باشد، دومی برمی‌گردد؛ در هر حالتِ دیگری (مساوی، یا اولی بزرگ‌تر)، اولی برمی‌گردد. `pair_larger_breaks_a_tie_toward_first` همین را تأیید می‌کند.

## `matches_count` — کران رویِ کلوژر، نه رویِ `T`

```rust
pub fn matches_count<T, F: Fn(&T) -> bool>(items: &[T], predicate: F) -> usize {
    let mut count = 0;
    for item in items {
        if predicate(item) {
            count += 1;
        }
    }
    count
}
```

این‌بار `T` اصلاً کرانی ندارد — تابع هیچ‌وقت دو تا `T` را با هم مقایسه نمی‌کند، فقط هرکدام را دستِ `predicate` می‌دهد و جوابش را می‌شمرد. کرانی که واقعاً لازم است رویِ پارامترِ دوم است: `F: Fn(&T) -> bool`. همین یک کران، به‌صورتِ درون‌خطی نوشته شده، نه با `where` — چون فقط یک کران رویِ یک پارامتر است، دقیقاً همان مرزی که «مفهوم» گفت `where` هنوز اختیاری است.

## `label_and_duplicate` — دو کران رویِ یک پارامتر

```rust
pub fn label_and_duplicate<T: std::fmt::Display + Clone>(item: T) -> (String, T) {
    let label = format!("label: {item}");
    (label, item.clone())
}
```

`Display` برایِ `format!("label: {item}")` لازم است، `Clone` برایِ `item.clone()` — دو قولِ کاملاً مستقل، هر دو رویِ همان `T`. اگر یکی‌شان را برداری، دقیقاً همان `E0599`ای می‌گیری که در «خطاهایی که خواهی دید» دیدی.

## این درس واقعاً درباره‌ی چه بود

- **یک کران، دقیقاً همان قولی است که تابع برایِ کامپایل‌شدن لازم دارد** — نه یک‌کلمه بیشتر، نه کمتر. `matches_count` به `T: PartialOrd` نیاز نداشت چون هیچ‌وقت دو `T` را مقایسه نکرد.
- **کران رویِ خودِ متد، دقیق‌تر از کران رویِ کلِ بلوکِ `impl` است.** `Pair<T>` سه متدِ دیگرش را برایِ هر `T`ای نگه داشت، چون فقط `larger` کرانش را داشت.
- **`where` انتخابِ نوشتاری است، نه یک قابلیتِ تازه** — `<T: Trait>` و `where T: Trait` دقیقاً یک قرارداد را به کامپایلر می‌گویند.
- **دو کران رویِ یک پارامتر با `+` می‌آید** — `T: Display + Clone` یعنی هر دو قول، هم‌زمان، لازم‌اند.
- **هیچ‌کدام از این چهار تابع رویِ یک نوعِ مشخص قفل نشد** — همان‌طور که تست‌ها نشان دادند، `i32`، `&str`، `String` و حتی نوع‌هایی که خودت تعریف می‌کنی، همه از همین یک تعریف عبور می‌کنند — و هرکدام، جداگانه، سرِ کامپایل.
