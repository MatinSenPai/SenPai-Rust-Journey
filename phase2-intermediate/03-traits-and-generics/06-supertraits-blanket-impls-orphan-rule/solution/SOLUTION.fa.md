# راه‌حل — ۲.۳.۶ ابرصفت‌ها، پیاده‌سازیِ فراگیر، قاعده‌ی یتیم و راهِ فرارِ newtype

```rust
pub trait Priced {
    fn price_cents(&self) -> u32;
}

pub trait Discounted: Priced {
    fn discounted_cents(&self) -> u32 {
        self.price_cents() * 90 / 100
    }
}

impl Discounted for Book {}
impl Discounted for Pen {}

pub fn amount_saved<T: Discounted>(item: &T) -> u32 {
    item.price_cents() - item.discounted_cents()
}

pub struct Cart(pub Vec<Pen>);

impl Cart {
    pub fn total_cents(&self) -> u32 {
        self.0.iter().map(Pen::price_cents).sum()
    }
}

impl std::fmt::Display for Cart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} pen(s), {} cents total",
            self.0.len(),
            self.total_cents()
        )
    }
}
```

هر چهار تکه یک ایده‌ی مشترک دارند: صفت‌ها به هم وابسته‌اند (`Discounted` به `Priced` تکیه دارد)، و newtype (`Cart`) صرفاً یک `Vec<Pen>` پوشیده‌شده است، دقیقاً برای اینکه بشود `Display` — یک صفتِ بیگانه — رویش پیاده‌سازی کرد.

## `Discounted::discounted_cents` — بدنه‌ی پیش‌فرضِ ابرصفت

```rust
fn discounted_cents(&self) -> u32 {
    self.price_cents() * 90 / 100
}
```

این بدنه داخلِ خودِ تعریفِ `Discounted` نشسته، نه داخلِ `impl Discounted for Book`. چون `Discounted: Priced` است، کامپایلر تضمین می‌کند هر `Self`ای که به این بدنه می‌رسد یک `price_cents()` هم دارد — دقیقاً همان چیزی که در «مفهوم» دیدی: صدا زدنِ متدِ ابرصفت از داخلِ صفتِ فرزند. ضربِ اول و بعد تقسیم (نه برعکس) با اعدادِ صحیح انجام می‌شود، پس `101 * 90 / 100 = 9090 / 100 = 90` می‌شود، نه ۹۰٫۹ — دقیقاً همان چیزی که کامنتِ مستندات گفته بود.

`impl Discounted for Book {}` و `impl Discounted for Pen {}` هر دو بدنه‌شان خالی است — همان همین بدنه‌ی پیش‌فرض را می‌گیرند، بدونِ بازنویسی.

## `amount_saved` — یک کراند، دو متد

```rust
pub fn amount_saved<T: Discounted>(item: &T) -> u32 {
    item.price_cents() - item.discounted_cents()
}
```

امضا فقط `T: Discounted` را می‌خواهد، ولی بدنه هم `.price_cents()` (از `Priced`) و هم `.discounted_cents()` (از `Discounted`) را صدا می‌زند. کامپایلر این را قبول می‌کند چون `Discounted: Priced` یعنی هر `T`ای که `Discounted` را ارضا می‌کند، از قبل `Priced` را هم ارضا کرده — نوشتنِ `T: Discounted + Priced` زائد بود.

## `Cart::total_cents` — یک forwarding method معمولی

```rust
pub fn total_cents(&self) -> u32 {
    self.0.iter().map(Pen::price_cents).sum()
}
```

`Cart` خودش هیچ متدی از `Vec` به ارث نمی‌برد — این دقیقاً همان قیمتی است که «مفهوم» گفت newtype می‌گیرد. `self.0.iter()` صریح باز می‌کند، `Pen::price_cents` (نامِ خودِ متد، بدونِ فراخوانی) به‌عنوانِ یک تابع به `.map()` داده می‌شود، و `.sum()` جمعشان می‌زند.

## `Display for Cart` — پرداختِ نهایی

```rust
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
        f,
        "{} pen(s), {} cents total",
        self.0.len(),
        self.total_cents()
    )
}
```

اینجا همان جایی است که کلِ درس به هم می‌رسد: `Display` یک صفتِ بیگانه است، `Vec` یک نوعِ بیگانه است، و `Cart` — پوششِ محلی‌مان — تنها چیزی است که اجازه می‌دهد این `impl` اصلاً کامپایل شود. تابع `self.total_cents()` را دوباره صدا می‌زند به‌جای اینکه جمع را دوباره حساب کند؛ تستِ `cart_display_matches_total_cents` دقیقاً همین را بررسی می‌کند — اگر فرمول اینجا دوباره‌نویسی می‌شد، یک روز می‌توانستند از هم فاصله بگیرند.

## چیزی که این درس واقعاً درباره‌اش بود

- **یک ابرصفت یک وابستگی است، نه ارث‌بری.** نوشتنِ `impl Discounted for Book {}` هیچ‌چیزی از `Priced` را برایت رایگان نمی‌سازد؛ باید `impl Priced for Book` را جدا بنویسی. ابرصفت فقط تضمین می‌کند این کار را قبلاً کرده‌ای.
- **از داخلِ یک کراندِ فرزند، متدهای ابرصفت هم در دسترس‌اند.** `amount_saved<T: Discounted>` نیازی به `+ Priced` نداشت.
- **پیاده‌سازیِ فراگیر یعنی نوشتنِ یک impl برای هزار نوع، یک‌بار.** همان کاری که کتابخانه‌ی استاندارد با `impl<T, U> Into<U> for T where U: From<T>` کرد.
- **قاعده‌ی یتیم یعنی: صفت یا نوع، دست‌کم یکی‌شان باید محلی باشد.** `Display` بیگانه است، `Vec` بیگانه است — `impl Display for Vec<Pen>` مستقیم کامپایل نمی‌شود.
- **newtype یک راهِ فرار است.** `Cart(Vec<Pen>)` یک نوعِ محلیِ تازه می‌سازد؛ حالا `impl Display for Cart` قانونی است، چون یک طرفش (`Cart`) محلی است.
