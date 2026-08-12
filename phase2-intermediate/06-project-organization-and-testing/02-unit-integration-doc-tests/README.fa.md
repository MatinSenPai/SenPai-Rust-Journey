# ۰۶.۲ — unit test، integration test و doc test

## unit test: داخل crate و آگاه از جزئیات private

module معمول `#[cfg(test)] mod tests` کنار کد قرار می‌گیرد. چون بخشی از همان crate است، می‌تواند تابع‌های private را هم مستقیم آزمایش کند. `#[cfg(test)]` باعث می‌شود این کد فقط در build تست compile شود.

## integration test: یک crate جدا و فقط API عمومی

هر فایل `tests/*.rs` مانند مصرف‌کننده‌ای بیرونی compile می‌شود و فقط itemهای `pub` را می‌بیند. این تست‌ها خطاهای واقعی API—re-export فراموش‌شده، visibility اشتباه یا قرارداد ناقص—را پیدا می‌کنند که unit test داخلی ممکن است پنهان کند.

## doc test: مثالی که همیشه باید درست بماند

code fenceهای `rust` در doc comment با `cargo test` compile و اجرا می‌شوند. خط‌های آغازشده با `# ` در تست حضور دارند اما در مستند تولیدشده پنهان‌اند؛ برای import و setup لازم عالی‌اند.

```senpai-visual
{"kind":"concept","labels":["unit: داخل crate","integration: مصرف‌کننده","doc: مثال قابل اجرا"]}
```

این سه لایه مثل بررسی قطعه، مونتاژ کامل و راهنمای مشتری‌اند. مرز تشبیه: هر سه کد واقعی compileشده‌اند و جای یکدیگر را کامل نمی‌گیرند.

## تمرین تو

تابع تبدیل دما، helper خصوصی، unit test، integration test و doc example را کامل کن.
