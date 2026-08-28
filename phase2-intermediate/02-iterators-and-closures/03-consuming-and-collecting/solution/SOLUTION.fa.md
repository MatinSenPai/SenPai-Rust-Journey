# راه‌حل — ۲.۲.۳ مصرف و جمع‌آوری، از جمله `Result<Vec<_>, E>`

```rust
pub fn doubled_long_runs(shows: &[u32], min_episodes: u32) -> Vec<u32> {
    shows
        .iter()
        .filter(|&&count| count >= min_episodes)
        .map(|count| count * 2)
        .collect()
}

pub fn initials(titles: &[&str]) -> String {
    titles.iter().filter_map(|title| title.chars().next()).collect()
}

pub fn episode_lookup(entries: &[(String, u32)]) -> HashMap<String, u32> {
    entries.iter().cloned().collect()
}

pub fn all_genres(shows: &[Vec<String>]) -> HashSet<String> {
    shows.iter().flatten().cloned().collect()
}

pub fn parse_all(inputs: &[&str]) -> Result<Vec<i32>, String> {
    inputs
        .iter()
        .map(|s| s.parse::<i32>().map_err(|e| e.to_string()))
        .collect()
}
```

هر پنج تابع همان چیزی است که کلِ درس نشانت داد: یک زنجیره‌ی آداپتور، و در انتها یک `.collect()`. هیچ‌کدام حلقه‌ی `for` ندارد.

## `doubled_long_runs` — فیلتر، بعد مپ، بعد جمع در `Vec`

```rust
shows
    .iter()
    .filter(|&&count| count >= min_episodes)
    .map(|count| count * 2)
    .collect()
```

`.filter()` همان چیزی است که از [۲.۲.۲](../../02-iterator-adapters/README.fa.md) می‌شناسی: فقط شماره‌هایی که `>= min_episodes` هستند رد می‌شوند. `.map()` هرکدام از آن‌ها را دوبرابر می‌کند. `.collect()` نوعِ مقصد را از رویِ نوعِ بازگشتیِ تابع — `Vec<u32>` — استنتاج می‌کند؛ اینجا نه توربوفیش لازم بود نه نوع‌نویسیِ جداگانه، چون امضایِ خودِ تابع همان اطلاعات را از قبل به Rust داده.

## `initials` — `filter_map` برایِ رد کردنِ رشته‌هایِ خالی، جمع در `String`

```rust
titles.iter().filter_map(|title| title.chars().next()).collect()
```

`title.chars().next()` یک `Option<char>` می‌دهد — `None` فقط وقتی `title` خالی باشد. `.filter_map()` دقیقاً همان کاری را می‌کند که اسمش می‌گوید: هر `Some(c)` را به `c` باز می‌کند و نگه می‌دارد، هر `None` را کامل حذف می‌کند — یک `.map()` و یک `.filter()` در یک قدم. نتیجه یک ایتریتور از `char` است، و چون نوعِ بازگشتیِ تابع `String` است، `.collect()` آن‌ها را می‌چسباند، نه به یک `Vec<char>`.

## `episode_lookup` — تبدیل به مالک، بعد جمع در `HashMap`

```rust
entries.iter().cloned().collect()
```

`entries` یک `&[(String, u32)]` است؛ `.iter()` رویش `&(String, u32)` می‌دهد. `HashMap<String, u32>` برایِ ساختنش به تاپل‌هایِ مالک نیاز دارد، نه ارجاع — پس `.cloned()` هر `&(String, u32)` را به یک `(String, u32)` کاملِ مالک تبدیل می‌کند (چون `String` و `u32` هردو `Clone`اند). `.collect()` باقی را انجام می‌دهد: هر تاپل یک جفتِ کلید-مقدار می‌شود، و اگر عنوانی تکرار شود، همان رفتارِ «آخرین برنده است» را می‌گیری که در بخشِ «مفهوم» دیدی — چون `.collect()` رویِ `HashMap` دقیقاً مثلِ یک حلقه‌ی `.insert()` پیاده شده.

## `all_genres` — `flatten` برایِ صاف‌کردنِ یک لیستِ توی لیست، جمع در `HashSet`

```rust
shows.iter().flatten().cloned().collect()
```

`shows` یک `&[Vec<String>]` است — یک لیست از لیست‌ها. `.iter()` رویش `&Vec<String>` می‌دهد؛ `.flatten()` هرکدام از آن زیرلیست‌ها را باز می‌کند و همه‌ی `&String`هایشان را در یک ایتریتورِ تخت پشتِ سرِ هم می‌گذارد — بدونِ اینکه لازم باشد دستی یک حلقه‌ی توی‌حلقه بنویسی. `.cloned()` هرکدام را مالک می‌کند، و `.collect()` — چون نوعِ بازگشتی `HashSet<String>` است — تکراری‌ها را همان‌طور که در بخشِ «مفهوم» دیدی خودکار حذف می‌کند.

## `parse_all` — همان `Result<Vec<T>, E>`، این‌بار خودت نوشتیش

```rust
inputs
    .iter()
    .map(|s| s.parse::<i32>().map_err(|e| e.to_string()))
    .collect()
```

`s.parse::<i32>()` یک `Result<i32, ParseIntError>` می‌دهد. `.map_err(|e| e.to_string())` — همان ترکیب‌گری که در [۱.۶.۳](../../../../phase1-fundamentals/06-absence-and-failure/03-result-and-question-mark/README.fa.md) دیدی — فقط طرفِ خطا را عوض می‌کند، از `ParseIntError` به `String`، بدونِ دست‌زدن به طرفِ `Ok`. پس ایتریتوری که به `.collect()` می‌رسد، آیتم‌هایش `Result<i32, String>`اند — دقیقاً همان چیزی که نوعِ بازگشتیِ تابع، `Result<Vec<i32>, String>`، می‌خواهد. بقیه‌اش همان مکانیزمِ مرکزِ ثقلِ درس است: اولین `Err` توقفِ زودهنگام می‌سازد؛ اگر همه `Ok` بودند، یک `Vec` از همه‌یِ اعدادِ باز‌شده می‌گیری.

این را با نسخه‌ی [۱.۶.۳](../../../../phase1-fundamentals/06-absence-and-failure/03-result-and-question-mark/README.fa.md) مقایسه کن — همان امضا، همان مشخصات، ولی آن‌جا یک حلقه‌ی صریح با `?` بود و اینجا سه متد زنجیر شده.

## این درس واقعاً درباره‌ی چه بود

- `.collect()` مصرف‌کننده‌ی همه‌کاره است؛ نوعِ بازگشتیِ تابع (یا یک نوع‌نویسی، یا یک توربوفیش) به‌اش می‌گوید دقیقاً چه بسازد — گاهی حتی بدونِ اینکه لازم باشد صریح بنویسیش، اگر امضایِ تابع از قبل کافی باشد.
- `.filter_map()` یک `.map()` و یک `.filter()` را در یک قدم انجام می‌دهد، وقتی خودِ تبدیل، `Option` برمی‌گرداند.
- `.flatten()` یک لیست‌توی‌لیست را به یک ایتریتورِ تخت تبدیل می‌کند، بدونِ حلقه‌یِ توی‌حلقه‌یِ دستی.
- `.cloned()` قبل از `.collect()` وقتی لازم است که مقصد به مقدارهایِ مالک نیاز دارد ولی خودت فقط ارجاع داری.
- `Result<Vec<T>, E>` از `.collect()` دقیقاً همان حلقه‌ی دستی + `?` است که در فازِ ۱ نوشتی — فقط این‌بار خودِ `.collect()` آن حلقه را برایت می‌نویسد.
