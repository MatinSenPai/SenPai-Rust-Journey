# راه‌حل — ۱.۷.۲ مرورِ فاز

```rust
pub fn grade_letter(score: u32) -> char {
    if score >= 90 {
        'A'
    } else if score >= 80 {
        'B'
    } else if score >= 70 {
        'C'
    } else if score >= 60 {
        'D'
    } else {
        'F'
    }
}

pub fn merge_unique(base: Vec<String>, extra: &[String]) -> Vec<String> {
    let mut merged = base;
    for item in extra {
        if !merged.contains(item) {
            merged.push(item.clone());
        }
    }
    merged
}

pub fn interior(values: &[i32]) -> &[i32] {
    if values.len() < 2 {
        &[]
    } else {
        &values[1..values.len() - 1]
    }
}

pub fn shorten(text: &str, max_chars: usize) -> &str {
    let mut end = text.len();
    let mut count = 0;
    for (index, _) in text.char_indices() {
        if count == max_chars {
            end = index;
            break;
        }
        count += 1;
    }
    &text[..end]
}

pub enum Status {
    Pending,
    Shipped { tracking: String },
    Cancelled { reason: String },
}

pub fn describe_status(status: &Status) -> String {
    match status {
        Status::Pending => "pending".to_string(),
        Status::Shipped { tracking } => format!("shipped, tracking {tracking}"),
        Status::Cancelled { reason } => format!("cancelled: {reason}"),
    }
}

pub fn safe_average(values: &[i32]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut total: i64 = 0;
    for value in values {
        total += *value as i64;
    }
    Some(total as f64 / values.len() as f64)
}
```

شش تابع، هرکدام از یک ماژولِ فاز ۱. هرکدام را جدا می‌خوانیم — نه چون کد پیچیده است، بلکه چون ارزشِ این تمرین در این است که بگویی *کدام* تصمیم پشتِ هرکدام است.

## `grade_letter` — یک تصمیم که مقدار است

```rust
if score >= 90 {
    'A'
} else if score >= 80 {
    'B'
} ...
```

نه `let mut letter` ای در کار است، نه `return` ای. کلِ زنجیره‌ی `if`/`else if`/`else` یک عبارت است و مقدارِ خودش همان چیزی است که تابع برمی‌گرداند. این دقیقاً همان نکته‌ای است که [۱.۱.۵](../../../01-foundations/05-control-flow/README.fa.md) با آن باز شد: در Rust جریانِ کنترل فقط شکل نمی‌دهد، مقدار هم تولید می‌کند.

ترتیبِ شرط‌ها هم مهم است: از بالا به پایین چک می‌شود، پس نوشتنِ `score >= 60` قبل از `score >= 90` هر نمره‌ای را `'D'` می‌کرد. اولین شرطی که برقرار باشد برنده است.

## `merge_unique` — مالکیت، نه فقط امضا

```rust
let mut merged = base;
for item in extra {
    if !merged.contains(item) {
        merged.push(item.clone());
    }
}
merged
```

`base: Vec<String>` بدونِ `&` گرفته شده — یعنی تابع مالکِ آن می‌شود، همان چیزی که [۱.۲.۴](../../../02-ownership-and-memory/04-ownership-across-functions/README.fa.md) بین `fn f(x: String)` و `fn f(x: &str)` فرق می‌گذاشت. اینجا انتخابِ درست مالکیت است چون تابع دقیقاً همین Vec را — گسترش‌یافته — پس می‌دهد؛ چیزی برای «قرض دادن و پس گرفتن» نیست.

`extra: &[String]` برعکس: فقط خوانده می‌شود، پس قرض کافی است — [۱.۳.۱](../../../03-borrowing-and-references/01-shared-and-mutable-refs/README.fa.md).

و نکته‌ی ظریف‌تر: `.clone()` فقط داخلِ شاخه‌ی `if` است. رشته‌هایی که از قبل در `merged` هستند هرگز کلون نمی‌شوند — چون از اول همان‌جا هستند و کسی مجبور نیست دوباره تخصیصشان بدهد. رشته‌هایی که تکراری‌اند هم کلون نمی‌شوند — چون اصلاً وارد نمی‌شوند. تنها چیزی که کلون می‌گیرد دقیقاً همان چیزی است که قرار است اضافه شود. این [۱.۲.۳](../../../02-ownership-and-memory/03-clone-and-copy/README.fa.md) است: کلون باید در کد دیده شود، و فقط جایی که واقعاً لازم است.

## `interior` — نگاه، نه کپی

```rust
&values[1..values.len() - 1]
```

هیچ `Vec` جدیدی ساخته نمی‌شود. `&values[1..values.len() - 1]` فقط یک نگاهِ قرضی به بخشی از حافظه‌ای است که از قبل وجود دارد — دقیقاً همان تعریفی که [۱.۳.۴](../../../03-borrowing-and-references/04-slices/README.fa.md) از اسلایس داد: دو کلمه، شروع و طول، بدونِ مالکیت.

شرطِ `values.len() < 2` قبل از تفریق می‌آید چون `values.len() - 1` وقتی طول صفر است زیرِ صفر می‌رود — و چون این عدد از نوعِ `usize` است (که هرگز منفی نمی‌شود)، آن تفریق panic می‌کند، نه اینکه یک عددِ منفیِ بی‌ضرر بدهد. این تله‌ی همان [سرریز](../../../01-foundations/02-scalar-types-and-overflow/README.fa.md) است که [۱.۱.۲](../../../01-foundations/02-scalar-types-and-overflow/README.fa.md) درباره‌اش هشدار داده بود، همین‌جا دوباره سر و کله‌اش پیدا شد.

## `shorten` — بایت نشمار، اسکالر بشمار

```rust
for (index, _) in text.char_indices() {
    if count == max_chars {
        end = index;
        break;
    }
    count += 1;
}
&text[..end]
```

`text.char_indices()` هر اسکالرِ یونیکد را همراهِ نشانیِ بایتی‌اش می‌دهد. حلقه شمارش می‌کند تا به `max_chars`مین کاراکتر برسد، نشانیِ بایتیِ *همان‌جا* را نگه می‌دارد، و برشِ نهایی روی همان مرز اتفاق می‌افتد — نه روی `max_chars`اُمین بایت.

این دقیقاً همان چیزی است که [۱.۴.۲](../../../04-text-and-strings/02-utf8-bytes-chars-graphemes/README.fa.md) با «سلام» نشان داد: چهار حرف، هشت بایت. اگر این تابع `&text[..max_chars]` را مستقیم می‌نوشت، `shorten("سلام", 2)` می‌شد `&text[..2]` — که دقیقاً وسطِ حرفِ «س» برش می‌خورد و panic می‌داد، همان چیزی که [تمرینِ تعمیر](../README.fa.md#تعمیر) همین درس رویش ساخته شده.

اگر `max_chars` از تعدادِ کاراکترها بیشتر باشد، حلقه هرگز `break` نمی‌شود و `end` همان `text.len()` اولیه می‌ماند — یعنی کلِ متن.

## `describe_status` — کامل بودنِ اجباری

```rust
match status {
    Status::Pending => "pending".to_string(),
    Status::Shipped { tracking } => format!("shipped, tracking {tracking}"),
    Status::Cancelled { reason } => format!("cancelled: {reason}"),
}
```

سه شاخه برای سه شکلِ `Status` — نه کمتر. اگر یک شاخه جا می‌ماند، این `match` اصلاً کامپایل نمی‌شد؛ کامپایلر خودش دقیقاً می‌گفت کدام شاخه غایب است. این «کاملیّتِ اجباری» چیزی است که [۱.۵.۴](../../../05-your-own-types/04-match-in-depth/README.fa.md) به‌عنوانِ بزرگ‌ترین هدیه‌ی `match` معرفی کرد: یک enum که فردا یک شکلِ چهارم بگیرد، هر `match` ناقصی را به خطای کامپایل تبدیل می‌کند، نه به یک باگِ ساکت.

الگوهای `{ tracking }` و `{ reason }` همان لحظه‌ای که شکل را تشخیص می‌دهند، داده‌ی داخلش را هم می‌گیرند — دقیقاً همان درسی که [۱.۵.۳](../../../05-your-own-types/03-enums-as-data/README.fa.md) با enumهای داده‌دار داد.

## `safe_average` — نبودن به‌جای تصادف

```rust
if values.is_empty() {
    return None;
}
...
Some(total as f64 / values.len() as f64)
```

قبل از هر محاسبه‌ای، حالتِ «هیچ عددی نیست» را جدا می‌کنیم و صریح می‌گوییم: `None`. اگر این چک نبود، `values.len() as f64` می‌شد صفر، و تقسیم بر صفر برای اعشاری‌ها panic نمی‌کند — `NaN` برمی‌گرداند، عددی که ساکت در برنامه پخش می‌شود و هزار خط بعد سر و کله‌اش پیدا می‌شود. `Option` این را غیرممکن می‌کند: تماس‌گیرنده مجبور است قبل از استفاده از میانگین، به `None` هم فکر کند. این همان چیزی است که [۱.۶.۱](../../../06-absence-and-failure/01-option-and-null-safety/README.fa.md) به‌جایِ یک مقدارِ نشانه‌ای مثلِ `-1.0` یا `0.0` پیشنهاد داد.

## چیزی که این درس واقعاً درباره‌اش بود

- شش تابع، شش تصمیمِ متفاوت — و هرکدام دقیقاً به یک درسِ فاز ۱ برمی‌گردد، نه به «Rust» به‌طورِ کلی.
- اگر یکی از این‌ها سخت بود، آن سختی به تو می‌گوید کدام فصل را دوباره باز کنی — این خودِ فایده‌ی این تمرین است.
- هیچ‌کدام از این توابع به چیزی بیرونِ فاز ۱ نیاز نداشت: نه `HashMap`، نه بستار، نه صفتِ خودت. همه‌چیز از همین سی‌ویک درس آمد.
