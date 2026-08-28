# راه‌حل — ۲.۲.۲ ترکیب‌گرهای ایتریتور

```rust
pub fn uppercase_titles(shows: &[Show]) -> Vec<String> {
    shows.iter().fold(Vec::new(), |mut acc, show| {
        acc.push(show.title.to_uppercase());
        acc
    })
}

pub fn completed_titles(shows: &[Show]) -> Vec<&str> {
    let mut out = Vec::new();
    for show in shows.iter().filter(|show| show.completed) {
        out.push(show.title.as_str());
    }
    out
}

pub fn total_episodes_watched(shows: &[Show]) -> u32 {
    shows
        .iter()
        .filter(|show| show.completed)
        .fold(0, |acc, show| acc + show.episodes)
}

pub fn first_n_in_progress(shows: &[Show], n: usize) -> Vec<&str> {
    let mut out = Vec::new();
    for show in shows.iter().filter(|show| !show.completed).take(n) {
        out.push(show.title.as_str());
    }
    out
}

pub fn ranked_titles(shows: &[Show]) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (index, show) in shows.iter().enumerate() {
        out.push((index + 1, show.title.clone()));
    }
    out
}
```

هیچ‌کدام از این پنج تابع به `.collect()` نیاز نداشت — هر پنج‌تا، یا با `.fold()`، یا با یک `for` معمولی، همان `Vec`ی را که می‌خواستند با دست ساختند. عمداً هر دو راه را اینجا می‌بینی؛ هر جا خواستی می‌شود جای‌شان را عوض کرد.

## `uppercase_titles` — `.fold()` جای یک `for`

```rust
shows.iter().fold(Vec::new(), |mut acc, show| {
    acc.push(show.title.to_uppercase());
    acc
})
```

این دقیقاً همان الگویِ «مفهوم» است: مقدارِ شروع یک `Vec::new()` خالی است، هر آیتم یک `push` به آن اضافه می‌کند، و کلوژر همان `acc` را — حالا با یکی بیشتر — پس می‌دهد. اگر با `for` هم می‌نوشتی («یک `Vec` خالی بساز، هر بار `push` کن، در آخر برش گردان») همین‌قدر درست بود؛ اینجا فقط `.fold()` انتخاب شد تا با چشمِ خودت ببینی این الگو چقدر مستقیم به یک ترکیب‌گر تبدیل می‌شود.

## `completed_titles` و `first_n_in_progress` — چرا `&str`، نه `String`

```rust
out.push(show.title.as_str());
```

مشخصات هر دو تابع گفته بود «قرض‌گرفته، نه کلون‌شده» — و `.as_str()` دقیقاً همین را می‌دهد: یک `&str` که به همان `String`ِ داخلِ `shows` اشاره می‌کند، بدونِ کپیِ هیچ بایتی. امضای هر دو تابع هم همین را روی خودش نشان می‌دهد: `shows: &[Show]` می‌آید، `Vec<&str>` می‌رود — نه یک لیفتایمِ صریح لازم بود، نه هیچ چیزِ دیگری؛ همان قاعده‌ی elision که در فازِ ۱ دیدی، اینجا هم بی‌سروصدا کارش را کرد.

## `first_n_in_progress` — چرا `.filter()` باید قبل از `.take()` بیاید

```rust
shows.iter().filter(|show| !show.completed).take(n)
```

این ترتیب اتفاقی نیست. `.take(n)` هیچ‌چیزی از «تمام‌نشده» نمی‌داند — فقط می‌گوید «بعد از `n` تا آیتمی که از هر چیزی که قبل از من در خط‌لوله بود رد شد، دست نگه‌دار». اگر جایشان را عوض می‌کردی (`shows.iter().take(n).filter(...)`)، اول `n` تای اولِ **کلِ لیستِ اصلی** را برمی‌داشتی — فارغ از اینکه تمام شده بودند یا نه — و تازه همان‌ها را فیلتر می‌کردی؛ به‌راحتی ممکن بود کمتر از `n` نتیجه بگیری، حتی وقتی جلوترِ لیست شوهای ناتمامِ کافی وجود داشت. تستِ `first_n_in_progress_stops_at_n_matches` دقیقاً همین را چک می‌کند — و دقیقاً همین سؤال در «گرم‌کردن» و در توضیحِ `.take_while()` هم تکرار شد: یک ترکیب‌گرِ محدودکننده فقط به چیزی که از داخلِ خط‌لوله به‌اش می‌رسد نگاه می‌کند، نه به کلِ منبعِ اصلی.

## `total_episodes_watched` — `.filter()` بعد `.fold()`، نه یک `if` داخلِ کلوژر

```rust
shows
    .iter()
    .filter(|show| show.completed)
    .fold(0, |acc, show| acc + show.episodes)
```

می‌شد این را هم با یک `.fold()` تنها نوشت — یک `if show.completed { acc + show.episodes } else { acc }` داخلِ کلوژرش — و همان جواب را می‌گرفت. ولی نسخه‌ی بالا هر قدم را جدا اسم می‌گذارد: «اول این‌ها را نگه دار»، «حالا جمع‌شان بزن» — دقیقاً همان چیزی که در بخشِ «فیلتر» و بخشِ «fold» جدا از هم دیدی، فقط این‌بار پشتِ‌سرِهم.

## `ranked_titles` — `enumerate` به‌علاوه‌ی `+ 1`

```rust
for (index, show) in shows.iter().enumerate() {
    out.push((index + 1, show.title.clone()));
}
```

`.enumerate()` از ۰ می‌شمارد، ولی مشخصات رتبه‌ای از ۱ می‌خواست — همان `+ 1` که تنها فرقش با یک اندیسِ خام است. این‌جا `.clone()` واقعاً لازم بود، بر خلافِ دو تابعِ قبلی: نوعِ بازگشتی `Vec<(usize, String)>` است، نه `Vec<(usize, &str)>` — یعنی نتیجه باید مالکِ رشته‌های خودش باشد، نه فقط قرضشان بگیرد.

## این درس واقعاً درباره‌ی چه بود

- **هیچ ترکیب‌گری به‌تنهایی چیزی نمی‌سازد.** پنج تابعِ بالا هرکدام چند ترکیب‌گر را زنجیر کردند، ولی این خودِ `for` یا خودِ `.fold()` بود که واقعاً `Vec`ِ خروجی را ساخت — همان نکته‌ی مرکزیِ «مفهوم»، این‌بار در کدِ خودت.
- **ترتیبِ ترکیب‌گرها معنا دارد.** `.filter()` قبل از `.take()` جوابی می‌دهد که `.take()` قبل از `.filter()` نمی‌دهد — چون هرکدام فقط چیزی را می‌بیند که از ترکیب‌گرِ قبل از خودش رد شده.
- **قرض‌گرفتن رایگان‌تر از کلون‌گرفتن است، وقتی کافی باشد.** `completed_titles` و `first_n_in_progress` هیچ رشته‌ای را کپی نکردند؛ فقط `ranked_titles`، چون نوعِ بازگشتی‌اش مالکیت می‌خواست، به `.clone()` نیاز داشت.
