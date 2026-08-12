# ۰۷.۱ — thread، `Mutex` و `Arc`

## thread واقعی سیستم‌عامل

`std::thread::spawn` یک OS thread می‌سازد و closure را روی آن اجرا می‌کند. `JoinHandle::join` منتظر پایان می‌ماند و نتیجه یا panic thread را برمی‌گرداند. اگر هر thread داده‌ی دارای مالکیت مستقل بگیرد، shared state و lock لازم نیست؛ moveکردن chunkهای جدا نمونه‌ی مناسب است.

## چرا closure مربوط به `spawn` باید `'static` باشد؟

thread ممکن است پس از پایان تابع سازنده زنده بماند. بنابراین closure نمی‌تواند ارجاع کوتاه‌عمر به stack محلی نگه دارد. `move` مالکیت captureها را به thread می‌دهد؛ `'static` اینجا معمولاً یعنی داده‌ی قرضی وابسته به scope کوتاه وجود ندارد، نه اینکه داده تا پایان برنامه حتماً زنده باشد.

## اشتراک داده با `Arc<Mutex<T>>`

`Arc` مالکیت مشترک thread-safe و `Mutex` دسترسی تغییرپذیر انحصاری در runtime می‌دهد. `.lock()` یک `MutexGuard` می‌سازد و با dropشدن guard قفل خودکار آزاد می‌شود.

```senpai-visual
{"kind":"concurrency","labels":["thread ۱","Arc","Mutex<T>","thread ۲","join"]}
```

اگر thread هنگام داشتن lock panic کند، mutex poisoned می‌شود و `.lock()` مقدار `Err(PoisonError)` می‌دهد؛ `.unwrap()` در این تمرین سیاست ساده‌ی توقف است، نه سیاست مناسب همه‌ی productionها.

مثل چند صندوق با یک دفتر مرکزی است که فقط یک کلید ویرایش دارد. مرز تشبیه: lock می‌تواند contention و deadlock بسازد و باید guard را کوتاه نگه داشت.

## تمرین تو

`sum_in_threads` و `count_matching_in_threads` را پیاده کن.
