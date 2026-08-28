# راه‌حل — ۲.۱.۴ انتخابِ مجموعه

```rust
use std::collections::{BTreeMap, HashSet, VecDeque};

pub fn unique_viewer_count(viewer_ids: &[u32]) -> usize {
    let mut seen: HashSet<u32> = HashSet::new();
    for id in viewer_ids {
        seen.insert(*id);
    }
    seen.len()
}

pub struct RecentEvents {
    capacity: usize,
    events: VecDeque<String>,
}

impl RecentEvents {
    pub fn new(capacity: usize) -> Self {
        RecentEvents {
            capacity,
            events: VecDeque::new(),
        }
    }

    pub fn record(&mut self, event: &str) {
        self.events.push_back(event.to_string());
        while self.events.len() > self.capacity {
            self.events.pop_front();
        }
    }

    pub fn oldest_to_newest(&self) -> Vec<String> {
        let mut out = Vec::new();
        for event in &self.events {
            out.push(event.clone());
        }
        out
    }
}

pub fn daily_report(entries: &[(u32, u32)]) -> String {
    let mut by_day: BTreeMap<u32, u32> = BTreeMap::new();
    for (day, count) in entries {
        by_day.insert(*day, *count);
    }

    let mut report = String::new();
    for (day, count) in &by_day {
        report.push_str(&format!("day {day}: {count}\n"));
    }
    report
}
```

هر سه‌تا دقیقاً همان استدلالِ بخشِ «مفهوم» را دنبال می‌کنند — نه یک متد جدید، فقط همان جدولِ تصمیم که این‌بار خودت پرش کردی.

## `unique_viewer_count` — عضویت، نه مقدار

```rust
let mut seen: HashSet<u32> = HashSet::new();
for id in viewer_ids {
    seen.insert(*id);
}
seen.len()
```

سؤالِ واقعی این تابع فقط «این شناسه را قبلاً دیده‌ام یا نه؟» است — نه کلید-به-مقدار، نه ترتیب، نه اندیس. دقیقاً همان محورِ اول: وقتی فقط عضویت مهم است، `HashSet` جستجویش `O(1)` است؛ با یک `Vec` مجبور بودی برای هر شناسه‌ی تازه کلِ لیستِ دیده‌شده‌ها را خطی بگردی — همان چیزی که مثالِ `01-lookup-by-key-vs-scan` نشانت داد، فقط اینجا سؤال «موجود است؟» است نه «مقدارش چیست؟».

`.insert()` روی یک مقدارِ تکراری هیچ کاری نمی‌کند و false برمی‌گرداند (اینجا نادیده‌اش گرفتیم) — دقیقاً همان رفتاری که «هر شناسه فقط یک‌بار شمرده شود» می‌خواهد، بدونِ هیچ چکِ صریحِ `if already_seen`.

## `RecentEvents` — هر دو سر، ارزان

```rust
pub fn record(&mut self, event: &str) {
    self.events.push_back(event.to_string());
    while self.events.len() > self.capacity {
        self.events.pop_front();
    }
}
```

این دقیقاً محورِ چهارم است: هر بار یک رویداد از **عقب** اضافه می‌شود (`push_back`)، و وقتی ظرفیت پر شد، از **جلو** برداشته می‌شود (`pop_front`) — دقیقاً همان دو عملیاتی که `Vec` روی سرِ جلو رایگان نمی‌دهد (مثالِ `06-vec-has-no-pop-front` را یادت هست) ولی `VecDeque` هر دو را با هزینه‌ی `O(1)` می‌دهد. اگر می‌خواستی این را با `Vec` بسازی، `record` باید `remove(0)` صدا می‌زد — یعنی هر رویدادِ تازه، جابه‌جاییِ کلِ بافر.

حلقه‌ی `while` (نه `if`) عمداً است: حتی اگر یک نفر `capacity` را بینِ دو `record` عوض کند، تابع هنوز درست کار می‌کند. برای `capacity == 0`، همان حلقه بلافاصله هر چیزی را که تازه `push_back` شده دوباره بیرون می‌کشد — دقیقاً «هیچ‌وقت چیزی نگه نمی‌دارد».

`oldest_to_newest` یک کپیِ مالکِ رشته‌ها برمی‌گرداند (نه ارجاع به `self.events`) چون امضایش `Vec<String>` قول داده، نه `&VecDeque<String>` — دقیقاً همان تمایزِ «قرض در برابرِ مالکیت» که فازِ ۱ یادت داد، اینجا رویِ محتوای یک مجموعه.

## `daily_report` — کلید تکراری یعنی جایگزینی، نه جمع

```rust
let mut by_day: BTreeMap<u32, u32> = BTreeMap::new();
for (day, count) in entries {
    by_day.insert(*day, *count);
}
```

`BTreeMap::insert` وقتی کلید از قبل هست، مقدارِ قدیمی را با مقدارِ تازه **عوض** می‌کند — نه جمع، نه خطا. همین یک خط دقیقاً قاعده‌ی «ورودیِ متأخر برنده است» را که مشخصات خواسته بود مجانی می‌دهد.

```rust
let mut report = String::new();
for (day, count) in &by_day {
    report.push_str(&format!("day {day}: {count}\n"));
}
report
```

اینجاست که انتخابِ `BTreeMap` به‌جایِ `HashMap` واقعاً خودش را نشان می‌دهد: پیمایشِ `&by_day` تضمین می‌کند روزها از کوچک به بزرگ بیرون می‌آیند — بدونِ اینکه لازم باشد جفت‌ها را جمع کنی و بعد خودت مرتبشان کنی (همان کاری که با `HashMap` مجبور بودی انجام بدهی، دقیقاً همان نکته‌ای که درسِ اول همین ماژول درباره‌ی `top_n` گفت).

## این درس واقعاً درباره‌ی چه بود

- **`unique_viewer_count`**: وقتی سؤال «آیا این عضو است؟» است، نه «مقدارش چیست؟»، `HashSet` دقیقاً همان چیزی است که لازم داری — نه کمتر، نه بیشتر.
- **`RecentEvents`**: وقتی باید از **هر دو سر** ارزان اضافه/برداری، `VecDeque` تنها گزینه‌ای است از این شش‌تا که هر دو عملیات را `O(1)` می‌دهد؛ `Vec` یکی از دو سر را همیشه `O(n)` می‌کند.
- **`daily_report`**: وقتی هم باید با کلید بازنویسی کنی و هم در پایان به ترتیبِ کلید نیاز داری، `BTreeMap` هر دو را در یک ساختار می‌دهد — یک `HashMap` مجبورت می‌کرد جداگانه جمع کنی و مرتب کنی.
- سه تابع، سه امضای متفاوت، و **هیچ‌کدام نگفت کدام مجموعه را به‌کار ببر.** همان جایی که تصمیم گرفتی، همان‌جا درس را واقعاً یاد گرفتی.
