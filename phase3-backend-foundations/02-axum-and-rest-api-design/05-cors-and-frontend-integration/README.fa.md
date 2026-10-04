# ۳.۲.۵ — CORS و اتصال به فرانت‌اند

## در یک نگاه

بعد از این درس می‌توانی:

- توضیح بدهی چرا یک API کاملاً سالم در کنسولِ مرورگر با «blocked by CORS policy» دیده می‌شود، و بگویی کدام طرف (مرورگر یا سرور) این قاعده را اعمال می‌کند.
- یک درخواستِ پیش‌پروازِ `OPTIONS` را توصیف کنی، بگویی مرورگر کِی آن را می‌فرستد، و با یک تست نشان بدهی که هندلرِ تو هیچ‌وقت آن را نمی‌بیند.
- یک `CorsLayer` برایِ توسعه و یکی برایِ تولید بسازی، و بدانی `tower-http` کدام پیکربندی را اصلاً نمی‌سازد.
- یک سیاستِ CORS را با `oneshot` تست کنی و یک پیش‌پرواز را با `curl` شبیه‌سازی کنی، بدونِ هیچ مرورگری.

**زمان:** حدود ۶۰ دقیقه · **پیش‌نیازها:**
[۳.۲.۴ — `tower::Service` و `Layer`: میان‌افزار با دست](../04-tower-service-and-layer-middleware/README.fa.md)،
[۳.۲.۱ — مسیریابی، هندلرها، اکسترکتورها](../01-routing-handlers-extractors/README.fa.md)،
[۳.۱.۳ — چیزهایی از HTTP که باید بدانی](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.fa.md)

---

## چرا اهمیت دارد

هر چیزی که در این ماژول ساخته‌ای تست‌هایش پاس می‌شود و به `curl` جواب می‌دهد. بعد یک نفر یک صفحه‌ی React یا Vite را به آن وصل می‌کند و کنسولِ مرورگر پر از قرمز می‌شود. در کدِ Rust هیچ‌چیز خراب نیست. این اولین مشکلی است که فقط به این دلیل وجود دارد که یک مرورگر در ماجرا هست، و تقریباً هر APIای که فرانت‌اندِ وب دارد در هفته‌ی اولش به آن می‌خورد.

اگر جنگو بلدی، همین دیوار را با اسمِ `django-cors-headers` می‌شناسی: یک تنظیمِ `CORS_ALLOWED_ORIGINS` و یک خط در `MIDDLEWARE`. در `axum` این کار با `CorsLayer` از `tower-http` انجام می‌شود که با `.layer(...)` وصل می‌شود. بعد از [۳.۲.۴](../04-tower-service-and-layer-middleware/README.fa.md) که در آن خودت یک `Layer` و یک `Service` را با دست نوشتی، `CorsLayer` دیگر جادو نیست: یک لایه‌ی آماده است، دقیقاً مثلِ همان که نوشتی، و یکی از کارهایش این است که عمداً سرویسِ داخلی را صدا نمی‌زند. این درس، که ماژول را می‌بندد، از همین استفاده می‌کند.

---

## مفهوم

### باگی که اصلاً باگ نیست

`examples/01-no-cors-layer.rs` یک روتر بدونِ هیچ لایه‌ی CORS است. درخواستی به آن می‌رسد که یک صفحه‌ی روی `http://localhost:5173` می‌فرستد:

```text
GET /anime -> 200 OK
  access-control-allow-origin: None
  body: [{"id":1,"title":"Frieren"}]
OPTIONS /anime -> 405 Method Not Allowed
```

سرور کارش را کرد: `200`، یک بدنه، بدونِ خطا. تنها چیزی که نیست یک هدر است. جاوااسکریپتِ صفحه هیچ‌وقت این بدنه را نمی‌بیند، چون **مرورگر** پاسخ را با مبدأِ صفحه مقایسه می‌کند، هدری نمی‌یابد که ازش ضمانت کند، و پاسخ را نگه می‌دارد. سیاستِ هم‌مبدأ (same-origin policy) را مرورگر اعمال می‌کند، نه سرور. سرور هیچ‌وقت کسی را «بلاک» نمی‌کند؛ فقط از ضمانت‌کردن (vouch) خودداری می‌کند، و بلاک‌کردن کارِ مرورگر است.

خطِ دوم نیمه‌ی دیگرِ ماجراست: پیش از یک `POST`ِ جیسون، مرورگر یک درخواستِ `OPTIONS` می‌فرستد، و وقتی هیچ لایه‌ای نیست `405` می‌گیرد. تو هرگز برایش هندلر ننوشتی، و (همان‌طور که می‌بینی) لازم هم نیست بنویسی.

### مبدأها، و اینکه چه کسی قاعده را اعمال می‌کند

**مبدأ (origin)** یعنی سه‌تایی *پروتکل + هاست + پورت*. نسبت به `http://localhost:3000`، هرکدام از این‌ها یک مبدأِ متفاوت است:

| URL | چرا فرق دارد |
|---|---|
| `https://localhost:3000` | پروتکل (`https` در برابرِ `http`) |
| `http://localhost:5173` | پورت |
| `http://127.0.0.1:3000` | هاست (`localhost` و `127.0.0.1` دو رشته‌ی متفاوت‌اند) |

چون مرورگر اعمال‌کننده است، هر چیزی که مرورگر نیست کلِ این ماجرا را نادیده می‌گیرد: `curl`، تست‌هایِ `oneshot`ِ تو، یک سرویسِ بک‌اندِ دیگر. CORS از *کاربرها* (کوکی‌ها و نشست‌هایِ واردشده‌شان) در برابرِ *وب‌سایت‌هایِ* مخرب محافظت می‌کند. هیچ‌وقت کنترلِ دسترسی برایِ API تو نبوده: اگر تو بتوانی با `curl` صدایش بزنی، همه می‌توانند.

### پیش‌پرواز: درخواستِ `OPTIONS`ای که برایش هندلر ننوشتی

برایِ یک درخواستِ «ساده» (`GET`، `HEAD`، یا `POST` با نوعِ محتوایِ شبیهِ فرم) مرورگر همان را با یک هدرِ `Origin` می‌فرستد و بعد از دریافتِ پاسخ بررسی‌اش می‌کند. هر چیزِ دیگر اول یک **پیش‌پرواز (preflight)** می‌گیرد: مرورگر یک `OPTIONS` می‌فرستد که `Origin`، `Access-Control-Request-Method` و، اگر هدرِ سفارشی در کار باشد، `Access-Control-Request-Headers` را دارد. فقط اگر جواب تأیید کند، درخواستِ واقعی را می‌فرستد. این‌ها پیش‌پرواز می‌گیرند:

- متدهایی بیرون از مجموعه‌ی ساده: `PUT`، `PATCH`، `DELETE`
- هدرهایِ سفارشیِ درخواست: `Authorization`، `X-Api-Key`
- **`Content-Type: application/json`** که نوعِ محتوایِ «ساده» نیست

مورد آخر نقطه‌ی اوج است: هر `POST`ِ جیسونی که API تو قبول می‌کند پیش‌پرواز می‌گیرد. جواب روی سه هدرِ پاسخ سوار است:

| هدر | یعنی | کجا می‌آید |
|---|---|---|
| `access-control-allow-origin` | کدام مبدأ حق دارد پاسخ را بخواند | پیش‌پرواز و پاسخ‌هایِ واقعی |
| `access-control-allow-methods` | درخواستِ واقعی چه متدهایی را می‌تواند بزند | فقط پیش‌پرواز |
| `access-control-allow-headers` | درخواستِ واقعی چه هدرهایی می‌تواند داشته باشد | فقط پیش‌پرواز |

چرا مرورگر می‌تواند بدونِ اجازه از کسی پیش‌پرواز بفرستد؟ چون `OPTIONS` در جدولِ [۳.۱.۳](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.fa.md) یک **متدِ ایمن** است: قرار نیست چیزی را رویِ سرور عوض کند، پس پرسیدنِ «این را قبول می‌کنی؟» ذاتاً بی‌خطر است.

### لایه جواب می‌دهد، و هندلر هیچ‌وقت اجرا نمی‌شود

این یک سیاست برایِ یک فرانت‌اند است، دورِ روتری که در هندلرِ `POST`اش یک شمارنده دارد (`examples/02-preflight-never-reaches-the-handler.rs`):

```rust
let cors = CorsLayer::new()
    .allow_origin(AllowOrigin::list([HeaderValue::from_static(
        "http://localhost:5173",
    )]))
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([header::CONTENT_TYPE]);
let app = Router::new()
    .route("/anime", post(create_anime))
    .with_state(hits.clone())
    .layer(cors);
```

اول پیش‌پرواز را می‌فرستد، بعد درخواستِ واقعی را:

```text
preflight -> 200 OK
  access-control-allow-origin: "http://localhost:5173"
  access-control-allow-methods: "GET,POST"
  access-control-allow-headers: "content-type"
  body: 0 bytes
  handler ran 0 times
real POST -> 201 Created
  access-control-allow-origin: "http://localhost:5173"
  handler ran 1 times
```

همان روتر، همان لایه؛ شمارنده بعد از پیش‌پرواز صفر می‌ماند و فقط با `POST`ِ واقعی جلو می‌رود. `CorsLayer` یک `Layer` است مثلِ همان که در ۳.۲.۴ نوشتی، و روی یک درخواستِ `OPTIONS` `Service`ِ آن همان کاری را می‌کند که میان‌افزارِ اتصال‌کوتاهِ (short-circuit) تو می‌کرد: خودش پاسخ را می‌سازد و برمی‌گرداند، **بدونِ صدازدنِ سرویسِ داخلی**. پیش‌پرواز هیچ‌وقت به روتر یا هندلرِ تو نمی‌رسد. برایِ همین هیچ مسیرِ `options(...)` لازم نیست و بدنه خالی است.

```senpai-visual
{"kind":"network","labels":["مرورگر: پیش‌پرواز OPTIONS","CorsLayer جواب می‌دهد، هندلر صدا زده نمی‌شود","مرورگر هدرهای allow را می‌سنجد","مرورگر درخواست واقعی را می‌فرستد","هندلر اجرا می‌شود، CorsLayer هدر allow-origin را اضافه می‌کند","مرورگر به صفحه اجازه می‌دهد یا بلاک می‌کند"]}
```

دقت کن پاسخِ واقعی `access-control-allow-origin` دارد ولی آن دو هدرِ دیگر را نه: آن‌ها مخصوصِ پیش‌پروازند. و چون پاسخی که یک مبدأِ مشخص را تأیید می‌کند برایِ هر درخواست فرق می‌کند، `tower-http` هدرِ `vary: origin, access-control-request-method, access-control-request-headers` را هم اضافه می‌کند تا یک کش جوابِ یک مبدأ را به مبدأِ دیگر ندهد.

### تنها پیکربندیِ ممنوع

`Access-Control-Allow-Origin: *` را نمی‌شود با `Access-Control-Allow-Credentials: true` ترکیب کرد. این ترکیب یعنی «هر وب‌سایتی در اینترنت می‌تواند کوکی‌هایِ این کاربر را به API تو بفرستد و جواب را بخواند»، یعنی ربودنِ نشست به‌شکلِ یک گزینه‌ی تنظیمات. مرورگرها ردش می‌کنند، و `tower-http` اصلاً نمی‌سازدش: وقتی لایه را اعمال می‌کنی پنیک می‌کند. «خطاهایی که خواهی دید» پنیکِ واقعی را نشان می‌دهد.

### وضعیتِ توسعه، وضعیتِ تولید

- **توسعه: آزاد.** مبدأها مدام عوض می‌شوند: Vite پورتِ دیگری برمی‌دارد، هم‌تیمی‌ات `127.0.0.1` می‌زند، گوشیِ روی شبکه‌ی محلی UI را باز می‌کند. روی سیستمِ خودت همه‌چیز را باز بگذار.
- **تولید: قفل.** دقیقاً مبدأیی که فرانت‌اند از آن سرو می‌شود، دقیقاً متدها و هدرهایی که استفاده می‌کند. وایلدکارد در تولید یعنی هر صفحه‌ای که کاربر باز می‌کند می‌تواند پاسخ‌هایِ API تو را از داخلِ مرورگرش بخواند.

انتخاب بین این دو کارِ پیکربندی است، و یک رشته‌ی هاردکدشده در `main` فقط جایگزینِ موقتِ آن است. [۳.۴.۱ — پیکربندیِ ۱۲فاکتوری و سکرت‌ها](../../04-configuration-and-app-structure/01-config-and-secrets/README.fa.md) جایی است که مبدأهایِ مجاز دیگر هاردکد نیستند.

### تست‌کردنِ CORS بدونِ مرورگر

پیش‌پرواز فقط یک درخواستِ HTTP است: `OPTIONS` به‌علاوه‌ی چند هدر. پس تکنیکِ `oneshot` از [۳.۲.۱](../01-routing-handlers-extractors/README.fa.md) می‌تواند یکی بسازد و مستقیم رویِ هدرهایِ `access-control-*`ِ پاسخ assert کند:

```rust
fn preflight(origin: &str, request_headers: &str) -> Request<Body> {
    Request::builder()
        .method("OPTIONS")
        .uri("/anime")
        .header("origin", origin)
        .header("access-control-request-method", "POST")
        .header("access-control-request-headers", request_headers)
        .body(Body::empty())
        .unwrap()
}
```

`tests/cors_test.rs` کلِ سیاستِ تو را این‌طور محکم می‌کند، از جمله حالتِ منفی: مبدأِ ناشناس باز هم `200` می‌گیرد، بدونِ `access-control-allow-origin`، چون ضمانت‌نکردن خطا نیست.

---

## دست‌به‌کد

```sh
cargo run -p p3-02-05-cors-and-frontend-integration --example 01-no-cors-layer
cargo run -p p3-02-05-cors-and-frontend-integration --example 02-preflight-never-reaches-the-handler
```

هر دو همان چیزی را چاپ می‌کنند که در «مفهوم» آمد. حالا یک سوکتِ واقعی. `03-serve-with-cors` اَپ را رویِ پورتِ ۳۰۰۲ فقط برایِ مبدأِ `http://localhost:5173` سرو می‌کند؛ در یک ترمینال اجرایش کن و بازش بگذار:

```sh
cargo run -p p3-02-05-cors-and-frontend-integration --example 03-serve-with-cors
```

در ترمینالِ دوم، کاری را که مرورگر پیش از یک `POST`ِ جیسون می‌کند شبیه‌سازی کن:

```sh
curl -si -X OPTIONS http://127.0.0.1:3002/anime \
  -H "Origin: http://localhost:5173" \
  -H "Access-Control-Request-Method: POST" \
  -H "Access-Control-Request-Headers: content-type"
```

```text
HTTP/1.1 200 OK
vary: origin, access-control-request-method, access-control-request-headers
access-control-allow-methods: GET,POST
access-control-allow-headers: content-type
access-control-allow-origin: http://localhost:5173
allow: GET,HEAD,POST
content-length: 0
date: Sun, 04 Oct 2026 10:13:26 GMT

```

خط‌هایِ `access-control-allow-*` همه‌ی چیزی‌اند که مرورگر لازم دارد. (خطِ `allow:` فهرستِ متدهایِ خودِ `axum` است که بعد از جواب‌دادنِ لایه اضافه شده؛ هندلر باز هم اجرا نشده. `date` هر بار عوض می‌شود.) حالا `Origin` را به `https://evil.example.com` عوض کن و دوباره بزن: کدِ وضعیت هنوز `200` است و `access-control-allow-origin` رفته. سرور را با Ctrl+C ببند.

بعد دو مثالِ خراب. اولی فیچرِ `broken` لازم دارد؛ دومی معمولی اجرا می‌شود و فقط غلط است:

```sh
cargo run -p p3-02-05-cors-and-frontend-integration --example 04-credentials-with-wildcard-broken --features broken
cargo run -p p3-02-05-cors-and-frontend-integration --example 05-origin-that-never-matches-trap
```

بعد این‌ها را امتحان کن:

۱. در `02-preflight-never-reaches-the-handler`، مقدارِ `access-control-request-method` در پیش‌پرواز را به `DELETE` عوض کن. حالا `access-control-allow-methods` چه می‌گوید، و برایِ مرورگر یعنی چه؟
۲. در `01-no-cors-layer`، `.layer(CorsLayer::permissive())` را به روتر اضافه کن. حالا آن دو خط چه می‌گویند؟

---

## خطاهایی که خواهی دید

### یک پنیکِ زمانِ اجرا: اعتبارنامه همراهِ مبدأِ وایلدکارد

```text
layer built, applying it to the router...

thread 'main' (31804) panicked at C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\tower-http-0.6.11\src\cors\mod.rs:797:9:
Invalid CORS configuration: Cannot combine `Access-Control-Allow-Credentials: true` with `Access-Control-Allow-Origin: *`
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(عددِ داخلِ پرانتز شناسه‌ی ریسمان است و هر بار عوض می‌شود؛ مسیر جایی است که Cargo روی این ماشین `tower-http` را باز کرده.)

**واقعاً چه چیزی خراب است:** `.allow_origin(Any).allow_credentials(true)` یعنی «هر وب‌سایتی می‌تواند کوکی بفرستد و جواب را بخواند». ساختنِ لایه موفق است (خطِ اول چاپ شد). پنیک وقتی می‌آید که لایه با `.layer(...)` اعمال می‌شود، یعنی وقتِ سرِ هم‌کردنِ اَپ، پیش از اولین درخواست. جای خوبی برایِ شکستن است: همان لحظه‌ی استارت می‌فهمی، نه در تولید.

**راه‌حل:** تصمیم بگیر چه کسانی می‌توانند اعتبارنامه بفرستند و اسمشان را بیاور:

```rust
CorsLayer::new()
    .allow_origin(AllowOrigin::list([HeaderValue::from_static(
        "https://anime.example.com",
    )]))
    .allow_credentials(true)
```

**چرا این راه‌حل است:** با اعتبارنامه، مرورگر باید دقیقاً بداند کدام مبدأ حق دارد پاسخ را بخواند. همین قاعده برایِ `allow_methods` و `allow_headers` هم هست: وقتی اعتبارنامه روشن است آن‌ها هم نمی‌توانند `Any` باشند.

### پیش‌پروازی که هدرِ تو را ندارد

```text
HTTP/1.1 200 OK
vary: origin, access-control-request-method, access-control-request-headers
access-control-allow-methods: GET,POST
access-control-allow-headers: content-type
access-control-allow-origin: http://localhost:5173
allow: GET,HEAD,POST
content-length: 0
date: Sun, 04 Oct 2026 10:13:26 GMT

```

این پاسخِ همان پیش‌پروازِ «دست‌به‌کد» است، با این فرق که حالا درخواست `Access-Control-Request-Headers: content-type, authorization` می‌خواهد. جواب بایت‌به‌بایت همان است: `access-control-allow-headers` فقط `content-type` می‌گوید.

**واقعاً چه چیزی خراب است:** فرانت‌اندِ تو یک هدرِ `Authorization` اضافه می‌کند و سیاستِ تو هرگز اجازه‌اش نداده. نه خطایی هست، نه خطِ لاگی، و کدِ وضعیت `200` است. مرورگر هدری را که می‌خواهد بفرستد با فهرستِ برگشتی مقایسه می‌کند، `authorization` را نمی‌بیند، و درخواستِ واقعی را لغو می‌کند. در کنسول یک خطای CORS می‌بینی و در تبِ شبکه یک `OPTIONS`ِ موفق و هیچ `POST`ی.

**راه‌حل:** هدر را به فهرستِ مجازِ سیاست اضافه کن (`header::AUTHORIZATION` کنارِ `header::CONTENT_TYPE`) و پیش‌پرواز را دوباره بزن تا ببینی ظاهر شد.

**چرا این راه‌حل است:** سرور هیچ‌وقت یک هدر را رد نمی‌کند. یک فهرست منتشر می‌کند و مرورگر درخواست را با آن می‌سنجد. پس راهِ دیباگ همان است که همین الان کردی: به پاسخِ پیش‌پرواز نگاه کن، نه به هندلرت.

### هیچ خطایی نیست: مبدأیی که هیچ‌وقت جور نمی‌شود

```text
allowed "https://anime.example.com/", Origin "https://anime.example.com"
  -> 200 OK, access-control-allow-origin: None
allowed "http://anime.example.com", Origin "https://anime.example.com"
  -> 200 OK, access-control-allow-origin: None
```

**واقعاً چه چیزی خراب است:** `examples/05-origin-that-never-matches-trap.rs` کامپایل می‌شود، اجرا می‌شود، و هر بار `200` می‌دهد. هر دو مبدأِ مجاز با دست تایپ شده‌اند. یکی اسلشِ آخر دارد، دیگری پروتکلِ غلط. هدرِ `Origin`ِ مرورگر دقیقاً پروتکل، هاست و پورت است: بدونِ مسیر و بدونِ `/`ِ آخر. سیاست بایت‌ها را مقایسه می‌کند، پس هیچ‌کدام جور نمی‌شود، و هر درخواستِ مرورگر بدونِ هیچ سرنخی در سمتِ سرور شکست می‌خورد.

**راه‌حل:** مبدأ را همان‌طور بنویس که مرورگر می‌فرستد:

```rust
AllowOrigin::list([HeaderValue::from_static("https://anime.example.com")])
```

**چرا این راه‌حل است:** مقایسه دقیق است، پس پیکربندیِ تو هم باید دقیق باشد. برایِ همین هم `prod_cors_from_list` در تمرینِ «بساز» ورودیِ با `/`ِ آخر را رد می‌کند: اشتباهِ تایپیِ همیشه‌ناجور بهتر است همان استارت گرفته شود تا اینکه کاربر پیدایش کند.

---

## تمرین

### گرم‌کردن

<details>
<summary>API تو به <code>curl</code> کاملاً جواب می‌دهد ولی کنسولِ مرورگر می‌گوید «blocked by CORS policy». کدام طرف بلاک کرد؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

مرورگر. سرور پاسخ را ساخت و فرستاد؛ مرورگر آن را از جاوااسکریپتِ صفحه نگه داشت، چون هیچ هدرِ `access-control-allow-origin`ای از مبدأِ آن صفحه ضمانت نکرده بود. سرور هیچ‌وقت بلاک نمی‌کند، فقط از ضمانت‌کردن خودداری می‌کند.

</details>

<details>
<summary>یک صفحه با <code>fetch</code> یک <code>GET</code> بدونِ هدرِ سفارشی می‌زند، بعد یک <code>POST</code> با <code>Content-Type: application/json</code>. کدام پیش‌پرواز می‌گیرد؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

فقط `POST`. یک `GET`ِ ساده درخواستِ ساده است و مستقیم با هدرِ `Origin` فرستاده می‌شود. `application/json` نوعِ محتوایِ ساده نیست، پس پیش از `POST` یک پیش‌پروازِ `OPTIONS` می‌آید.

</details>

<details>
<summary>چرا <code>allow_origin(Any)</code> همراهِ <code>allow_credentials(true)</code> ممنوع است؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

با هم یعنی هر وب‌سایتی در اینترنت می‌تواند کوکی‌هایِ کاربرِ واردشده را به API تو بفرستد و پاسخ‌ها را بخواند. اسپک ممنوعش کرده، و `tower-http` وقتی چنین لایه‌ای را اعمال کنی به‌جایِ اجازه‌ی اجرا پنیک می‌کند.

</details>

<details>
<summary>یک مبدأِ ناشناس به لایه‌ی تولیدِ تو پیش‌پرواز می‌فرستد. چه کدی برمی‌گردد و چرا؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

`200 OK`، بدونِ `access-control-allow-origin`. سرور کارش بلاک‌کردن نیست؛ صادقانه جواب می‌دهد و تصمیم را به مرورگر می‌سپارد. ضمناً برایِ این درخواست هم هندلر اجرا نمی‌شود، چون لایه هر `OPTIONS` را خودش جواب می‌دهد.

</details>

### تعمیر

هر دو مثالِ خراب را درست کن:

۱. `examples/04-credentials-with-wildcard-broken.rs` بدونِ پنیک روتر را بسازد، و باز هم اعتبارنامه را برایِ یک مبدأِ مشخص مجاز بگذارد.
۲. `examples/05-origin-that-never-matches-trap.rs` در هر دو حالت یک هدرِ `access-control-allow-origin` برایِ مبدأِ مرورگر چاپ کند.

### پیاده‌سازی

دو تابع در `src/lib.rs`:

```sh
cargo test -p p3-02-05-cors-and-frontend-integration
```

هر کدام در کامنتِ مستنداتِ خودش کامل مشخص شده (کدام مبدأ، کدام متدها، کدام هدرها، و اعتبارنامه مجاز است یا نه)، پس لازم نیست تست‌ها را بخوانی:

- `dev_cors`: همه‌چیز مجاز، با وایلدکارد `*`.
- `prod_cors`: یک مبدأ، `GET` و `POST`، فقط `content-type`. اگر مبدأ مقدارِ معتبرِ هدر نباشد پنیک می‌کند.

### بساز

`prod_cors_from_list`: همان سیاست برایِ چند فرانت‌اند، که از یک رشته‌ی جداشده با ویرگول خوانده می‌شود، مثلِ `"https://anime.example.com, http://localhost:5173"`، شکلی که یک تنظیمِ `ALLOWED_ORIGINS` معمولاً دارد. ورودی‌ها را trim می‌کند، خالی‌ها را رد می‌کند، و روی ورودیِ با `/`ِ آخر پنیک می‌کند و اسمش را می‌آورد. سه تستِ دیگر در همان اجرایِ `cargo test` بررسی‌اش می‌کنند.

### چالش (اختیاری)

جاوااسکریپت فقط چند هدرِ پاسخِ «ایمن» را می‌تواند بخواند. یک هدر مثلِ `x-total-count` به پاسخِ یک هندلر اضافه کن، بعد با `expose_headers`ِ `CorsLayer` کاری کن که `tower-http` آن را در معرض بگذارد. یک تستِ خودت، در یک فایلِ تازه زیرِ `tests/`، بنویس که یک درخواستِ ساده با هدرِ `Origin` بفرستد و رویِ `access-control-expose-headers` assert کند. آیا پیش‌پرواز هم این هدر را دارد؟

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| مبدأ (origin) | پروتکل + هاست + پورت | تعیین اینکه فرانت‌اندِ تو کیست |
| سیاستِ هم‌مبدأ (same-origin policy) | مرورگر پاسخِ بین‌مبدأ را نگه می‌دارد مگر سرور ضمانت کند | هر فرانت‌اندِ وب |
| CORS | هدرهایی که سرور با آن‌ها از یک مبدأ ضمانت می‌کند | یک `CorsLayer` رویِ هر APIی که مرورگر صدایش می‌زند |
| پیش‌پرواز (preflight) | یک `OPTIONS` که مرورگر پیش از درخواستِ غیرساده می‌فرستد | هر `POST`ِ جیسون، `PUT`، `DELETE` یا هدرِ سفارشی |
| `CorsLayer` | یک `Layer`ِ آماده که پیش‌پرواز را خودش جواب می‌دهد | `.layer(...)` رویِ روتر |
| اعتبارنامه (credentials) | کوکی‌هایی که به درخواستِ بین‌مبدأ چسبیده‌اند | نشست‌هایِ کوکی‌ای، هیچ‌وقت همراهِ وایلدکارد |

### الان می‌دانی

- مرورگر سیاستِ هم‌مبدأ را اعمال می‌کند. سرور فقط ضمانت می‌کند یا نمی‌کند، و `curl` و `oneshot` کلِ این ماجرا را نادیده می‌گیرند.
- پیش‌پرواز یک `OPTIONS` است، ایمن به معنایِ ۳.۱.۳، که `CorsLayer` بدونِ صدازدنِ روتر یا هندلرِ تو جوابش را می‌دهد.
- `*` همراهِ اعتبارنامه ممنوع است، و `tower-http` به‌جایِ ساختنش پنیک می‌کند.
- هدری که سیاستِ تو فهرست نکرده در مرورگر بی‌صدا شکست می‌خورد، و مبدأیی با اسلشِ آخر یا پروتکلِ غلط هیچ‌وقت جور نمی‌شود.
- کلِ یک سیاستِ CORS را با `oneshot` تست می‌کنی و یک پیش‌پرواز را با `curl` شبیه‌سازی می‌کنی.

### بعداً کامل‌تر می‌بینی

- **مبدأهایی که از محیط می‌آیند، نه از کد** — [۳.۴.۱ — پیکربندیِ ۱۲فاکتوری و سکرت‌ها](../../04-configuration-and-app-structure/01-config-and-secrets/README.fa.md)
- **چرا کوکی `allow_credentials` می‌خواهد، و جایگزینش** — [۳.۷.۲ — Session در برابرِ JWT: مصالحه‌یِ واقعی](../../07-auth-and-security/02-sessions-vs-jwt/README.fa.md)
- **structهایِ درخواست و پاسخ، این بار با قوانینِ واقعیِ فیلد** — [۳.۳.۱ — عمقِ serde](../../03-serialization-and-validation/01-serde-depth/README.fa.md)
- **همین endpointها رویِ یک دیتابیسِ واقعی** — [ماژول ۵ — دیتابیس PostgreSQL و `sqlx`](../../05-postgres-and-sqlx/README.fa.md)

نگاهی به کلِ ماژول: [۳.۲.۱](../01-routing-handlers-extractors/README.fa.md) مسیر، هندلر و اکسترکتور را داد؛ [۳.۲.۲](../02-writing-your-own-extractor/README.fa.md) نوشتنِ اکسترکتورِ خودت را؛ [۳.۲.۳](../03-anime-catalog-crud-in-memory/README.fa.md) یک منبعِ کامل ساخت و خطاها را به پاسخ تبدیل کرد؛ [۳.۲.۴](../04-tower-service-and-layer-middleware/README.fa.md) `.layer(...)` را باز کرد؛ و این درس یک لایه‌ی آماده را به‌کار برد و بدونِ سوکت تستش کرد. ماژول ۳ با [serde](../../03-serialization-and-validation/01-serde-depth/README.fa.md) شروع می‌شود، و ماژول ۵ یک دیتابیس پشتِ این endpointها می‌گذارد.

### می‌توانی توضیح بدهی؟

- چرا سرور به درخواستی که مرورگر بعدش به صفحه نشانش نمی‌دهد `200` جواب می‌دهد؟
- چرا یک `POST`ِ جیسون همیشه پیش‌پرواز می‌گیرد، و چرا مرورگر اصلاً می‌تواند پیش‌پرواز بفرستد؟
- چطور با یک تست ثابت می‌کنی که پیش‌پرواز هیچ‌وقت به هندلرت نمی‌رسد؟
- چرا `*` همراهِ اعتبارنامه ممنوع است، و `tower-http` کِی بابتش پنیک می‌کند؟
- چرا نبودنِ `authorization` در `access-control-allow-headers` در سمتِ سرور هیچ خطایی نمی‌دهد؟

---

## بیشتر

- [MDN — Cross-Origin Resource Sharing (CORS)](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS): کلِ سازوکار از سمتِ مرورگر، از جمله اینکه کدام درخواست‌ها «ساده» هستند.
- [Fetch Standard — CORS protocol](https://fetch.spec.whatwg.org/#http-cors-protocol): اسپکی که هدرهایِ امروزت را تعریف می‌کند.
- [`tower_http::cors`](https://docs.rs/tower-http/0.6.11/tower_http/cors/index.html): همه‌ی متدهایِ `CorsLayer`، از جمله `max_age` و `AllowOrigin::predicate`.
- [فهرستِ ماژول ۲](../README.fa.md): پنج درس به ترتیب.
