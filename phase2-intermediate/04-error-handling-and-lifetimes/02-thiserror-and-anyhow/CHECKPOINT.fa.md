# ایست بازرسی

۱. attribute به‌شکل `#[error("...")]` را از یکی از variantهای `ConfigError` حذف و `cargo check` را اجرا کن. نتیجه خطای compile است یا فقط تفاوت runtime؟ این رفتار درباره‌ی ضروری‌بودن `#[error(...)]` برای `#[derive(thiserror::Error)]` چه می‌گوید؟

۲. در آخرین تست، `err.source()` همان `ConfigError` اصلی زیر پوشش `anyhow::Context` را برمی‌گرداند. چرا `.context(...)` خطای اصلی را نگه می‌دارد و جایگزین نمی‌کند؟ اگر تنها راه بررسی شکست، متن `Display` مربوط به `anyhow::Error` بود چه اطلاعاتی از دست می‌رفت؟

۳. قاعده‌ی درس این است: «`thiserror` در `lib.rs` و `anyhow` در `main.rs`». تابع `parse_config` مقدار `Result<Config, ConfigError>` و `load_and_parse` مقدار `anyhow::Result<Config>` می‌دهد. در یک ابزار CLI واقعی کدام امضا را در crate کتابخانه‌ی reusable می‌گذاری و کدام فقط در `main.rs` binary می‌ماند؟ چرا؟

۴. `load_and_parse` را با `.with_context(|| ...)` به‌جای `.context(...)` بنویس. چه زمانی نسخه‌ی lazy از نظر کارایی واقعاً مهم است و چه زمانی، مثل اینجا، فقط تفاوت سبکی دارد؟
