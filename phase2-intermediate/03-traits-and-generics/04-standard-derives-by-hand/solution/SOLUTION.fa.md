# راه‌حل — ۲.۳.۴ مشتق‌های استاندارد، دستی پیاده‌سازی‌شده

```rust
impl fmt::Debug for Track {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Track")
            .field("title", &self.title)
            .field("artist", &self.artist)
            .field("seconds", &self.seconds)
            .finish()
    }
}

impl fmt::Display for Track {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} by {}", self.title, self.artist)
    }
}

impl Default for Track {
    fn default() -> Self {
        Track { title: String::new(), artist: String::new(), seconds: 0 }
    }
}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title && self.artist == other.artist
    }
}

impl Ord for Track {
    fn cmp(&self, other: &Self) -> Ordering {
        self.seconds.cmp(&other.seconds).then_with(|| self.title.cmp(&other.title))
    }
}

impl Hash for Track {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.title.hash(state);
        self.artist.hash(state);
    }
}
```

هیچ‌کدام از این شش تکه به چیزی فراتر از همان ابزارهایی نیاز نداشت که در «مفهوم» دیدی — فقط این‌بار روی نوعِ خودت.

## `Debug` — دقیقاً همان builderی که در مثالِ ۰۱ دیدی

```rust
f.debug_struct("Track")
    .field("title", &self.title)
    .field("artist", &self.artist)
    .field("seconds", &self.seconds)
    .finish()
```

مشخصات دقیقاً همان قالبی را خواسته بود که `#[derive(Debug)]` تولید می‌کند — اسمِ نوع، بعد فیلدها به همان ترتیبِ اعلانشان. از `debug_struct` استفاده کردن، نه `write!` خام، دقیقاً همان چیزی است که تستِ دوم — `debug_alternate_form_is_indented` — رویش تکیه دارد: بدونِ builder، `{:#?}` هیچ‌وقت تورفتگی نمی‌گرفت.

## `Display` — یک `write!`، بدونِ گیومه

```rust
write!(f, "{} by {}", self.title, self.artist)
```

مشخصات دقیقاً همین رشته را خواسته بود: عنوان، کلمه‌ی «by»، هنرمند — بدونِ گیومه، بدونِ چیزِ اضافه. `seconds` اصلاً اینجا حاضر نمی‌شود؛ `Display` برایِ کاربر است، نه برایِ اشکال‌زدایی.

## `Default` — سه فیلد، سه مقدارِ خالی

```rust
Track { title: String::new(), artist: String::new(), seconds: 0 }
```

چیزِ پیچیده‌ای نیست — فقط باید هر سه فیلد را صریح صفر/خالی بگذاری. `String::new()` یک `String`ِ خالی می‌سازد، نه یک `Option` یا یک پنیک.

## `PartialEq` — فقط `title` و `artist`، `seconds` اصلاً دیده نمی‌شود

```rust
self.title == other.title && self.artist == other.artist
```

مشخصات صریح گفته بود: `seconds` هیچ‌وقت جزوِ برابری نیست — دو ضبط از همان آهنگ، حتی با زمان‌بندیِ متفاوتِ ثبت‌شده، هنوز همان یک آهنگ‌اند. `impl Eq for Track {}` که از قبل نوشته شده بود، همین را رسمی می‌کند: چون `PartialEq` بالا واقعاً بازتابی است (هیچ‌کدام از `String`/`u32` مشکلِ `NaN` را ندارند)، قولِ اضافه‌ی `Eq` را بی‌دردسر می‌شود داد.

## `Ord` — اول `seconds`، بعد `title` برایِ رفعِ تساوی

```rust
self.seconds.cmp(&other.seconds).then_with(|| self.title.cmp(&other.title))
```

`.then_with(...)` دقیقاً همان چیزی است که مشخصات می‌خواست: «اول بر اساسِ `seconds` مرتب کن؛ اگر مساوی بودند، برو سراغِ `title`.» کلوژرِ داخلِ `.then_with()` فقط وقتی واقعاً صدا زده می‌شود که مقایسه‌ی اول مساوی داده باشد — دقیقاً همان تنبلیِ لازم برایِ اینکه یک تساوی، واقعاً تساوی بماند تا رفعش کنی.

## `Hash` — دقیقاً همان دو فیلدی که `PartialEq` می‌بیند

```rust
self.title.hash(state);
self.artist.hash(state);
```

اینجا نکته‌ی اصلیِ درس است: `Hash` باید دقیقاً همان فیلدهایی را ببیند که `PartialEq` بالا می‌بیند — `title` و `artist`، نه `seconds`. اگر `self.seconds.hash(state);` هم اضافه می‌شد، دو `Track` که `PartialEq` برابر می‌داندشان (چون فقط `title`/`artist` را چک می‌کند) هشِ متفاوتی می‌گرفتند، و تستِ `hash_matches_eq_so_a_hashset_dedupes` دقیقاً همین را می‌گیرد: یک `HashSet` که باید یک عضو نگه دارد، دوتا نگه می‌داشت.

## این درس واقعاً درباره‌ی چه بود

- **هر شش صفت، یک قرارداد جدا دارند، نه یک مکانیزمِ مشترک.** `Debug` مکانیکی است، `Display` یک تصمیم است، `Eq` یک قولِ اضافه روی `PartialEq` است، `Ord` همان قول را یک پله بالاتر می‌برد، و `Hash` باید همیشه با `Eq` هم‌قدم بماند.
- **`f64` هیچ‌کدام از `Eq`/`Ord`/`Hash` را ندارد، به یک دلیلِ ریشه‌ای واحد:** `NaN` بازتابی نیست، پس نه برابری‌اش کامل است، نه ترتیبش، نه هشش می‌تواند رویِ چیزی که خودش تعریف‌نشده تکیه کند.
- **کامپایلر فقط تا جایی می‌تواند کمک کند.** ناسازگاریِ `Hash` با `Eq` — درست مثلِ همین `Track` اگر `seconds` را هم هش می‌کرد — هیچ خطایی نمی‌دهد. تنها دفاعت این است که خودت بدانی این دو باید هماهنگ بمانند.
