# راه‌حل — ۱.۵.۵ `if let`، `while let`، `let else`

```rust
pub fn watching_line(entry: &Entry) -> String {
    if let Entry::Watching { episode } = entry {
        format!("on episode {episode}")
    } else {
        "not watching".to_string()
    }
}

pub fn ratings_only(entries: Vec<Entry>) -> Vec<u8> {
    let mut out = Vec::new();
    for entry in entries {
        if let Entry::Completed { rating } = entry {
            out.push(rating);
        }
    }
    out
}

pub fn pop_all(stack: Vec<i32>) -> Vec<i32> {
    let mut stack = stack;
    let mut out = Vec::with_capacity(stack.len());
    while let Some(value) = stack.pop() {
        out.push(value);
    }
    out
}

pub fn episode_gap(entry: &Entry, latest_episode: u32) -> u32 {
    let Entry::Watching { episode } = entry else {
        return 0;
    };
    latest_episode.saturating_sub(*episode)
}

pub fn pop_until_negative(stack: Vec<i32>) -> Vec<i32> {
    let mut stack = stack;
    let mut out = Vec::new();
    while let Some(value) = stack.pop() {
        if value < 0 {
            break;
        }
        out.push(value);
    }
    out
}
```

پنج تابع، چهار شکل. هیچ‌کدام‌شان به `match` احتیاج نداشت — ولی هر پنج تا را می‌شد با `match` نوشت، و همین است که سؤالِ جالب را می‌سازد.

## `watching_line` — همان `if let ... else`، بی هیچ حاشیه‌ای

```rust
if let Entry::Watching { episode } = entry {
    format!("on episode {episode}")
} else {
    "not watching".to_string()
}
```

دو حالت، هر دو کار دارند، و فقط یکی‌شان داده باز می‌کند. این دقیقاً همان جایی است که `if let ... else` از `match` بهتر است: یک بازو داده می‌خواهد و بقیه یک جوابِ مشترک دارند.

دقت کن که `entry` نوعش `&Entry` است و الگو `Entry::Watching { episode }` — بدونِ هیچ `&`ای. کامپایلر خودش می‌فهمد که داری به یک ارجاع نگاه می‌کنی و `episode` را هم به‌صورتِ ارجاع می‌بندد. برای `format!` مهم نیست، چون فقط می‌خواند.

نسخه‌ی `match` این می‌شود:

```rust
match entry {
    Entry::Watching { episode } => format!("on episode {episode}"),
    _ => "not watching".to_string(),
}
```

تقریباً هم‌اندازه است — و اگر آن `_` را برداری، کامپایلر مجبورت می‌کند هر سه واریانتِ دیگر را جدا بنویسی. این خودش نشانه است: **آن `_` همان چیزی است که `if let` به‌طورِ ضمنی می‌نویسد.** هر جا در نسخه‌ی `match` مجبور شدی `_` بنویسی، `if let` هم همان‌قدر امن است. هر جا نه، `if let` دارد چیزی را از تو پنهان می‌کند.

## `ratings_only` — `if let` بدونِ `else`، داخلِ حلقه

```rust
for entry in entries {
    if let Entry::Completed { rating } = entry {
        out.push(rating);
    }
}
```

اینجا واقعاً «بقیه‌ی حالت‌ها هیچ کاری ندارند» — پس `else` هم لازم نیست و نوشتنش فقط یک بلوکِ خالی می‌شد.

`entries` را مصرف می‌کنیم (`Vec<Entry>` نه `&Vec<Entry>`)، پس `rating` یک `u8` واقعی است و نه ارجاع؛ همان را مستقیم داخلِ خروجی می‌گذاریم. اگر امضا `&[Entry]` بود باید `*rating` می‌نوشتی — که همان تفاوتِ [۱.۳.۱](../../../03-borrowing-and-references/01-shared-and-mutable-refs/README.fa.md) است، نه یک قاعده‌ی تازه.

## `pop_all` — چرا `while let` و نه `for`

```rust
while let Some(value) = stack.pop() {
    out.push(value);
}
```

`for` اینجا کار نمی‌کند، چون `for` روی چیزی می‌چرخد که از قبل شکل دارد؛ اینجا هر دور از یک **تماس با تابع** می‌آید که ممکن است جواب ندهد. این تفاوتِ `for` و `while let` است: `for` روی یک مجموعه راه می‌رود، `while let` یک قدمِ ممکن‌الشکست را تکرار می‌کند.

`Vec::with_capacity(stack.len())` مجانی است و از رشدِ پله‌پله‌ی خروجی جلوگیری می‌کند، چون طولِ نهایی را از قبل می‌دانیم.

و `let mut stack = stack;` را ببین: `stack` را با مالکیت گرفته‌ایم ولی امضا `mut` ندارد. یک `let` تازه با همان اسم آن را تغییرپذیر می‌کند — همان سایه‌اندازی (shadowing) از [۱.۱.۱](../../../01-foundations/01-variables-mutability-shadowing/README.fa.md). می‌شد `mut stack: Vec<i32>` هم در خودِ امضا نوشت؛ فرقی در رفتار ندارد، ولی `mut` در امضا برای صداکننده هیچ معنایی ندارد و بهتر است داخل بماند.

## `episode_gap` — نگهبان روی یک الگو

```rust
let Entry::Watching { episode } = entry else {
    return 0;
};
latest_episode.saturating_sub(*episode)
```

این کوتاه‌ترین تابعِ این پنج تاست و درعین‌حال آن‌که بیشترین حرف را دارد. کارِ اصلیِ تابع یک خط است و در سطحِ اولِ تابع نشسته — نه داخلِ یک `if let`، نه داخلِ یک بازوی `match`.

سه نکته:

- `return 0;` واگرا می‌شود، پس بلوکِ `else` قانونی است. اگر به‌جایش `0` تنها می‌نوشتی، `E0308` می‌گرفتی: بلوک تمام می‌شد و به خطِ بعد می‌رسید.
- `episode` بعد از این خط در دسترس است. با `if let` نبود، و مجبور می‌شدی همه‌چیز را داخلِ بلوک ببری.
- `*episode` چون `entry` یک ارجاع است و در نتیجه `episode` هم `&u32` است. `saturating_sub` یک `u32` می‌خواهد.

و `saturating_sub` به‌جای `-` عمدی است: اگر کسی قسمتِ ۱۴ را می‌بیند و آخرین قسمتِ پخش‌شده ۱۲ است، تفریقِ ساده روی `u32` پنیک می‌کند. این همان `saturating_` از [۱.۱.۲](../../../01-foundations/02-scalar-types-and-overflow/README.fa.md) است، و مستندِ تابع صریحاً گفته بود «هرگز نمی‌پیچد».

## `pop_until_negative` — `while let` به‌علاوه‌ی یک `break`

```rust
while let Some(value) = stack.pop() {
    if value < 0 {
        break;
    }
    out.push(value);
}
```

دو دلیلِ متفاوت برای تمام شدنِ حلقه، و مهم است که با هم قاتی نشوند:

- **الگو نخورد** — یعنی وکتور خالی شد. این را `while let` خودش می‌فهمد.
- **`break`** — یعنی یک عددِ منفی درآمد. این تصمیمِ توست و ربطی به الگو ندارد.

`break` قبل از `push` می‌آید، چون خودِ عددِ منفی نباید در خروجی باشد. و صفر منفی نیست، پس `value < 0` درست است و `value <= 0` غلط — تستِ `vec![5, 0, 6]` دقیقاً همین را می‌گیرد.

## چیزی که این درس واقعاً درباره‌اش بود

- **`if let` یک `match` است که بازوی `_ => {}` را ننوشته.** اگر در نسخه‌ی `match` به یک `_` راضی بودی، `if let` هم امن است.
- **`while let` یک قدمِ ممکن‌الشکست را تکرار می‌کند**، و اولین شکست تمامش می‌کند.
- **`let ... else` اسم را در دامنه‌ی جاری می‌بندد**، و همین است که آن را برای نگهبان مناسب می‌کند.
- **بلوکِ `else` باید برود.** نوعش `!` است، همان نوعِ هرگز.
- **هر بار که از `match` پایین می‌آیی، جامعیت را جا می‌گذاری.** گاهی معامله‌ی خوبی است. همیشه باید آگاهانه باشد.
