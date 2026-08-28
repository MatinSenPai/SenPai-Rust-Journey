# راه‌حل — ۲.۱.۱ `Vec` از نزدیک

```rust
pub fn retain_unwatched(entries: &mut Vec<WatchEntry>) {
    entries.retain(|entry| !entry.watched);
}

pub fn drain_first_n(entries: &mut Vec<WatchEntry>, n: usize) -> Vec<WatchEntry> {
    let n = n.min(entries.len());
    let mut removed = Vec::new();
    for item in entries.drain(0..n) {
        removed.push(item);
    }
    removed
}

pub fn dedup_adjacent_titles(entries: &mut Vec<WatchEntry>) {
    entries.dedup_by_key(|entry| entry.title.clone());
}

pub fn sorted_by_rating(entries: Vec<WatchEntry>) -> Vec<WatchEntry> {
    let mut entries = entries;
    entries.sort_by(|a, b| a.rating.total_cmp(&b.rating));
    entries
}

pub fn find_by_rating(entries: &[WatchEntry], target: f64) -> Option<usize> {
    entries
        .binary_search_by(|entry| entry.rating.total_cmp(&target))
        .ok()
}
```

هیچ‌کدام از این پنج تابع به کلوژرهای پیچیده، جنریک یا `HashMap` نیاز نداشت — همان پنج متدی که در «مفهوم» دیدی، فقط این‌بار روی داده‌ی خودت.

## `retain_unwatched` — یک خط، همان چیزی که در مثالِ ۰۳ دیدی

```rust
entries.retain(|entry| !entry.watched);
```

`.retain()` خودش «فقط عنصرهایی که این شرط را پاس کنند بمانند» را انجام می‌دهد؛ کارِ تو فقط نوشتنِ شرط بود. نکته: کلوژر `!entry.watched` می‌گیرد، نه `entry.watched` — چون `.retain()` می‌پرسد «این بماند؟» نه «این برود؟».

## `drain_first_n` — `.min()` قبل از `.drain()`، بدونِ `.collect()`

```rust
let n = n.min(entries.len());
let mut removed = Vec::new();
for item in entries.drain(0..n) {
    removed.push(item);
}
removed
```

`n.min(entries.len())` همان تضمینِ «اگر `n` از طول بیشتر بود، همه برداشته شوند» را می‌دهد — بدونِ آن، `entries.drain(0..n)` با یک `n` بزرگ‌تر از طول پنیک می‌گرفت (بازه‌ی نامعتبر). بعدش، به‌جایِ `.drain(0..n).collect()` (که فازِ ۲.۲ هنوز نداده)، یک حلقه‌ی ساده هر عنصرِ برداشته‌شده را `push` می‌کند — همان کار، با ابزاری که تا اینجا داری.

## `dedup_adjacent_titles` — `dedup_by_key`، نه `dedup_by`

```rust
entries.dedup_by_key(|entry| entry.title.clone());
```

چون معیارِ برابری یک **کلید مشتق‌شده** از عنصر است — فقط `title`، نه کلِ `WatchEntry` — `.dedup_by_key()` دقیقاً همین را می‌خواهد: یک کلوژر که کلید را برمی‌گرداند، نه یک مقایسه‌ی دستی. `.clone()` لازم است چون کلوژر باید یک `String` مالکانه برگرداند، نه یک ارجاع به فیلدی که همان لحظه ممکن است حذف شود. و چون `dedup_by_key` همیشه **اولینِ** هر رانِ همسایه را نگه می‌دارد، امتیاز و وضعیتِ `watched` هم از همان اولین ورودی می‌مانند — دقیقاً همان چیزی که تست چک می‌کند.

## `sorted_by_rating` — `sort_by` با `total_cmp`، نه `sort_unstable_by`

```rust
let mut entries = entries;
entries.sort_by(|a, b| a.rating.total_cmp(&b.rating));
entries
```

دو انتخاب اینجا عمدی بودند. اول، `f64: Ord` نیست، پس `.sort()`ِ ساده اصلاً کامپایل نمی‌شد؛ `total_cmp` یک `Ordering` واقعی می‌دهد، حتی برایِ `NaN`. دوم — و اینجا مهم‌تر است — مشخصات صراحتاً خواسته بود امتیازهای مساوی ترتیبِ نسبیِ اصلی‌شان را حفظ کنند. `.sort_by()` این را **تضمین** می‌کند؛ `.sort_unstable_by()` نه. همین یک کلمه («unstable») تستِ `sorted_by_rating_orders_ascending_and_keeps_ties_stable` را رد می‌کرد.

## `find_by_rating` — `binary_search_by` به‌علاوه‌ی `.ok()`

```rust
entries
    .binary_search_by(|entry| entry.rating.total_cmp(&target))
    .ok()
```

`binary_search_by` یک `Result<usize, usize>` می‌دهد: `Ok(index)` اگر پیدا شود، `Err(insert_at)` اگر نه. مشخصاتِ تابع فقط `Option<usize>` می‌خواست — همان جواب، بدونِ محلِ درج وقتی پیدا نشده. `.ok()` دقیقاً همین تبدیل را می‌کند: `Ok(x)` به `Some(x)`، و `Err(_)` به `None`، بدونِ اینکه به‌طورِ صریح بنویسی‌اش با یک `match`.

## این درس واقعاً درباره‌ی چه بود

- **متدهای امروز، جایگزینِ حلقه‌های دستی‌اند، نه ابزارِ جدید.** `.retain()`، `.drain()`، `.dedup_by_key()` — هرکدام همان کاری را می‌کنند که یک `for` با یک `if` می‌کرد، فقط با یک اسم، و با گارانتی‌هایی که خودت مجبور نیستی هربار دوباره درست بنویسی.
- **`.sort_by()` و `.sort_unstable_by()` جواب یکسان می‌دهند، مگر وقتی مساوی‌ها مهم‌اند.** آنجا که مهم‌اند — دقیقاً مثلِ `sorted_by_rating` — کلمه‌ی «stable» تنها چیزی است که بینِ درست و نادرست فاصله می‌اندازد.
- **`binary_search_by` یک قراردادِ خاموش دارد.** کد هیچ‌جا نمی‌نویسد «`entries` باید مرتب باشد» — این مسئولیتِ صداکننده است، همیشه، و امضا کمکی به یادآوریش نمی‌کند.
