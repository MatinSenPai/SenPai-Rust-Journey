# راه‌حل — ۲.۳.۱ تعریف و پیاده‌سازیِ صفت‌ها

## `AnimeSeries`

```rust
impl Summarize for AnimeSeries {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn summary(&self) -> String {
        format!("{} — {} episodes", self.title(), self.episodes)
    }
}
```

`title` فقط یک کلونِ فیلد است — چیزِ دیگری برایِ برگرداندن نیست. `summary` پیش‌فرض را بازنویسی می‌کند تا تعدادِ قسمت‌ها را هم اضافه کند، و برایِ گرفتنِ عنوان به‌جایِ خواندنِ مستقیمِ فیلد (`self.title`)، متدِ `title()` را صدا می‌زند — همان الگویی که بدنه‌ی پیش‌فرضِ خودِ trait هم استفاده می‌کند.

## `MangaVolume`

```rust
impl Summarize for MangaVolume {
    fn title(&self) -> String {
        self.title.clone()
    }
}
```

همین. هیچ بازنویسیِ `summary`ای نیست، چون هیچ لازم نبود. `vol.summary()` نسخه‌ی پیش‌فرضِ نوشته‌شده در خودِ trait را اجرا می‌کند — که `self.title()` را صدا می‌زند و این‌بار نسخه‌ی `MangaVolume` جواب می‌دهد. هیچ‌چیزی اینجا کپی یا از نو نوشته نشده؛ همان یک بدنه، برایِ هر نوعی که پیاده‌سازی‌اش نمی‌کند.

## `GameTitle`

```rust
impl Summarize for GameTitle {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn summary(&self) -> String {
        format!("{} — {}h to beat", self.title(), self.hours_to_beat)
    }
}
```

شکلِ کاملاً یکسانِ `AnimeSeries` — یک بازنویسیِ `title` اجباری، یک بازنویسیِ `summary` اختیاری که فرمتِ خودش را دارد. نکته: `GameTitle` هیچ ربطی به `AnimeSeries` یا `MangaVolume` ندارد — نه فیلد مشترک، نه ساختارِ پایه — و بازهم بدونِ هیچ زحمتِ اضافه‌ای همان قراردادِ `Summarize` را برآورده می‌کند.

## `shelf_summary`

```rust
pub fn shelf_summary(series: &AnimeSeries, volume: &MangaVolume) -> String {
    format!("{}; {}", series.summary(), volume.summary())
}
```

این تابع اصلاً جنریک نیست — دو نوعِ *ملموس* و *متفاوت* را به‌عنوانِ پارامتر می‌گیرد، رویِ هرکدام `summary()` را صدا می‌زند، و دو رشته را با `; ` کنارِ هم می‌گذارد. هیچ نیازی به جنریک یا `dyn` نبود، چون از قبل می‌دانستیم پارامترِ اول همیشه `AnimeSeries` است و پارامترِ دوم همیشه `MangaVolume`. اگر می‌خواستی همین ایده را برایِ *هر* ترکیبی از نوع‌هایی که `Summarize` دارند تعمیم بدهی — یک تابعِ واحد، به‌جایِ نوشتنِ یک نسخه‌ی جدا برایِ هر جفتِ نوع — دقیقاً همان چیزی است که [۲.۳.۲](../../02-generic-functions-and-structs/README.fa.md) به‌ات یاد می‌دهد.
