# راه‌حل — ۱.۷.۱ پروژه‌ی کوچکِ راهنمایی‌شده

```rust
pub fn add(&mut self, entry: Entry) {
    self.entries.push(entry);
}

pub fn find(&self, title: &str) -> Option<&Entry> {
    for entry in &self.entries {
        if entry.title == title {
            return Some(entry);
        }
    }
    None
}

pub fn titles(&self) -> Vec<&str> {
    let mut out = Vec::with_capacity(self.entries.len());
    for entry in &self.entries {
        out.push(entry.title.as_str());
    }
    out
}

pub fn rate(&mut self, title: &str, rating: Rating) -> Result<(), WatchlistError> {
    for entry in &mut self.entries {
        if entry.title == title {
            entry.status = Status::Finished { rating };
            return Ok(());
        }
    }
    Err(WatchlistError::NotFound(title.to_string()))
}

pub fn rate_from_text(&mut self, title: &str, rating_text: &str) -> Result<(), WatchlistError> {
    let raw: u8 = rating_text.trim().parse()?;
    self.rate(title, Rating::new(raw))
}
```

پنج متد، پنج تصمیم — و هیچ‌کدام‌شان تصادفی نیستند. تایپ‌ها (`Rating`، `Status`، `Entry`، `WatchlistError`) از قبل کامل داده شده بودند؛ کاری که اینجا اتفاق افتاده فقط پُر کردنِ بدنه‌ی این پنج متد است، هرکدام دقیقاً همان‌طور که doc comment‌اش گفته بود.

## `add` — مالکیت، چون کسی باید نگهش دارد

```rust
self.entries.push(entry);
}
```

یک خط. هیچ چیزِ هوشمندانه‌ای در کار نیست، و نباید هم باشد — `add` قرار است `entry` را برای همیشه نگه دارد، و `push` روی `Vec<Entry>` دقیقاً همین را می‌کند. امضا خودش کارِ اصلی را از قبل انجام داده بود: `entry: Entry`، نه `&Entry`، یعنی از لحظه‌ای که وارد این تابع می‌شود دیگر مالِ صدازننده نیست.

اگر این را با `entry: &Entry` می‌نوشتی، اصلاً کامپایل نمی‌شد — `self.entries.push(entry)` یک `Entry` مالکانه می‌خواهد، نه یک نگاهِ قرضی به یکی. امضا، پیش از نوشتنِ یک خط بدنه، مسیر را بسته بود.

## `find` — یک حلقه‌ی دستی، چون هنوز به iteratorها نرسیده‌ایم

```rust
for entry in &self.entries {
    if entry.title == title {
        return Some(entry);
    }
}
None
```

این دقیقاً همان چیزی‌ست که فاز ۲ با `self.entries.iter().find(|e| e.title == title)` در یک خط می‌نویسد — ولی این درس هنوز آن‌جا نیست، پس یک حلقه‌ی معمولی، با یک `return` زودهنگام، همان کار را می‌کند.

نکته‌ی ظریف‌تر در `&self.entries` است: حلقه روی **قرض‌ها** پیمایش می‌کند، نه روی خودِ عنصرها. `entry` در هر دور از نوعِ `&Entry` است، پس `Some(entry)` دقیقاً همان `Option<&Entry>`ای می‌سازد که امضا وعده داده بود — بدونِ هیچ `&` اضافه‌ای که خودت بنویسی. اگر روی `self.entries` (بدونِ `&`) پیمایش می‌کردی، همان اولین دور، مالکیتِ کلِ `Vec` را می‌بُرد و کامپایلر با یک خطای مالکیت متوقفت می‌کرد — همان قاعده‌ی موردِ ۱.۲.۴ که یک تابع نباید چیزی را که فقط قرار است بخواند مصرف کند.

## `titles` — یک `Vec<&str>`، ساخته‌شده با `with_capacity`

```rust
let mut out = Vec::with_capacity(self.entries.len());
for entry in &self.entries {
    out.push(entry.title.as_str());
}
out
```

`with_capacity` اجباری نبود — `Vec::new()` هم درست کار می‌کرد — ولی چون تعدادِ دقیقِ عنصرها از قبل معلوم است (`self.entries.len()`)، این یک تخصیصِ رایگان است که از تخصیصِ دوباره در وسطِ حلقه جلوگیری می‌کند؛ همان استدلالِ ۱.۲.۳ درباره‌ی ظرفیت.

`entry.title.as_str()` تبدیلِ یک `&String` به یک `&str` است — بدونِ کپی، بدونِ تخصیص، فقط یک نگاهِ دیگر به همان بافر. این خطِ حلقه دقیقاً می‌گوید چه چیزی داری می‌سازی: یک وکتور از نگاه‌ها، نه رونوشت‌ها.

## `rate` — تنها عملیاتی که `Result` دارد

```rust
for entry in &mut self.entries {
    if entry.title == title {
        entry.status = Status::Finished { rating };
        return Ok(());
    }
}
Err(WatchlistError::NotFound(title.to_string()))
```

اینجا حلقه روی `&mut self.entries` است، نه `&self.entries` — چون این‌بار قرار است `entry.status` را عوض کنیم، و برای عوض‌کردن، ارجاعِ انحصاری لازم است، همان قاعده‌ی ۱.۳.۱.

نکته‌ی مهم‌تر جای `Err` است: بیرونِ حلقه، بعد از این‌که همه‌ی عنصرها را بی‌نتیجه گشتیم. `title.to_string()` تنها جایی‌ست که این تابع تخصیص می‌گیرد، و فقط وقتی گرفته می‌شود که واقعاً لازم باشد — اگر عنوان پیدا شود، تابع پیش از رسیدن به آن خط با `return Ok(())` برگشته.

## `rate_from_text` — جایی که `?` کارِ دوتایی می‌کند

```rust
let raw: u8 = rating_text.trim().parse()?;
self.rate(title, Rating::new(raw))
```

دو خط، ولی هر خط یک تصمیم دارد.

خطِ اول: `.trim()` فاصله‌های اضافیِ دورِ ورودی را برمی‌دارد — چیزی که یک ورودیِ واقعی از طرفِ کاربر تقریباً همیشه دارد. `.parse::<u8>()` رشته را به عدد تبدیل می‌کند و `Result<u8, ParseIntError>` برمی‌گرداند. `?` روی آن یعنی: اگر `Ok` بود، مقدار را بگیر و ادامه بده؛ اگر `Err` بود، آن `ParseIntError` را با `From<ParseIntError> for WatchlistError` (که بالای همین فایل نوشته شده) تبدیل کن، و همین الان از `rate_from_text` با آن خطا برگرد. یک علامت، دو کار: بازکردنِ مقدار و تبدیلِ نوعِ خطا.

خطِ دوم چیزِ تازه‌ای نمی‌سازد — کارش را به `rate` می‌سپارد. چون `rate` از قبل دقیقاً همان `Result<(), WatchlistError>` را برمی‌گرداند که `rate_from_text` هم قرار است برگرداند، نیازی به `?` یا `match` دیگری نیست؛ همان مقدار، همان‌طور که هست، آخرین خطِ تابع است.

و ترتیب اینجا مهم است: `parse` پیش از هر جست‌وجویی اجرا می‌شود. اگر متن نامعتبر باشد، تابع پیش از این‌که اصلاً `self.rate` صدا زده شود، با `InvalidRating` برمی‌گردد — حتی اگر `title` هم در فهرست نباشد. تستِ `rate_from_text_reports_bad_text_before_checking_the_title` دقیقاً همین را بررسی می‌کند.

## چیزی که این درس واقعاً درباره‌اش بود

- **امضای یک متد، تصمیمِ مالکیت را قبل از بدنه‌اش می‌گیرد.** `Entry` مالکانه در `add` یعنی «نگهش می‌دارم»؛ `&Entry` در خروجیِ `find` یعنی «فقط نشانت می‌دهم». وقتی امضا درست باشد، بدنه معمولاً خودش را می‌نویسد.
- **یک `enum` با واریانت‌های داده‌دار، و یک `match` جامع روی آن، با هم حالتِ نامعتبر را نانوشتنی و فراموش‌کردنِ یک حالت را غیرقابلِ کامپایل می‌کنند.**
- **پدینگِ رشته و برشِ رشته دو قاعده‌ی کاملاً جدا هستند.** یکی نویسه می‌شمارد، دیگری بایت — و این تفاوت دقیقاً همان‌جایی‌ست که یک عنوانِ فارسی، یک باگِ خاموش را آشکار می‌کند.
- **`Result` فقط برای عملیاتی نوشته می‌شود که واقعاً می‌تواند شکست بخورد.** `add`، `find`، `titles` هرگز شکست نمی‌خورند، و بدونِ `Result` ساده‌تر و صادق‌ترند.
- **`?` بیشتر از یک `return` زودهنگام است — یک `From` هم صدا می‌زند.** همین یک علامت، `ParseIntError` را بی‌آن‌که خودت یک خط `match` بنویسی، به `WatchlistError` تبدیل کرد.
- و مهم‌تر از همه: هیچ‌کدامِ این پنج متد، مفهومِ تازه‌ای نبود. سی درسِ قبل، همه‌شان اینجا بودند — فقط این‌بار، کنارِ هم.
