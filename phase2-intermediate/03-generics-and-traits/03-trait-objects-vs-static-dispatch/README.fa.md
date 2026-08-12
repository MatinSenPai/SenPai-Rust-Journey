# ۰۳.۳ — trait object در برابر static dispatch

درس قبل با یک پرسش تمام شد: آیا `print_all_summaries<T: Summarize>` می‌تواند sliceای شامل **هم** `AnimeSeries` و **هم** `MangaVolume` بگیرد؟ پاسخ منفی است. هدف این درس فهمیدن دلیل دقیق آن و هزینه‌ی راه‌حل است.

crate این درس مستقل است و نمی‌تواند از crate درس قبل import کند؛ بنابراین نسخه‌ای کوچک از trait را دوباره تعریف می‌کند: `pub trait Summarize { fn summary(&self) -> String; }`.

## ارسال ایستا (Static dispatch): جنریک‌ها

```rust
pub fn total_summary_length_generic<T: Summarize>(items: &[T]) -> usize {
    items.iter().map(|i| i.summary().len()).sum()
}
```

مثل `largest<T>` و `Stack<T>`، این تابع monomorphize می‌شود. اگر آن را با `&[AnimeSeries]` صدا بزنی، کامپایلر نسخه‌ای با `T = AnimeSeries` تولید می‌کند. فراخوانی دیگری با `&[MangaVolume]` نسخه‌ی مجزایی می‌سازد. هر **call site** دقیقاً یک نوع concrete برای `T` انتخاب می‌کند؛ پس `&[T]` هرگز ترکیب واقعی چند نوع نیست، چون `T` برای کل slice یک نوع است.

این رفتار **static dispatch** نام دارد: کامپایلر هنگام compile دقیقاً می‌داند هر فراخوانی `summary` به کدام پیاده‌سازی می‌رود و اغلب می‌تواند آن را inline کند؛ حتی بدون هزینه‌ی فراخوانی تابع.

## ارسال پویا (Dynamic dispatch): trait objectها

```rust
pub fn total_summary_length_dyn(items: &[Box<dyn Summarize>]) -> usize {
    items.iter().map(|i| i.summary().len()).sum()
}
```

`Box<dyn Summarize>` یک **trait object** است: مقداری از یک نوع نامشخصِ پیاده‌کننده‌ی `Summarize` که با `Box` روی heap قرار گرفته و نوع concrete آن پاک شده است. کامپایلر فقط می‌داند «این مقدار `Summarize` را پیاده می‌کند»، نه اینکه دقیقاً کدام struct است.

کنار داده‌ی boxed، Rust یک **vtable** یا جدول مجازی متدها نگه می‌دارد: جدول کوچکی از function pointerها، یکی برای هر متد trait، که به پیاده‌سازی نوع concrete همان مقدار اشاره می‌کنند. فراخوانی `.summary()` روی `dyn Summarize` در runtime ابتدا pointer درست را از vtable می‌خواند و سپس به آن تابع می‌پرد؛ یک indirection اضافه نسبت به فراخوانی مستقیم و ازپیش‌معلوم.

```senpai-visual
{"kind":"concept","labels":["dyn Summarize","vtable","AnimeSeries::summary","MangaVolume::summary"]}
```

همین پاک‌شدن نوع امکان ناهمگونی را می‌خرد:

```rust
pub fn make_mixed_collection() -> Vec<Box<dyn Summarize>> {
    vec![
        Box::new(AnimeSeries { /* ... */ }),
        Box::new(MangaVolume { /* ... */ }),
    ]
}
```

یک `Vec<Box<dyn Summarize>>` واقعاً می‌تواند structهای concrete مختلف را کنار هم نگه دارد، چون از دید `Vec` نوع ایستای همه‌ی عضوها یکی است: `Box<dyn Summarize>`. هیچ `Vec<T>` با یک `T` ثابت نمی‌تواند این ترکیب را بیان کند.

## کدام را انتخاب کنیم؟

حالت پیش‌فرض **جنریک** باشد. سریع‌تر است، چون indirection مربوط به vtable ندارد و کامپایلر معمولاً می‌تواند فراخوانی را inline و آن‌سوی آن optimize کند. خطاهای نوع هم روی call site مشخص هنگام compile پیدا می‌شوند. مشخصاً وقتی سراغ `dyn Trait` برو که:

- واقعاً یک **مجموعه‌ی ناهمگون** لازم داری؛ `Vec`ای از نوع‌های concrete متفاوت با trait مشترک.
- می‌خواهی از بزرگ‌شدن اندازه‌ی binary بر اثر monomorphization یک تابع جنریک روی تعداد زیادی نوع جلوگیری کنی؛ چون هر نوع یک کپی کامل از کد می‌سازد.

در فاز سه دوباره این بده‌بستان را می‌بینی. router در `axum` و registryهای شبیه plugin باید handlerهای متفاوت را در یک مجموعه نگه دارند، پس از `dyn Trait` یا الگوهای مشابه trait object بهره می‌برند.

می‌توانی vtable را مثل دفترچه‌ی داخلی مرکز تماس ببینی که برای هر نوع درخواست شماره‌ی کارشناس مناسب را دارد. مرز تشبیه اینجاست: دفترچه در runtime ساخته یا جست‌وجوی متنی نمی‌شود؛ ساختار و pointerهای لازم را کامپایلر فراهم کرده و dispatch فقط یک پرش غیرمستقیم است.

## تمرین تو

یک trait کوچک `Summarize`، دو struct به نام‌های `AnimeSeries` و `MangaVolume` و سه تابع بالا را در `src/lib.rs` پیاده کن.

## ایست بازرسی

ابتدا `CHECKPOINT.fa.md` و بعد `solution/SOLUTION.fa.md` را بخوان.
