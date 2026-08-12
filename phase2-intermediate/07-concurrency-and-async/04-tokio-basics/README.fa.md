# ۰۷.۴ — مبانی Tokio

## `#[tokio::main]` و `#[tokio::test]`

تابع `async fn` به executor نیاز دارد. attributeهای Tokio runtime می‌سازند و future مربوط به بدنه را تا پایان اجرا می‌کنند. `#[test] async fn` ساده معتبر نیست؛ test harness معمولی نمی‌داند future را چگونه poll کند.

## `tokio::spawn`: همتای async برای `thread::spawn`

`tokio::spawn(future)` یک task سبک روی runtime زمان‌بندی و `JoinHandle` می‌دهد. `.await` روی handle نتیجه‌ی task را می‌گیرد؛ `Err(JoinError)` می‌تواند از panic یا cancellation task بیاید. taskها معمولاً بسیار سبک‌تر از OS thread هستند.

## نکته‌ی اصلی: انتظار هم‌زمان، نه محاسبه‌ی موازی

`async` به‌تنهایی concurrency ایجاد نمی‌کند. اگر در حلقه هر future را فوری `.await` کنی، عملیات ترتیبی‌اند. باید چند future را spawn یا با ابزارهایی مانند `join!` هم‌زمان در حال پیشرفت نگه داری. سود بزرگ برای I/O-bound است: هنگام انتظار شبکه، task دیگری اجرا می‌شود. محاسبه‌ی CPU-bound طولانی runtime را block می‌کند و به thread pool مخصوص مانند `spawn_blocking` نیاز دارد.

```senpai-visual
{"kind":"async","labels":["request ۱: wait","request ۲: wait","Tokio runtime","هر دو آماده"]}
```

مثل یک نانواست که هنگام پخت سفارش اول، سفارش دوم را آماده می‌کند؛ نه دو نانوای واقعی که CPU را موازی مصرف کنند. این مرز تشبیه تفاوت concurrency و parallelism است.

## تمرین تو

توابع sleep شبیه‌سازی‌شده و `fetch_all_concurrently` را کامل کن.
