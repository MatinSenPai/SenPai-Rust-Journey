# پاسخ تشریحی

```rust
pub fn dev_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}
```

`Any` یک انتخاب type-level از `tower_http::cors` است و در header به `*` تبدیل می‌شود. `CorsLayer::permissive()` همین سه مجوز را آماده دارد، اما نوشتن صریح نشان می‌دهد permissive دقیقاً چه می‌بخشد.

```rust
pub fn prod_cors(allowed_origin: &str) -> CorsLayer {
    let origin = allowed_origin.parse::<HeaderValue>().expect("invalid allowed origin");
    CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE])
}
```

origin نامعتبر خطای deployment است، پس fail-fast در startup از process سالم‌نما با browser client خراب بهتر است. origin دقیق روی match header را می‌گذارد و روی mismatch فقط آن را حذف می‌کند. محدودکردن method و header نیز باعث می‌شود endpoint آینده مانند DELETE تا افزوده‌شدن آگاهانه‌ی مجوز، از browser fail closed باشد.

پاسخ checkpoint: origin ناشناس `200 OK` بدون `access-control-allow-origin` می‌گیرد؛ browser هنگام inspect پاسخ preflight request واقعی را نمی‌فرستد. GET ساده preflight ندارد؛ JSON POST و Authorization GET دارند. خود `CorsLayer` پیش از router به OPTIONS پاسخ می‌دهد. wildcard همراه credentials هر سایت را قادر به خواندن response session کاربر می‌کند. حتی بدون cookie، wildcard هر صفحه را قادر به درخواست و خواندن endpointهای public، IP-allowlisted یا درون شبکه‌ی داخلی بازدیدکننده می‌کند؛ و policy امروز ممکن است از auth امروز عمر بیشتری داشته باشد.
