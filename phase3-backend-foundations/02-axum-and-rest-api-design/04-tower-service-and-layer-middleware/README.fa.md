# ۳.۲.۴ — `tower::Service` و `Layer`: میان‌افزار با دست

## در یک نگاه

بعد از این درس می‌توانی:

- بگویی `poll_ready` و `call` هرکدام چه قولی می‌دهند، و توضیح بدهی چرا `.oneshot(...)` در [۳.۲.۱](../01-routing-handlers-extractors/README.fa.md) رویِ `Router` کار می‌کرد: `Router` یک `tower::Service` است.
- یک `Layer` و `Service`ِ آن را با دست بنویسی، با `.layer(...)` وصلش کنی، و سه خطایِ `E0277` را که `axum` وقتی این جفت قاعده‌هایش را بشکند می‌دهد بخوانی.
- پیش‌بینی کنی از بینِ دو لایه کدام اول درخواست را می‌بیند، هم با چند `.layer(...)`ِ پشت‌سرِهم و هم با `ServiceBuilder`.
- برایِ یک میان‌افزارِ مشخص بینِ `Service`ِ دست‌نویس و `axum::middleware::from_fn` انتخاب کنی.

**زمان:** حدود ۹۰ دقیقه · **پیش‌نیاز:**
[۳.۲.۱ — مسیریابی، هندلرها، اکسترکتورها](../01-routing-handlers-extractors/README.fa.md)،
[۳.۲.۳ — عملیاتِ CRUD روی کاتالوگِ انیمه (در حافظه)](../03-anime-catalog-crud-in-memory/README.fa.md)،
[۲.۸.۵ — Futureها و محیط‌هایِ اجرا](../../../phase2-intermediate/08-concurrency/05-futures-and-runtimes/README.fa.md)

---

## چرا اهمیت دارد

از ۳.۲.۱ تا حالا در هر تست `.oneshot(request)` نوشته‌ای، بدونِ اینکه بپرسی از کجا آمده. این متدِ `Router` نیست. از یک صفت (trait) می‌آید، و `Router` فقط یکی از نوع‌هایی است که آن را پیاده کرده‌اند. همین trait پشتِ `.layer(...)` هم هست، یعنی همان فراخوانی‌ای که لاگ، تایم‌اوت، احراز هویت و CORS را به یک اپِ `axum` اضافه می‌کند.

ایده را از جنگو می‌شناسی: `MIDDLEWARE` یک فهرست است و هر عضوش بعدی را در خودش می‌پیچد. هیچ‌وقت خودت پیچیدن را ننوشتی، چون یک میان‌افزار یک تابعِ کوچک بود. درسِ بعد، [۳.۲.۵ — CORS و اتصال به فرانت‌اند](../05-cors-and-frontend-integration/README.fa.md)، `CorsLayer` را دستت می‌دهد، یک لایه‌یِ آماده از `tower-http`. اگر بدونِ دانستنِ اینکه لایه چیست ازش استفاده کنی، `.layer(CorsLayer::permissive())` فقط یک وردِ دیگر است. این درس اول یک لایه را از هیچ می‌سازد، تا وقتی لایه‌ای بدرفتاری کرد، یا وقتی [۳.۸.۲ — ردیابیِ درخواست و correlation ID](../../08-error-handling-and-testing-at-scale/02-request-tracing-and-correlation-ids/README.fa.md) ازت خواست یکی بنویسی که هر درخواست را برچسب بزند، بدانی زیرِ آن فراخوانی چه خبر است.

---

## مفهوم

### `oneshot` یعنی `poll_ready` و بعد `call`

`examples/01-router-is-a-service.rs` یک درخواست را دو بار به یک `Router` می‌فرستد. بار اول روتر را با دست می‌راند. بار دوم از `oneshot` استفاده می‌کند:

```rust
let ready = ServiceExt::<Request<Body>>::ready(&mut app);
let response = ready.await.unwrap().call(request).await.unwrap();
println!("by hand:  {}", response.status());

let response = app.oneshot(request).await.unwrap();
println!("oneshot:  {}", response.status());
```

```text
by hand:  200 OK
oneshot:  200 OK
```

دو قدم، `ready` و بعد `call`، و `oneshot` هر دو را در یک متد دارد. هر دو از traitِ `Service`ِ `tower` می‌آیند. این تعریفِ واقعی است، از `tower-service` نسخه‌ی 0.3.3:

```rust
pub trait Service<Request> {
    type Response;
    type Error;
    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    fn call(&mut self, req: Request) -> Self::Future;
}
```

با چیزی که فاز ۲ بهت داد بخوانش. `Response` و `Error` و `Future` **نوعِ وابسته** هستند ([۲.۳.۵](../../../phase2-intermediate/03-traits-and-generics/05-associated-types/README.fa.md)): هر پیاده‌سازی هرکدام را یک بار پر می‌کند، مثلِ `Item` در `Iterator`. `call` پاسخ برنمی‌گرداند. یک `Future` برمی‌گرداند ([۲.۸.۵](../../../phase2-intermediate/08-concurrency/05-futures-and-runtimes/README.fa.md))، و پاسخ وقتی هست که چیزی آن future را `poll` کند و به `Ready` برسد. `Request` پارامترِ جنریکِ trait است، نه نوعِ وابسته، چون یک سرویس می‌تواند بیش از یک نوعِ درخواست بپذیرد.

پس **سرویس (service)** هر چیزی است که یک درخواست را به یک future پاسخ تبدیل کند. `Router` همین کار را می‌کند، و یک هندلر هم، بعد از اینکه `axum` پیچیدش، همین است و بس.

### `poll_ready`: الان می‌توانی درخواست بگیری؟

`call` یک `&mut self` می‌گیرد و هیچ‌وقت نمی‌گوید «شلوغم». پرسشِ «الان می‌توانم درخواست بفرستم؟» یک متدِ جداست، `poll_ready`، و راهِ نه‌گفتنِ یک سرویس همین است. تا وقتی نتواند درخواست بگیرد `Poll::Pending` برمی‌گرداند، همان `Pending`ای که در `Future::poll` دیدی. `examples/02-poll-ready-backpressure.rs` یک سرویسِ ساده را در `ConcurrencyLimit`ِ `tower` با سقفِ یک می‌پیچد، یک درخواست می‌فرستد بدونِ اینکه تمامش کند، و دوباره آمادگی می‌پرسد:

```rust
let first = svc.ready().await.unwrap().call(1);
println!("first request is in flight (its future is not awaited yet)");

let second = tokio::time::timeout(Duration::from_millis(50), svc.ready()).await;
// ... prints "still waiting" when the timeout fires ...
println!("first request finishes: {:?}", first.await.unwrap());
svc.ready().await.unwrap();
println!("now ready again");
```

```text
first request is in flight (its future is not awaited yet)
second ready() after 50 ms: still waiting
first request finishes: 1
now ready again
```

این **پس‌فشار (backpressure)** است ([۲.۸.۳](../../../phase2-intermediate/08-concurrency/03-channels-message-passing/README.fa.md) آن را به‌شکلِ کانالِ کراندارِ پر دید): سرویس به فراخواننده می‌گوید صبر کن، به‌جایِ اینکه کار را انبار کند یا شکست بخورد. قرارداد این است که فراخواننده پیش از هر `call` باید `Ready` را از `poll_ready` ببیند، و سرویس اجازه دارد اگر ندیده باشد پنیک کند. این پنیک را در «خطاهایی که خواهی دید» می‌بینی.

یک `Router` هر بار `Ready` می‌دهد، چون هندلرهایِ `axum` همیشه آماده‌اند. مستنداتِ `axum` صریح می‌گویند: `axum` انتظار دارد هیچ سرویسی در اپت به پس‌فشار اهمیت ندهد. برای همین میان‌افزارِ این درس می‌تواند `poll_ready` را فقط رد کند و دیگر به آن فکر نکند.

```senpai-visual
{"kind":"async","labels":["فراخواننده: poll_ready","سرویس: Pending، هنوز نه","سرویس فراخواننده را بیدار می‌کند","poll_ready: Ready(Ok)","فراخواننده: call(request)","سرویس یک Future برمی‌گرداند و poll می‌شود تا Ready(response)"]}
```

### `Layer` تابعی است از یک سرویس به سرویسی دیگر

یک **میان‌افزار (middleware)** یک سرویس را می‌پیچد و خودش هم سرویس است، پس پوشش‌ها روی هم سوار می‌شوند. نسخه‌ی جنگو تابعی است که `get_response` را می‌گیرد و یک callableِ تازه برمی‌گرداند. یک میان‌افزارِ زمان‌سنج به شکلِ جنگو:

```python
def timing_middleware(get_response):
    def middleware(request):
        start = time.monotonic()
        response = get_response(request)
        elapsed = int((time.monotonic() - start) * 1000)
        response["X-Response-Time-Ms"] = str(elapsed)
        return response
    return middleware
```

پایتونِ توضیحی است و جزوِ این crate نیست. دو لایه تابع دارد. `timing_middleware`ِ بیرونی یک بار، موقعِ راه‌افتادن، اجرا می‌شود و چیزی را که باید بپیچد می‌گیرد. `middleware`ی درونی برایِ هر درخواست اجرا می‌شود و همان چیزِ پیچیده‌شده را صدا می‌زند. `tower` به هرکدام یک اسم و یک trait داده. تابعِ بیرونی یک **`Layer`** است:

```rust
pub trait Layer<S> {
    type Service;

    fn layer(&self, inner: S) -> Self::Service;
}
```

تابعِ درونی همان `Service`ای است که `layer` برمی‌گرداند، با `inner` که در یک فیلد نگه داشته می‌شود، همان‌طور که کلوژرِ پایتون `get_response` را نگه می‌دارد. یک `Layer` یک کارخانه با تنظیمات است، و هر سرویسی که می‌سازد یک `inner`ِ پیچیده‌شده نگه می‌دارد. برایِ همین `Router::layer(...)` یک لایه می‌گیرد نه یک سرویس: هر مسیر را در نسخه‌ی خودش می‌پیچد.

جایی که تصویرِ جنگو دقیق نیست: میان‌افزارِ جنگو یک تابع است که هر دو کار را می‌کند، و `get_response`اش همگام است. در `tower` دو کار به دو نوع تقسیم شده. هرچه در تابعِ درونی هست ناهمگام است، پس `call` یک future برمی‌گرداند. و پایتون `poll_ready` ندارد، چون یک callableِ WSGI راهی برایِ گفتنِ «الان نه» ندارد.

### همان چیز، با دست: `Log`

`src/lib.rs` یک لایه‌یِ کامل دارد، `LogLayer`، که وقتی درخواستی وارد می‌شود یک خط چاپ می‌کند و وقتی پاسخش بیرون می‌آید یک خط. خودِ لایه بخشِ کوتاه است:

```rust
#[derive(Clone)]
pub struct LogLayer {
    name: &'static str,
}

impl<S> Layer<S> for LogLayer {
    type Service = Log<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Log { inner, name: self.name }
    }
}
```

سرویس بخشِ بلند است. اول structای که لایه می‌سازد، که `inner` را همان‌طور نگه می‌دارد که کلوژرِ پایتون `get_response` را:

```rust
#[derive(Clone)]
pub struct Log<S> {
    inner: S,
    name: &'static str,
}
```

بعد پیاده‌سازیِ `Service`اش، با شروع از نوع‌ها و `poll_ready`:

```rust
impl<S> Service<Request> for Log<S>
where
    S: Service<Request, Response = Response>,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), S::Error>> {
        self.inner.poll_ready(cx)
    }
```

`poll_ready` فقط رد می‌کند: پوشش دقیقاً وقتی آماده است که چیزِ پیچیده‌شده آماده باشد. `Error = S::Error` یعنی پوشش راهِ شکستِ تازه‌ای اضافه نمی‌کند. بعد `call`:

```rust
    fn call(&mut self, request: Request) -> Self::Future {
        let name = self.name;
        println!("{name}: request in  ({} {})", request.method(), request.uri());
        let future = self.inner.call(request);
        Box::pin(async move {
            let response = future.await?;
            println!("{name}: response out ({})", response.status());
            Ok(response)
        })
    }
}
```

هنوز چیزی اجرا نشده: `call` همان لحظه «request in» را چاپ کرد، درخواست را پایین داد، و future درونی را گرفت. چیزی که برمی‌گرداند یک futureِ تازه است که منتظرِ درونی می‌ماند و بعد «response out» را چاپ می‌کند. `examples/03-log-layers-onion.rs` از آن استفاده می‌کند:

```text
B: request in  (GET /)
A: request in  (GET /)
A: response out (200 OK)
B: response out (200 OK)
```

### چرا future جعبه‌ای است

`type Future = Pin<Box<dyn Future<Output = ...> + Send>>` قسمتی است که سنگین به نظر می‌رسد. دلیلش این است که بلوکِ `async` در `call` نوعی دارد که کامپایلر می‌سازد و هیچ‌کس نمی‌تواند بنویسدش، دقیقاً مثلِ نوعِ یک کلوژر ([۲.۳.۷](../../../phase2-intermediate/03-traits-and-generics/07-static-vs-dynamic-dispatch/README.fa.md)). یک نوعِ وابسته یک اسم لازم دارد. دو راهِ اسم‌گرفتن هست.

می‌توانی یک structِ futureِ نام‌دار بنویسی، با `poll` و pinِ خودش ([۲.۸.۵](../../../phase2-intermediate/08-concurrency/05-futures-and-runtimes/README.fa.md) این کار را برای `FlipOnce` کرد). تخصیصِ هیپ ندارد و کدِ زیادی می‌خواهد. یا future را پشتِ یک شیءِ صفتی بگذاری، `Pin<Box<dyn Future<...>>>`: همان پاک‌کردنِ نوعِ ۲.۳.۷، با یک تخصیصِ هیپ به‌ازایِ هر درخواست و یک `Box::pin(async move { ... })`ِ راحت. این درس جعبه می‌کند. کران‌ها از جایی می‌آیند که future زندگی می‌کند. `+ Send` هست چون یک runtimeِ چندریسمانیِ `tokio` تسک را بینِ دو `poll` بین ریسمان‌ها جابه‌جا می‌کند ([۲.۸.۴](../../../phase2-intermediate/08-concurrency/04-send-and-sync/README.fa.md)). `S::Future: Send + 'static` هست چون futureِ درونی به داخلِ جعبه منتقل می‌شود.

### `.layer(...)` چه می‌خواهد

`Router::layer`ِ `axum` نسخه‌ی 0.8.9 این امضا را دارد (از سورسِ resolveشده، `src/routing/mod.rs`):

```rust
pub fn layer<L>(self, layer: L) -> Router<S>
where
    L: Layer<Route> + Clone + Send + Sync + 'static,
    L::Service: Service<Request> + Clone + Send + Sync + 'static,
    <L::Service as Service<Request>>::Response: IntoResponse + 'static,
    <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
    <L::Service as Service<Request>>::Future: Send + 'static,
```

هر خط یک قاعده است که می‌شود شکستش، و در «خطاهایی که خواهی دید» سه‌تایش را می‌شکنی. `Clone` هست چون `axum` سرویس‌ها را کلون می‌کند: یکی به‌ازایِ هر مسیر، و یکی به‌ازایِ هر اتصال. `Error: Into<Infallible>` یعنی میان‌افزاری که بتواند شکست بخورد اینجا مجاز نیست: باید هر شکست را به یک پاسخ تبدیل کند، چون وگرنه `hyper` اتصال را بدونِ جواب می‌بندد. سمتِ `Router`ِ تصویر هم همین trait است: `Route`، نوعی که `L` می‌پیچدش، خودش یک `Service<Request, Error = Infallible>` است.

### تله‌یِ کلون، و `std::mem::replace`

`Log` همان اول `self.inner.call(request)` را داخلِ `call` صدا زد و فقط future را در جعبه گذاشت. این ساده‌ترین راهِ نوشتنِ یک پوشش است و از یک تله فرار می‌کند. تله وقتی پیش می‌آید که میان‌افزار لازم دارد `inner` را داخلِ بلوکِ `async`، بعد از یک `.await`، صدا بزند. بلوکِ `async` باید مالکِ `inner` باشد، و حرکتِ بدیهی `self.inner.clone()` است. ولی سرویسی که `Ready` داد `self.inner` بود، و کلونِ یک سرویس تضمین نمی‌شود آماده باشد، حتی وقتی اصلی آماده است. `ConcurrencyLimit` روشن‌ترین نمونه است: کلون بدونِ permit شروع می‌کند. مستنداتِ خودِ `tower` راه‌حل را نشان می‌دهد. سرویسِ آماده را نگه دار، و کلونِ تازه را در `self` جا بگذار:

```rust
fn call(&mut self, request: u32) -> Self::Future {
    let clone = self.inner.clone();
    let mut inner = std::mem::replace(&mut self.inner, clone);
    Box::pin(async move { inner.call(request).await })
}
```

```text
answer = 7
```

`mem::replace` مقدارِ `self.inner` را با کلون عوض می‌کند و اصلی را بهت می‌دهد، پس future مالکِ سرویسی است که آماده اعلام شده بود و `self` یک کلون نگه می‌دارد که دفعه‌ی بعد `poll_ready` رویش می‌آید. این الگو فقط وقتی لازم است که `call` بعد از یک `.await` از `inner` استفاده کند. «خطاهایی که خواهی دید» نشان می‌دهد بدونِ آن چه می‌شود.

### پیازِ لایه‌ها: کدام لایه اول می‌آید

هر `.layer(L)` همه‌چیزِ موجود را می‌پیچد، پس لایه‌ای که آخر اضافه شود بیرونی‌ترین است: درخواست را اول و پاسخ را آخر می‌بیند. `examples/03-log-layers-onion.rs` همان `.layer(LogLayer::new("A")).layer(LogLayer::new("B"))` است، و خروجیِ بالا نشان می‌دهد B اول وارد می‌شود.

```senpai-visual
{"kind":"concept","labels":["درخواست به بیرونی‌ترین لایه می‌رسد: B","B آن را به A می‌دهد","A آن را به Router و هندلر می‌دهد","هندلر پاسخ را برمی‌گرداند","پاسخ از A بیرون می‌رود","بعد از B بیرون می‌رود"]}
```

`tower::ServiceBuilder` برعکس می‌خواند. لایه‌هایش را طوری می‌چیند که اولی که نوشته‌ای بیرونی‌ترین باشد، یعنی بالا به پایین مثلِ فهرستِ `MIDDLEWARE`. `examples/04-service-builder-order.rs` اول A و بعد B را می‌نویسد:

```text
A: request in  (GET /)
B: request in  (GET /)
B: response out (200 OK)
A: response out (200 OK)
```

مستنداتِ `axum` برایِ چند لایه `ServiceBuilder` را توصیه می‌کنند، دقیقاً به همین دلیل. یک قاعده‌ی دیگر از همان مستندات را هم یادت بماند: `.layer` فقط مسیرهایی را می‌پیچد که پیش از آن اضافه شده‌اند. مسیری که بعدش اضافه کنی پیچیده نمی‌شود.

### میان‌بر، و جایی که نباید سراغش بروی

`axum::middleware::from_fn` کارِ Log را در چند خط انجام می‌دهد (`examples/05-from-fn-version.rs`):

```rust
async fn log(request: Request, next: Next) -> Response {
    println!("fn: request in  ({} {})", request.method(), request.uri());
    let response = next.run(request).await;
    println!("fn: response out ({})", response.status());
    response
}
// ...
Router::new().route("/", get(|| async { "hello" })).layer(from_fn(log))
```

```text
fn: request in  (GET /)
fn: response out (200 OK)
```

`from_fn` یک `Layer`ِ آماده است: futureِ جعبه‌ای و پیاده‌سازیِ `Clone` را خودش می‌سازد، و `next.run(request)` همان صدا زدنِ `inner` است. برایِ میان‌افزاری که مالِ اپِ خودت است ازش استفاده کن. `Service` را با دست وقتی بنویس که:

- میان‌افزار باید بیرون از `axum` هم کار کند، یا به‌شکلِ یک crate منتشر شود، چون میان‌افزارِ `from_fn` فقط با `axum` کار می‌کند؛
- نوعِ تنظیماتِ خودش را لازم دارد، با متدهایِ builder رویِ لایه (شکلِ `TraceLayer` و `CorsLayer`)؛
- باید خودش `poll_ready` را کنترل کند، که یک تابعِ `from_fn` هرگز نمی‌بیندش؛
- یک تخصیص به‌ازایِ هر درخواست مهم است، و حاضری یک futureِ نام‌دار بنویسی.

یک چیز که نسخه‌ی دست‌نویس همین حالا نشان می‌دهد و درسِ بعد رویش تکیه می‌کند: یک میان‌افزار می‌تواند بدونِ صدا زدنِ `inner` جواب بدهد. خودش یک پاسخ برمی‌گرداند و بقیه‌ی پیاز اصلاً اجرا نمی‌شود. `CorsLayer` برایِ یک درخواستِ preflightِ `OPTIONS` همین کار را می‌کند. تمرینِ «بساز» همین را از تو می‌خواهد.

---

## دست‌به‌کد

```sh
cargo run -p p3-02-04-tower-service-and-layer-middleware --example 01-router-is-a-service
cargo run -p p3-02-04-tower-service-and-layer-middleware --example 02-poll-ready-backpressure
cargo run -p p3-02-04-tower-service-and-layer-middleware --example 03-log-layers-onion
cargo run -p p3-02-04-tower-service-and-layer-middleware --example 04-service-builder-order
cargo run -p p3-02-04-tower-service-and-layer-middleware --example 05-from-fn-version
```

بعد چهار مثالِ خراب. `06` یک پنیکِ زمانِ اجراست، سه‌تایِ دیگر کامپایل نمی‌شوند:

```sh
cargo run -p p3-02-04-tower-service-and-layer-middleware --example 06-clone-without-ready-broken --features broken
cargo build -p p3-02-04-tower-service-and-layer-middleware --example 07-future-not-send-broken --features broken
cargo build -p p3-02-04-tower-service-and-layer-middleware --example 08-service-not-clone-broken --features broken
cargo build -p p3-02-04-tower-service-and-layer-middleware --example 09-error-not-infallible-broken --features broken
```

بعد این‌ها را امتحان کن:

۱. در `03-log-layers-onion` بعد از `B` یک لایه‌یِ سوم `C` با `.layer(LogLayer::new("C"))` اضافه کن. `C` کجا چاپ می‌شود؟
۲. در `04-service-builder-order` دو خطِ `.layer(...)` را جابه‌جا کن. چه چیزی عوض شد، و چه چیزی نه؟
۳. در `02-poll-ready-backpressure` سقف را از `1` به `2` ببر. حالا `ready()`ِ دوم چه می‌کند؟

---

## خطاهایی که خواهی دید

هر رونوشتِ زیر خروجیِ واقعیِ مثالی است که اسمش آمده، بدونِ هشدارهایِ `todo!()`ِ همین درس.

### `E0277` — futureِ جعبه‌ای که `Send` نیست

```text
error[E0277]: `(dyn Future<Output = Result<Response<Body>, Infallible>> + 'static)` cannot be sent between threads safely
    --> phase3-backend-foundations\02-axum-and-rest-api-design\04-tower-service-and-layer-middleware\examples\07-future-not-send-broken.rs:54:16
     |
  54 |         .layer(NoopLayer);
     |          ----- ^^^^^^^^^ `(dyn Future<Output = Result<Response<Body>, Infallible>> + 'static)` cannot be sent between threads safely
     |          |
     |          required by a bound introduced by this call
     |
     = help: the trait `Send` is not implemented for `(dyn Future<Output = Result<Response<Body>, Infallible>> + 'static)`
     = note: required for `std::ptr::Unique<(dyn Future<Output = Result<Response<Body>, Infallible>> + 'static)>` to implement `Send`
note: required because it appears within the type `Box<(dyn Future<Output = Result<Response<Body>, Infallible>> + 'static)>`
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\alloc\src\boxed.rs:234:12
     |
 234 | pub struct Box<
     |            ^^^
note: required because it appears within the type `Pin<Box<(dyn Future<Output = Result<Response<Body>, Infallible>> + 'static)>>`
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\pin.rs:1092:12
     |
1092 | pub struct Pin<Ptr> {
     |            ^^^
note: required by a bound in `Router::<S>::layer`
    --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:309:51
     |
 303 |     pub fn layer<L>(self, layer: L) -> Router<S>
     |            ----- required by a bound in this associated function
...
 309 |         <L::Service as Service<Request>>::Future: Send + 'static,
     |                                                   ^^^^ required by this bound in `Router::<S>::layer`

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-02-04-tower-service-and-layer-middleware` (example "07-future-not-send-broken") due to 1 previous error
```

**کامپایلر به چه ایراد می‌گیرد:** خطا رویِ `.layer(...)` گزارش شده، نه رویِ خطی که نوع را نوشتی، و خطِ اول نوعی را اسم می‌برد که هرگز ننوشته‌ای. همان `Pin<Box<dyn Future<...>>>`ی توست که `dyn Future`اش `+ Send` ندارد. آخرین `note:` قاعده‌ی اصلی است: `Router::layer` شرط می‌گذارد `Future: Send + 'static`.

**راه‌حل:** کران را به نوعِ جعبه‌ای اضافه کن:

```rust
type Future = Pin<Box<dyn Future<Output = Result<Response, S::Error>> + Send>>;
```

**چرا این راه‌حل است:** `dyn Future` نوعِ ملموس را پاک می‌کند، و همراهش این دانستن که نوع `Send` است ([۲.۸.۴](../../../phase2-intermediate/08-concurrency/04-send-and-sync/README.fa.md)). پاک‌کردن فقط چیزی را نگه می‌دارد که فهرست کرده‌ای. نوشتنِ `+ Send` راهِ قول‌دادنِ آن است. و آن قول باید راست باشد: بلوکِ `async` داخلِ `call` فقط باید چیزهایِ `Send` را نگه دارد، و برایِ همین `Log` مقدارِ `name` را از `self` بیرون کپی می‌کند به‌جایِ اینکه `&self` را بگیرد.

### `E0277` — سرویس `Clone` نیست

```text
error[E0277]: the trait bound `Noop<Route>: Clone` is not satisfied
   --> phase3-backend-foundations\02-axum-and-rest-api-design\04-tower-service-and-layer-middleware\examples\08-service-not-clone-broken.rs:53:16
    |
 53 |         .layer(NoopLayer);
    |          ----- ^^^^^^^^^ the trait `Clone` is not implemented for `Noop<Route>`
    |          |
    |          required by a bound introduced by this call
    |
note: required by a bound in `Router::<S>::layer`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:306:40
    |
303 |     pub fn layer<L>(self, layer: L) -> Router<S>
    |            ----- required by a bound in this associated function
...
306 |         L::Service: Service<Request> + Clone + Send + Sync + 'static,
    |                                        ^^^^^ required by this bound in `Router::<S>::layer`
help: consider annotating `Noop<Route>` with `#[derive(Clone)]`
    |
 28 + #[derive(Clone)]
 29 | struct Noop<S> {
    |

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-02-04-tower-service-and-layer-middleware` (example "08-service-not-clone-broken") due to 1 previous error
```

**کامپایلر به چه ایراد می‌گیرد:** `Noop<Route>` سرویسی است که لایه‌ات ساخته، با `inner`ای که با `Route`ِ `axum` پر شده. `Router::layer` می‌خواهد آن `Clone` باشد، و structِ تو derive ندارد.

**راه‌حل:** کامپایلر خودش نوشته: `#[derive(Clone)]` رویِ structِ سرویس.

**چرا این راه‌حل است:** `axum` دائم سرویس‌ها را کلون می‌کند: یک نسخه به‌ازایِ هر مسیر، و یکی به‌ازایِ هر اتصال. derive کار می‌کند چون `Route` هم `Clone` است و فیلدهایِ دیگرت هم. `Log` یک `&'static str` دارد که `Copy` است. اگر فیلدی `Clone` نیست، آن را در `Arc` بپیچ و `Arc` را کلون کن، همان‌طور که `MaintenanceLayer` در «بساز» می‌کند.

### `E0277` — نوعِ خطا `Infallible` نیست

```text
error[E0277]: the trait bound `Infallible: From<String>` is not satisfied
   --> phase3-backend-foundations\02-axum-and-rest-api-design\04-tower-service-and-layer-middleware\examples\09-error-not-infallible-broken.rs:55:16
    |
 55 |         .layer(NoopLayer);
    |          ----- ^^^^^^^^^ the trait `From<String>` is not implemented for `Infallible`
    |          |
    |          required by a bound introduced by this call
    |
help: the trait `From<String>` is not implemented for `Infallible`
      but trait `From<!>` is implemented for it
   --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\convert\mod.rs:988:1
    |
988 | impl const From<!> for Infallible {
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    = help: for that trait implementation, expected `!`, found `String`
    = note: required for `String` to implement `Into<Infallible>`
note: required by a bound in `Router::<S>::layer`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:308:50
    |
303 |     pub fn layer<L>(self, layer: L) -> Router<S>
    |            ----- required by a bound in this associated function
...
308 |         <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
    |                                                  ^^^^^^^^^^^^^^^^ required by this bound in `Router::<S>::layer`

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-02-04-tower-service-and-layer-middleware` (example "09-error-not-infallible-broken") due to 1 previous error
```

**کامپایلر به چه ایراد می‌گیرد:** میان‌افزار `type Error = String` اعلام کرده. `Router::layer` شرط می‌گذارد `Error: Into<Infallible>`، و `help:` می‌گوید چرا شکست می‌خورد: هیچ‌چیز به `Infallible` تبدیل نمی‌شود جز نوعِ هرگز `!`.

**راه‌حل:** خطایِ میان‌افزار را همان خطایِ سرویسِ درونی کن، `type Error = S::Error`، و هر شکستِ خودت را به‌جایِ یک `Err` به یک پاسخ (`500` یا `503`) تبدیل کن.

**چرا این راه‌حل است:** `Infallible` نوعِ خطایِ یک `Router` است چون هر هندلر باید پاسخ بسازد. مستنداتِ `axum` دلیلش را می‌گویند: اگر میان‌افزاری خطا برگرداند، `hyper` اتصال را بدونِ فرستادنِ چیزی می‌بندد. اگر واقعاً خطایِ سفارشی لازم داری، `axum` یک `HandleErrorLayer` دارد که آن را به پاسخ تبدیل کند، ولی «هرگز شکست نخور، همیشه جواب بده» پیش‌فرضی است که باید ترجیحش بدهی.

### یک پنیکِ زمانِ اجرا: صدا زدنِ کلونی که هرگز آماده نشده

```text
thread 'main' (6552) panicked at C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\tower-0.5.3\src\limit\concurrency\service.rs:86:14:
max requests in-flight; poll_ready must be called first
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(عددِ داخلِ پرانتز شناسه‌ی ریسمان است و هر بار عوض می‌شود.)

**واقعاً چه چیزی خراب است:** `examples/06-clone-without-ready-broken.rs` رویِ `self.inner` تابعِ `poll_ready` را صدا می‌زند، بعد رویِ `self.inner.clone()` تابعِ `call` را. کامپایلر راضی است: هر دو یک نوع‌اند. ولی `ConcurrencyLimit` در `poll_ready` یک permit رزرو می‌کند و در `call` آن را برمی‌دارد، و یک کلونِ تازه permit ندارد، پس `call`اش به `expect(...)` می‌خورد. مستنداتِ `Service` مستقیم می‌گویند: یک سرویس اجازه دارد اگر `call` بدونِ `Ready` از `poll_ready` صدا زده شود پنیک کند.

**راه‌حل:** کلون را جایگزین کن و سرویسِ آماده را نگه دار، همان الگوی «مفهوم»:

```rust
let clone = self.inner.clone();
let mut inner = std::mem::replace(&mut self.inner, clone);
```

**چرا این راه‌حل است:** سرویسی که `call` می‌شود باید همان باشد که `Ready` جواب داده. بعد از جابه‌جایی، `inner` همان است، و `self.inner` کلون است که `poll_ready` پیش از درخواستِ بعدی بررسی می‌کند. با جابه‌جایی، مثال `answer = 7` چاپ می‌کند.

---

## تمرین

### گرم‌کردن

<details>
<summary><code>poll_ready</code> به چه پرسشی جواب می‌دهد، و <code>Poll::Pending</code> از آن یعنی چه؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

«الان می‌توانی درخواست بگیری؟» `Pending` یعنی نه: سرویس پر است، و فراخواننده باید صبر کند تا بیدار شود و دوباره بپرسد. اگر با این حال `call` را صدا بزنی، سرویس اجازه دارد پنیک کند.

</details>

<details>
<summary>یک <code>Router</code> با <code>.layer(A).layer(B)</code> داریم. کدام لایه اول درخواست را می‌بیند، و کدام آخر پاسخ را؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

`B` اول درخواست را می‌بیند، چون آخرین لایه‌یِ اضافه‌شده بیرونی‌ترین است. `B` آخر پاسخ را هم می‌بیند، چون پاسخ از همان لایه‌ها به ترتیبِ برعکس بیرون می‌آید.

</details>

<details>
<summary>در <code>timing_middleware</code>ِ جنگویِ بالا، کدام تابع <code>Layer</code> است و کدام <code>Service</code>؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

`timing_middleware(get_response)`ِ بیرونی همان `Layer` است: یک بار اجرا می‌شود و پوشش را از چیزی که می‌پیچد می‌سازد. `middleware(request)`ِ درونی همان `Service` است: برایِ هر درخواست اجرا می‌شود، و `get_response` همان `inner`ِ آن است.

</details>

<details>
<summary><code>ServiceBuilder::new().layer(A).layer(B)</code> با <code>.layer(A).layer(B)</code> رویِ روتر چه فرقی دارد؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

`ServiceBuilder` بالا به پایین می‌خواند، پس `A` بیرونی‌ترین است و درخواست را اول می‌بیند. چند `.layer`ِ پشتِ‌هم رویِ روتر آخری را بیرونی‌ترین می‌کند.

</details>

### تعمیر

هر چهار مثالِ خراب را درست کن:

۱. `examples/06-clone-without-ready-broken.rs` به‌جایِ پنیک `answer = 7` چاپ کند.
۲. `examples/07-future-not-send-broken.rs` کامپایل شود.
۳. `examples/08-service-not-clone-broken.rs` کامپایل شود.
۴. `examples/09-error-not-infallible-broken.rs` کامپایل شود، و حالتِ شکستش به‌جایِ یک `Err` به یک پاسخ تبدیل شود.

### پیاده‌سازی

`ResponseTimeLayer` و `ResponseTime` در `src/lib.rs`: میان‌افزاری که رویِ هر پاسخ یک هدرِ `x-response-time-ms` می‌زند.

```sh
cargo test -p p3-02-04-tower-service-and-layer-middleware --test response_time_test
```

کامنتِ مستنداتِ بالایِ `ResponseTime` کلِ مشخصات است: هدر چه دارد، اگر از قبل بوده چه می‌شود، و چه چیزی نباید عوض شود. هیچ‌وقت لازم نیست تست‌ها را بخوانی. `Log`ِ بالاتر از آن یک نمونه‌ی کاملِ همین شکل است. تست‌ها حضور و قالبِ هدر را بررسی می‌کنند، نه مقدارش را، چون زمان‌ها هر بار فرق دارند. تست‌ها درخواست‌ها را با `oneshot` می‌فرستند، مثلِ ۳.۲.۱.

### بساز

`MaintenanceLayer` و `Maintenance`، در همان فایل: میان‌افزاری که تا وقتی یک پرچمِ مشترک روشن است `503 Service Unavailable` جواب می‌دهد، بدونِ اینکه اصلاً سرویسِ درونی را صدا بزند.

```sh
cargo test -p p3-02-04-tower-service-and-layer-middleware --test maintenance_test
```

باز هم کامنتِ مستندات مشخصات است. نکته‌ی تمرین همان کوتاه‌کردنِ مسیر است: وقتی پرچم روشن است هندلرِ درونی نباید اجرا شود، که یک تست با شمردنِ تعدادِ اجرایش بررسی می‌کند. همان حرکتی است که `CorsLayer` برایِ یک درخواستِ preflight در [۳.۲.۵](../05-cors-and-frontend-integration/README.fa.md) می‌کند. پرچم یک `Arc<AtomicBool>` است ([۲.۸.۲](../../../phase2-intermediate/08-concurrency/02-rwlock-semaphore-oncelock-atomics/README.fa.md))، پس می‌توانی وقتی اپ در حالِ کار است از بیرون عوضش کنی.

### چالش (اختیاری)

میان‌افزارِ زمان‌سنج را بار دوم با `axum::middleware::from_fn`، در یک فایلِ موقتِ خودت بنویس و مقایسه کن: هرکدام چند خط شد، و چه چیزی را در یکی می‌توانی تنظیم کنی و در دیگری نه. بعد به `ResponseTimeLayer` یک فیلد بده، تا `ResponseTimeLayer::new("x-took-ms")` نامِ هدر را انتخاب کند. هیچ تستی این یکی را بررسی نمی‌کند. با یک درخواستِ `oneshot`ِ خودت امتحانش کن.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| `tower::Service` | یک trait: `poll_ready` به‌علاوه‌ی `call` که یک future برمی‌گرداند | `Router`، هندلرها و میان‌افزارها همه سرویس‌اند |
| `poll_ready` | می‌پرسد «الان می‌توانی درخواست بگیری؟»؛ `Pending` یعنی صبر کن | پس‌فشار، و رد‌کردن به `inner` در هر پوشش |
| `Layer` | کارخانه‌ای که یک سرویس را به سرویسِ پیچنده تبدیل می‌کند | `.layer(...)`، `CorsLayer`، `TraceLayer` |
| میان‌افزار | سرویسی که یک سرویسِ درونی نگه می‌دارد و صدایش می‌زند | لاگ، زمان‌سنجی، احراز هویت، CORS |
| پیازِ لایه‌ها | هر لایه همه‌یِ قبلی‌ها را می‌پیچد؛ آخری بیرونی‌ترین است | پیش‌بینیِ ترتیبِ میان‌افزارها |
| `ServiceBuilder` | لایه‌ها را طوری می‌چیند که اولی بیرونی‌ترین باشد | اعمالِ چند لایه با هم |
| `from_fn` | لایه‌یِ آماده‌یِ `axum` که از یک `async fn` ساخته می‌شود | میان‌افزارِ مخصوصِ اپ |
| کوتاه‌کردنِ مسیر | میان‌افزار خودش جواب می‌دهد و `inner` را صدا نمی‌زند | حالتِ تعمیر، preflightِ CORS |

### الان می‌دانی

- `oneshot` همان `poll_ready` به‌علاوه‌یِ `call` بود، و `Router` سرویسی است که `Error`اش `Infallible` است.
- `poll_ready` پس‌فشار است، و فراخواننده باید پیش از هر `call` از آن `Ready` ببیند.
- `Layer` تابعی است از یک سرویس به یک سرویسِ پیچنده، همان شکلِ میان‌افزارِ جنگو، تقسیم‌شده به یک کارخانه و یک نمونه.
- یک میان‌افزارِ دست‌نویس برایِ گذشتن از `Router::layer` به `Clone`، یک futureِ `Send + 'static` (معمولاً جعبه‌ای) و `Error = Infallible` نیاز دارد.
- وقتی `call` باید بعد از یک `.await` از `inner` استفاده کند، با `std::mem::replace` جابه‌جا کن، نه با یک کلونِ لخت.
- آخرین `.layer(...)` بیرونی‌ترین است، `ServiceBuilder` بالا به پایین می‌خواند، و یک میان‌افزار می‌تواند بدونِ صدا زدنِ `inner` جواب بدهد.

### بعداً کامل‌تر می‌بینی

- **`CorsLayer`، یک `Layer`ِ آماده، و کوتاه‌کردنِ مسیرِ preflight** — [۳.۲.۵ — CORS و اتصال به فرانت‌اند](../05-cors-and-frontend-integration/README.fa.md)
- **میان‌افزاری که هر درخواست را با یک ID برچسب می‌زند** — [۳.۸.۲ — ردیابیِ درخواست و correlation ID](../../08-error-handling-and-testing-at-scale/02-request-tracing-and-correlation-ids/README.fa.md)
- **میان‌افزاری که درخواستِ احرازنشده را رد می‌کند** — [۳.۷.۳ — JWT و میان‌افزار در `tower`](../../07-auth-and-security/03-jwt-and-tower-middleware/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا `.oneshot(request)` رویِ یک `Router` کار می‌کند، و دو قدمِ داخلش چیست؟
- `poll_ready` از چه محافظت می‌کند، و چرا کلونِ یک سرویسِ آماده تضمین نمی‌شود آماده باشد؟
- میان‌افزارِ جنگو چطور شبیهِ یک `Layer` به‌علاوه‌یِ یک `Service` است، و مقایسه کجا دیگر دقیق نیست؟
- چرا future در یک میان‌افزارِ دست‌نویس معمولاً `Pin<Box<dyn Future<...> + Send>>` است، و وقتی یک تکه را جا بیندازی کدام یک از سه خطایِ `E0277` را می‌گیری؟
- با `.layer(A).layer(B)`، و بعد `ServiceBuilder` با `A` و `B`، در هرکدام کدام لایه اول درخواست را می‌بیند؟
- چه وقت `Service` را با دست می‌نویسی به‌جایِ `from_fn`؟

---

## بیشتر

- [راهنمایِ tower: ساختنِ یک میان‌افزار از صفر](https://github.com/tower-rs/tower/blob/master/guides/building-a-middleware-from-scratch.md): همین میان‌افزار، با futureِ نام‌دار به‌جایِ جعبه‌ای.
- [`tower::Service` در docs.rs](https://docs.rs/tower-service/0.3.3/tower_service/trait.Service.html): trait با مستنداتِ کاملش درباره‌یِ پس‌فشار و کلون‌کردنِ سرویس‌هایِ درونی.
- [`axum::middleware`](https://docs.rs/axum/0.8.9/axum/middleware/index.html): راه‌هایِ نوشتنِ میان‌افزار در `axum`، ترتیب، و پس‌فشار.
- [`tower::ServiceBuilder`](https://docs.rs/tower/0.5.3/tower/struct.ServiceBuilder.html): لایه‌هایش چطور ترکیب می‌شوند، با قاعده‌ی ترتیب.
