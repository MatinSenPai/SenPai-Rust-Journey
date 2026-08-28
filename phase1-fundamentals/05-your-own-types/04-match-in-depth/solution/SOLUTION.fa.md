# پاسخ — ۱.۵.۴ `match` از نزدیک

```rust
pub fn describe(progress: &Progress) -> String {
    match progress {
        Progress::NotStarted => "not started".to_string(),
        Progress::Reading { chapter, of } => format!("chapter {chapter} of {of}"),
        Progress::Finished { rating } => format!("finished, {rating}/10"),
        Progress::Dropped { at } => format!("dropped at chapter {at}"),
    }
}

pub fn band(stars: u8) -> String {
    match stars {
        0 => "unrated".to_string(),
        1..=3 => "weak".to_string(),
        4..=6 => "watchable".to_string(),
        7 | 8 => "good".to_string(),
        9 | 10 => "top shelf".to_string(),
        _ => "not a score".to_string(),
    }
}

pub fn shelf(progress: &Progress) -> String {
    match progress {
        Progress::Finished { rating: 9 | 10 } => "hall of fame".to_string(),
        Progress::Finished { .. } => "read".to_string(),
        Progress::Reading { chapter, of } if chapter == of => {
            "waiting for the next chapter".to_string()
        }
        Progress::Reading { .. } => "reading".to_string(),
        Progress::NotStarted => "the pile".to_string(),
        Progress::Dropped { .. } => "gone".to_string(),
    }
}

pub fn chapter_label(chapter: u32) -> String {
    match chapter {
        0 => "no chapters yet".to_string(),
        n @ 1..=9 => format!("early (chapter {n})"),
        n @ 10..=99 => format!("mid (chapter {n})"),
        n => format!("long runner (chapter {n})"),
    }
}

pub fn pair_verdict(mine: &Progress, theirs: &Progress) -> String {
    use Progress::{Finished, NotStarted};
    match (mine, theirs) {
        (Finished { rating: a }, Finished { rating: b }) if a == b => "we agree".to_string(),
        (Finished { .. }, Finished { .. }) => "we disagree".to_string(),
        (NotStarted, NotStarted) => "neither of us has started".to_string(),
        (Finished { .. }, _) | (_, Finished { .. }) => "one of us finished".to_string(),
        _ => "still reading".to_string(),
    }
}

pub fn release_note(latest: Option<u32>, read: u32) -> String {
    match latest {
        None => "nothing published yet".to_string(),
        Some(n) if n == read => "caught up".to_string(),
        Some(n) if n > read => format!("{} behind", n - read),
        Some(_) => "ahead of the release".to_string(),
    }
}
```

شش تابع، و هیچ‌کدام یک `if` ندارد. این تصادفی نیست.

## `describe` — شکلِ پایه

```rust
Progress::Reading { chapter, of } => format!("chapter {chapter} of {of}"),
```

یک بازو به‌ازای هر واریانت، و هر بازو داده‌ی خودش را بیرون می‌کشد. هیچ `_`ای نیست — و همین است که این تابع را در آینده امن می‌کند: روزی که یک واریانتِ پنجم اضافه شود، این `match` خطا می‌دهد و از تو می‌پرسد جوابِ آن حالت چیست.

اگر به‌جای این، `if let` یا یک زنجیره‌ی `if` می‌نوشتی هم کار می‌کرد و تست‌ها هم سبز می‌شدند. تفاوت فقط در روزِ تغییر معلوم می‌شود.

نکته‌ی ریز: `chapter` این‌جا `&u32` است چون `progress` یک `&Progress` است. `format!` خودش ارجاع را دنبال می‌کند، پس این‌جا `*` لازم نیست. اگر می‌خواستی جمع و تفریق کنی، لازم می‌شد.

## `band` — بازه، جایگزین، و یک `_` که این‌بار درست است

```rust
1..=3 => "weak".to_string(),
7 | 8 => "good".to_string(),
_ => "not a score".to_string(),
```

سه ابزار در سه خط: بازه، جایگزین، جانشین.

این‌جا `_` انتخابِ درستی است و در `describe` نبود — تفاوت در نوع است. یک `u8` دویست‌و‌پنجاه‌وشش مقدار دارد و هرگز هم بیشتر نمی‌شود؛ نوشتنِ `11..=255` هم درست بود ولی چیزی اضافه نمی‌کرد. یک شمارشی برعکس است: تعدادِ واریانت‌هایش دقیقاً همان چیزی است که فردا عوض می‌شود.

می‌شد `9 | 10` را `9..=10` نوشت. برای دو مقدار فرقی ندارد؛ برای پنج مقدار بازه خواناتر است.

## `shelf` — سه ابزار در یک تابع

```rust
Progress::Finished { rating: 9 | 10 } => "hall of fame".to_string(),
```

اول: `|` می‌تواند **داخلِ** یک الگو هم بیاید، نه فقط بینِ دو بازوی کامل. این یعنی «یک `Finished` که `rating`اش ۹ یا ۱۰ است».

```rust
Progress::Reading { chapter, of } if chapter == of => {
    "waiting for the next chapter".to_string()
}
```

دوم: این تنها جایی است که نگهبان واقعاً لازم است. هیچ الگویی نمی‌تواند بگوید «این دو فیلد با هم برابرند» — الگو دربار‌ه‌ی شکل حرف می‌زند، نه دربار‌ه‌ی رابطه‌ی بینِ مقدارها. مقایسه‌ی `chapter == of` روی دو `&u32` انجام می‌شود و درست کار می‌کند، چون Rust برابریِ ارجاع‌ها را به برابریِ مقدارها ترجمه می‌کند.

سوم: ترتیب. بازوی `Finished { rating: 9 | 10 }` باید بالای `Finished { .. }` باشد. اگر جابه‌جایشان کنی، تست‌ها می‌شکنند و کامپایلر هم `unreachable pattern` می‌دهد — همان چیزی که در مثالِ `03` دیدی، این‌بار روی کدِ خودت.

و باز هم هیچ `_`ای نیست. شش بازو برای چهار واریانت، چون دو واریانت دو رفتار دارند.

## `chapter_label` — `@` دقیقاً برای همین است

```rust
n @ 1..=9 => format!("early (chapter {n})"),
```

بدونِ `@` مجبور بودی دو بار همان کار را بنویسی: یک بار بازه را بسنجی و یک بار عدد را بگیری. با `@` هر دو در یک الگو انجام می‌شود.

بازوی آخر `n` است بدونِ هیچ بازه‌ای — یک الگوی «هر چیزی، و اسمش را بگذار `n`». نوشتنِ `100..=u32::MAX` هم درست بود، ولی `n` هم همان را می‌گوید و هم کوتاه‌تر است. توجه کن که این‌جا `_` **نمی‌توانست** جوابگو باشد، چون به خودِ عدد احتیاج داشتیم.

## `pair_verdict` — چهار حالت از شانزده، در یک `match`

```rust
(Finished { rating: a }, Finished { rating: b }) if a == b => "we agree".to_string(),
(Finished { .. }, Finished { .. }) => "we disagree".to_string(),
```

دو بازوی اول یک الگوی رایج‌اند: **خاص با نگهبان، بعد همان الگو بدونِ نگهبان.** چون بازوها به ترتیب امتحان می‌شوند، بازوی دوم فقط وقتی نوبتش می‌شود که نگهبانِ اولی رد شده باشد — یعنی «هر دو `Finished`اند و امتیازها فرق دارند»، بدونِ اینکه لازم باشد `a != b` بنویسی.

```rust
(Finished { .. }, _) | (_, Finished { .. }) => "one of us finished".to_string(),
```

این بازو «دقیقاً یکی‌شان `Finished` است» را می‌گوید — ولی فقط به این دلیل که بعد از دو بازوی «هر دو `Finished`» آمده. اگر بالاتر می‌بردی‌اش، حالتِ «هر دو» را هم می‌بلعید و دو تستِ اول می‌شکستند. ترتیب این‌جا بخشی از منطق است، نه سلیقه.

`use Progress::{Finished, NotStarted};` داخلِ تابع فقط برای خوانایی است. با نامِ کاملِ `Progress::Finished { .. }` هم دقیقاً همان است، فقط سه برابر طولانی‌تر.

## `release_note` — `Option` هیچ چیزِ تازه‌ای نمی‌خواهد

```rust
None => "nothing published yet".to_string(),
Some(n) if n == read => "caught up".to_string(),
Some(n) if n > read => format!("{} behind", n - read),
Some(_) => "ahead of the release".to_string(),
```

چهار بازو برای شمارشی‌ای که دو واریانت دارد — چون دو تا از حالت‌ها نگهبان دارند. و همان قاعده‌ی همیشگی: زنجیره را با یک بازوی بی‌نگهبان تمام کن. اگر آخری را `Some(n) if n < read` بنویسی، `E0004` می‌گیری، چون کامپایلر بازوهای نگهبان‌دار را نمی‌شمارد.

بازوی آخر `Some(_)` است نه `Some(n)`، چون در آن حالت به عدد کاری نداریم. اگر `Some(n)` بنویسی، کامپایلر هشدارِ «متغیرِ استفاده‌نشده» می‌دهد و پیشنهاد می‌کند `_n` بنویسی. `_` گفتنِ صریح‌ترِ همان چیز است.

و `n - read` این‌جا امن است چون بازوی قبلی‌اش تضمین کرده که `n > read`. اگر ترتیب را عوض کنی، همان تفریق در حالتِ `n < read` سرریز می‌کند و در بیلدِ دیباگ پنیک می‌گیرد — همان چیزی که در [۱.۱.۲](../../../01-foundations/02-scalar-types-and-overflow/README.fa.md) دیدی. نگهبان‌ها این‌جا فقط دسته‌بندی نمی‌کنند؛ کدِ بعدی را هم امن می‌کنند.

## این درس واقعاً دربار‌ه‌ی چه بود

- **جامع بودن یک ویژگی نیست، یک اهرم است.** ارزشش را روزی می‌دهد که یک واریانت اضافه کنی و کامپایلر فهرستِ کارها را بنویسد.
- **`_` روی یک شمارشی، همان اهرم را می‌شکند.** پنج تا از این شش تابع `_` ندارند و همین عمدی است. `band` استثناست، چون روی `u8` است نه روی یک شمارشی.
- **یک بازو در یک قدم هم می‌سنجد و هم بیرون می‌کشد.** هیچ‌جای این فایل یک `if` و بعد یک دسترسی به فیلد نیست.
- **ترتیب معنا دارد.** در `shelf` و در `pair_verdict`، جابه‌جا کردنِ دو بازو منطق را عوض می‌کند و کامپایلر فقط بعضی وقت‌ها هشدار می‌دهد.
- **نگهبان‌ها چیزی را می‌گویند که الگو نمی‌تواند** — و در عوض، در شمارشِ کامپایلر حساب نمی‌شوند.

درسِ بعدی ([۱.۵.۵](../../05-if-let-while-let-let-else/README.fa.md)) همین را برای وقتی که فقط یک بازو برایت مهم است کوتاه می‌کند. چیزِ تازه‌ای نیست؛ همین است با نوشتنِ کمتر.
