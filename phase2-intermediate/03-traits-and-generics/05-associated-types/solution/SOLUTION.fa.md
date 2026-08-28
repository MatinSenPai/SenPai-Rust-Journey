# راه‌حل — ۲.۳.۵ نوع‌های وابسته در برابرِ پارامترهای جنریک

```rust
impl Measures for Rectangle {
    type Output = u32;

    fn measure(&self) -> u32 {
        self.width * self.height
    }
}

impl DescribesAs<u32> for Rectangle {
    fn describe(&self) -> u32 {
        2 * (self.width + self.height)
    }
}

impl DescribesAs<String> for Rectangle {
    fn describe(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }
}
```

هیچ‌کدام نه یک صفتِ تازه لازم داشت، نه هیچ نوع‌نویسیِ اضافه‌ای در خودِ `impl` — فقط بدنه‌هایی که به قولِ همان دو شکلی که «مفهوم» نشانت داد وفادار مانده‌اند.

## `Measures::measure` — یک ضرب، یک نوعِ ثابت

```rust
self.width * self.height
```

`type Output = u32;` از قبل، در تعریفِ صفت، ثابت شده بود — نه در بدنه‌ی متد، نه در نقطه‌ی صدا زدن (`r.measure()`، بدونِ هیچ نوع‌نویسی‌ای). بدنه فقط باید همان چیزی را برگرداند که آن قول گفته بود.

## `DescribesAs<u32>::describe` و `DescribesAs<String>::describe` — دو بدنه، دو `impl` جدا

```rust
2 * (self.width + self.height)
```

```rust
format!("{}x{}", self.width, self.height)
```

این دو، دو `impl` کاملاً جدایند — نه یک متدِ مشترک که رویِ یک شرط انشعاب بزند. `Rectangle` هم‌زمان `DescribesAs<u32>` و `DescribesAs<String>` را پیاده کرده، دقیقاً همان‌طور که `Celsius` در «مفهوم» هم‌زمان `Converts<f64>` و `Converts<String>` را پیاده کرد. تست‌ها هم دقیقاً به همین دلیل هرکدام یک `let` با نوعِ صریح دارند (`let perimeter: u32 = ...`، `let label: String = ...`) — بدونِ آن، همان `E0283`ای می‌گرفتی که «خطاهایی که خواهی دید» نشانت داد.

## نکته‌ای که این درس واقعاً درباره‌اش بود

- **نوعِ وابسته یک بار مشخص می‌شود، همه‌جا اعمال می‌شود.** `Output = u32` را فقط یک‌بار نوشتی؛ هر صدا زدنِ `.measure()`، رویِ هر `Rectangle`ای، همان یک نوع را می‌دهد — بدونِ استثنا.
- **پارامترِ جنریک یعنی دو `impl` کاملاً مستقل.** `describe()` رویِ `DescribesAs<u32>` و `describe()` رویِ `DescribesAs<String>` دو تابعِ کاملاً جدا هستند که فقط اسمشان یکی است؛ کامپایلر بر اساسِ نوعِ خواسته‌شده (از رویِ نوع‌نویسیِ `let`) تصمیم می‌گیرد کدام‌یک صدا زده شود.
- **مشخصات، مشخصات بود.** فرمتِ دقیقِ `"3x4"` — بدونِ فاصله، `width` قبل از `height` — در کامنتِ مستنداتِ خودِ تمرین آمده بود؛ چیزی حدسی در تست‌ها نبود.
