# راه‌حل — ۲.۶.۴ `Weak` و چرخه‌های ارجاع

```rust
pub fn is_reachable<T>(weak: &Weak<T>) -> bool {
    weak.upgrade().is_some()
}

pub fn describe<T: std::fmt::Debug>(weak: &Weak<T>) -> String {
    match weak.upgrade() {
        Some(value) => format!("alive: {value:?}"),
        None => "gone".to_string(),
    }
}

pub fn parent_name(folder: &Folder) -> Option<String> {
    folder.parent.upgrade().map(|parent| parent.name.clone())
}

pub fn depth(folder: &Rc<Folder>) -> usize {
    let mut steps = 0;
    let mut current = Rc::clone(folder);
    while let Some(parent) = current.parent.upgrade() {
        steps += 1;
        current = parent;
    }
    steps
}
```

هر چهار تابع فقط با `.upgrade()` و آنچه از قبل بلد بودی نوشته شده‌اند — بدونِ `RefCell`، بدونِ ساختنِ درخت. درخت از قبل ساخته شده بود؛ کارِ تو فقط خواندنش بود.

## `is_reachable` — خودِ تعریفِ `Weak`

```rust
weak.upgrade().is_some()
```

این تقریباً کلمه‌به‌کلمه تعریفِ `Weak` است: یک `Weak<T>` «زنده» است اگر و فقط اگر `.upgrade()`اش `Some` بدهد. نیازی به `match` نیست — `Option::is_some()` دقیقاً همین سؤال را می‌پرسد.

## `describe` — دو حالت، دقیقاً همان دو رشته

```rust
match weak.upgrade() {
    Some(value) => format!("alive: {value:?}"),
    None => "gone".to_string(),
}
```

اینجا واقعاً به مقدارِ داخلِ `Some` نیاز داری (برایِ چاپش)، پس `match` به‌جایِ `is_some()`. کرانِ `T: std::fmt::Debug` روی امضایِ تابع دقیقاً به همین دلیل آنجاست — بدونش کامپایلر نمی‌داند `{value:?}` را چطور بنویسد.

## `parent_name` — `.upgrade()` بعلاوه `.map()`

```rust
folder.parent.upgrade().map(|parent| parent.name.clone())
```

`folder.parent.upgrade()` یک `Option<Rc<Folder>>` می‌دهد. `.map()` — همان ترکیب‌گرِ [۱.۶.۲](../../../../phase1-fundamentals/06-absence-and-failure/02-option-combinators/README.fa.md) — وقتی `Some` است رویِ مقدارِ داخلش اجرا می‌شود و وقتی `None` است دست‌نخورده رد می‌شود؛ دقیقاً همان `match` بالا را کوتاه‌تر می‌نویسد. `parent.name.clone()` یک `String` مالک برمی‌گرداند، نه یک `&String` که به عمرِ آن `Rc`ِ موقت گره خورده باشد.

## `depth` — بالا رفتن، یک پله در هر بار

```rust
let mut steps = 0;
let mut current = Rc::clone(folder);
while let Some(parent) = current.parent.upgrade() {
    steps += 1;
    current = parent;
}
steps
```

`current` با خودِ `folder` شروع می‌شود (یک `Rc::clone` ارزان، نه یک کپیِ عمیق). هر دور از حلقه سعی می‌کند یک پله بالاتر برود: اگر `.upgrade()` چیزی بدهد، `steps` یکی زیاد می‌شود و `current` همان والد می‌شود؛ وگرنه (والدی نیست، یا زنده نیست) حلقه با `while let` تمام می‌شود. برایِ ریشه — که `parent`ش یک `Weak::new()` خالی است — `.upgrade()` همان بارِ اول `None` می‌دهد، پس `steps` همان صفرِ اولیه می‌ماند.

## این درس واقعاً درباره‌ی چه بود

- **`Weak<T>` فقط یک راه برایِ رسیدن به مقدار دارد: `.upgrade()`.** و چون نمی‌تواند قول بدهد مقدار هنوز هست، آن راه یک `Option<Rc<T>>` می‌دهد، نه خودِ `Rc<T>`.
- **نگه‌داشتنِ یک `Weak` هرگز شمارنده‌ی قوی را عوض نمی‌کند** — دقیقاً همان چیزی که اجازه می‌دهد این تمرین‌ها هیچ‌کدام `RefCell` لازم نداشته باشند: چون هیچ‌کدام نمی‌خواستند مالکیتِ چیزی را بگیرند، فقط نگاهش کنند.
- **درختِ نمونه با `Rc::new_cyclic` ساخته شده بود** — همان سازنده‌ای که در `README.fa.md` دیدی، دقیقاً برایِ همین شکل: هر فرزند از همان اول یک `Weak<Folder>` کارآمد به والدش دارد.
- **`.map()` رویِ یک `Option<Rc<T>>` همان کاری را می‌کند که رویِ هر `Option`ِ دیگری می‌کرد** — چیزی «مخصوصِ» `Weak` اینجا نیست؛ بعد از `.upgrade()`، فقط یک `Option` معمولی داری.
