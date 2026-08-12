# ایست بازرسی

۱. اگر fetchها را در حلقه یکی‌یکی `.await` کنی، تست زمان concurrency چه نتیجه‌ای دارد و چه چیزی را ثابت می‌کند؟
۲. تبدیل `#[tokio::test]` به `#[test]` روی `async fn` چرا رد می‌شود و attribute چه زیرساختی می‌سازد؟
۳. `.await.unwrap()` روی `JoinHandle` کدام `Result` را باز می‌کند و چه چیزی `JoinError` می‌سازد؟
۴. برای resize ده‌هزار تصویر و ده‌هزار HTTP request به‌ترتیب OS thread/CPU pool یا async task را چگونه انتخاب می‌کنی؟
