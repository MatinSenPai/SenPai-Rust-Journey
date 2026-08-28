# راه‌حل — ۲.۲.۴ پیاده‌سازیِ `Iterator` و `IntoIterator`

```rust
impl Iterator for Collatz {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let value = self.current?;
        self.current = if value == 1 {
            None
        } else if value % 2 == 0 {
            Some(value / 2)
        } else {
            Some(3 * value + 1)
        };
        Some(value)
    }
}

impl IntoIterator for EpisodeLog {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
```

هیچ‌کدام از این دو به جنریک، طول‌عمر، یا صفتِ سفارشیِ تازه نیاز نداشت — فقط همان دو متدی که «مفهوم» نشانت داد، این‌بار روی داده‌ی خودت.

## `Collatz::next` — یک `?`، یک `if`/`else`، یک `Some` آخر

```rust
let value = self.current?;
self.current = if value == 1 {
    None
} else if value % 2 == 0 {
    Some(value / 2)
} else {
    Some(3 * value + 1)
};
Some(value)
```

خطِ اول دو کار را با هم انجام می‌دهد: اگر `self.current` از قبل `None` بود (دنباله تمام شده)، `?` همان‌جا از تابع با `None` برمی‌گردد — دقیقاً همان رفتاری که مشخصات خواسته بود («اگر تمام شده، `None`»). اگر `None` نبود، `value` مقدارِ داخلش می‌شود.

بعد، **حالتِ بعدی** حساب و ذخیره می‌شود — نه مقدارِ برگشتی. این نکته‌ی ظریف را جدی بگیر: تابع قرار است `value` را برگرداند (همان چیزی که همین الان خوانده)، نه چیزی که بعدش می‌آید؛ فقط چیزی که بعدش می‌آید را برایِ صدایِ *بعدی* ذخیره می‌کند. اگر `value` همان `1` بود، دیگر چیزی برایِ ادامه نیست — `self.current` می‌شود `None`، و صدایِ بعدی همان‌جا متوقف می‌شود. در غیرِ این‌صورت، قاعده‌ی زوج/فرد دقیقاً همانی است که مستندات گفته بودند.

خطِ آخر، `Some(value)`، همان چیزی است که این متد را از یک تابعِ معمولی به یک `Iterator::next` واقعی تبدیل می‌کند — همیشه یک `Option` برمی‌گرداند، هیچ‌وقت خودِ مقدار را.

## `EpisodeLog::into_iter` — تحویل‌دادنِ چیزی که از قبل داری

```rust
self.0.into_iter()
```

همین. `self` را که با مقدار گرفته بودی، همان بلافاصله رویِ فیلدِ داخلی‌اش (`self.0`، یک `Vec<String>`) صدا می‌زند `.into_iter()` — دقیقاً همان چیزی که `Vec` خودش از قبل پیاده کرده. کارِ تو فقط این بود که بگویی «هرچه لازم است، از همانی که داخلش داری قرض بگیر» — نه اینکه از صفر یک پیمایشگر بسازی. `type IntoIter = std::vec::IntoIter<String>` هم دقیقاً به همین خاطر همان نوعی بود که انتخاب کردیم: چیزی که `Vec<String>::into_iter()` خودش برمی‌گرداند، بدونِ هیچ تبدیلِ اضافه.

## این درس واقعاً درباره‌ی چه بود

- **یک متد، همه‌چیز رایگان.** `Collatz` هیچ‌جا `.filter()` یا `.count()` یا `.collect()` را برایِ خودش ننوشت — تستِ `collatz_works_with_adapters_nobody_wrote_here` دقیقاً همین را ثابت کرد.
- **`IntoIterator` جدا از `Iterator` است.** `EpisodeLog` هرگز `Iterator` را پیاده نکرد؛ فقط گفت «چطور بشو یک `Iterator`» (`std::vec::IntoIter` را تحویل داد)، و همین برایِ `for title in log` کافی بود.
- **نوعِ وابسته، فقط یک اسم‌گذاری است.** `type Item = u64` روی `Collatz` و `type Item = String` روی `EpisodeLog` — هیچ‌کدام `Iterator` یا `IntoIterator` را جنریک نکردند؛ هرکدام فقط گفتند «مالِ من این است».
