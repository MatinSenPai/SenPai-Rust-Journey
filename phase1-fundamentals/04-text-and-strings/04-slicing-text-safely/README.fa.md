# ۱.۴.۴ — برشِ امنِ متن

## در یک نگاه

بعد از این درس می‌توانی:

- به یک `&text[a..b]` نگاه کنی و *قبل از اجرا* بگویی پنیک می‌کند یا نه.
- پیامِ `end byte index N is not a char boundary` را بخوانی و بگویی دقیقاً کدام حرف را نصف کرده‌ای.
- یک `truncate_to_chars` بنویسی که روی هیچ ورودی‌ای، به هیچ زبانی، پنیک نکند.
- بین `[..]`، `.get(..)` و `floor_char_boundary` برای یک کارِ مشخص انتخاب کنی.

**زمان:** حدود ۶۰ دقیقه · **پیش‌نیاز:**
[۱.۴.۲ — یوتی‌اف-۸: بایت، کاراکتر، گرافیم](../02-utf8-bytes-chars-graphemes/README.fa.md) ·
[۱.۴.۳ — ساختن و دگرگون کردنِ رشته‌ها](../03-building-and-transforming-strings/README.fa.md)

---

## چرا اهمیت دارد

این درس، نتیجه‌گیریِ کلِ ماژولِ متن است و به‌احتمالِ زیاد کاربردی‌ترین درسِ کلِ فاز ۱ برای تو.

سناریو را می‌شناسی. یک تیکت می‌آید: «عنوانِ کارت حداکثر ۲۰ کاراکتر باشد، بقیه‌اش با سه‌نقطه.» کسی می‌نویسد `&title[..20]`. تست‌ها با `"Fullmetal Alchemist: Brotherhood"` سبز می‌شوند. کد merge می‌شود. یک هفته بعد، سرویس روی یک عنوانِ فارسی می‌ترکد — نه یک بار، بلکه روی تقریباً نیمی از عنوان‌های فارسی — و لاگ می‌گوید:

```text
end byte index 20 is not a char boundary; it is inside 'ی' (bytes 19..21 of string)
```

این باگ برای یک کدبیسِ انگلیسی‌زبان عملاً نامرئی است، چون در ASCII شمارشِ بایت و شمارشِ حرف یک عددند. برای تو نامرئی نیست: تو کاربرِ فارسی داری و در فارسی این دو عدد هرگز یکی نیستند.

خبرِ خوب این است که Rust این باگ را ساکت رد نمی‌کند. به‌جای اینکه نیمِ یک حرف را به تو بدهد و بگذارد کاربرت `Ø±` ببیند، برنامه را متوقف می‌کند. این سخت‌گیری آزاردهنده است تا روزی که بفهمی جایگزینش داده‌ی خراب در پایگاه داده است.

---

## مفهوم

### `&text[a..b]` بایت می‌شمارد، نه حرف

نحوِ بازه را از برش‌ها می‌شناسی: `&v[1..4]`. روی متن هم همان نحو کار می‌کند — ولی آن دو عدد **اندیسِ بایت** هستند.

```rust
let english = "programming";
println!("{:?}", &english[0..7]);
println!("{} chars, {} bytes", english.chars().count(), english.len());
```

```text
"program"
11 chars, 11 bytes
```

هفت بایت شد هفت حرف. دو عدد یکی درآمدند، و همین است که این باگ را از review رد می‌کند: در انگلیسی هیچ‌وقت مجبور نمی‌شوی بینشان فرق بگذاری.

### همان برش، روی فارسی

```rust
let persian = "برنامه‌نویسی";
println!("{} chars, {} bytes", persian.chars().count(), persian.len());
println!("persian: {:?}", &persian[0..7]);
```

```text
12 chars, 25 bytes

thread 'main' (15184) panicked at phase1-fundamentals\04-text-and-strings\04-slicing-text-safely\examples\04-cut-in-half.rs:15:39:
end byte index 7 is not a char boundary; it is inside 'ا' (bytes 6..8 of string)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

دوازده حرف، بیست‌وپنج بایت. و بایتِ شماره‌ی ۷ اصلاً جای مجازی برای بریدن نیست.

(عددِ داخلِ پرانتز بعد از `main` شناسه‌ی ریسه است و هر بار فرق می‌کند؛ بقیه‌ی پیام همیشه همین است.)

### پنیک را خط به خط بخوان

پیام سه چیز به تو می‌گوید و هر سه به‌دردبخورند:

| تکه‌ی پیام | یعنی چه |
|---|---|
| `end byte index 7` | انتهای بازه‌ات، بایتِ شماره‌ی ۷ |
| `is not a char boundary` | آنجا مرزِ یک حرف نیست |
| `it is inside 'ا' (bytes 6..8 of string)` | وسطِ این حرف است، که بایت‌های ۶ تا ۸ را گرفته |

یعنی کامپایلر حتی حرفِ قربانی را هم به تو نشان می‌دهد. نقشه‌ی بایتیِ اولِ همان رشته این است:

```text
byte index:  0    1    2    3    4    5    6    7    8
             +---------+---------+---------+---------+
             | letter1 | letter2 | letter3 | letter4 |
             +---------+---------+---------+---------+
                                                ^
                            byte 7 sits inside letter 4
```

هر حرفِ فارسی دو بایت گرفته، پس مرزهای مجاز فقط عددهای زوج‌اند: ۰، ۲، ۴، ۶، ۸. عددهای فرد همه وسطِ یک حرف‌اند.

**قاعده‌ای که باید حفظ کنی:** برشِ رشته با `[a..b]` مجاز است فقط وقتی هم `a` و هم `b` مرزِ حرف باشند. وگرنه پنیک — نه خروجیِ خراب، نه هشدار. توقف.

### `is_char_boundary` — پرسیدن به‌جای حدس زدن

قبل از اینکه ببری، می‌توانی بپرسی:

```rust
let persian = "برنامه‌نویسی";
for index in 0..=8 {
    println!("{index}: {}", persian.is_char_boundary(index));
}
```

```text
0: true
1: false
2: true
3: false
4: true
5: false
6: true
7: false
8: true
```

`is_char_boundary` هیچ‌وقت پنیک نمی‌کند و برای اندیسِ بزرگ‌ترِ از طول هم فقط `false` می‌دهد. دو حقیقتِ همیشگی: `0` همیشه مرز است و `text.len()` هم همیشه مرز است — حتی برای رشته‌ی خالی.

### `char_indices` — فهرستِ مرزهای واقعی

به‌جای حدس زدن، فهرست را بگیر. `char_indices` که در [۱.۴.۲](../02-utf8-bytes-chars-graphemes/README.fa.md) دیدی دقیقاً همین را می‌دهد: اندیسِ بایتی که هر حرف از آن شروع می‌شود.

```rust
for (index, character) in "می‌روم".char_indices() {
    println!("{index:>3} {character:?}  {} bytes", character.len_utf8());
}
```

```text
  0 'م'  2 bytes
  2 'ی'  2 bytes
  4 '\u{200c}'  3 bytes
  7 'ر'  2 bytes
  9 'و'  2 bytes
 11 'م'  2 bytes
```

آن `'\u{200c}'` نیم‌فاصله است — همان نویسه‌ای که در «می‌روم» بین «می» و «روم» می‌گذاری. نامرئی است، **سه** بایت می‌گیرد در حالی که بقیه‌ی حروف دو بایت می‌گیرند، و دقیقاً به همین دلیل حسابِ بایتیِ فارسی حتی «همیشه ضربدر دو» هم نیست.

### `.get(a..b)` — برشی که به‌جای مردن، جواب می‌دهد

`.get()` همان برش را می‌زند، ولی وقتی برش غیرمجاز باشد برنامه را نمی‌کشد:

```rust
let persian = "برنامه‌نویسی";
println!("{:?}", persian.get(0..7));
println!("{:?}", persian.get(0..6));
println!("{:?}", persian.get(0..7).unwrap_or(""));
```

```text
None
Some("برن")
""
```

آن `Some`/`None` یک `Option<&str>` است: نوعی که می‌گوید «شاید یک `&str` اینجا باشد، شاید هم نباشد». همان `.get()`ای است که در [۱.۱.۳](../../01-foundations/03-compound-types-and-destructuring/README.fa.md) روی آرایه دیدی، با همان قرارداد.

برای امروز دو کارِ ساده با آن کافی است. اولی `unwrap_or` است: «اگر بود بده، اگر نبود این را بده». دومی `if let`:

```rust
if let Some(piece) = persian.get(0..6) {
    println!("cut: {piece:?}");
} else {
    println!("byte 6 is not a legal cut");
}
```

```text
cut: "برن"
```

داستانِ کاملِ `Option` در [۱.۶.۱](../../06-absence-and-failure/01-option-and-null-safety/README.fa.md) است و `if let` هم درسِ خودش را در [ماژولِ ۱.۵](../../05-your-own-types/README.fa.md) دارد. اینجا فقط همین‌قدر لازم داریم.

> `&text[a..b]` و `text.get(a..b)` یک کار می‌کنند. تفاوتشان این است که اولی روی ورودیِ بد برنامه را می‌کشد و دومی جواب می‌دهد. انتخاب بینشان انتخاب بینِ «این ورودی هرگز نباید بد باشد» و «این ورودی ممکن است بد باشد» است.

### `floor_char_boundary` و `ceil_char_boundary` — چسبیدن به نزدیک‌ترین مرز

گاهی نمی‌خواهی برشِ بد را رد کنی؛ می‌خواهی به نزدیک‌ترین برشِ *مجاز* بچسبی. مثلاً «حداکثر ۲۰ بایت» برای یک ستونِ پایگاه داده.

```rust
let text = "می‌روم";
println!("{} {}", text.floor_char_boundary(5), text.ceil_char_boundary(5));
println!("{:?}", &text[..text.floor_char_boundary(5)]);
```

```text
4 7
"می"
```

`floor_char_boundary` بزرگ‌ترین مرزِ کوچک‌تر-یا-مساویِ عددِ توست و `ceil_char_boundary` کوچک‌ترین مرزِ بزرگ‌تر-یا-مساوی. بایتِ ۵ وسطِ نیم‌فاصله (بایت‌های ۴ تا ۷) است، پس یکی می‌رود عقب به ۴ و آن یکی جلو به ۷. هیچ‌کدام پنیک نمی‌کنند و هر دو برای عددِ بزرگ‌تر از طول، به `text.len()` بسنده می‌کنند.

این دو از نسخه‌ی **۱.۹۱** پایدارند و ابزارِ روی این مخزن (rustc 1.97.0) داردشان. اگر روی کامپایلرِ قدیمی‌تری گیر کرده‌ای، معادلِ دستی‌اش سه خط است:

```rust
let mut cut = 5;
while !text.is_char_boundary(cut) {
    cut -= 1;
}
println!("{cut} -> {:?}", &text[..cut]);
```

```text
4 -> "می"
```

این حلقه همیشه تمام می‌شود، چون بایتِ صفر همیشه مرز است. بدترین حالت سه قدم عقب رفتن است، چون هیچ حرفی در UTF-8 بیشتر از چهار بایت نیست.

### `split_at` هم دقیقاً همین‌طور می‌ترکد

```rust
let salam = "سلام";
let (head, tail) = salam.split_at(3);
```

```text
thread 'main' (28348) panicked at C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\str\mod.rs:846:21:
end byte index 3 is not a char boundary; it is inside 'ل' (bytes 2..4 of string)
```

به مسیرِ فایل نگاه کن: پنیک به **درونِ کتابخانه‌ی استاندارد** اشاره می‌کند، نه به خطِ کدِ تو. یعنی روی یک پروژه‌ی بزرگ باید خودت بگردی که کدام `split_at` بوده. نسخه‌ی امنش همان الگوی `.get()` را دارد:

```rust
println!("{:?}", salam.split_at_checked(3));
println!("{:?}", salam.split_at_checked(4));
```

```text
None
Some(("سل", "ام"))
```

### «حداکثر N حرف»، درست انجام‌شده

حالا ابزار کامل است. برای بریدن به تعدادِ **حرف**، اندیسِ بایتیِ حرفِ شماره‌ی N را لازم داری — و `char_indices` همان را می‌دهد:

```rust
fn truncate_to_chars(text: &str, max_chars: usize) -> &str {
    let mut seen = 0;
    for (index, _) in text.char_indices() {
        if seen == max_chars {
            return &text[..index];
        }
        seen += 1;
    }
    text
}
```

```text
truncate_to_chars("برنامه\u{200c}نویسی سیستمی", 8) -> "برنامه\u{200c}ن"
truncate_to_chars("Rust systems", 8)              -> "Rust sys"
truncate_to_chars("سلام", 8)                      -> "سلام"
```

سه نکته در نه خط:

۱. `index`ای که برمی‌گردانیم از خودِ `char_indices` آمده، پس **حتماً** مرز است. برش هرگز پنیک نمی‌کند.
۲. اگر حلقه تا آخر برود یعنی متن از `max_chars` کوتاه‌تر بوده، و کلِ `text` را برمی‌گردانیم. بدونِ نیاز به شمردنِ اولیه.
۳. برای `max_chars == 0` همان دورِ اول برمی‌گردد و `""` می‌دهد؛ برای متنِ خالی حلقه اصلاً اجرا نمی‌شود. هر دو حالتِ لبه، رایگان.

خروجی `&str` است نه `String` — هیچ تخصیصی در کار نیست. این تابع فقط یک پنجره‌ی کوچک‌تر به همان بافر می‌دهد.

### ...و سه‌نقطه، فقط اگر واقعاً بریدی

سه‌نقطه گذاشتن سرِ متنی که اصلاً بریده نشده، باگِ کوچکی است که همه‌جا هست:

```rust
fn with_ellipsis(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    format!("{}…", truncate_to_chars(text, max_chars))
}
```

```text
"برنامه\u{200c}ن…"
"سلام"
""
```

شرط روی `chars().count()` است نه `len()`. اگر `len()` بگذاری، «سلام» با چهار حرف و هشت بایت در برابرِ حدِ ۵ ناگهان «بریده‌شده» حساب می‌شود و یک سه‌نقطه‌ی دروغ می‌گیرد.

و آن `…` یک نویسه است (U+2026)، نه سه تا نقطه. سه نقطه سه حرف است و بودجه‌ات را سه برابر خرج می‌کند.

### چرا «حداکثر ۲۰ کاراکتر» یک باگِ واقعیِ production است

حالا نسخه‌ی غلط را روی داده‌ی واقعی ببین. این `card_title` است، همان‌طور که واقعاً نوشته می‌شود:

```rust
fn card_title(title: &str) -> &str {
    if title.len() <= 20 {
        return title;
    }
    &title[..20]
}
```

```text
32 chars in, 20 chars out -> "Fullmetal Alchemist:"
14 chars in, 11 chars out -> "حمله به تای"

thread 'main' (2676) panicked at phase1-fundamentals\04-text-and-strings\04-slicing-text-safely\examples\05-max-twenty-characters.rs:13:11:
end byte index 20 is not a char boundary; it is inside 'ی' (bytes 19..21 of string)
```

سه ردیف، سه رفتارِ متفاوت:

| عنوان | حرف | بایت | `&title[..20]` |
|---|---|---|---|
| `Fullmetal Alchemist: Brotherhood` | ۳۲ | ۳۲ | ۲۰ حرف — درست |
| `حمله به تایتان` | ۱۴ | ۲۶ | ۱۱ حرف — **بی‌صدا غلط** |
| `برنامه‌نویسی سیستمی با Rust` | ۲۷ | ۴۸ | **پنیک** |
| `شکارچی شیاطین` | ۱۳ | ۲۵ | پنیک (هرگز نمی‌رسد به آن) |

ردیفِ دوم بدتر از ردیفِ سوم است. ردیفِ سوم سر و صدا می‌کند؛ ردیفِ دوم فقط یازده حرف نشان می‌دهد جایی که قرار بود بیست حرف باشد، و هیچ‌کس تا وقتی کاربر شکایت نکند خبردار نمی‌شود.

و ریاضی‌اش ساده است: حروفِ فارسی در UTF-8 دو بایت‌اند. پس مرزهای مجاز روی یک متنِ تماماً فارسی، بایت‌های زوج‌اند و یک عددِ ثابتِ فرد مثل ۱۹ یا ۲۱ همیشه می‌ترکد. حتی عددِ زوج هم امن نیست، چون یک فاصله (یک بایت) یا یک نیم‌فاصله (سه بایت) کلِ حساب را جابه‌جا می‌کند. عملاً روی ورودیِ فارسیِ واقعی، یک برشِ بایتیِ ثابت تقریباً نصفِ مواقع پنیک می‌کند.

همان مسئله برای ستونِ پایگاه داده هم هست، فقط بی‌صداتر:

```text
20 chars = 20 bytes     (Fullmetal Alchemist:)
19 chars = 38 bytes     (برنامه‌نویسی سیستمی)
```

یک `VARCHAR(20)` که با بایت می‌شمارد، برای کاربرِ فارسی حدودِ ده حرف است.

```senpai-visual
{"kind":"concept","labels":["اندیس بایت","مرز است؟","بچسبان یا بپرس","برش امن"]}
```

---

## دست‌به‌کد

```sh
cargo run -p p1-04-04-slicing-text-safely --example 01-bytes-not-characters
cargo run -p p1-04-04-slicing-text-safely --example 02-where-the-boundaries-are
cargo run -p p1-04-04-slicing-text-safely --example 03-safe-cuts
```

بعد چهار تای خراب. این‌ها پشتِ یک feature هستند تا اجرا کردنشان یک کارِ عمدی باشد:

```sh
cargo run -p p1-04-04-slicing-text-safely --example 04-cut-in-half --features broken
cargo run -p p1-04-04-slicing-text-safely --example 05-max-twenty-characters --features broken
cargo run -p p1-04-04-slicing-text-safely --example 06-get-returns-an-option --features broken
cargo run -p p1-04-04-slicing-text-safely --example 07-indexing-by-number --features broken
```

بعد این‌ها را امتحان کن:

۱. در `04-cut-in-half`، `0..7` را به `0..40` عوض کن. پیامِ پنیک عوض می‌شود — چطور؟
۲. در `04-cut-in-half`، `0..7` را به `0..6` عوض کن. حالا چه چاپ می‌شود، و چند حرف است؟
۳. در `02-where-the-boundaries-are`، متن را به `"سلام"` عوض کن. کدام اندیس‌ها مرز شدند؟ چرا این بار خبری از عددِ فرد نیست؟
۴. در `05-max-twenty-characters`، عنوانِ سوم را حذف کن و دوباره اجرا کن. حالا برنامه تا آخر می‌رود — ولی خروجی‌اش هنوز غلط است. کجایش؟

---

## خطاهایی که خواهی دید

### پنیک — `end byte index N is not a char boundary`

```text
english: "program"

thread 'main' (15184) panicked at phase1-fundamentals\04-text-and-strings\04-slicing-text-safely\examples\04-cut-in-half.rs:15:39:
end byte index 7 is not a char boundary; it is inside 'ا' (bytes 6..8 of string)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**به چه اعتراض دارد:** خواستی رشته را سرِ بایتِ ۷ ببری، و بایتِ ۷ وسطِ یک حرفِ دوبایتی است. اگر Rust اطاعت می‌کرد، حاصل یک `&str` بود که UTF-8 معتبر نیست — و کلِ تضمینِ نوعِ `&str` همین است که همیشه UTF-8 معتبر باشد. پس نمی‌تواند اطاعت کند.

**راه‌حل، بسته به اینکه چه می‌خواستی:**

| می‌خواستی | بنویس |
|---|---|
| اگر برش بد بود، خودم تصمیم می‌گیرم | `text.get(a..b)` |
| نزدیک‌ترین برشِ مجاز، بدونِ ردشدن از بودجه | `&text[..text.floor_char_boundary(b)]` |
| N حرف، نه N بایت | `truncate_to_chars(text, n)` |

**چرا این راه‌حل است:** هر سه ورودیِ بد را به یک تصمیمِ صریح تبدیل می‌کنند. `[a..b]` تنها گزینه‌ای است که تصمیم را به runtime واگذار می‌کند، و runtime تصمیمش «توقف» است.

توجه کن که این یک پنیک است، نه خطای کامپایل. کامپایلر نمی‌تواند جلویش را بگیرد چون `b` معمولاً تا زمانِ اجرا معلوم نیست. تفاوتِ «پنیک» و «خطایی که برمی‌گردانی» درسِ [۱.۶.۴](../../06-absence-and-failure/04-panic-vs-result/README.fa.md) است.

### پنیک — `end byte index N is out of bounds`

اگر در همان `04-cut-in-half` عددِ `7` را به `40` تبدیل کنی:

```text
english: "program"

thread 'main' (11808) panicked at phase1-fundamentals\04-text-and-strings\04-slicing-text-safely\examples\04-cut-in-half.rs:15:39:
end byte index 40 is out of bounds for string of length 25
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**به چه اعتراض دارد:** این دیگر ربطی به مرزِ حرف ندارد. رشته ۲۵ بایت است و تو بایتِ ۴۰ را خواستی.

**راه‌حل:** باز هم `.get()` — که برای این حالت هم `None` می‌دهد — یا `floor_char_boundary` که برای عددِ بزرگ‌تر از طول به `text.len()` بسنده می‌کند و پنیک نمی‌کند.

**چرا مهم است:** این دو پیام کاملاً فرق دارند و اشتباه گرفتنشان وقت تلف می‌کند. `not a char boundary` یعنی «عددت در محدوده هست ولی جای بدی است»؛ `out of bounds` یعنی «عددت اصلاً بزرگ است». اولی مسئله‌ی یونیکد است، دومی مسئله‌ی حساب.

### `E0308` — `.get()` یک `&str` نمی‌دهد، یک `Option` می‌دهد

```text
error[E0308]: mismatched types
  --> phase1-fundamentals\04-text-and-strings\04-slicing-text-safely\examples\06-get-returns-an-option.rs:11:23
   |
11 |     let piece: &str = persian.get(0..6);
   |                ----   ^^^^^^^^^^^^^^^^^ expected `&str`, found `Option<&str>`
   |                |
   |                expected due to this
   |
   = note: expected reference `&str`
                   found enum `Option<&str>`
help: consider using `Option::expect` to unwrap the `Option<&str>` value, panicking if the value is an `Option::None`
   |
11 |     let piece: &str = persian.get(0..6).expect("REASON");
   |                                        +++++++++++++++++
```

**به چه اعتراض دارد:** اولین کاری که همه بعد از اولین پنیک می‌کنند این است که `&text[a..b]` را با `text.get(a..b)` عوض کنند و انتظار داشته باشند همان نوع برگردد. برنمی‌گردد. `.get()` دقیقاً به این دلیل امن است که نوعِ متفاوتی می‌دهد: نوعی که «شاید نباشد» را در خودش دارد.

**راه‌حل:** یکی از این دو، بسته به اینکه نبودنش را چطور می‌خواهی مدیریت کنی:

```rust
let piece = persian.get(0..6).unwrap_or("");

if let Some(piece) = persian.get(0..6) {
    println!("{piece}");
}
```

```text
برن
```

**چرا این راه‌حل است:** آن `help` که کامپایلر پیشنهاد می‌دهد — `.expect("REASON")` — کار می‌کند ولی معمولاً غلط است: دقیقاً همان پنیکی را برمی‌گرداند که فرار کرده بودی، فقط با پیامی که خودت نوشته‌ای. `.expect()` جایی درست است که واقعاً باور داری غیرممکن است `None` باشد. اینجا نیست.

### `E0277` — `str` را نمی‌شود با عدد اندیس زد

دومین کاری که همه می‌کنند: بی‌خیالِ بازه می‌شوند و یک اندیسِ تکی می‌گذارند.

```text
error[E0277]: the type `str` cannot be indexed by `{integer}`
   --> phase1-fundamentals\04-text-and-strings\04-slicing-text-safely\examples\07-indexing-by-number.rs:11:25
    |
 11 |     let third = persian[2];
    |                         ^ string indices are ranges of `usize`
    |
    = help: the trait `SliceIndex<str>` is not implemented for `{integer}`
    = note: you can use `.chars().nth()` or `.bytes().nth()`
            for more information, see chapter 8 in The Book: <https://doc.rust-lang.org/book/ch08-02-strings.html#indexing-into-strings>
```

(کامپایلر بعد از این چند خطِ دیگر هم چاپ می‌کند که نشان می‌دهند `usize` برای *برش‌های معمولی* اندیسِ معتبری هست؛ آن خط‌ها به فایل‌های داخلِ خودِ toolchain اشاره می‌کنند.)

**به چه اعتراض دارد:** `persian[2]` باید چه بدهد؟ یک بایت؟ یک حرف؟ در ASCII این دو یکی‌اند و سؤال بی‌معنی است؛ در UTF-8 دو جوابِ متفاوتند و Rust حاضر نیست یکی را حدس بزند. پس این عملگر اصلاً برای `str` وجود ندارد. این را در [۱.۱.۶](../../01-foundations/06-vec-and-string-basics/README.fa.md) هم دیدی.

**راه‌حل:** همان که خودِ پیام می‌گوید — `.chars().nth(2)` برای حرفِ سوم، `.bytes().nth(2)` برای بایتِ سوم. هر دو `Option` می‌دهند، چون ممکن است رشته آن‌قدر بلند نباشد.

**چرا این راه‌حل است:** مجبور شدنت به انتخابِ بینِ `chars` و `bytes` خودِ آموزش است. هر باری که این خطا را می‌گیری یعنی داشتی سؤالی می‌پرسیدی که جوابش به زبانِ متن بستگی دارد.

---

## تمرین

### گرم‌کردن

هیچ تایپی لازم نیست. جواب بده، بعد باز کن.

<details>
<summary><code>"سلام".len()</code> چند است؟</summary>

۸. چهار حرف، هرکدام دو بایت. `len()` بایت می‌شمارد.

</details>

<details>
<summary><code>&"سلام"[0..2]</code> چه می‌دهد؟ <code>&"سلام"[0..3]</code> چطور؟</summary>

اولی `"س"` می‌دهد. دومی پنیک می‌کند: بایتِ ۳ وسطِ حرفِ دوم است.

</details>

<details>
<summary><code>"سلام".get(0..3)</code> چه می‌دهد؟</summary>

`None`. همان برشِ غیرمجاز، ولی این بار به‌جای کشتنِ برنامه جواب می‌دهد.

</details>

<details>
<summary><code>"سلام".floor_char_boundary(3)</code> و <code>ceil_char_boundary(3)</code> چند می‌شوند؟</summary>

۲ و ۴. مرزهای همسایه‌ی بایتِ ۳.

</details>

<details>
<summary>آیا <code>text.len()</code> همیشه یک مرزِ مجاز است؟ <code>0</code> چطور؟</summary>

بله، هر دو. حتی برای رشته‌ی خالی، که در آن هر دو صفرند.

</details>

<details>
<summary>یک برشِ بایتیِ ثابت مثل <code>&title[..20]</code> روی یک عنوانِ فارسی چند درصد مواقع پنیک می‌کند؟</summary>

تقریباً نصف. حروفِ فارسی دو بایت‌اند، پس مرزها یکی‌درمیان‌اند؛ فاصله‌ها و نیم‌فاصله‌ها این الگو را جابه‌جا می‌کنند و نتیجه عملاً سکه انداختن است.

</details>

<details>
<summary>چرا ردیفِ «بی‌صدا غلط» از ردیفِ «پنیک» بدتر است؟</summary>

چون پنیک را می‌بینی و همان روز درستش می‌کنی. برشِ بی‌صدا داده‌ی ناقص را تا پایگاه داده و تا چشمِ کاربر می‌برد و ماه‌ها می‌ماند.

</details>

### تعمیر

`examples/04-cut-in-half.rs` را **سه** جور درست کن، بی‌آنکه معنیِ برنامه عوض شود:

۱. با `.get()` و یک مقدارِ جایگزین.
۲. با `floor_char_boundary`.
۳. با `char_indices`، بدونِ استفاده از هیچ‌کدام از آن دو.

بعد بگو کدام‌شان برای «عنوانِ کارت» درست‌تر است و چرا این سؤال بی‌جواب می‌ماند تا وقتی بدانی آن ۷ از کجا آمده.

`examples/06-get-returns-an-option.rs` را دو جور درست کن: یک بار با `unwrap_or`، یک بار با `if let`. بعد نسخه‌ی سومی را که کامپایلر پیشنهاد می‌دهد (`.expect(...)`) هم امتحان کن و بگو چرا اینجا انتخابِ بدی است.

`examples/07-indexing-by-number.rs` را طوری درست کن که حرفِ سوم را چاپ کند، نه بایتِ سوم.

### پیاده‌سازی

پنج تابع در `src/lib.rs`:

```sh
cargo test -p p1-04-04-slicing-text-safely
```

تست‌ها فارسی، انگلیسی، متنِ ترکیبی، رشته‌ی خالی و هر دو سرِ بازه را می‌گیرند. هیچ‌کدام از این پنج تابع اجازه ندارد روی هیچ ورودی‌ای پنیک کند — و دو تا از تست‌ها دقیقاً همین را برای هر ترکیبِ ممکن بررسی می‌کنند.

`truncated_with_ellipsis` را با دقت بخوان: تفاوتِ «۴ حرف از یک متنِ ۴ حرفی» و «۴ حرف از یک متنِ ۹ حرفی» کلِ تمرین است.

### بساز

یک `pub fn fits_in(text: &str, max_chars: usize) -> String` بنویس که کارِ `truncated_with_ellipsis` را بکند، با یک تفاوت: **سه‌نقطه هم داخلِ بودجه حساب می‌شود.** یعنی خروجی هرگز، تحتِ هیچ شرایطی، بیشتر از `max_chars` حرف ندارد.

خودت تصمیم بگیر برای `max_chars` برابرِ ۰ و ۱ چه باید بشود، و تصمیمت را در یک جمله در doc comment بنویس.

بعد اندازه بگیر: برای `max_chars` برابرِ ۲۰، خروجی روی یک عنوانِ انگلیسی چند بایت می‌شود و روی یک عنوانِ فارسی چند بایت؟ همان نسبت است که یک `VARCHAR(20)` را برای کاربرِ فارسی به نصف تبدیل می‌کند.

### چالش (اختیاری)

**بخشِ یک — نیم‌فاصله‌ی آویزان.** این را اجرا کن:

```rust
let word = "می‌روم";
println!("{:?}", truncate_to_chars(word, 3));
```

سه حرفِ اول «م»، «ی» و خودِ نیم‌فاصله‌اند. خروجی از نظرِ Rust کاملاً معتبر است — ولی برای یک کاربرِ فارسی چه به نظر می‌رسد؟ یک قاعده بنویس که سه‌نقطه‌ی آویزان را حذف کند.

**بخشِ دو — حرف در برابرِ آنچه کاربر می‌بیند.** یک `char` هنوز آن چیزی نیست که کاربر «یک حرف» می‌داند. `"👨‍👩‍👧"` را با `truncate_to_chars(_, 1)` امتحان کن و ببین چه در می‌آید. واحدِ درست اینجا **گرافیم** است، که std اصلاً نمی‌شناسدش و به یک crate بیرونی مثل `unicode-segmentation` نیاز دارد. این حرف را [۱.۴.۲](../02-utf8-bytes-chars-graphemes/README.fa.md) شروع کرده بود و اینجا هم تمام نمی‌شود.

**بخشِ سه — رو به جلو.** `truncate_to_chars` را با `.char_indices()` و آداپتورهای ایتریتور در یک خط بنویس. این ابزار مالِ [فاز ۲](../../../phase2-intermediate/02-iterators-and-closures/02-iterator-adapters/README.fa.md) است، پس اگر گیر کردی مشکلی نیست — ولی وقتی به آن درس رسیدی، این تمرین را دوباره باز کن.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به کارت می‌آید |
|---|---|---|
| مرزِ حرف (char boundary) | اندیسِ بایتی که یک حرف از آن شروع می‌شود | تنها جای مجاز برای برش |
| `&text[a..b]` | برشِ بایتی؛ روی مرزِ بد پنیک | وقتی مطمئنی هر دو سر مرزند |
| `.get(a..b)` | همان برش، در قالبِ `Option<&str>` | وقتی ورودی از بیرون می‌آید |
| `is_char_boundary` | مرز است یا نه؛ هرگز پنیک نمی‌کند | بررسی قبل از برش |
| `char_indices` | فهرستِ (اندیسِ بایت، حرف) | پیدا کردنِ مرزِ حرفِ N اُم |
| `floor_char_boundary` | نزدیک‌ترین مرز، رو به پایین | بودجه‌ی بایتی که نباید رد شود |
| `ceil_char_boundary` | نزدیک‌ترین مرز، رو به بالا | وقتی حرفِ ناقص را می‌خواهی کامل نگه داری |
| `split_at_checked` | نسخه‌ی بی‌پنیکِ `split_at` | دو نیم کردنِ متن |
| نیم‌فاصله (ZWNJ) | نویسه‌ی نامرئیِ سه‌بایتیِ `U+200C` | چیزی که حسابِ بایتیِ فارسی را هم بر هم می‌زند |

### الان می‌دانی

- دو عددِ داخلِ `&text[a..b]` اندیسِ بایت‌اند، نه شماره‌ی حرف.
- برش فقط وقتی مجاز است که هر دو سرش مرزِ حرف باشند؛ وگرنه پنیک، نه خروجیِ خراب.
- پیامِ پنیک خودِ حرفِ قربانی و بازه‌ی بایتی‌اش را می‌گوید.
- `is_char_boundary`، `char_indices`، `.get()`، `floor_char_boundary` و `split_at_checked` هیچ‌کدام پنیک نمی‌کنند.
- `.get()` یک `Option<&str>` می‌دهد و همین تفاوتِ نوع است که امنش می‌کند.
- بریدن به تعدادِ حرف یعنی گرفتنِ اندیسِ بایتیِ حرفِ N اُم از `char_indices`.
- سه‌نقطه فقط وقتی اضافه می‌شود که واقعاً چیزی حذف شده باشد، و `…` یک نویسه است نه سه تا.
- «حداکثر ۲۰ کاراکتر» که با بایت پیاده شود، برای کاربرِ فارسی یا نصفه‌متن می‌دهد یا پنیک.

### بعداً کامل‌تر می‌بینی

- **`Option` به‌طورِ کامل، با `match` و همه‌ی متدهایش** — [۱.۶.۱ — `Option` و ایمنیِ null](../../06-absence-and-failure/01-option-and-null-safety/README.fa.md)
- **`if let`، که اینجا فقط در حدِ نیاز دیدی** — [ماژولِ ۱.۵ — نوع‌های خودت](../../05-your-own-types/README.fa.md)
- **پنیک در برابرِ خطایی که برمی‌گردانی** — [۱.۶.۴ — پنیک یا `Result`](../../06-absence-and-failure/04-panic-vs-result/README.fa.md)
- **آداپتورهای ایتریتور، که این توابع را یک‌خطی می‌کنند** — [فاز ۲ — آداپتورهای ایتریتور](../../../phase2-intermediate/02-iterators-and-closures/02-iterator-adapters/README.fa.md)
- **گرافیم و crate بیرونی برای شمردنِ آنچه کاربر می‌بیند** — [۱.۴.۲ — یوتی‌اف-۸](../02-utf8-bytes-chars-graphemes/README.fa.md)

### می‌توانی توضیح بدهی؟

- دو عددِ داخلِ `&text[2..6]` چه چیزی را می‌شمارند؟
- چرا Rust به‌جای برگرداندنِ نیمِ یک حرف، برنامه را متوقف می‌کند؟
- سه تکه‌ی پیامِ `end byte index 7 is not a char boundary; it is inside 'ا' (bytes 6..8 of string)` هرکدام چه می‌گویند؟
- تفاوتِ `&text[a..b]` و `text.get(a..b)` در یک جمله؟
- `floor_char_boundary` و `ceil_char_boundary` روی بایتِ ۵ از «می‌روم» چه می‌دهند و چرا؟
- چرا برای بریدن به N حرف باید از `char_indices` رد شوی و نمی‌شود از `len()` حساب کرد؟
- چرا برشِ بی‌صدا و غلط از پنیک بدتر است؟

---

## بیشتر

- [`str::get`](https://doc.rust-lang.org/std/primitive.str.html#method.get) — مستنداتش دقیقاً همان جدولِ «چه‌وقت `None` می‌دهد» را دارد.
- [`str::floor_char_boundary`](https://doc.rust-lang.org/std/primitive.str.html#method.floor_char_boundary) — پایدار از ۱.۹۱.
- [کتابِ Rust — فصلِ ۸، رشته‌ها](https://doc.rust-lang.org/book/ch08-02-strings.html#indexing-into-strings) — همان استدلال، رسمی. همان لینکی که خودِ کامپایلر در `E0277` می‌دهد.
- [`unicode-segmentation`](https://docs.rs/unicode-segmentation) — وقتی «یک حرف» یعنی آنچه کاربر می‌بیند، نه آنچه `char` می‌گوید.
