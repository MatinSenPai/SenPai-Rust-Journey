# راه‌حل — ۱.۶.۴ پنیک در برابرِ `Result`

```rust
pub fn parse_priority(input: &str) -> Result<u8, String> {
    let value: u8 = match input.trim().parse() {
        Ok(value) => value,
        Err(_) => return Err(format!("'{input}' is not a whole number")),
    };
    if !(1..=5).contains(&value) {
        return Err(format!("priority must be between 1 and 5, got {value}"));
    }
    Ok(value)
}

pub fn priority_label(level: u8) -> &'static str {
    match level {
        1 | 2 => "low",
        3 => "normal",
        4 | 5 => "high",
        other => unreachable!(
            "priority_label: level {other} was never validated by parse_priority (must be 1..=5)"
        ),
    }
}

pub fn checked_midpoint(sorted_ascending: &[i32]) -> i32 {
    assert!(
        !sorted_ascending.is_empty(),
        "checked_midpoint: caller must not pass an empty slice"
    );
    sorted_ascending[sorted_ascending.len() / 2]
}

pub fn last_digit_of(values: &[u32]) -> u32 {
    let last = values
        .last()
        .expect("last_digit_of: caller must not pass an empty values slice");
    last % 10
}
```

دو تا از این چهارتا هرگز پنیک نمی‌زنند. دوتای دیگر باید بزنند. فرقشان در کد نیست — در این است که ورودی از کجا آمده.

## `parse_priority` — ورودی‌ای که هیچ‌وقت به آن اعتماد نمی‌کنی

```rust
let value: u8 = match input.trim().parse() {
    Ok(value) => value,
    Err(_) => return Err(format!("'{input}' is not a whole number")),
};
if !(1..=5).contains(&value) {
    return Err(format!("priority must be between 1 and 5, got {value}"));
}
Ok(value)
```

دو راهِ جداگانه برای شکست، دو پیامِ جداگانه. `.parse()` روی چیزی مثلِ `"abc"` یک `Err` می‌دهد که هیچ ربطی به این ندارد که عدد چه بوده — برای همین به‌جای اینکه آن خطای داخلی را نشان بدهیم، یک پیامِ خودمان می‌سازیم که خودِ `input` (نه نسخه‌ی trim‌شده‌اش) را نام می‌برد. مرحله‌ی دوم فقط وقتی می‌رسد که پارس موفق بوده — این‌جا `value` را داریم و می‌توانیم آن را در پیامِ خطا بگذاریم.

توجه کن که هیچ‌جای این تابع پنیک نمی‌زند. حتی وقتی `input` یک رشته‌ی کاملاً بی‌معنی است، تابع می‌میرد نه با یک crash بلکه با یک `Err` — چون این ورودی از بیرونِ برنامه آمده و بودنِ نامعتبرش، طبقِ تعریف، عادی است.

## `priority_label` — همان مقدار، اعتمادِ کاملاً متفاوت

```rust
match level {
    1 | 2 => "low",
    3 => "normal",
    4 | 5 => "high",
    other => unreachable!(
        "priority_label: level {other} was never validated by parse_priority (must be 1..=5)"
    ),
}
```

`level` هم یک `u8` است، دقیقاً مثلِ چیزی که از `parse_priority` بیرون آمد — ولی این‌جا دیگر یک ورودیِ خام نیست؛ مستندِ تابع می‌گوید که فقط با خروجیِ معتبرِ `parse_priority` صدا زده می‌شود. `match` هنوز باید همه‌ی مقدارهای ممکنِ `u8` را پوشش دهد (این قانونِ خودِ زبان است، از ۱.۵.۴)، ولی شاخه‌ی `other` قرار نیست هرگز واقعاً اجرا شود. `unreachable!()` این را دقیق می‌گوید و اسمِ فرضِ شکسته‌شده را هم می‌برد — نه فقط «این نباید اتفاق بیفتد»، بلکه «چه چیزی این را تضمین می‌کرد و کجا آن تضمین دور زده شد».

## `checked_midpoint` — یک `assert!` که یک قاعده‌ی داخلی را نگهبانی می‌کند

```rust
assert!(
    !sorted_ascending.is_empty(),
    "checked_midpoint: caller must not pass an empty slice"
);
sorted_ascending[sorted_ascending.len() / 2]
```

`sorted_ascending` را یک کاربر نساخته؛ یک بخشِ دیگرِ همین برنامه ساخته و فرستاده. اگر خالی برسد، این نشانه‌ی این نیست که «ورودی بد بود» — نشانه‌ی این است که یک‌جای دیگرِ همین کدبیس قولش را زیرِ پا گذاشته. پیامِ `assert!` هم دقیقاً همین را می‌گوید: نامِ تابع، و این‌که چه چیزی از فراخواننده انتظار می‌رفت.

## `last_digit_of` — `.expect()`ی که فرض را نام می‌برد، نه شکست را

```rust
let last = values
    .last()
    .expect("last_digit_of: caller must not pass an empty values slice");
last % 10
```

`values.last()` یک `Option<&u32>` می‌دهد. می‌شد این‌جا `.unwrap()` نوشت — درست هم کار می‌کرد — ولی پیامش «called `Option::unwrap()` on a `None` value» می‌بود: چیزی که خودِ نوعِ `Option` هم مجانی می‌گفت. پیامِ `.expect()` این‌جا به‌جایش می‌گوید *چرا* فکر می‌کردیم `values` هرگز خالی نیست: چون این قراردادِ خودِ تابع است. کسی که این خط را در لاگ می‌بیند دیگر مجبور نیست حدس بزند کدام `None` بوده — همان‌جا اسمِ تابع و فرضش نوشته شده.

## این چهار تابع واقعاً چه چیزی را نشان می‌دادند

- **قاعده هیچ‌وقت خودِ کد نیست، منشأِ داده است.** `parse_priority` و `priority_label` هر دو یک `u8` می‌گیرند؛ یکی `Result` برمی‌گرداند، آن یکی پنیک می‌زند — چون یکی‌شان از یک آدم می‌آید و آن یکی از داخلِ همین برنامه.
- **`assert!`/`unreachable!()`/`.expect()` همه یک خانواده‌اند: «دارم ادعا می‌کنم این نمی‌تواند اتفاق بیفتد».** تفاوتشان فقط در شکلِ چیزی است که چک می‌کنند — یک شرطِ بولین، یک شاخه‌ی `match`، یا یک `Option`.
- **پیامِ خوب همیشه قولِ شکسته‌شده را نام می‌برد، نه علامتِ شکست را.** چهارتای این توابع، هر جا پنیک می‌زنند، اسمِ خودِ تابع و اسمِ قراردادش را در پیام آورده‌اند — نه فقط «این خالی بود».
- **هیچ‌کدام از این چهار تابع روی ورودیِ یک کاربر پنیک نمی‌زند.** فقط `parse_priority` مستقیماً با متنِ بیرونی سروکار دارد، و او تنها کسی است که هرگز پنیک نمی‌زند.
