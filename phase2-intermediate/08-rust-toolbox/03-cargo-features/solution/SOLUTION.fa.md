# پاسخ تشریحی

feature `json-export` دو dependency اختیاری را فعال می‌کند. `dep:` نام را فقط به dependency وصل می‌کند و feature ضمنی هم‌نام نمی‌سازد. بدون `optional = true` آن crateها در همه‌ی buildها وارد می‌شدند و feature دیگر هزینه‌ی dependency را کنترل نمی‌کرد.

```rust
#[cfg_attr(feature = "json-export", derive(serde::Serialize))]
pub struct Report { /* ... */ }

#[cfg(feature = "json-export")]
pub fn to_json(report: &Report) -> Result<String, serde_json::Error> {
    serde_json::to_string(report)
}
```

خود `Report` در هر build لازم است؛ فقط derive و تابع JSON افزوده می‌شوند. Cargo در graph معمولاً union featureها را انتخاب می‌کند، پس قابلیت‌ها باید additive باشند. برای verification هم plain test و هم `--features json-export` را اجرا کن؛ وگرنه شاخه‌ی JSON و تست‌های `cfg(all(test, feature = ...))` اصلاً compile نمی‌شوند.
