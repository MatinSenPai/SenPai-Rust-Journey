# ۰۸.۳ — featureهای Cargo

## تعریف feature و dependency اختیاری

در `Cargo.toml`، dependency با `optional = true` فقط وقتی فعال می‌شود که feature مربوط آن را با `dep:name` روشن کند:

```toml
[features]
default = []
json-export = ["dep:serde", "dep:serde_json"]
```

## محدودکردن کد با `#[cfg]` و `#[cfg_attr]`

`#[cfg(feature = "json-export")]` کل item را فقط در build فعال نگه می‌دارد. `#[cfg_attr(..., derive(...))]` خود struct را همیشه نگه می‌دارد، اما attribute اضافه را مشروط می‌کند؛ برای derive کردن `Serialize` مناسب است.

## featureها باید افزایشی باشند

Cargo featureهای درخواست‌شده‌ی وابستگی‌های مختلف را unify می‌کند و معمولاً crate را یک بار با union آن‌ها می‌سازد. پس feature باید قابلیت اضافه کند، نه رفتار موجود را حذف یا عوض کند؛ وگرنه وابستگی دیگری می‌تواند ناخواسته semantics برنامه را تغییر دهد.

```senpai-visual
{"kind":"concept","labels":["crate A: json","crate B: default","feature unification","build واحد"]}
```

## فرمان‌های واقعی

```text
cargo test -p PACKAGE
cargo test -p PACKAGE --features json-export
cargo check -p PACKAGE --all-features
```

feature مثل افزونه‌ی اختیاری گزارش است. مرز تشبیه: انتخاب هر مصرف‌کننده خصوصی باقی نمی‌ماند؛ Cargo درخواست‌ها را در graph dependency یکپارچه می‌کند.

## تمرین تو

feature مربوط به JSON، derive مشروط و `to_json` را کامل کن و هر دو build را تست کن.
