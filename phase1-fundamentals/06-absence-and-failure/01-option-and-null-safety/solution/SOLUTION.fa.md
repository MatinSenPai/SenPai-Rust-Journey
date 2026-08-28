# راه‌حل — ۱.۶.۱ `Option` و ایمنی در برابرِ نبودن

```rust
pub fn index_of_first_negative(readings: [i32; 6]) -> Option<usize> {
    for position in 0..readings.len() {
        if readings[position] < 0 {
            return Some(position);
        }
    }
    None
}

pub fn safe_average(total: i32, count: u32) -> Option<f64> {
    if count == 0 {
        return None;
    }
    Some(total as f64 / count as f64)
}

pub fn nickname_len(nickname: &Option<String>) -> Option<usize> {
    match nickname.as_ref() {
        Some(name) => Some(name.len()),
        None => None,
    }
}

pub fn first_word_upper(words: &[String]) -> Option<String> {
    match words.first() {
        Some(word) => Some(word.to_uppercase()),
        None => None,
    }
}

pub struct Profile {
    pub nickname: Option<String>,
}

pub fn greeting(profile: &Profile) -> String {
    match &profile.nickname {
        Some(name) => format!("Hey, {name}!"),
        None => "Hey, stranger!".to_string(),
    }
}
```

هر پنج تابع فقط با `match` و `if let` نوشته شده‌اند — بدونِ `.map()`، `.and_then()`، `.unwrap_or()` یا `.ok_or()`. آن‌ها [۱.۶.۲](../../02-option-combinators/README.fa.md) هستند.

## `index_of_first_negative` — همان تابع، امضایِ درست

```rust
for position in 0..readings.len() {
    if readings[position] < 0 {
        return Some(position);
    }
}
None
```

بدنه‌ی تابع تقریباً کلمه‌به‌کلمه همان چیزی است که در [۱.۱.۵](../../../01-foundations/05-control-flow/README.fa.md) نوشتی — همان حلقه، همان `return` زودهنگام. تنها چیزی که عوض شده نوعِ برگشتی است: `readings.len()` جای خودش را به `None` داده، و `position` حالا داخلِ یک `Some` پیچیده شده.

فرقش کوچک به نظر می‌رسد تا وقتی صداکننده‌اش را بنویسی. با نسخه‌ی قدیمی، `readings[index_of_first_negative(r)]` کامپایل می‌شد — و اگر هیچ منفی‌ای نبود، با `readings[6]` پنیک می‌گرفت. با این نسخه، همان خط اصلاً کامپایل نمی‌شود؛ اول باید `match` یا `if let` بنویسی. باگ از یک چیزی که *ممکن است* پیش بیاید تبدیل شده به یک چیزی که *نمی‌تواند* رخ بدهد — کامپایلر بینِ تو و اشتباه ایستاده.

## `safe_average` — نگهبان، بعد تقسیم

```rust
if count == 0 {
    return None;
}
Some(total as f64 / count as f64)
```

یک نگهبان از [۱.۱.۵](../../../01-foundations/05-control-flow/README.fa.md): مورد استثنایی را اول رفعِ زحمت کن، بقیه‌ی تابع مجبور نیست به آن فکر کند. بعد از آن نگهبان، `count` قطعاً غیرصفر است، پس تقسیم امن است.

نکته‌ی ظریف: نگهبان **قبل از** تبدیلِ `as f64` است. اگر ترتیب را عوض کنی و اول تبدیل کنی، فرقی نمی‌کند — تقسیم بر صفرِ اعشاری در Rust پنیک نمی‌کند، `inf` یا `NaN` می‌دهد. ولی آن یک `f64` است، نه `None`ای که امضا قول داده؛ تست دقیقاً همین را می‌گیرد.

## `nickname_len` — قرض‌گرفتن، نه مالکیت

```rust
match nickname.as_ref() {
    Some(name) => Some(name.len()),
    None => None,
}
```

پارامتر از قبل یک ارجاع است (`&Option<String>`)، پس `nickname.as_ref()` یک `Option<&String>` می‌سازد که رویش تطبیق می‌دهی — بدونِ اینکه چیزی را از داخلِ آن ارجاع بیرون بکشی. اگر به‌جایش مستقیم `match *nickname { ... }` یا `match nickname { ... }` (روی خودِ ارجاع، بدونِ `as_ref`) می‌نوشتی هم کار می‌کرد، چون تطبیقِ الگو رویِ یک ارجاع خودکار قرض می‌گیرد — ولی `.as_ref()` همان قصد را با نامِ صریح می‌نویسد، و همان اصطلاحی است که بقیه‌ی درس و اکوسیستمِ Rust به کار می‌برد.

تست دو بار پشتِ سرِ هم صدایش می‌زند تا مطمئن شود چیزی حرکت نکرده:

```rust
assert_eq!(nickname_len(&missing), None);
assert_eq!(nickname_len(&missing), None);
```

اگر امضا `nickname: Option<String>` بود (بدونِ `&`)، صدای دوم اصلاً کامپایل نمی‌شد — دقیقاً همان `E0382`ای که در «خطاهایی که خواهی دید» دیدی.

## `first_word_upper` — `Option<&T>` که خودش قبلاً یک نگاه بود

```rust
match words.first() {
    Some(word) => Some(word.to_uppercase()),
    None => None,
}
```

`.first()` از قبل `Option<&String>` می‌دهد — نیازی به `.as_ref()` نیست چون هیچ‌وقت مالکِ `words` نبودی. `word` داخلِ بازوی `Some` یک `&String` است؛ `.to_uppercase()` رویِ آن یک `String`ِ نو و مالک می‌سازد، پس چیزی که برمی‌گردانی از پارامترِ ورودی مستقل است.

## `greeting` — `Option` در یک فیلد، دقیقاً همان دو رشته

```rust
match &profile.nickname {
    Some(name) => format!("Hey, {name}!"),
    None => "Hey, stranger!".to_string(),
}
```

`&profile.nickname` رویِ فیلد قرض می‌گیرد نه اینکه از `Profile` بیرونش بکشد — همان الگویِ `nickname_len`، این‌بار با استفاده از قرض‌گیریِ خودکارِ تطبیقِ الگو به‌جایِ `.as_ref()` صریح. هر دو راه یک نتیجه می‌دهند؛ کدام را بنویسی سلیقه است، تا وقتی که یادت باشد چرا کار می‌کند.

فرمت دقیقاً همانی است که مستنداتِ تابع قول داد: `"Hey, {nickname}!"` یا `"Hey, stranger!"` — نه شبیه‌اش، دقیقاً همان.

## این درس واقعاً درباره‌ی چه بود

- **نوعِ برگشتیِ متفاوت یعنی سوءاستفاده‌ی تصادفی ممکن نیست.** `index_of_first_negative` را با `Option<usize>` بنویس، و کامپایلر خودش صداکننده را مجبور می‌کند اول با «پیدا نشد» روبه‌رو شود.
- **`match` روی یک ارجاع، خودکار قرض می‌گیرد؛ `.as_ref()` همان کار را با اسمِ صریح می‌کند.** هر دو از حرکت دادنِ یک `Option`ِ که فقط می‌خواستی نگاهش کنی جلوگیری می‌کنند.
- **`Option<&T>`ی که `.first()` می‌دهد از قبل قرض‌شده است** — نیازی به `.as_ref()` نیست وقتی از اول مالک نبودی.
- **`.unwrap()`/`.expect()` در هیچ‌کدام از این پنج تابع لازم نبود** — چون هیچ‌کدام مجبور نبودند فرض کنند چیزی قطعاً `Some` است؛ همیشه هر دو حالت را جواب دادند.
- **`match` و `if let` برای هر مسئله‌ای که «یک مقدار یا هیچ» است کافی‌اند.** ترکیب‌گرها ([۱.۶.۲](../../02-option-combinators/README.fa.md)) همین‌ها را کوتاه‌تر می‌نویسند، نه کارِ متفاوتی می‌کنند.
