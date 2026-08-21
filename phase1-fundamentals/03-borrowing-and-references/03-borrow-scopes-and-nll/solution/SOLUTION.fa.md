# راه‌حل — ۱.۳.۳ دامنه‌ی قرض و NLL

```rust
pub fn with_first_repeated(values: Vec<i32>) -> Vec<i32> {
    let mut values = values;
    if values.is_empty() {
        return values;
    }
    values.push(values[0]);
    values
}

pub fn with_total_appended(values: Vec<i32>) -> Vec<i32> {
    let mut values = values;
    let mut total = 0;
    for value in &values {
        total += value;
    }
    values.push(total);
    values
}

pub fn with_longest_repeated(names: Vec<String>) -> Vec<String> {
    let mut names = names;
    if names.is_empty() {
        return names;
    }
    let mut longest = 0;
    for index in 1..names.len() {
        if names[index].len() > names[longest].len() {
            longest = index;
        }
    }
    names.push(names[longest].clone());
    names
}

pub fn with_length_appended(text: String) -> String {
    let mut text = text;
    text.push_str(&text.len().to_string());
    text
}

pub fn doubled_then_totalled(values: Vec<i32>) -> Vec<i32> {
    let mut values = values;
    for value in &mut values {
        *value *= 2;
    }
    let mut total = 0;
    for value in &values {
        total += value;
    }
    values.push(total);
    values
}
```

در هیچ‌کدام یک آکولادِ اضافه نیست. اگر تو گذاشتی، اشتباه نیست — ولی هیچ‌کدامشان لازم نبود، و همین حرفِ اصلیِ درس است.

## `with_first_repeated` — یک خط، به‌لطفِ قرضِ دومرحله‌ای

```rust
values.push(values[0]);
```

این خط `Vec::push(&mut values, values[0])` است: یک ارجاعِ تغییرپذیر و یک خواندن از همان وکتور، در یک دستور. کامپایل می‌شود چون آن `&mut` را کامپایلر ساخته، پس اول رزرو است و تا لحظه‌ی خودِ فراخوانی مثلِ یک ارجاعِ اشتراکی رفتار می‌کند.

نسخه‌ی دوخطی هم کاملاً درست است و به هیچ لطفی احتیاج ندارد:

```rust
let first = values[0];
values.push(first);
```

`i32` نوعِ `Copy` است، پس `first` یک کپی است نه یک ارجاع، و هیچ قرضی در کار نیست. اگر تازه‌کار هستی این نسخه را بنویس؛ چیزی که باید بدانی این است که **چرا** نسخه‌ی یک‌خطی هم مجاز است.

نگهبانِ `is_empty` لازم است چون `values[0]` روی وکتورِ خالی پنیک می‌کند. این یک خطای قرض نیست، خطای اندیس است — و دو چیزِ متفاوت‌اند.

## `with_total_appended` — قرضِ حلقه سرِ حلقه تمام می‌شود

```rust
for value in &values {
    total += value;
}
values.push(total);
```

`&values` یک ارجاعِ اشتراکی است که تا آخرین دورِ حلقه زنده می‌ماند. `values.push(total)` یک `&mut` می‌خواهد. این دو با هم جور درنمی‌آیند — ولی با هم نیستند: آخرین استفاده‌ی آن ارجاعِ اشتراکی داخلِ حلقه است، پس دامنه‌اش سرِ آکولادِ حلقه تمام می‌شود و `push` بعد از آن است.

هیچ بلوکِ اضافه‌ای دورِ `for` لازم نیست. اگر گذاشتی، خروجی همان است و چیزی هم عوض نشده — دقیقاً همان چیزی که `examples/03-ending-a-borrow-early.rs` نشان می‌داد.

حالا امتحان کن: `push` را ببر **داخلِ** حلقه. آن‌وقت ارجاع در دورِ بعد هنوز لازم است و `E0502` می‌گیری — و این بار برچسبِ سوم روی خودِ خطِ `for` می‌نشیند، چون همان‌جاست که ارجاع دوباره استفاده می‌شود.

## `with_longest_repeated` — اندیس نگه دار، نه ارجاع

این آنی است که غریزه‌ی اول را رد می‌کند. غریزه این است:

```rust
let longest = &names[0];      // یک &String
names.push(longest.clone());
```

و جالب اینکه این هم کامپایل می‌شود — باز هم قرضِ دومرحله‌ای، چون `longest.clone()` آخرین استفاده‌ی آن ارجاع است و قبل از فعال شدنِ `&mut` تمام می‌شود. ولی یک خطِ دیگر اضافه کن که بعد از `push` از `longest` استفاده کند و بلافاصله `E0502` می‌گیری.

نسخه‌ی راه‌حل این ریسک را اصلاً برنمی‌دارد:

```rust
let mut longest = 0;
for index in 1..names.len() {
    if names[index].len() > names[longest].len() {
        longest = index;
    }
}
```

`longest` یک `usize` است. یک عدد. هیچ چیزی را قرض نگرفته، پس هیچ چیزی هم نمی‌تواند نگهش دارد. حلقه هر بار `names` را برای خواندن قرض می‌گیرد و در همان خط تمامش می‌کند.

بعد:

```rust
names.push(names[longest].clone());
```

`.clone()` لازم است، و اگر برداری‌اش خطایی می‌گیری که خطای قرض **نیست**:

```text
error[E0507]: cannot move out of index of `Vec<String>`
```

آن `E0507` از [۱.۲.۲](../../../02-ownership-and-memory/02-move-semantics/README.fa.md) است: نمی‌توانی چیزی را از وسطِ یک وکتور بیرون بکشی و وکتور را سالم بگذاری. تشخیصِ اینکه چه وقت با یک خطای مالکیت طرفی و چه وقت با یک خطای قرض، خودش یک مهارت است.

مساوی‌ها: `>` می‌نویسیم و نه `>=`، پس اولین رشته‌ی بلند برنده می‌ماند. مشخصات همین را می‌خواست.

## `with_length_appended` — همان الگو، روی `String`

```rust
text.push_str(&text.len().to_string());
```

باز هم `String::push_str(&mut text, ...)` که آرگومانش `text` را می‌خواند. باز هم رزرو، باز هم مجاز.

و طولی که نوشته می‌شود طولِ **قبل** از اضافه شدن است، چون آرگومان قبل از فعال شدنِ `&mut` محاسبه می‌شود. مشخصات این را صریح گفته بود؛ اگر نگفته بود، این کد دو جوابِ ممکن داشت و آن یک نقصِ مشخصات بود، نه یک چالش.

## `doubled_then_totalled` — سه قرض، پشتِ سرِ هم

```rust
for value in &mut values {
    *value *= 2;
}
let mut total = 0;
for value in &values {
    total += value;
}
values.push(total);
```

سه قرضِ متفاوت از یک وکتور، در یک تابع:

| قرض | از کجا تا کجا |
|---|---|
| `&mut values` | داخلِ حلقه‌ی اول |
| `&values` | داخلِ حلقه‌ی دوم |
| `&mut values` (خودکارِ `push`) | فقط همان یک خط |

هیچ‌کدام با هم برخورد نمی‌کنند، چون هیچ نقطه‌ای نیست که داخلِ دو تایشان باشد. این تصویرِ کاملِ چیزی است که NLL به تو می‌دهد: در مدلِ لغوی هر سه تا آخرِ تابع زنده بودند و این تابع کامپایل نمی‌شد.

`*value *= 2` به یک `*` احتیاج دارد چون `value` یک `&mut i32` است و باید از فلش رد شوی تا به عدد برسی — همان `*` از [۱.۳.۱](../../../03-borrowing-and-references/01-shared-and-mutable-refs/README.fa.md).

## این درس واقعاً درباره‌ی چه بود

- **دامنه‌ی یک قرض از ساخته شدنش تا آخرین استفاده‌اش است**، نه تا آکولادِ بسته.
- **برچسبِ سومِ خطا خطِ پایان را نام می‌برد.** اول آن را بخوان، بعد کد را عوض کن.
- **«بعداً» یعنی بعداً در اجرا.** یک حلقه می‌تواند خطی بالاتر را استفاده‌ی بعدی کند.
- **آکولاد آخرین ابزار است، نه اولی.** اول ببین می‌شود استفاده را جابه‌جا کرد یا اصلاً قرض نگرفت.
- **قرضِ دومرحله‌ای فقط برای `&mut`ِ خودکارِ متدهاست.** به‌همین‌خاطر `v.push(v.len())` کامپایل می‌شود و نسخه‌ی نام‌گذاری‌شده‌اش نه.
- و وقتی به [برش‌ها](../../04-slices/README.fa.md) برسی، همه‌ی این‌ها دوباره برمی‌گردند — یک برش هم یک قرض است، فقط از بخشی از یک مجموعه.
