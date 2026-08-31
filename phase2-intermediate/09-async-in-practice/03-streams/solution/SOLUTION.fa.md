# راه‌حل

```rust
pub async fn sum_stream(values: Vec<i32>) -> i32 {
    let mut stream = tokio_stream::iter(values);
    let mut total = 0;
    while let Some(value) = stream.next().await {
        total += value;
    }
    total
}

pub async fn first_n_even(values: Vec<i32>, limit: usize) -> Vec<i32> {
    let mut stream = tokio_stream::iter(values)
        .filter(|n| n % 2 == 0)
        .take(limit);

    let mut kept = Vec::new();
    while let Some(value) = stream.next().await {
        kept.push(value);
    }
    kept
}
```

هر دو تابع دقیقاً همان ایدیومی را دنبال می‌کنند که «مفهوم» چند بار نشانت داد: `tokio_stream::iter` یک `Vec` معمولی را به `Stream` تبدیل می‌کند، و یک حلقه‌ی `while let Some(...) = ... .next().await` واقعاً رانده‌اش می‌کند. نکته‌ی مهم دربارهٔ `first_n_even` ترتیبِ زنجیره است: `.filter()` قبل از `.take()` می‌آید — اگر برعکس بود، `.take(limit)` را رویِ همهٔ `values` (نه فقط زوج‌ها) می‌گذاشتی و بعد فیلترشان می‌کردی، که ممکن بود کمتر از `limit` عددِ زوجِ واقعی برگرداند حتی وقتی به‌اندازه‌ی کافی تویِ `values` وجود داشت. چون هر دو آداپتور تنبل‌اند، این ترتیب هیچ هزینه‌ی اضافه‌ای هم ندارد — استریم دقیقاً همان‌قدر عنصر از `values` می‌کشد که لازم است، نه بیشتر.

## «بساز»: مقدارها در طولِ زمانِ واقعی

```rust
pub async fn ticks_after(delays_ms: Vec<u64>) -> Vec<u64> {
    let stream = tokio_stream::iter(delays_ms).then(|delay_ms| async move {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        delay_ms
    });
    tokio::pin!(stream);

    let mut ticks = Vec::new();
    while let Some(delay_ms) = stream.next().await {
        ticks.push(delay_ms);
    }
    ticks
}
```

دقیقاً همان الگویِ `03-time-spaced-stream` تویِ «مفهوم» — فقط این‌بار به‌جایِ چاپ‌کردن، هر مقدار را تویِ یک `Vec` نگه می‌داریم. تستِ `ticks_after_actually_takes_real_time` دقیقاً همان چیزی را با عدد ثابت می‌کند که «مفهوم» با چشمِ خودت دیدی: پنج تأخیرِ ۲۰میلی‌ثانیه‌ای رویِ هم دستِ‌کم ۹۰ میلی‌ثانیه طول می‌کشند — نه چیزی نزدیکِ صفر، که یعنی `sleep`ها واقعاً اتفاق افتاده‌اند، نه فقط شبیه‌سازیِ فوری.

## «چالش»: یک `Stream` دستی

```rust
pub struct CountdownStream {
    remaining: u32,
}

impl Stream for CountdownStream {
    type Item = u32;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<u32>> {
        if self.remaining == 0 {
            Poll::Ready(None)
        } else {
            self.remaining -= 1;
            Poll::Ready(Some(self.remaining))
        }
    }
}
```

هیچ `tokio_stream::iter`ی، هیچ آداپتوری در کار نیست — این دقیقاً همان چیزی است که «مفهوم» با `poll_next` نشانت داد، این‌بار با دستِ خودت نوشته شده. چون `CountdownStream` فقط یک `u32` نگه می‌دارد و هیچ ارجاعِ داخلی‌ای ندارد، خودش به‌طور خودکار `Unpin` است — دقیقاً همان دلیلی که [۲.۸.۵](../../08-concurrency/05-futures-and-runtimes/README.fa.md) برایِ `FlipOnce` هم داد؛ برایِ همین `stream.next().await` بدونِ هیچ `Box::pin` یا `tokio::pin!`ی رویِ این نوع کار می‌کند. `CountdownStream::new(3)` سه بار `Some(2)`، `Some(1)`، `Some(0)` می‌دهد و بعد `None` — چون هر `poll_next` اول چک می‌کند `remaining` صفر است یا نه، *بعد* کم می‌کند.
