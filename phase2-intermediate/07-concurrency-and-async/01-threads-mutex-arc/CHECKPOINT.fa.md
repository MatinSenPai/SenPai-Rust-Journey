# ایست بازرسی

۱. چرا `sum_in_threads` با وجود چند thread به `Arc` یا `Mutex` نیاز ندارد؟
۲. چرا نمی‌توان یک `Mutex<i32>` ساده را مالک هم‌زمان چند thread کرد و `Arc` چه چیزی اضافه می‌کند؟
۳. تفاوت `predicate: fn(i32) -> bool` با closure دارای capture چیست؟ آیا closureای که `threshold` می‌گیرد تبدیل به function pointer می‌شود؟
۴. `.lock()` چه زمانی به‌علت lock poisoning مقدار `Err` می‌دهد؟
۵. چرا tail expression به‌شکل `*counter.lock().unwrap()` ممکن است با عمر temporary/guard خطا بدهد و binding میانی آن را حل می‌کند؟
