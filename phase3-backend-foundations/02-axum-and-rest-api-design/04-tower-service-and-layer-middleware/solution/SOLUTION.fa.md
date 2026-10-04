# راه‌حل — ۳.۲.۴ `tower::Service` و `Layer`: میان‌افزار با دست

## `ResponseTimeLayer` و `ResponseTime`

```rust
impl<S> Layer<S> for ResponseTimeLayer {
    type Service = ResponseTime<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ResponseTime { inner }
    }
}
```

```rust
fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), S::Error>> {
    self.inner.poll_ready(cx)
}

fn call(&mut self, request: Request) -> Self::Future {
    let start = Instant::now();
    let future = self.inner.call(request);
    Box::pin(async move {
        let mut response = future.await?;
        let elapsed_ms = start.elapsed().as_millis() as u64;
        response
            .headers_mut()
            .insert("x-response-time-ms", HeaderValue::from(elapsed_ms));
        Ok(response)
    })
}
```

لایه فقط `inner` را در structِ سرویس می‌گذارد. `poll_ready` رد می‌کند، چون پوشش دقیقاً وقتی آماده است که چیزِ پیچیده‌شده آماده باشد. در `call`، ساعت وقتی شروع می‌شود که `call` صدا زده می‌شود و بعد از `future.await?` خوانده می‌شود، پس کلِ هندلر را می‌سنجد، نه فقط صدا زدنی که future را ساخت. تستِ `times_the_whole_handler_not_just_the_call` یک هندلر را ۲۰ میلی‌ثانیه می‌خواباند تا همین را ثابت کند.

`?` خطایِ درونی را دست‌نخورده برمی‌گرداند، پس در شکست هیچ هدری اضافه نمی‌شود (`passes_an_inner_error_through_unchanged`). `HeaderValue::from(u64)` ارقامِ ساده‌یِ دهدهی می‌نویسد، و `headers_mut().insert` هدرِ موجود را جایگزین می‌کند، نه اینکه یکی دیگر اضافه کند، که کارِ `append` می‌بود. اینجا چیزی کلون نمی‌شود، پس `mem::replace` لازم نیست: `call` همان اول از `inner` استفاده می‌کند و فقط futureاش را به داخلِ جعبه منتقل می‌کند.

## `MaintenanceLayer` و `Maintenance`

```rust
fn layer(&self, inner: S) -> Self::Service {
    Maintenance {
        inner,
        flag: Arc::clone(&self.flag),
    }
}
```

```rust
fn call(&mut self, request: Request) -> Self::Future {
    if self.flag.load(Ordering::SeqCst) {
        let response = Response::builder()
            .status(StatusCode::SERVICE_UNAVAILABLE)
            .body(Body::from("down for maintenance"))
            .unwrap();
        return Box::pin(async move { Ok(response) });
    }
    Box::pin(self.inner.call(request))
}
```

`Arc::clone` به هر سرویسِ پیچیده‌شده یک اشاره‌گر به همان `AtomicBool` می‌دهد، پس عوض‌کردنِ پرچم از بیرون در درخواستِ بعدیِ همه‌شان اثر می‌کند. کپی‌کردنِ خودِ bool به هر سرویس یک پرچمِ جدا می‌داد که کسی به آن دسترسی ندارد.

کوتاه‌کردنِ مسیر همان `return`ِ پیش از `self.inner.call(request)` است: وقتی پرچم روشن است `call`ِ سرویسِ درونی هرگز صدا زده نمی‌شود، پس هندلر اجرا نمی‌شود (`flag_on_never_runs_the_handler` تعدادِ اجراها را می‌شمارد). پرچم یک بار به‌ازایِ هر درخواست، در لحظه‌ی `call` خوانده می‌شود.

## درباره‌ی چالش (اختیاری)

نسخه‌ی `from_fn`ِ زمان‌سنج کوتاه‌تر است:

```rust
async fn timed(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        "x-response-time-ms",
        HeaderValue::from(start.elapsed().as_millis() as u64),
    );
    response
}
```

`Layer` ندارد، `Service` ندارد، futureِ جعبه‌ای ندارد و deriveِ `Clone` هم نیست که یادت بماند. کاری که نمی‌تواند بکند این است که نوعِ تنظیماتی با متدهایِ builderِ خودش داشته باشد، بیرون از `axum` کار کند، یا `poll_ready` را کنترل کند. برایِ `ResponseTimeLayer::new("x-took-ms")` یک فیلدِ `header: &'static str` به لایه اضافه می‌کنی، در `layer` آن را به سرویس کپی می‌کنی، و در `call` از آن استفاده می‌کنی: همان شکلِ `name` در `LogLayer`.

## این درس واقعاً درباره‌ی چه بود

هیچ‌کدام از این میان‌افزارها بلند نیست. چیزی که سخت به نظر می‌رسد این است که شکایتِ کامپایلر رویِ `.layer(...)` ظاهر می‌شود، دور از خطی که غلط است. حالا می‌توانی بخوانیشان: `Clone` همان deriveای است که یادت رفته بود، `Send` کرانِ futureِ جعبه‌ایِ توست، و `Infallible` یعنی میان‌افزارت باید جواب بدهد، نه شکست بخورد.
