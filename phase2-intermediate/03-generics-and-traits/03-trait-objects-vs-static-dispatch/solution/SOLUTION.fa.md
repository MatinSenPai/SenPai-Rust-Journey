# پاسخ تشریحی

```rust
pub fn make_mixed_collection() -> Vec<Box<dyn Summarize>> {
    vec![
        Box::new(AnimeSeries { title: "Trigun".to_string(), episodes: 26 }),
        Box::new(MangaVolume { title: "Blame!".to_string(), chapters: 10 }),
    ]
}
```

این خط چیزی است که فقط `dyn Trait` می‌تواند بیان کند. `Box::new(AnimeSeries { .. })` یک `Box<AnimeSeries>` و `Box::new(MangaVolume { .. })` یک `Box<MangaVolume>` می‌سازد؛ دو نوع واقعاً متفاوت.

annotation نوع عضوهای `Vec` به‌شکل `Vec<Box<dyn Summarize>>` باعث می‌شود هر دو در یک مجموعه type-check شوند. هنگام درج هر عضو، Rust یک **unsizing coercion** ضمنی از `Box<AnimeSeries>` به `Box<dyn Summarize>` و همین‌طور برای `MangaVolume` انجام می‌دهد. نوع concrete هرکدام به «یک Box از چیزی که `Summarize` است، همراه vtable مربوط» پاک می‌شود. همین پاک‌کردن نوع دلیل ناممکن‌بودن `Vec<T>` است؛ `T` جنریک برای کل `Vec` یک بار انتخاب می‌شود، اما اینجا دو نوع می‌خواهیم.

پاسخ پرسش سوم: `Vec<dyn Summarize>` بدون `Box` compile نمی‌شود. backing array مربوط به `Vec` عضوها را inline و پشت سر هم نگه می‌دارد و برای محاسبه‌ی فاصله‌ی آن‌ها باید اندازه‌ی دقیق هر عضو را از پیش بداند. خود `dyn Summarize` اندازه‌ی ثابتی ندارد؛ `AnimeSeries` و `MangaVolume` اندازه‌های متفاوت دارند و «هر نوع پیاده‌کننده‌ی `Summarize`» می‌تواند هر اندازه‌ای داشته باشد.

`Box<dyn Summarize>` این مشکل را حل می‌کند: خود pointer مربوط به `Box` اندازه‌ی ثابت و شناخته‌شده دارد، مستقل از چیزی که روی heap به آن اشاره می‌کند. پس از دید `Vec`، نوع عضو ثابت است و فقط محتوای آن‌سوی pointer ناشناخته می‌ماند. به همین دلیل `dyn Trait` تقریباً همیشه پشت pointerهایی مانند `Box<dyn T>`، `&dyn T` یا `Rc<dyn T>` دیده می‌شود.

پاسخ پرسش پنجم: `total_summary_length_generic::<AnimeSeries>` به نسخه‌ای compile می‌شود که `i.summary()` فراخوانی مستقیم و ایستای `AnimeSeries::summary` است. کامپایلر حتی می‌تواند آن را داخل حلقه inline کند و هیچ indirectionای باقی نگذارد.

در `total_summary_length_dyn`، عبارت `i.summary()` باید pointer مربوط به vtable را کنار داده‌ی boxed بخواند، function pointer مربوط به `summary` را از جدول پیدا کند و سپس به آن بپرد. هزینه‌ی «یک پرش pointer برای هر فراخوانی» کوچک اما واقعی است و معمولاً inlineکردن آن‌سوی call را نیز دشوار می‌کند.
