# راه‌حل — ۳.۲.۵ CORS و اتصال به فرانت‌اند

## `dev_cors`

```rust
CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any)
```

`Any` یک unit structِ `tower_http::cors` است. هر متدِ `allow_*` هر چیزی را که به نوعِ هدرِ خودش تبدیل شود می‌پذیرد، و `Any` در پاسخ یک `*`ِ واقعی می‌شود؛ برایِ همین تستِ dev دقیقاً `*` را چک می‌کند. `CorsLayer::permissive()` همین لایه‌ی از پیش‌ساخته است (و همه‌ی هدرهایِ پاسخ را هم در معرض می‌گذارد). `allow_credentials` نیست، پس هدرِ اعتبارنامه فرستاده نمی‌شود.

## `prod_cors`

```rust
let origin = allowed_origin.parse::<HeaderValue>().expect("invalid allowed origin");
CorsLayer::new()
    .allow_origin(AllowOrigin::list([origin]))
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([header::CONTENT_TYPE])
```

سه تصمیم که ارزشِ دفاع دارند:

- **`AllowOrigin::list`، نه یک `HeaderValue`ِ لخت.** با یک `HeaderValue`، `tower-http` آن مقدارِ ثابت را رویِ *هر* پاسخ می‌زند، حتی برایِ `https://evil.example.com`، و ردکردن را به مرورگر می‌سپارد. با `list`، لایه `Origin`ِ درخواست را با فهرست مقایسه می‌کند و فقط در صورتِ جورشدن `access-control-allow-origin` می‌فرستد. تستِ «مبدأِ ناشناس ضمانت نمی‌شود» دقیقاً همین را محکم می‌کند.
- **`.expect` رویِ پارس.** مبدأِ خراب اشتباهِ دیپلوی است. پنیک در استارت بهتر از پروسه‌ای است که سالم به نظر می‌رسد ولی تک‌تکِ کلاینت‌هایِ مرورگرش بی‌صدا شکست می‌خورند.
- **فقط `GET`/`POST` و `content-type`.** همان را بده که فرانت‌اند استفاده می‌کند: یک مسیرِ `DELETE`ِ آینده تا وقتی کسی عمداً `Method::DELETE` را اضافه نکرده بسته می‌ماند. جوابِ پیش‌پرواز همین ثابت‌ها را فهرست می‌کند و هیچ‌وقت آنچه را خواسته شده تکرار نمی‌کند؛ برایِ همین درخواستِ `authorization` هم باز `content-type` برمی‌گردد.

## `prod_cors_from_list` (تمرینِ «بساز»)

```rust
let origins = allowed_origins
    .split(',')
    .map(str::trim)
    .filter(|entry| !entry.is_empty())
    .map(|entry| {
        assert!(!entry.ends_with('/'), "allowed origin {entry:?} ends with a slash and can never match");
        entry.parse::<HeaderValue>().expect("invalid allowed origin")
    })
    .collect();
```

بعدش همان `AllowOrigin::list` و `allow_methods` و `allow_headers`ِ `prod_cors` رویِ `origins` اعمال می‌شود (راه‌حل آن‌ها را در یک `prod_layer`ِ خصوصی مشترک گذاشته و `prod_cors` آن را با یک مبدأ صدا می‌زند). `{entry:?}` ورودی را داخلِ گیومه چاپ می‌کند و تست فقط دنبالِ متنِ ورودی داخلِ پیام می‌گردد. فهرستِ خالی یک `AllowOrigin::list([])` می‌دهد که از هیچ‌کس ضمانت نمی‌کند.

## درباره‌ی چالش

```rust
CorsLayer::new().expose_headers([HeaderName::from_static("x-total-count")])
```

`expose_headers` فقط به پاسخ‌هایِ **غیرِ پیش‌پرواز** `access-control-expose-headers` اضافه می‌کند؛ پیش‌پرواز آن را ندارد. بدونِ آن، جاوااسکریپت پاسخ را می‌بیند ولی `response.headers.get("x-total-count")` برابرِ `null` است.
