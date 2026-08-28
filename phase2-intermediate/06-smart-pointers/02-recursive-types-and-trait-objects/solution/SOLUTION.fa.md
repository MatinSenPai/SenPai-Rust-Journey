# راه‌حل — ۲.۶.۲ نوع‌های بازگشتی و شیء‌های صفتِ جعبه‌ای

```rust
pub fn depth(expr: &Expr) -> usize {
    match expr {
        Expr::Num(_) => 1,
        Expr::Add(left, right) | Expr::Mul(left, right) => 1 + depth(left).max(depth(right)),
    }
}

pub fn count_nums(expr: &Expr) -> usize {
    match expr {
        Expr::Num(_) => 1,
        Expr::Add(left, right) | Expr::Mul(left, right) => count_nums(left) + count_nums(right),
    }
}

pub fn total_area(shapes: &[Box<dyn Shape>]) -> f64 {
    shapes.iter().map(|shape| shape.area()).sum()
}

pub fn count_shapes_over(shapes: &[Box<dyn Shape>], threshold: f64) -> usize {
    shapes
        .iter()
        .filter(|shape| shape.area() > threshold)
        .count()
}
```

## `depth` و `count_nums` — یک الگویِ بازگشتیِ مشترک

هر دو تابع دقیقاً همان شکلی را دنبال می‌کنند که خودِ `eval` بالای همین فایل دارد: یک حالتِ پایه برایِ `Num`، و یک قدم که دو فرزند را با هم ترکیب می‌کند. تنها فرقشان این است که `eval` باید بینِ `Add` و `Mul` فرق بگذارد (چون `+` و `*` جواب‌هایِ متفاوتی می‌دهند)، ولی `depth` و `count_nums` به این فرق هیچ کاری ندارند — عمقِ یک گره و تعدادِ برگ‌هایِ زیرش، فارغ از اینکه گره جمع است یا ضرب، یکی است. برایِ همین هر دو تابع از یک الگویِ جایگزین (`Add(left, right) | Mul(left, right)`) استفاده می‌کنند: یک بازو، هر دو گونه را می‌گیرد، چون بدنه‌شان یکسان است.

`left` و `right` اینجا از نوعِ `&Box<Expr>` درمی‌آیند (چون خودِ `expr` یک `&Expr` است)، و صدا زدنِ بازگشتیِ `depth(left)` بدونِ هیچ نشان‌زداییِ دستی کار می‌کند — همان تبدیلِ ضمنیِ ارجاعی که خودِ درس رویِ `eval` نشان داد.

## `total_area` و `count_shapes_over` — دوباره، یک بدنه‌ی تقریباً یکسان

```rust
shapes.iter().map(|shape| shape.area()).sum()
```

```rust
shapes.iter().filter(|shape| shape.area() > threshold).count()
```

هر دو تابع یک خط‌لوله‌ی ایتریتورِ ساده رویِ همان `&[Box<dyn Shape>]` هستند — یکی جمع می‌زند، آن یکی فیلتر و می‌شمارد. نکته‌ای که ارزشِ دیدن دارد: `.area()` از پشتِ *دو لایه* بازمی‌شود — اول `&Box<dyn Shape>` تا `Box<dyn Shape>` (مرجع‌گیریِ `.iter()`)، بعد `Box<dyn Shape>` تا `dyn Shape` (خودِ `Box`) — و هیچ‌کدام از این دو لایه را دستی باز نکردی. همین دقیقاً همان نکته‌ای است که «شیءِ صفتِ جعبه‌ای» در درس گفت: `Box` اینجا فقط برایِ اندازه‌ی ثابت است، هیچ چیزِ تازه‌ای درباره‌ی ارسالِ پویا نیست — همان چیزی که [۲.۳.۷](../../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.fa.md) قبلاً کامل توضیح داد.

## این درس واقعاً درباره‌ی چه بود

- **بازگشت هنگامِ کامپایل باید به یک عددِ نهایی برسد؛ `Box` این محدودیت را به زمانِ اجرا منتقل می‌کند، نه حذفش می‌کند.** `depth` و `count_nums` ثابت می‌کنند که خودِ بازگشتِ *الگوریتمی* (تابعی که خودش را صدا می‌زند) هیچ مشکلی نداشت و هیچ‌وقت نداشت — مشکل فقط در تعریفِ *نوع* بود.
- **هر مسیرِ بازگشتی باید از یک `Box` رد شود، نه فقط یکی.** الگویِ جایگزینِ `Add(left, right) | Mul(left, right)` فقط زمانی درست کامپایل می‌شود که هر دو گونه واقعاً همان شکل — دو فیلدِ `Box<Expr>` — را داشته باشند.
- **`Box<dyn Trait>` همان ترفند را، رویِ یک دلیلِ دیگر برایِ اندازه‌ی نامعلوم، به‌کار می‌برد.** `total_area` و `count_shapes_over` هیچ کدِ اضافه‌ای برایِ «این شکل، `Circle` است یا `Rectangle`؟» ندارند — دقیقاً همان بی‌تفاوتی‌ای که ارسالِ پویا قول می‌دهد.
