# راه‌حل — ۲.۵.۴ طراحیِ رده‌بندیِ خطا برای یک سرویس

```rust
fn validate(title: &str, rating: u8) -> Result<(), ValidationError> {
    if title.trim().is_empty() {
        return Err(ValidationError::EmptyTitle);
    }
    if rating > 10 {
        return Err(ValidationError::RatingOutOfRange { rating });
    }
    Ok(())
}

impl WatchlistStore {
    pub fn new() -> Self {
        WatchlistStore {
            entries: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn add_entry(&mut self, title: &str, rating: u8) -> Result<u64, ServiceError> {
        validate(title, rating)?;
        let id = self.next_id;
        self.next_id += 1;
        self.entries.insert(
            id,
            Entry {
                title: title.to_string(),
                rating,
            },
        );
        Ok(id)
    }

    pub fn rating_of(&self, id: u64) -> Result<u8, ServiceError> {
        self.entries
            .get(&id)
            .map(|entry| entry.rating)
            .ok_or(ServiceError::NotFound { id })
    }

    pub fn restore_from_file(&mut self, path: &str) -> Result<usize, ServiceError> {
        let contents = std::fs::read_to_string(path)?;
        let mut added = 0;
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let Some((title, rating_text)) = line.split_once('=') else {
                continue;
            };
            let Ok(rating) = rating_text.trim().parse::<u8>() else {
                continue;
            };

            let id = self.next_id;
            self.next_id += 1;
            self.entries.insert(
                id,
                Entry {
                    title: title.trim().to_string(),
                    rating,
                },
            );
            added += 1;
        }
        Ok(added)
    }
}
```

هر پنج‌تا دقیقاً همان شکلی را پیاده می‌کنند که `ServiceError` و
`ValidationError` از پیش قول داده بودند — هیچ متدِ تازه‌ای نیست، فقط منطقی
که آن‌ها را تولید و مصرف می‌کند.

## `validate` — رده را انتخاب نمی‌کند، فقط داخلِ رده تصمیم می‌گیرد

```rust
fn validate(title: &str, rating: u8) -> Result<(), ValidationError> {
    if title.trim().is_empty() {
        return Err(ValidationError::EmptyTitle);
    }
    if rating > 10 {
        return Err(ValidationError::RatingOutOfRange { rating });
    }
    Ok(())
}
```

عنوان اول چک می‌شود: اگر هر دو خراب باشند، جوابِ همیشگی `EmptyTitle` است —
دقیقاً همان چیزی که مشخصات در کامنتِ مستندات گفته بود. `validate` اصلاً
نمی‌داند بعداً کجا استفاده می‌شود؛ فقط یک `ValidationError` برمی‌گرداند.
تصمیمِ اینکه این را چطور به `ServiceError` تبدیل کند، بیرونِ این تابع است —
دقیقاً همان تفکیکِ رده که «مفهوم» توضیح داد.

## `add_entry` — `?` رده را از پیش‌نوشته‌شده صدا می‌زند

```rust
pub fn add_entry(&mut self, title: &str, rating: u8) -> Result<u64, ServiceError> {
    validate(title, rating)?;
    let id = self.next_id;
    self.next_id += 1;
    self.entries.insert(id, Entry { title: title.to_string(), rating });
    Ok(id)
}
```

`validate(title, rating)?` یک `ValidationError` را، اگر باشد، خودش به
`ServiceError::Validation` تبدیل می‌کند — چون `#[from]` رویِ آن گونه از
پیش نوشته شده. بعدش، منطقِ شمارشِ شناسه ساده است: مقدارِ فعلیِ `next_id` را
بردار، بعد یکی اضافه‌اش کن. همین ترتیب — اول خواندن، بعد افزایش — است که
اولین ورودیِ موفق همیشه `0` می‌گیرد.

## `rating_of` — داده‌ی ساده، بدونِ زیرخطا

```rust
pub fn rating_of(&self, id: u64) -> Result<u8, ServiceError> {
    self.entries
        .get(&id)
        .map(|entry| entry.rating)
        .ok_or(ServiceError::NotFound { id })
}
```

`.get(&id)` یک `Option<&Entry>` می‌دهد؛ `.map(...)` رویِ آن رَتینگ را
بیرون می‌کشد اگر باشد؛ `.ok_or(...)` همان `Option` را به یک `Result`
تبدیل می‌کند، با `ServiceError::NotFound { id }` وقتی چیزی نبود. توجه کن
همان `id`ای که فراخواننده پرسیده، عیناً در خطا برمی‌گردد — نه یک پیامِ
عمومی، چون این جزئیات دقیقاً همان چیزی است که برایِ رده‌ی «پیدا نشدن»
امن و مفید است.

## `restore_from_file` — فقط یک راهِ شکست

```rust
pub fn restore_from_file(&mut self, path: &str) -> Result<usize, ServiceError> {
    let contents = std::fs::read_to_string(path)?;
    let mut added = 0;
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((title, rating_text)) = line.split_once('=') else {
            continue;
        };
        let Ok(rating) = rating_text.trim().parse::<u8>() else {
            continue;
        };
        // ... insert and count
    }
    Ok(added)
}
```

`?` رویِ `read_to_string` دقیقاً همان‌جایی است که یک `io::Error` واقعی
می‌تواند رخ بدهد، و `#[from]` رویِ `Internal` آن را خودکار تبدیل می‌کند.
بعدِ آن، هیچ خط دیگری نمی‌تواند خطا برگرداند — یک خطِ بدشکل یا یک رَتینگِ
غیرقابل‌پارس فقط با `continue` رد می‌شود، نه با یک `Err`. این عمدی است:
مشخصات گفته بود این تابع «دقیقاً یک راهِ شکست» دارد، و آن راه فقط خودِ
خواندنِ فایل است — نه محتوایِ داخلش.

## این درس واقعاً درباره‌ی چه بود

- **`validate`**: منطقِ داخلِ یک رده، جدا از تصمیمِ اینکه آن رده چیست.
- **`add_entry`**: `?` فقط وقتی رده را خودش انتخاب می‌کند که از پیش
  گفته باشی چطور — `#[from]` همان گفتن است.
- **`rating_of`**: یک رده‌ی کوچک می‌تواند فقط داده باشد؛ جزئیاتش (اینجا،
  `id`) وقتی امن و مفید است، بدونِ ترس بیرون می‌رود.
- **`restore_from_file`**: «این تابع دقیقاً یک راهِ شکست دارد» یک ادعا
  نیست که رایگان می‌آید — با تصمیمِ آگاهانه‌یِ رد‌کردنِ خطوطِ بدشکل به‌جایِ
  خطا دادن رویشان ساخته شده.
- هیچ‌کدام از این چهار تابع رده‌بندی را از نو اختراع نکرد — فقط منطقی
  نوشتند که همان رده‌بندیِ از پیش‌طراحی‌شده را درست پر می‌کند.
