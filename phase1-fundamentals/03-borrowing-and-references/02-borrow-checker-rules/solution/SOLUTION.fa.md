# راه‌حل — ۱.۳.۲ قواعدِ بررسی‌کننده‌ی قرض

```rust
pub fn duplicate_in_place(values: &mut Vec<i32>) {
    let original = values.len();
    for index in 0..original {
        let value = values[index];
        values.push(value);
    }
}

pub fn move_first_to_last(values: &mut Vec<i32>) {
    if values.len() < 2 {
        return;
    }
    let front = values.remove(0);
    values.push(front);
}

pub fn append_longest(lines: &mut Vec<String>) {
    if lines.is_empty() {
        return;
    }
    let mut best = 0;
    for index in 1..lines.len() {
        if lines[index].len() > lines[best].len() {
            best = index;
        }
    }
    let copy = lines[best].clone();
    lines.push(copy);
}

pub fn drop_duplicates(values: &mut Vec<i32>) {
    let mut kept: Vec<i32> = Vec::new();
    for value in values.iter() {
        if !kept.contains(value) {
            kept.push(*value);
        }
    }
    *values = kept;
}

pub fn apply_bonus(scores: &mut Vec<i32>, bonus: i32) -> i32 {
    if scores.is_empty() {
        return 0;
    }
    let mut total = 0;
    for score in scores.iter_mut() {
        *score += bonus;
        total += *score;
    }
    scores.push(total);
    total
}
```

پنج تابع، یک `.clone()`. اگر بیشتر نوشتی، جایی به‌جای بازچینی، خطا را خریده‌ای.

## `duplicate_in_place` — همان تله‌ی کلاسیک، در لباسِ تمرین

طبیعی‌ترین چیزی که آدم می‌نویسد این است:

```rust
for value in values.iter() {
    values.push(*value);
}
```

```text
error[E0502]: cannot borrow `*values` as mutable because it is also borrowed as immutable
 --> src\main.rs:3:9
  |
2 |     for value in values.iter() {
  |                  -------------
  |                  |
  |                  immutable borrow occurs here
  |                  immutable borrow later used here
3 |         values.push(*value);
  |         ^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
```

همان `E0502`ی مثالِ ۰۴، فقط با `Vec<i32>` به‌جای `Vec<String>`. و این‌بار خطر ملموس‌تر است: اگر اجازه داشت، `push` می‌توانست بافر را جابه‌جا کند در حالی که حلقه هنوز از روی بافرِ قدیمی می‌خواند.

جوابْ پیمایش با اندیس است، با یک نکته‌ی مهم:

```rust
let original = values.len();
```

طول **قبل** از حلقه خوانده می‌شود. اگر `0..values.len()` می‌نوشتی، هر `push` مقصد را جلوتر می‌برد و حلقه هرگز تمام نمی‌شد — دقیقاً همان حلقه‌ی بی‌پایانِ پایتون، این بار با اجازه‌ی کامپایلر. **بررسی‌کننده‌ی قرض جلوی حافظه‌ی نامعتبر را می‌گیرد، نه جلوی منطقِ غلط.** آن یکی هنوز کارِ توست.

نکته‌ی سوم: `let value = values[index];` عمدی است. `i32` نوعِ `Copy` است، پس این یک عدد است نه یک قرض، و آن قرضِ کوتاه همان‌جا تمام می‌شود.

اگر `values.push(values[index]);` را در یک خط بنویسی هم کامپایل می‌شود — چون آرگومان قبل از فعال شدنِ قرضِ تغییرپذیر ارزیابی می‌شود. اسمِ آن حالتِ خاص «two-phase borrow» است و [۱.۳.۳](../../03-borrow-scopes-and-nll/README.fa.md) جایش است. دو خط نوشتن هیچ هزینه‌ای ندارد و به هیچ حالتِ خاصی تکیه نمی‌کند.

## `move_first_to_last` — بیرون بکش، بعد برگردان

```rust
let front = values.remove(0);
values.push(front);
```

دو فراخوانی، و بینشان هیچ قرضی زنده نیست. `remove` مالکیتِ عنصر را به تو می‌دهد و `Vec` را یکی کوتاه می‌کند؛ `push` همان مقدار را برمی‌گرداند.

نسخه‌ی «بگذار به عنصر اشاره کنم» شکست می‌خورد:

```rust
let front = &mut values[0];
values.push(*front);
```

```text
error[E0499]: cannot borrow `values` as mutable more than once at a time
 --> src\main.rs:4:5
  |
3 |     let front = &mut values[0];
  |                      ------ first mutable borrow occurs here
4 |     values.push(*front);
  |     ^^^^^^      ------ first borrow later used here
  |     |
  |     second mutable borrow occurs here
```

سه برچسب روی دو خط، و برچسبِ `first borrow later used here` دقیقاً روی `*front` نشسته — یعنی «همین استفاده است که قرضِ اول را زنده نگه داشته». اگر آن را قبلاً در یک متغیر کپی می‌کردی، خطا از بین می‌رفت.

نگهبانِ `values.len() < 2` هم لازم است: مشخصات می‌گوید `Vec` با کمتر از دو عنصر دست‌نخورده می‌ماند، و بدونِ آن `remove(0)` روی یک `Vec` خالی پنیک می‌کند.

## `append_longest` — اندیس نگه دار، نه ارجاع

```rust
let mut best = 0;
for index in 1..lines.len() {
    if lines[index].len() > lines[best].len() {
        best = index;
    }
}
let copy = lines[best].clone();
lines.push(copy);
```

`best` یک `usize` است. هیچ چیزی را قرض نگرفته، پس هر کاری بعدش با `lines` بکنی مشکلی نیست.

نسخه‌ای که `best` را یک `&String` می‌گیرد هم اینجا کامپایل می‌شود، چون آخرین استفاده‌اش داخلِ خودِ `push` است. ولی روزی که یک خط بعدش اضافه کنی که `best` را بخواند، `E0502` می‌گیری. **اندیس این سؤال را کلاً حذف می‌کند** و همان چیزی است که در کدِ واقعی می‌خواهی.

این تنها تابعی است که `.clone()` دارد، و دلیلش در خودِ مشخصات است: قرار است یک `String`ِ مستقل به `Vec` اضافه شود. تستِ `the_appended_line_owns_its_own_buffer` دقیقاً همین را می‌سنجد — اگر جوری می‌نوشتی که بافر مشترک بماند، آن `assert` می‌گرفتش (که در Rust بدونِ ابزارهای [فاز ۲](../../../../phase2-intermediate/05-smart-pointers/02-rc-and-arc/README.fa.md) اصلاً بیان‌شدنی نیست).

«اولی برنده است» هم از `>` می‌آید نه از `>=`؛ با `>=` آخرینِ هم‌طول‌ها برنده می‌شد و تستِ `the_earlier_of_two_equal_lines_wins` قرمز.

## `drop_duplicates` — بساز، بعد جایگزین کن

```rust
let mut kept: Vec<i32> = Vec::new();
for value in values.iter() {
    if !kept.contains(value) {
        kept.push(*value);
    }
}
*values = kept;
```

حذف کردن حین پیمایش همان خطای همیشگی است:

```text
error[E0502]: cannot borrow `*values` as mutable because it is also borrowed as immutable
 --> src\main.rs:4:13
  |
2 |     for value in values.iter() {
  |                  -------------
  |                  |
  |                  immutable borrow occurs here
  |                  immutable borrow later used here
3 |         if *value == 3 {
4 |             values.remove(0);
  |             ^^^^^^^^^^^^^^^^ mutable borrow occurs here
```

پس یک `Vec` تازه ساخته می‌شود و آخرِ کار جایگزین: `*values = kept;`. آن `*` همان تهی‌سازیِ ارجاعِ [۱.۳.۱](../../01-shared-and-mutable-refs/README.fa.md) است — «به چیزی که این ارجاع به آن اشاره می‌کند مقدار بده»، نه «خودِ ارجاع را عوض کن». `Vec`ِ قدیمی همان‌جا رها و آزاد می‌شود.

قرضِ `values.iter()` تا پایانِ حلقه زنده است و `*values = kept;` بعد از آن می‌آید، پس تعارضی نیست. اگر همان خط را داخلِ حلقه می‌بردی، `E0502` می‌گرفتی.

هزینه: `contains` برای هر عنصر کلِ `kept` را می‌گردد، پس این پیاده‌سازی درجه‌ی دوم است. برای فهرست‌های کوچک دقیقاً همین درست است؛ راهِ سریع‌ترش یک `HashSet` است که در [فاز ۲](../../../../phase2-intermediate/01-collections/01-vec-and-hashmap/README.fa.md) می‌آید.

## `apply_bonus` — یک گذر برای نوشتن، بعد یک `push`

```rust
for score in scores.iter_mut() {
    *score += bonus;
    total += *score;
}
scores.push(total);
```

`iter_mut` به تو `&mut i32` می‌دهد، یکی در هر لحظه، و هر کدام قبل از ساخته شدنِ بعدی تمام می‌شود. برای همین «یک نویسنده» نقض نمی‌شود، هرچند داری کلِ `Vec` را می‌نویسی.

`push` باید **بیرونِ** حلقه باشد:

```text
error[E0499]: cannot borrow `scores` as mutable more than once at a time
 --> src\main.rs:7:9
  |
4 |     for score in scores.iter_mut() {
  |                  -----------------
  |                  |
  |                  first mutable borrow occurs here
  |                  first borrow later used here
...
7 |         scores.push(total);
  |         ^^^^^^ second mutable borrow occurs here
```

`iter_mut()` خودش یک قرضِ تغییرپذیر از کلِ `Vec` است و تا آخرِ حلقه زنده می‌ماند. `push` دومی را می‌خواهد. و این بار قاعده یک باگِ واقعی را هم گرفته: `push` داخلِ حلقه‌ای که همان `Vec` را پیمایش می‌کند، دوباره همان حلقه‌ی بی‌پایان است.

نگهبانِ `is_empty` هم مشخصات را رعایت می‌کند: `Vec` خالی چیزی اضافه نمی‌کند و `0` برمی‌گرداند. بدونِ آن، خالی به `[0]` تبدیل می‌شد و تستِ `an_empty_score_list_gains_nothing` قرمز.

## الگویی که در هر پنج تا تکرار شد

هیچ‌کدام از این‌ها با بررسی‌کننده نجنگید. هر پنج تا یکی از سه حرکت را زدند:

| حرکت | کجا |
|---|---|
| مقدار را بیرون بکش تا قرض تمام شود | `duplicate_in_place`، `move_first_to_last` |
| اندیس نگه دار نه ارجاع | `append_longest` |
| در یک ساختارِ جدا بساز و آخرش جایگزین کن | `drop_duplicates` |
| بخوان و بنویس در دو مرحله‌ی جدا | `apply_bonus` |

و در هیچ‌کدام `.clone()` راهِ فرار نبود. تنها کلونِ درسْ آن یکی است که مشخصات صریحاً خواسته بود.
