# راه‌حل

```rust
pub fn build_report(label: &str, samples: &[i64]) -> Option<Report> {
    let min = samples.iter().copied().min()?;
    let max = samples.iter().copied().max()?;
    let sum: i64 = samples.iter().sum();

    Some(Report {
        label: label.to_string(),
        count: samples.len(),
        mean: sum as f64 / samples.len() as f64,
        min,
        max,
    })
}
```

هیچ‌جا صریحاً از `is_empty` استفاده نشده — متدِ `Iterator::min` روی خروجی یک `Option<i64>` برمی‌گرداند (`None` روی یک پیمایشگرِ خالی)، و `?` روی یک `Option`، وقتی داخلِ تابعی هستی که خودش `Option` برمی‌گرداند، به‌محضِ رسیدن به `None` همان‌جا زودتر برمی‌گردد. تا وقتی اجرا به خطِ تقسیم برسد، `min()?` از قبل ثابت کرده بُرش خالی نیست، پس `samples.len()` نمی‌تواند صفر باشد. یک نکته‌ی ظریف در ترتیبِ خطوط: این فقط به این دلیل کار می‌کند که سیگنالِ خالی‌بودن *زودتر* آمده — اگر `mean` را قبل از خط‌هایِ `?` حساب می‌کردی، رویِ یک بُرشِ خالی تقسیم بر صفر می‌شد (یا دقیق‌تر: چون تقسیمِ اعشاری است، `NaN` می‌شد — که شاید بدتر هم باشد: بدونِ کرش، فقط یک مقدارِ مسموم که همچنان جلو می‌رود).

```rust
#[cfg(feature = "json-export")]
pub fn to_json(report: &Report) -> String {
    serde_json::to_string(report)
        .expect("a plain struct of strings and numbers cannot fail to serialize")
}
```

چرا `.expect` به‌جایِ برگرداندنِ `serde_json::Result<String>`؟ متدِ `to_string` واقعاً می‌تواند شکست بخورد — ولی فقط برایِ نوع‌هایی که پیاده‌سازیِ `Serialize`شان می‌تواند خطا بدهد (مَپ‌هایی با کلیدهایِ غیرِرشته‌ای، پیاده‌سازی‌هایِ دستی‌ای که خودشان bail می‌کنند). `Report` پنج فیلدِ ساده است، پس شکست در عمل غیرِممکن است، و مجبورکردنِ هر فراخواننده به هندل‌کردنِ یک `Result`ی که هرگز رخ نمی‌دهد فقط نویز است. پیامِ `.expect` همان استدلال را دقیقاً همان‌جایی که ممکن است کرش کند مستند می‌کند.

```rust
#[cfg(feature = "csv-export")]
pub fn to_csv(report: &Report) -> String {
    format!(
        "{},{},{},{},{}",
        report.label, report.count, report.mean, report.min, report.max
    )
}
```

فیچرِ خودِ پله‌ی «بساز» — هم‌شکلِ `json-export` (یک لیستِ `[]` خالی، چون چیزِ اختیاری‌ای برایِ روشن‌کردن نیست) و بدونِ هیچ وابستگیِ تازه‌ای. `format!` و پیاده‌سازیِ `Display`ِ خودِ هر فیلد کافی است؛ Rust یک `f64`ِ عددِ صحیح مثلِ `30.0` را به‌شکلِ `30` چاپ می‌کند، برایِ همین تستِ پایین `"30"` را چک می‌کند، نه `"30.0"`.

```rust
#[cfg(feature = "pretty")]
pub fn to_json_pretty(report: &Report) -> String {
    serde_json::to_string_pretty(report)
        .expect("a plain struct of strings and numbers cannot fail to serialize")
}
```

پله‌ی «چالش». تمامِ ترفند همین یک خط تویِ `Cargo.toml` است: `pretty = ["json-export"]` — روشن‌کردنِ `pretty` خودش `json-export` را هم روشن می‌کند، همان شکلی که `derive = ["serde/derive"]` یک فیچرِ *وابستگی* را روشن می‌کرد، منتها اینجا رویِ فیچرِ خودِ همین کریت. `pretty_tests` اصلاً فقط به همین دلیل کامپایل می‌شود که این کار کرده باشد: `to_json` را صدا می‌زند — تابعی که پشتِ `json-export` است — درحالی‌که از Cargo فقط `pretty` خواسته شده.

## سیم‌کشیِ فیچرها، که درسِ اصلی همین است

- `Cargo.toml`: وابستگی‌هایِ `serde`/`serde_json` با `optional = true` علامت خورده‌اند، و `json-export = ["dep:serde", "dep:serde_json"]` تنها چیزی است که روشنشان می‌کند. `csv-export = []` و `pretty = ["json-export"]` درست کنارش‌اند — همان جدول، همان شکل؛ یکی هیچ‌چیزی جز خودش برایِ روشن‌کردن ندارد، یکی هم به‌جایِ یک وابستگی، یک فیچرِ دیگر را روشن می‌کند. `default = []` کریتِ پایه را بدونِ وابستگی نگه می‌دارد.
- ساختار از صفتِ `#[cfg_attr(feature = "json-export", derive(serde::Serialize))]` استفاده می‌کند — یک *صفتِ* شرطی، پس `Report` تویِ هر دو بیلد وجود دارد. یک `#[cfg]`ِ ساده رویِ ساختار آن را از بیلدِ پیش‌فرض حذف می‌کرد و `build_report` را می‌شکست، چون نوعِ برگشتی‌اش بی‌قیدوشرط به آن اشاره می‌کند.
- `to_json`، `to_csv` و `to_json_pretty` هرکدام از یک `#[cfg(feature = "...")]`ِ ساده رویِ خودِ تابع استفاده می‌کنند — کلِ تابع بدونِ فیچرش نباید وجود داشته باشد. با نبودنِ `to_json`، بیلدِ پیش‌فرض اصلاً به `serde_json` ارجاع نمی‌دهد، که دقیقاً همان چیزی است که اجازه می‌دهد این وابستگی از اول اختیاری باشد.
- هر ماژولِ تستِ مخصوصِ یک فیچر، پشتِ `#[cfg(all(test, feature = "..."))]` است، پس یک `cargo test`ِ ساده هرگز حتی سعی نمی‌کند (و شکست نمی‌خورد) کامپایلش کند.

هر دنیا را از رویِ همین مانیفستِ داخلِ `solution/` بررسی کن:

```bash
cargo test --manifest-path solution/Cargo.toml
cargo test --manifest-path solution/Cargo.toml --features json-export
cargo test --manifest-path solution/Cargo.toml --features csv-export
cargo test --manifest-path solution/Cargo.toml --features pretty
cargo test --manifest-path solution/Cargo.toml --all-features
```

اگر فقط همیشه دستورِ اول را بزنی، هیچ‌کدام از `to_json`، `to_csv` یا `to_json_pretty` — و نه تست‌هایشان — اصلاً کامپایل نمی‌شوند. سبزبودنِ بیلدِ پیش‌فرض هیچ‌چیزی درباره‌ی هیچ‌کدام از بیلدهایِ فیچردار نمی‌گوید. کریت‌هایِ واقعی این را تویِ CI با یک جاب به‌ازایِ هر ترکیبِ پشتیبانی‌شده خودکار می‌کنند (یا با `cargo hack --each-feature` وقتی ترکیب‌ها زیاد می‌شوند)؛ انضباطِ «هر ترکیبی که ادعایش را کرده‌ای باید سبز بماند» همان قانونِ جمع‌پذیریِ همین README است، این‌بار در عمل.
