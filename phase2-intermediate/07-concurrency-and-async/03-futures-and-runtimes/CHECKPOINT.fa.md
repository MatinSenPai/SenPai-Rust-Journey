# ایست بازرسی

1. در `Countdown::poll` چاپ اضافه کن. `block_on(Countdown::new(3))` چند بار poll می‌کند؟
2. چرا `block_on` آموزشی شرط `F: Unpin` دارد؟
3. busy-poll برای Countdown چه فرقی با انتظار ۲۰۰ms شبکه دارد؟ `Waker` چه چیزی را صرفه‌جویی می‌کند؟
4. فرق تابع عادی، Rust Future، Python coroutine و JavaScript Promise چیست؟
