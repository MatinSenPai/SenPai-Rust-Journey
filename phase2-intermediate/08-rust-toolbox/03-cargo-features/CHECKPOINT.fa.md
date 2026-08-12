# ایست بازرسی

۱. `json-export = ["dep:serde", "dep:serde_json"]` و `optional = true` را جزءبه‌جزء توضیح بده.
۲. اگر دو crate یکی feature را روشن و دیگری خاموش کند، library چند بار و با کدام featureها compile می‌شود؟
۳. چرا feature حذف‌کننده یا تغییردهنده‌ی رفتار با unification باگ طراحی است؟
۴. چرا `Report` به `cfg_attr` نیاز دارد و `cfg` روی خود struct در build بدون feature چه می‌کرد؟
۵. کدام دو فرمان هر دو جهان default و json را تست می‌کنند و plain `cargo test` چه چیزی را جا می‌اندازد؟
