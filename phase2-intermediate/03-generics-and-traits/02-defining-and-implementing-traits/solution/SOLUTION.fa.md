# پاسخ تشریحی

```rust
fn summary(&self) -> String {
    format!("{} (no summary available)", self.title())
}
```

این بدنه‌ی پیش‌فرض، خط اصلی درس است. یک بار داخل تعریف trait نوشته می‌شود؛ پیش از آنکه نوع concreteای مانند `AnimeSeries` یا `MangaVolume` وجود داشته باشد. کد compile می‌شود چون امضای trait، یعنی `fn title(&self) -> String;`، یک **تعهد** است: هر نوع واقعی `self` در call site باید متد `title` با دقیقاً همین امضا داشته باشد.

کامپایلر این تعهد را از بلوک حل‌شده‌ی `impl Summarize for X` می‌داند. متد پیش‌فرض لازم نیست `X` را concretely بشناسد؛ کافی است `X: Summarize` باشد. این شرط خودبه‌خود برقرار است، چون روی چیزی که اصلاً `Summarize` را پیاده نکرده نمی‌توانی `.summary()` را صدا بزنی. این پاسخ پرسش اول است.

پاسخ پرسش دوم: `vol.summary()` همان بدنه‌ی پیش‌فرض نوشته‌شده در trait را اجرا می‌کند. این بدنه برای هر پیاده‌کننده در سورس کپی نمی‌شود. بلوک `impl Summarize for MangaVolume` فقط `title` را دارد؛ Rust هنگام resolveکردن `vol.summary()` overrideای پیدا نمی‌کند و سراغ default trait می‌رود و در زمان compile، `MangaVolume` را جای `Self` می‌گذارد.

در سطح source چیزی تکرار نمی‌شود. فقط اگر `summary` از یک تابع جنریک مانند `print_all_summaries::<MangaVolume>` فراخوانی شود، ممکن است به همان دلیل monomorphization درس قبل نسخه‌های کد ماشین جدا ببینی.

پاسخ پرسش سوم: نبودن `title` در `LightNovel` هنگام `cargo build` یا `cargo check` پیدا می‌شود. خطای compile متد دقیق گمشده را نام می‌برد؛ چیزی در این معنا: `not all trait items implemented, missing: title`. هیچ تست یا binaryای پیش از اصلاح اجرا نمی‌شود.

در Python ABC، خطای `TypeError: Can't instantiate abstract class LightNovel with abstract method title` فقط وقتی رخ می‌دهد که برنامه بخواهد `LightNovel()` بسازد؛ شاید مدت‌ها پس از شروع یک فرایند در حال اجرا. با duck typing یا `Protocol` ساده حتی ممکن است تا اجرای همان خط فراخوانی متد گمشده خطایی نبینی. نسخه‌ی Rust از این باگ نمی‌تواند در build موفق پنهان بماند.
