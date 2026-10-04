# ۳.۷.۳ — JWT و میان‌افزار در `tower`

## در یک نگاه

بعد از این درس می‌توانی:

- یک JWT را با دست به سه تکه‌یِ هدر، پی‌لود و امضا باز کنی، و دقیق بگویی امضا از چه چیزی محافظت می‌کند و از چه چیزی نمی‌کند.
- با `jsonwebtoken` نسخه‌ی ۹ توکنِ HS256 صادر و راستی‌آزمایی کنی: الگوریتم را قفل کنی، `exp` را اجباری کنی، و انقضا و مهلتِ تحمل را با یک ساعتِ تزریق‌شده بسنجی، نه با ساعتِ واقعی.
- مسیرهایِ `axum` را با میان‌افزارِ `from_fn_with_state` پشتِ در بگذاری، هویتِ راستی‌آزمایی‌شده را از راهِ افزونه‌هایِ درخواست به هندلرها برسانی، و `401` را از `403` جدا کنی.
- اشتباه‌هایِ JWT را که واقعاً سرویس‌ها را شکسته‌اند نام ببری، و نشان بدهی کدام خطِ کدِ این درس جلویِ هرکدام را می‌گیرد.

**زمان:** حدود ۱۰۰ دقیقه · **پیش‌نیاز:**
[۳.۲.۴ — `tower::Service` و `Layer`: میان‌افزار با دست](../../02-axum-and-rest-api-design/04-tower-service-and-layer-middleware/README.fa.md)،
[۳.۲.۱ — مسیریابی، هندلرها، اکسترکتورها](../../02-axum-and-rest-api-design/01-routing-handlers-extractors/README.fa.md)،
[۳.۷.۱ — هش‌کردنِ پسورد با argon2](../01-password-hashing-argon2/README.fa.md)،
[۳.۷.۲ — Session در برابرِ JWT: مصالحه‌یِ واقعی](../02-sessions-vs-jwt/README.fa.md)

---

## چرا اهمیت دارد

[۳.۷.۱](../01-password-hashing-argon2/README.fa.md) به این پرسش جواب داد: «این واقعاً خودِ توست؟»، یک بار، موقعِ ورود. هر درخواستِ بعدی همین پرسش را دوباره لازم دارد، و فرستادنِ رمز با هر درخواست اصلاً شدنی نیست. نشست (session) این را با «سرور به‌خاطر می‌سپارد» حل می‌کند. **JWT** (JSON Web Token) با «کلاینت گواه را با خودش می‌آورد»: سرور یک جمله‌یِ کوچک را امضا می‌کند، «این `user-42` است، تا ساعتِ ۳ بعدازظهر معتبر است»، موقعِ ورود تحویلش می‌دهد، و بعد فقط باید امضایِ خودش را بررسی کند. [۳.۷.۲](../02-sessions-vs-jwt/README.fa.md) بحث می‌کند هرکدام کِی انتخابِ درست است. این درس فرض می‌کند JWT را انتخاب کرده‌ای و درستش را می‌سازد.

نیمه‌یِ دیگرش را در جنگو می‌شناسی. `rest_framework_simplejwt` کلاسی به نامِ `JWTAuthentication` دارد که `Authorization: Bearer ...` را می‌خواند، توکن را راستی‌آزمایی می‌کند و `request.user` را پر می‌کند. راستی‌آزمایی را هیچ‌وقت ندیدی، و بیشترِ نفوذهایِ مربوط به JWT دقیقاً داخلِ همان اتفاق می‌افتند: راستی‌آزمایی که به فیلدِ `alg`ِ خودِ توکن اعتماد می‌کند، توکنی که انقضا ندارد، رازی که تویِ مخزنِ کد مانده. در `axum` کلاسِ فریم‌ورکی نیست که پشتش پنهان شوی. میان‌افزار را خودت می‌نویسی، پس این درس وادارت می‌کند یک بار بنویسی‌اش و هر تصمیمِ داخلش را ببینی.

این درس قولی را هم که آخرِ [۳.۲.۴](../../02-axum-and-rest-api-design/04-tower-service-and-layer-middleware/README.fa.md) داده شد ادا می‌کند: «میان‌افزاری که درخواستِ احرازنشده را رد می‌کند». آنجا `Layer` و `Service` را با دست نوشتی. اینجا از میان‌بُری استفاده می‌کنی که آن درس به آن ختم شد، یعنی `from_fn`، چون میان‌افزارِ احرازِ هویت مالِ خودِ اپ است و به چیزی که میان‌بُر ندارد نیاز ندارد.

---

## مفهوم

### JWT سه تکه‌یِ base64url است که با نقطه به هم وصل شده‌اند

`examples/01-anatomy-of-a-jwt.rs` یک توکن را با عددهایِ ثابت امضا می‌کند (پس خروجی هیچ‌وقت عوض نمی‌شود) و بعد فقط با `split('.')` و یک دیکودرِ base64url آن را از هم باز می‌کند. هیچ کلیدی برایِ خواندنش به کار نمی‌رود:

```rust
let claims = Claims {
    sub: "user-42".to_string(),
    iat: 1_700_000_000,
    exp: 1_700_003_600,
};
let key = EncodingKey::from_secret(b"demo-secret-for-3-7-3");
let token = encode(&Header::default(), &claims, &key).unwrap();
let parts: Vec<&str> = token.split('.').collect();
```

```text
token:     eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ1c2VyLTQyIiwiaWF0IjoxNzAwMDAwMDAwLCJleHAiOjE3MDAwMDM2MDB9.E2gWSvvYZVXaJTL1pz8Iu8meUGFH0SK4zZewTyYtefI
parts:     3
header:    {"typ":"JWT","alg":"HS256"}
payload:   {"sub":"user-42","iat":1700000000,"exp":1700003600}
signature: 43 base64url characters (32 bytes of HMAC-SHA256)
```

کلِ قالب همین است، از RFC 7519. تکه‌یِ اول **هدر** (header) است: کدام الگوریتم این توکن را امضا کرده. تکه‌یِ دوم **پی‌لود** (payload) است و فیلدهایش **ادعا** (claim) نام دارند: `sub` («subject»: توکن درباره‌یِ چه کسی است)، `iat` («issued at»: زمانِ صدور) و `exp` («expires at»: زمانِ انقضا)، که دو تایِ آخر ثانیه‌هایِ کامل از Unix epoch‌اند. تکه‌یِ سوم **امضا** (signature) است: یک HMAC-SHA256 رویِ دو تکه‌یِ اول که با یک نقطه به هم وصل شده‌اند، و با رازی محاسبه شده که فقط سرور می‌داند.

```senpai-visual
{"kind":"concept","labels":["هدر: alg و typ، به شکلِ base64url","پی‌لود: sub و iat و exp، به شکلِ base64url","امضا: HMAC-SHA256 رویِ هدر نقطه پی‌لود","با نقطه به هم وصل‌شده: خودِ توکن","فقط امضا به راز نیاز دارد"]}
```

### امضا شده، نه رمزگذاری‌شده

دوباره به خروجی نگاه کن. هدر و پی‌لود به شکلِ JSONِ ساده درآمدند، و تنها چیزی که برایش به کار رفت یک دیکودر بود بدونِ هیچ کلید. **base64url یک کدگذاری (encoding) است، نه رمزگذاری.** هر کسی که توکن را دارد می‌تواند همه‌یِ ادعاها را بخواند. امضا قولِ دیگری می‌دهد: اگر کسی یک نویسه‌یِ پی‌لود را عوض کند، امضا دیگر جور درنمی‌آید و سرور می‌فهمد.

پس JWT یک یادداشتِ «دستکاری‌اش را می‌شود فهمید» است، نه یک جعبه‌یِ قفل‌شده. هیچ‌وقت رمز، کلیدِ API یا هر چیزِ خصوصی را تویِ ادعاها نگذار. شناسه‌یِ کاربر، نقش و انقضا اشکالی ندارند. این اولین اشتباهِ واقعی است، و دلیلی که `Claims` در `src/lib.rs` یک توضیحِ مستند دارد که همین را می‌گوید.

پلِ جنگو، و جایی که تمام می‌شود: کوکیِ نشستِ جنگو هم یک شناسه‌یِ مات دارد که برایِ مرورگر معنایی ندارد. JWT برعکس است: کلاینت می‌تواند محتوا را بخواند، و سرور چیزی ذخیره نمی‌کند. هزینه‌اش مسئله‌یِ بعدی است. سرور نمی‌تواند توکنی را پس بگیرد، چون هیچ فهرستی نگه نداشته. [۳.۷.۴](../04-refresh-token-rotation-and-revocation/README.fa.md) به این می‌پردازد.

### راستی‌آزمایی با `jsonwebtoken`: یک `Validation` فهرستِ بازبینی است

`jsonwebtoken` نسخه‌ی ۹.۳.۱ رمزنگاری را انجام می‌دهد. به `decode` توکن، یک کلید و یک `Validation` می‌دهی که می‌گوید بر چه چیزهایی پافشاری کند:

```rust
let key = DecodingKey::from_secret(SECRET);
let validation = Validation::new(Algorithm::HS256);
decode::<serde_json::Value>(token, &key, &validation)
```

`examples/02-validation-knobs.rs` `decode` را رویِ هشت توکن اجرا می‌کند و چاپ می‌کند چه برگشت:

```text
default leeway: 60 s
valid until 2100         ok, sub = "user-42"
expired in 1970          ExpiredSignature
expired 30 s ago         ok, sub = "user-42"
30 s ago, leeway 0       ExpiredSignature
HS512, HS256 pinned      InvalidAlgorithm
HS512, both allowed      ok, sub = "user-42"
no exp claim             MissingRequiredClaim("exp")
alg none                 Json(Error("unknown variant `none`, expected one of `HS256`, `HS384`, `HS512`, `ES256`, `ES384`, `RS256`, `RS384`, `RS512`, `PS256`, `PS384`, `PS512`, `EdDSA`", line: 1, column: 13))
```

هر خط یکی از فیلدهایِ `Validation::new(Algorithm::HS256)` را نشان می‌دهد:

- `exp` اجباری است و بررسی می‌شود. توکنِ بدونِ `exp` رد می‌شود (`MissingRequiredClaim`)، و توکنی که در گذشته است `ExpiredSignature` می‌گیرد. توکنی که هیچ‌وقت منقضی نمی‌شود دومین اشتباهِ واقعی است، و `jsonwebtoken` به‌طورِ پیش‌فرض خودش جلویش را می‌گیرد.
- `leeway` به‌طورِ پیش‌فرض **۶۰ ثانیه** است. توکنی که ۳۰ ثانیه پیش منقضی شده بود هنوز پذیرفته شد، تا وقتی که مثال `leeway` را صفر کرد. مهلتِ تحمل (leeway) برایِ این است که دو ماشین هیچ‌وقت دقیقاً روی ساعت توافق ندارند (اختلافِ ساعت، clock skew). یعنی هر `exp` در عمل `exp + 60` است، مگر اینکه خلافش را بگویی.
- `algorithms` یک فهرست است و در ابتدا فقط همانی است که دادی. توکنی که هدرش چیزِ دیگری را نام ببرد `InvalidAlgorithm` می‌گیرد.

### الگوریتم را قفل کن؛ `alg: none` گزینه نیست

هدر می‌گوید کدام الگوریتم توکن را امضا کرده، و توکن از کلاینت می‌آید. راستی‌آزمایی که هرچه هدر بگوید انجام بدهد، به مهاجم اجازه می‌دهد خودش انتخاب کند چطور بررسی شود. این حمله‌یِ کلاسیکِ JWT است، در دو شکل. اولی `alg: none` است: هدر ادعا می‌کند «امضا لازم نیست»، و یک کتابخانه‌یِ ساده‌لوح بررسی را رد می‌کند. دومی **سردرگم‌سازیِ الگوریتم** (algorithm confusion) است: سروری توکن‌هایِ RS256 را با یک کلیدِ عمومی راستی‌آزمایی می‌کند، و مهاجم یک توکنِ HS256 می‌فرستد که با همان کلیدِ عمومی، به‌عنوانِ رازِ HMAC، امضا شده.

قفل‌کردن هردو را می‌کُشد. `algorithms` فهرستِ چیزهایی است که می‌پذیری، هدر با آن سنجیده می‌شود، و چیزِ دیگری امتحان نمی‌شود. در خروجیِ بالا، یک توکنِ HS512 با رازِ درست وقتی فقط HS256 مجاز بود `InvalidAlgorithm` گرفت، و همین که فهرست گسترده شد پذیرفته شد. پس گسترده‌کردنِ فهرست یک تصمیم است، نه یک راحتی. و `none` هیچ‌وقت به مرحله‌یِ تصمیم نمی‌رسد: `jsonwebtoken` چنین الگوریتمی ندارد، پس خودِ هدر پارس نمی‌شود، و همان خطِ بلندِ `Json(Error(...))` است.

یک جزئیاتِ دیگر از سورسِ کتابخانه (`crypto/mod.rs`): مقایسه‌یِ امضا از `verify_slices_are_equal`ِ `ring` استفاده می‌کند که زمانِ ثابت دارد، پس لازم نیست خودت نگرانِ مقایسه‌یِ بایت‌به‌بایت باشی.

### زمان یک ورودی است: ساعتِ تزریق‌شده

`decode` انقضا را با ساعتِ واقعیِ سیستم می‌سنجد. در محیطِ واقعی خوب است و در تست غیرممکن: تستی که «۳۰ ثانیه بعد از `exp`» می‌خواهد باید بخوابد یا ساعتِ ماشین را جعل کند. پس این کریت ساعت را پارامتر می‌کند، همان ساعتِ تزریق‌شدهِ [۳.۷.۲](../02-sessions-vs-jwt/README.fa.md)، این بار با یک closure به‌جایِ صفت. تمامِ ایده، از `src/lib.rs`:

```rust
pub type Clock = Arc<dyn Fn() -> u64 + Send + Sync>;

pub struct JwtConfig {
    pub secret: String,
    pub ttl_secs: u64,
    pub leeway_secs: u64,
    pub clock: Clock,
}
```

`JwtConfig::new` ساعتِ واقعی (`system_now`) را نصب می‌کند. یک تست `.with_clock(Arc::new(|| 1_700_000_000))` را صدا می‌زند و «اکنون» برایِ همیشه همان عدد است. `verify_token`ـی که تو می‌نویسی بررسیِ `exp`ِ خودِ کتابخانه را خاموش می‌کند و خودش `now > exp + leeway_secs` را با `config.clock` مقایسه می‌کند. امضا، قفلِ الگوریتم و پارسِ ادعاها هنوز کارِ کتابخانه است. فقط همان یک مقایسه‌ای که ساعت می‌خواهد به کدِ تو آمد.

`JwtConfig` یک `Debug`ِ دست‌نویس هم دارد که به‌جایِ راز `<redacted>` چاپ می‌کند. `Debug`ِ مشتق‌شده راز را به هر خطِ لاگی که پیکربندی را فرمت کند می‌برد، از آن نشتی‌هایی که یک سال کسی نمی‌فهمد.

### میان‌افزارِ سریع: `from_fn_with_state`

۳.۲.۴ با `from_fn` تمام شد: یک `async fn(Request, Next) -> Response` که `axum` آن را به یک `Layer` تبدیل می‌کند. برایِ احرازِ هویت دو چیزِ دیگر لازم است، و هر دو در آرگومان‌هایِ تابع‌اند. راز باید از جایی بیاید، پس `from_fn_with_state` حالتِ تو را به‌شکلِ اکسترکتورِ `State(...)` تحویل می‌دهد، همان اکسترکتوری که یک هندلر استفاده می‌کند. و تابع می‌تواند یک `Result` برگرداند، پس می‌تواند رد کند. `examples/03-protected-server.rs` نسخه‌یِ سریع است، با ساعتِ واقعی و یک `401`ِ لخت. یک تابعِ کمکیِ سه‌خطیِ `bearer` بالایِ آن (اینجا نشان داده نشده) هدر را می‌خواند؛ نسخه‌یِ سخت‌گیرانه‌اش اولین تمرینِ توست:

```rust
async fn require_auth(
    State(secret): State<&'static str>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let key = DecodingKey::from_secret(secret.as_bytes());
    let token = bearer(&request).ok_or(StatusCode::UNAUTHORIZED)?;
    let data = decode::<Claims>(token, &key, &Validation::new(Algorithm::HS256))
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    request.extensions_mut().insert(AuthUser(data.claims.sub));
    Ok(next.run(request).await)
}
```

سه نتیجه در آن پنهان است. هدرِ غایب یا ناقص یک `Err`ِ زودهنگام است، و `next` هیچ‌وقت صدا زده نمی‌شود: میان‌افزار **مسیر را کوتاه کرد**، دقیقاً مثلِ `CorsLayer` رویِ یک preflight در ۳.۲.۴. توکنی که `decode` را رد نکند هم همین‌طور. فقط توکنی که بگذرد به `next.run(request)` می‌رسد، که همان صدا زدنِ سرویسِ درونی است. سرور را اجرا کن و با `curl` با آن حرف بزن:

```text
$ curl -i http://127.0.0.1:3190/health
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 2
date: Sun, 04 Oct 2026 11:17:39 GMT

ok
$ curl -i http://127.0.0.1:3190/whoami
HTTP/1.1 401 Unauthorized
content-length: 0
date: Sun, 04 Oct 2026 11:17:39 GMT

$ curl -s -X POST -H 'content-type: application/json' -d '{"user":"matin"}' http://127.0.0.1:3190/login
{"token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJtYXRpbiIsImlhdCI6MTc5MTExMjY2MCwiZXhwIjoxNzkxMTE2MjYwfQ.Q42TLWeqdNBZG7S4renyFyPJmJPGhAy9H1efRF-T46g"}
$ curl -i -H "Authorization: Bearer $TOKEN" http://127.0.0.1:3190/whoami
HTTP/1.1 200 OK
content-type: application/json
content-length: 19
date: Sun, 04 Oct 2026 11:17:40 GMT

{"user_id":"matin"}
$ curl -i -H "Authorization: Bearer ${TOKEN}x" http://127.0.0.1:3190/whoami
HTTP/1.1 401 Unauthorized
content-length: 0
date: Sun, 04 Oct 2026 11:17:40 GMT

```

خطِ `date:` و خودِ توکن در هر اجرا فرق می‌کنند، چون توکن زمانِ صدورش را همراه دارد. اضافه‌کردنِ یک نویسه به توکن امضا را شکست و درخواست هیچ‌وقت به هندلر نرسید. `POST /login` اینجا به هر کسی که بخواهد توکن می‌دهد. این جایگزینِ ورودِ واقعیِ ۳.۷.۱ است که اول یک هشِ `argon2` را بررسی می‌کند.

```senpai-visual
{"kind":"concept","labels":["درخواست با هدرِ Authorization می‌رسد","bearer_token: توکن را از هدر بیرون می‌کشد","verify_token: الگوریتم، بعد امضا، بعد انقضا","معتبر: AuthUser را در افزونه‌هایِ درخواست می‌گذارد","next.run: هندلر Extension از AuthUser را می‌خواند","نامعتبر در هر قدم: جوابِ 401، هندلر هیچ‌وقت اجرا نمی‌شود"]}
```

تصویرِ جنگو: این `AuthenticationMiddleware` به‌علاوه‌یِ `JWTAuthentication` در یک تابع است، و همان جایی که `request.user` پر می‌شود. در یک نکته که مهم است دقیق نیست. `request.user`ِ جنگو در هر درخواست از پایگاه‌داده بار می‌شود، پس کاربرِ حذف‌شده یا غیرفعال‌شده فوراً از کار می‌افتد. این میان‌افزار تا `exp` به ادعایِ امضاشده اعتماد می‌کند و هیچ‌وقت از پایگاه‌داده چیزی نمی‌پرسد. این همان سرعت است، و دوباره همان مسئله‌یِ ابطال.

### افزونه‌هایِ درخواست هویت را می‌رسانند

`request.extensions_mut().insert(AuthUser(...))` یک مقدار را در **افزونه‌هایِ** (extensions) درخواست می‌گذارد: یک نقشه‌یِ کوچک که کلیدش نوعِ Rust است و از [۳.۲.۲](../../02-axum-and-rest-api-design/02-writing-your-own-extractor/README.fa.md) به بعد بخشی از `Parts`ِ درخواست است. هندلری که جلوتر است با اکسترکتورِ `Extension` آن را از رویِ نوع می‌خواهد:

```rust
pub async fn whoami(Extension(user): Extension<AuthUser>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "user_id": user.0 }))
}
```

هندلر هیچ‌وقت هدری پارس نمی‌کند و چیزی را راستی‌آزمایی نمی‌کند. فرض می‌کند اگر اجرا شد، میان‌افزار پیش‌تر اجرا شده. این اعتماد یک واقعیتِ سیم‌کشی است، نه چیزی که کامپایلر بسنجد، و `examples/04-forgot-the-layer.rs` بهایش را نشان می‌دهد. `whoami` را رویِ روتری سوار می‌کند که لایه‌یِ احرازِ هویت ندارد و یک درخواست می‌فرستد:

```text
status: 500 Internal Server Error
body:   Missing request extension: Extension of type `p3_07_03_jwt_and_tower_middleware::AuthUser` was not found. Perhaps you forgot to add it? See `axum::Extension`.
```

کامپایل می‌شود، اجرا می‌شود، و `500` جواب می‌دهد. هیچ چیزِ تویِ نوع‌ها «این هندلر `AuthUser` می‌خواهد» را به «چیزی باید آن را بگذارد» وصل نمی‌کند. فقط تستی که به هر مسیرِ محافظت‌شده درخواستِ بدونِ توکن بفرستد این را می‌گیرد، و تست‌هایِ این درس دقیقاً همین کار را می‌کنند.

### لایه کجا می‌نشیند: `route_layer` و آنچه بعدش می‌آید

`app` در `src/lib.rs` سیم‌کشی است:

```rust
pub fn app(config: JwtConfig) -> Router {
    Router::new()
        .route("/whoami", get(whoami))
        .route_layer(from_fn_with_state(config, require_auth))
        .route("/health", get(|| async { "ok" }))
}
```

دو قاعده اینجا به هم می‌رسند. اولی از ۳.۲.۴ است: یک لایه فقط مسیرهایی را می‌پیچد که پیش از آن اضافه شده‌اند، پس `/health` که بعدش اضافه شده عمومی است: اولین `curl`ِ بالا بدونِ توکن `200 ok` گرفت. این عمدی است، چون یک load balancer باید بتواند بدونِ ورود بپرسد «زنده‌ای؟»، و در عین حال ساده‌ترین راهِ ناخواسته در معرضِ عموم گذاشتنِ یک مسیر هم هست. دومی تازه است: `route_layer` است، نه `layer`. `route_layer` فقط رویِ درخواست‌هایی اجرا می‌شود که به یک مسیر بخورند. درخواست به مسیری که وجود ندارد چه توکن داشته باشد چه نه `404` می‌گیرد. با `layer` اول `401` می‌گرفت، که به پرسشِ اشتباه جواب می‌دهد: فراخواننده هیچ‌وقت چیزی را که وجود دارد نخواسته بود.

### خوب شکست‌خوردن: یک `AuthError`ِ نوع‌دار

نسخه‌یِ سریع به هر شکستی با یک `401`ِ خالی جواب می‌دهد. این هم دیباگ را سخت می‌کند و هم تست را، پس کریت یک شمارشی دارد، `AuthError`، با پنج حالت (`Missing`، `Malformed`، `BadSignature`، `WrongAlgorithm`، `Expired`)، و یک `IntoResponse` برایش، همان حرکتِ `AnimeError` در [۳.۲.۳](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.fa.md). `require_auth` یک `Result<Response, AuthError>` برمی‌گرداند، پس یک `Err` به پاسخ تبدیل می‌شود. سرورِ تمام‌شده (`examples/08-admin-server.rs`، که اول باید کدِ خودت پاس شود) این‌طور جواب می‌دهد:

```text
$ curl -i http://127.0.0.1:3191/whoami
HTTP/1.1 401 Unauthorized
content-type: application/json
www-authenticate: Bearer
content-length: 25
date: Sun, 04 Oct 2026 11:19:44 GMT

{"error":"missing_token"}
$ curl -i -H "Authorization: Bearer nope" http://127.0.0.1:3191/whoami
HTTP/1.1 401 Unauthorized
content-type: application/json
www-authenticate: Bearer
content-length: 25
date: Sun, 04 Oct 2026 11:19:44 GMT

{"error":"invalid_token"}
```

دو تصمیم در آن است. هدرِ `WWW-Authenticate: Bearer` همان چیزی است که RFC 6750 می‌گوید یک `401` برایِ توکنِ bearer باید داشته باشد؛ نامِ طرحی را که کلاینت باید می‌فرستاد می‌گوید. و بدنه پنج حالتِ درونی را به سه کد فشرده می‌کند. `missing_token` به کلاینت می‌گوید وارد شود، `token_expired` می‌گوید تازه کند یا دوباره وارد شود، و `invalid_token` همه‌یِ بقیه را می‌پوشاند: امضایِ بد، الگوریتمِ غلط، داده‌یِ بی‌معنی. مهاجمی که سرور را می‌کاود هیچ نمی‌فهمد توکنِ جعلی *چرا* رد شد. `AuthError`ِ پنج‌حالته برایِ لاگ و تست‌هایِ توست. کدِ سه‌حالته برایِ سیم است. شکلِ بدنه‌یِ خطا در کلِ یک API موضوعِ [۳.۸.۱](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.fa.md) است.

ترتیبِ بررسی‌ها در `verify_token` هم بخشی از طراحی است. امضا پیش از انقضا بررسی می‌شود، پس توکنِ جعلی بدونِ توجه به `exp`اش `BadSignature` است، و کسی نمی‌تواند از جوابِ انقضا بفهمد کدام توکن‌ها زمانی واقعی بوده‌اند.

### `401` یعنی «تو کی هستی»، `403` یعنی «اجازه نداری»: میان‌افزارِ دوم

[۳.۱.۳](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.fa.md) خط را کشید: `401` یعنی هویتِ معتبری نیست، `403` یعنی هویتِ شناخته‌شده‌ای که مجوز ندارد. در کریت این دو میان‌افزار است، هرکدام یک کار، روی هم. `require_auth` مشخص می‌کند چه کسی. دومی، که در پله‌یِ «بساز» خودت می‌نویسی، `Extension<AuthUser>` را می‌خواند و تصمیم می‌گیرد آن سوژه اجازه‌یِ عبور دارد یا نه. این همان **مجوزسنجی** (authorization) است، در برابرِ احرازِ هویت (authentication). `admin_app` هر دو را جلویِ `GET /admin` می‌گذارد، و فقط `require_auth` را جلویِ `GET /whoami`.

کدام اول اجرا شود همان پیازِ ۳.۲.۴ است: لایه‌یِ آخر اضافه‌شده بیرونی‌ترین است. پس بررسیِ ادمین در درون می‌نشیند، اول اضافه می‌شود، و `require_auth` بعد از آن اضافه می‌شود و آن را می‌پیچد:

```senpai-visual
{"kind":"concept","labels":["درخواست می‌رسد","require_auth: بیرونی‌ترین، اول اجرا می‌شود، جوابِ 401 می‌دهد","require_subject: درونِ آن، AuthUser لازم دارد، جوابِ 403 می‌دهد","هندلر: admin_ping یا whoami","درخواستِ بدونِ توکن هیچ‌وقت به بررسیِ 403 نمی‌رسد"]}
```

همین ترتیب دلیلِ این است که درخواستِ بدونِ توکن به `/admin` `401` می‌گیرد، نه `403`: میان‌افزارِ دوم حتی نمی‌تواند `AuthUser` را پیدا کند، چون هیچ‌وقت اجرا نمی‌شود. اگر برعکس بودند، `require_subject` اول اجرا می‌شد، تویِ افزونه‌ها `AuthUser` پیدا نمی‌کرد و با `500` شکست می‌خورد، همان اشتباهِ `04`. این هم سرورِ تمام‌شده، یک بار با `matin` (ادمین) و یک بار با `someone-else` که توکنِ کاملاً معتبری دارد:

```text
$ curl -i -H "Authorization: Bearer $MATIN" http://127.0.0.1:3191/admin
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 4
date: Sun, 04 Oct 2026 11:19:44 GMT

pong
$ curl -i -H "Authorization: Bearer $OTHER" http://127.0.0.1:3191/admin
HTTP/1.1 403 Forbidden
content-length: 0
date: Sun, 04 Oct 2026 11:19:45 GMT

$ curl -i http://127.0.0.1:3191/admin
HTTP/1.1 401 Unauthorized
content-type: application/json
www-authenticate: Bearer
content-length: 25
date: Sun, 04 Oct 2026 11:19:45 GMT

{"error":"missing_token"}
```

مقایسه‌یِ سوژه با رشته‌یِ `admin` جایگزینی برایِ یک مدلِ واقعیِ مجوز است. مدلِ واقعی می‌پرسد «این کاربر اجازه دارد این کار را با آن چیز بکند؟»، که موضوعِ [۳.۷.۵ — مدل‌سازیِ RBAC و مجوزها](../05-modelling-rbac-and-permissions/README.fa.md) است.

### اشتباه‌هایِ JWT که واقعاً سرویس‌ها را شکسته‌اند

همه‌یِ آنچه بالا آمد یکی‌یکی بود. این هم همه با هم، با خطِ کدِ این درس که جوابِ هرکدام است:

| اشتباه | چه خراب می‌شود | چه چیزی اینجا جلویش را می‌گیرد |
|---|---|---|
| اعتماد به فیلدِ `alg`ِ هدر (`none`، سردرگمیِ RS256/HS256) | مهاجم انتخاب می‌کند توکن چطور بررسی شود | `Validation::new(Algorithm::HS256)`؛ `verify_token` فقط HS256 را می‌پذیرد |
| بدونِ انقضا | توکنِ دزدیده‌شده برایِ همیشه کار می‌کند | `exp` اجباری است، و `verify_token` آن را با ساعت می‌سنجد |
| راز یا داده‌یِ شخصی تویِ ادعاها | هر کس توکن را دارد می‌خواندشان | ادعاها فقط `sub` و `iat` و `exp`‌اند |
| رازِ HMACِ کوتاه یا قابل‌حدس | مهاجمِ آفلاین رازِ HMAC را از رویِ هر یک توکن حدس می‌زند و بعد هر توکنی را جعل می‌کند | ۳۲ بایتِ تصادفی یا بیشتر از پیکربندی ([۳.۴.۱](../../04-configuration-and-app-structure/01-config-and-secrets/README.fa.md))، هیچ‌وقت از سورس |
| عمرِ طولانی بدونِ راهِ ابطال | توکنِ لورفته تا `exp` معتبر می‌ماند | `ttl_secs`ِ کوتاه، و [۳.۷.۴](../04-refresh-token-rotation-and-revocation/README.fa.md) |
| نگه‌داشتنِ توکن جایی که اسکریپت بخواندش | یک باگِ XSS و همه‌یِ توکن‌ها از `localStorage` دزدیده می‌شوند | این درس فقط توکن صادر و بررسی می‌کند؛ اینکه مرورگر کجا نگهش دارد مبادله‌یِ [۳.۷.۲](../02-sessions-vs-jwt/README.fa.md) است |
| گفتنِ دلیلِ شکستِ راستی‌آزمایی به کلاینت | یک غیب‌گویِ رایگان برایِ کاویدن | سه کدِ خطایِ عمومی، پنج کدِ خصوصی |

ردیفِ `localStorage` یادداشتِ بلندتری می‌خواهد. `localStorage` را هر اسکریپتی که رویِ صفحه اجرا شود می‌خواند، از جمله اسکریپتِ تزریق‌شده. کوکیِ علامت‌خورده با `HttpOnly` را اسکریپت اصلاً نمی‌تواند بخواند، به بهایِ مشکل‌هایِ دیگری که ۳.۷.۲ می‌سنجد. جایی نیست که هم بی‌هزینه باشد و هم از همه‌چیز امن، و دقیقاً برایِ همین آن درس وجود دارد.

---

## دست‌به‌کد

مثال‌ها را اجرا کن. `03` و `08` سرور هستند: یکی را در یک ترمینال راه بینداز، از ترمینالِ دیگر `curl` بزن، و وقتی کارت تمام شد با Ctrl+C متوقفش کن. `08` فقط بعد از اینکه نردبانِ تو پاس شود جواب می‌دهد.

```sh
cargo run -p p3-07-03-jwt-and-tower-middleware --example 01-anatomy-of-a-jwt
cargo run -p p3-07-03-jwt-and-tower-middleware --example 02-validation-knobs
cargo run -p p3-07-03-jwt-and-tower-middleware --example 03-protected-server
cargo run -p p3-07-03-jwt-and-tower-middleware --example 04-forgot-the-layer
cargo run -p p3-07-03-jwt-and-tower-middleware --example 08-admin-server
```

بعد مثال‌هایِ خراب. پشتِ یک feature قفل‌اند، و هر سه کامپایل نمی‌شوند:

```sh
cargo build -p p3-07-03-jwt-and-tower-middleware --example 05-extension-not-clone-broken --features broken
cargo build -p p3-07-03-jwt-and-tower-middleware --example 06-wrong-state-type-broken --features broken
cargo build -p p3-07-03-jwt-and-tower-middleware --example 07-error-not-a-response-broken --features broken
```

بعد تست‌ها. الان شکست می‌خورند، و اولین شکست پیامِ یک `todo!()` را نشانت می‌دهد:

```sh
cargo test -p p3-07-03-jwt-and-tower-middleware --test jwt_test bearer
```

```text
running 2 tests
test bearer_token_accepts_the_scheme_in_any_case ... FAILED
test bearer_token_rejects_everything_else ... FAILED

failures:

---- bearer_token_accepts_the_scheme_in_any_case stdout ----

thread 'bearer_token_accepts_the_scheme_in_any_case' (39828) panicked at phase3-backend-foundations\07-auth-and-security\03-jwt-and-tower-middleware\src\lib.rs:175:5:
not yet implemented: return the token from a well-formed Bearer header value, or None
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- bearer_token_rejects_everything_else stdout ----

thread 'bearer_token_rejects_everything_else' (2620) panicked at phase3-backend-foundations\07-auth-and-security\03-jwt-and-tower-middleware\src\lib.rs:175:5:
not yet implemented: return the token from a well-formed Bearer header value, or None


failures:
    bearer_token_accepts_the_scheme_in_any_case
    bearer_token_rejects_everything_else

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p p3-07-03-jwt-and-tower-middleware --test jwt_test`
```

عددِ داخلِ پرانتز بعد از `thread '...'` شناسه‌یِ ریسمان است و در هر اجرا عوض می‌شود. حالا اینها را امتحان کن:

۱. در `02-validation-knobs`، `strict.leeway = 0` را به `strict.leeway = 31` عوض کن. کدام خطِ خروجی عوض می‌شود، و چرا؟
۲. در `03-protected-server`، `Validation::new(Algorithm::HS256)` را طوری عوض کن که `HS512` را هم بپذیرد. کدام توکن حالا پذیرفته می‌شود که قبلاً نمی‌شد؟
۳. در `04-forgot-the-layer`، روتر را با همان لایه‌یِ سریعِ `from_fn_with_state` از `03` بپیچ. همان درخواست حالا چه می‌گیرد؟

---

## خطاهایی که خواهی دید

هر رونوشتِ زیر خروجیِ واقعیِ مثالِ نام‌برده است، با هشدارهایِ `todo!()`ِ خودِ این درس حذف‌شده. عددِ بلند در نامِ فایل‌هایِ `long-type-...txt` در هر اجرا فرق می‌کند. سه‌تایِ اول `E0277` هستند، و دومی و سومی از آن نوعی‌اند که `axum` به آن مشهور است: پیام نوعی را نام می‌برد که هیچ‌وقت ننوشتی و نمی‌گوید کدام آرگومانت غلط است.

### `E0277` — `Extension<T>` به `T: Clone` نیاز دارد

```text
error[E0277]: the trait bound `fn(Extension<AuthUser>) -> impl Future<Output = String> {whoami}: Handler<_, _>` is not satisfied
   --> phase3-backend-foundations\07-auth-and-security\03-jwt-and-tower-middleware\examples\05-extension-not-clone-broken.rs:17:62
    |
 17 |     let _router: Router = Router::new().route("/whoami", get(whoami));
    |                                                          --- ^^^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(Extension<AuthUser>) -> impl Future<Output = String> {whoami}`
    |                                                          |
    |                                                          required by a bound introduced by this call
    |
    = note: Consider using `#[axum::debug_handler]` to improve the error message
note: required by a bound in `axum::routing::get`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\method_routing.rs:167:16
    |
167 |             H: Handler<T, S>,
    |                ^^^^^^^^^^^^^ required by this bound in `get`
...
441 | top_level_handler_fn!(get, GET);
    | -------------------------------
    | |                     |
    | |                     required by a bound in this function
    | in this macro invocation
    = note: this error originates in the macro `top_level_handler_fn` (in Nightly builds, run with -Z macro-backtrace for more info)

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-07-03-jwt-and-tower-middleware` (example "05-extension-not-clone-broken") due to 1 previous error
```

**کامپایلر به چه اعتراض دارد:** خطا به `get(whoami)` اشاره می‌کند، نه `AuthUser`، و فقط می‌گوید تابع یک `Handler` نیست. یعنی یکی از آرگومان‌هایش اکسترکتور نیست، و کامپایلر نمی‌گوید کدام. سرنخِ واقعی خطِ `note:` است: `#[axum::debug_handler]` را رویِ `whoami` بگذار و همین اشتباه علیهِ خودِ آرگومان گزارش می‌شود.

**راه‌حل:** رویِ نوع `Clone` را derive کن.

```rust
#[derive(Clone)]
struct AuthUser(String);
```

**چرا این راه‌حل است:** اکسترکتورِ `Extension` مقدار را از درخواست بیرون نمی‌کشد، کلونش می‌کند، چون یک افزونه ممکن است چند هندلر و لایه بخوانندش. پس نوعِ ذخیره‌شده باید `Clone` باشد. `AuthUser` در `src/lib.rs` `Debug, Clone` را derive می‌کند. اگر کلون‌کردنِ یک نوع گران است، یک `Arc` از آن را ذخیره کن.

### `E0277` — میان‌افزار حالتی غیر از حالتِ لایه می‌خواهد

```text
error[E0277]: the trait bound `FromFn<fn(..., ..., ...) -> ... {check}, ..., ..., _>: Service<...>` is not satisfied
   --> phase3-backend-foundations\07-auth-and-security\03-jwt-and-tower-middleware\examples\06-wrong-state-type-broken.rs:22:22
    |
 22 |         .route_layer(from_fn_with_state(config, check));
    |          ----------- ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
    |          |
    |          required by a bound introduced by this call
    |
    = help: the trait `tower_service::Service<axum::http::Request<Body>>` is not implemented for `FromFn<fn(State<String>, ..., ...) -> ... {check}, ..., ..., _>`
    = help: the following other types implement trait `tower_service::Service<Request>`:
              axum::middleware::FromFn<F, S, I, (T1, T2)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8, T9)>
            and 8 others
note: required by a bound in `Router::<S>::route_layer`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:324:21
    |
321 |     pub fn route_layer<L>(self, layer: L) -> Self
    |            ----------- required by a bound in this associated function
...
324 |         L::Service: Service<Request> + Clone + Send + Sync + 'static,
    |                     ^^^^^^^^^^^^^^^^ required by this bound in `Router::<S>::route_layer`
    = note: the full name for the type has been written to 'M:\SenPai-Rust-Journey\target\debug\examples\06_wrong_state_type_broken.long-type-12508816414026072018.txt'
    = note: consider using `--verbose` to print the full type name to the console

error[E0277]: the trait bound `FromFn<fn(..., ..., ...) -> ... {check}, ..., ..., _>: Service<...>` is not satisfied
  --> phase3-backend-foundations\07-auth-and-security\03-jwt-and-tower-middleware\examples\06-wrong-state-type-broken.rs:20:27
   |
20 |       let _router: Router = Router::new()
   |  ___________________________^
21 | |         .route("/", get(|| async { "ok" }))
22 | |         .route_layer(from_fn_with_state(config, check));
   | |_______________________________________________________^ unsatisfied trait bound
   |
   = help: the trait `tower_service::Service<axum::http::Request<Body>>` is not implemented for `FromFn<fn(State<String>, ..., ...) -> ... {check}, ..., ..., _>`
   = help: the following other types implement trait `tower_service::Service<Request>`:
             axum::middleware::FromFn<F, S, I, (T1, T2)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8, T9)>
           and 8 others
   = note: the full name for the type has been written to 'M:\SenPai-Rust-Journey\target\debug\examples\06_wrong_state_type_broken.long-type-12508816414026072018.txt'
   = note: consider using `--verbose` to print the full type name to the console

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-07-03-jwt-and-tower-middleware` (example "06-wrong-state-type-broken") due to 2 previous errors
```

**کامپایلر به چه اعتراض دارد:** `from_fn_with_state` یک مقدارِ `FromFn` ساخت، و `route_layer` لازم دارد آن یک `tower::Service` باشد. نیست، و کامپایلر نمی‌تواند در یک خط بگوید چرا، چون `FromFn` فقط وقتی سرویس است که *هر* آرگومانِ تابعت چیزی باشد که `axum` بتواند تأمین کند. تنها سرنخ در خطِ `help:` است: آرگومانِ اولِ تابع `State<String>` است، و حالتی که به لایه داده شد یک `JwtConfig` بود. دو خطا یک واقعیت‌اند که دو بار گزارش شده، یک بار رویِ فراخوانی و یک بار رویِ کلِ عبارت.

**راه‌حل:** نوع‌ها را هم‌خوان کن. یا تابع `State<JwtConfig>` بگیرد، یا به لایه یک `String` داده شود:

```rust
async fn check(State(_config): State<JwtConfig>, request: Request, next: Next) -> Response {
    next.run(request).await
}
```

**چرا این راه‌حل است:** `from_fn_with_state(state, f)` مقدارِ `state` را نگه می‌دارد و در هر درخواست `State<S>` را برایِ `f` از آن بیرون می‌کشد. اگر `f` یک `S`ِ دیگر بخواهد، راهی برایِ ساختنِ آرگومان‌هایش نیست، و قیدِ «سرویس است» می‌شکند. وقتی این خطا را رویِ یک میان‌افزارِ `from_fn` دیدی، آرگومان‌ها را به این ترتیب بررسی کن: نوعِ حالت، اکسترکتورهایی که پیش از `Request` می‌آیند (باید `FromRequestParts` را پیاده کرده باشند، و `Request` و `Next` باید آخر بیایند)، بعد نوعِ برگشتی، که خطایِ بعدی است.

### `E0277` — نوعِ خطایِ میان‌افزار یک `IntoResponse` نیست

```text
error[E0277]: the trait bound `FromFn<fn(Request<Body>, ...) -> ... {gate}, (), ..., _>: Service<...>` is not satisfied
   --> phase3-backend-foundations\07-auth-and-security\03-jwt-and-tower-middleware\examples\07-error-not-a-response-broken.rs:26:22
    |
 26 |         .route_layer(from_fn(gate));
    |          ----------- ^^^^^^^^^^^^^ unsatisfied trait bound
    |          |
    |          required by a bound introduced by this call
    |
    = help: the trait `tower_service::Service<axum::http::Request<Body>>` is not implemented for `FromFn<fn(Request<Body>, Next) -> ... {gate}, (), ..., _>`
    = help: the following other types implement trait `tower_service::Service<Request>`:
              axum::middleware::FromFn<F, S, I, (T1, T2)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8)>
              axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8, T9)>
            and 8 others
note: required by a bound in `Router::<S>::route_layer`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:324:21
    |
321 |     pub fn route_layer<L>(self, layer: L) -> Self
    |            ----------- required by a bound in this associated function
...
324 |         L::Service: Service<Request> + Clone + Send + Sync + 'static,
    |                     ^^^^^^^^^^^^^^^^ required by this bound in `Router::<S>::route_layer`
    = note: the full name for the type has been written to 'M:\SenPai-Rust-Journey\target\debug\examples\07_error_not_a_response_broken.long-type-15268988596771203291.txt'
    = note: consider using `--verbose` to print the full type name to the console

error[E0277]: the trait bound `FromFn<fn(Request<Body>, ...) -> ... {gate}, (), ..., _>: Service<...>` is not satisfied
  --> phase3-backend-foundations\07-auth-and-security\03-jwt-and-tower-middleware\examples\07-error-not-a-response-broken.rs:24:27
   |
24 |       let _router: Router = Router::new()
   |  ___________________________^
25 | |         .route("/", get(|| async { "ok" }))
26 | |         .route_layer(from_fn(gate));
   | |___________________________________^ unsatisfied trait bound
   |
   = help: the trait `tower_service::Service<axum::http::Request<Body>>` is not implemented for `FromFn<fn(Request<Body>, Next) -> ... {gate}, (), ..., _>`
   = help: the following other types implement trait `tower_service::Service<Request>`:
             axum::middleware::FromFn<F, S, I, (T1, T2)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8)>
             axum::middleware::FromFn<F, S, I, (T1, T2, T3, T4, T5, T6, T7, T8, T9)>
           and 8 others
   = note: the full name for the type has been written to 'M:\SenPai-Rust-Journey\target\debug\examples\07_error_not_a_response_broken.long-type-15268988596771203291.txt'
   = note: consider using `--verbose` to print the full type name to the console

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-07-03-jwt-and-tower-middleware` (example "07-error-not-a-response-broken") due to 2 previous errors
```

**کامپایلر به چه اعتراض دارد:** همان پیامِ قبلی، بدونِ هیچ حرفی دربارهِ علت. این بار همه‌یِ آرگومان‌ها درست‌اند: خطِ `help:` نشان می‌دهد `fn(Request<Body>, Next)`. چیزی که غلط است نوعِ برگشتی است، `Result<Response, NotAllowed>`: تابعِ `from_fn` باید چیزی برگرداند که `IntoResponse` را پیاده کرده باشد، و `Result<T, E>` فقط وقتی پیاده کرده که هر دوی `T` و `E` کرده باشند.

**راه‌حل:** برایِ خطا `IntoResponse` پیاده کن، یا نوعی برگردان که از قبل دارد. `AuthError`ِ این کریت اولی را می‌کند؛ نسخه‌یِ سریع در `03` دومی را با `StatusCode`:

```rust
impl IntoResponse for NotAllowed {
    fn into_response(self) -> Response {
        StatusCode::UNAUTHORIZED.into_response()
    }
}
```

**چرا این راه‌حل است:** میان‌افزاری که شکست می‌خورد باید *جواب بدهد*، چون کدِ بیرونی‌ای نیست که خطا را بگیرد و `hyper` اتصال را می‌بست. `axum` یک `Err` را با صدا زدنِ `into_response` رویِ آن به پاسخ تبدیل می‌کند، و برایِ همین ۳.۲.۴ برایِ لایه‌هایِ دست‌نویس `Error = Infallible` را لازم داشت. `from_fn` این تبدیل را برایت می‌کند، ولی فقط اگر خطا بلد باشد. برایِ همین هم `AuthError` یک شمارشی با `IntoResponse` است: یک جا تصمیم می‌گیرد هر شکستِ احرازِ هویت رویِ سیم چه شکلی باشد.

### خطای کامپایلر نیست: `500` با «Missing request extension»

چهارمین چیزی که می‌بینی کد خطا ندارد، چون کامپایل می‌شود. هندلری که `Extension<AuthUser>` می‌گیرد، رویِ مسیری که جلویش لایه‌یِ احرازِ هویت نیست، `500` و همان پیامی را می‌دهد که در `examples/04-forgot-the-layer.rs` دیدی. راه‌حل هیچ‌وقت در هندلر نیست. مسیر را پشتِ `require_auth` بگذار، همان‌طور که `app` می‌کند، و تستی بنویس که بدونِ توکن به آن درخواست بفرستد. اگر `401` جواب داد، لایه آنجاست. اگر `500` داد، نیست.

---

## تمرین

### گرم‌کردن

<details>
<summary>دوستی می‌گوید «توکن امن است، base64 است، کسی نمی‌تواند بخواندش». درست می‌گوید؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

نه. base64url کدگذاری است، نه رمزگذاری: هر کس توکن را دارد می‌تواند بی‌هیچ کلیدی پی‌لود را دیکود کند. چیزی که امضا از آن محافظت می‌کند *یکپارچگی* است: یک نویسه را عوض کنی راستی‌آزمایی شکست می‌خورد. از *محرمانگی* محافظت نمی‌کند. پس پی‌لود نباید هیچ چیزِ خصوصی داشته باشد.

</details>

<details>
<summary>توکنی در <code>1_700_000_000</code> با عمرِ ۳۶۰۰ ثانیه و ۳۰ ثانیه مهلتِ تحمل صادر شده. در <code>1_700_003_630</code> معتبر است؟ در <code>1_700_003_631</code>؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

در `1_700_003_630` معتبر است، در `1_700_003_631` منقضی. `exp` برابرِ `1_700_003_600` است و قاعده می‌گوید «وقتی منقضی است که now بزرگ‌تر از `exp + leeway` باشد»، پس دقیقاً `exp + leeway` هنوز معتبر است.

</details>

<details>
<summary>یک روتر اول <code>.route("/whoami", ...)</code> دارد، بعد <code>.route_layer(from_fn_with_state(config, require_auth))</code>، بعد <code>.route("/health", ...)</code>. آیا <code>/health</code> توکن لازم دارد؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

نه. یک لایه فقط مسیرهایی را می‌پیچد که پیش از آن اضافه شده‌اند، پس `/health` عمومی است. برایِ بررسیِ سلامت مفید است و وقتی ناخواسته اتفاق بیفتد خطرناک.

</details>

<details>
<summary>درخواستی به <code>/admin</code> توکنِ معتبرِ کاربرِ <code>someone-else</code> را دارد، و ادمین <code>matin</code> است. جواب <code>401</code> است یا <code>403</code>؟ اگر اصلاً توکن نباشد چه؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

برایِ توکنِ معتبر `403 Forbidden`: سرور می‌داند تو کیستی و اجازه نداری. بدونِ توکن `401 Unauthorized` است، چون `require_auth` لایه‌یِ بیرونی است و پیش از آنکه بررسیِ سوژه اجرا شود جواب می‌دهد.

</details>

<details>
<summary>راستی‌آزمایی‌ای با <code>algorithms = [HS256, HS512]</code> «برایِ انعطاف» تنظیم شده. چه چیزی را از دست داد؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

حالا راهِ دومی برایِ بررسی‌شدن می‌پذیرد، و هر عضوِ اضافه جایِ بیشتری برایِ باگ‌هایِ سردرگمیِ الگوریتم است. باید دقیقاً الگوریتم‌هایی را فهرست کند که خودِ سرور صادر می‌کند، و در این درس یکی است.

</details>

### تعمیر

هر مثالِ خراب را طوری درست کن که کامپایل شود، یا درست جواب بدهد:

۱. `examples/05-extension-not-clone-broken.rs` کامپایل شود.
۲. `examples/06-wrong-state-type-broken.rs` کامپایل شود.
۳. `examples/07-error-not-a-response-broken.rs` کامپایل شود، و شکستش `401` جواب بدهد، نه یک `Err`ِ بی‌صاحب.
۴. `examples/04-forgot-the-layer.rs` به‌جایِ `500` جوابِ `401` بدهد. این یکی اول پله‌یِ «پیاده‌سازی» را لازم دارد، چون راه‌حلش گذاشتنِ مسیر پشتِ `require_auth`ِ خودت است.

### پیاده‌سازی

چهار تابع در `src/lib.rs`، به همین ترتیب. همه‌چیزِ بالایِ خطِ «the ladder» داده شده است: `Claims`، `Clock`، `JwtConfig`، `AuthError` با `IntoResponse`اش، `AuthUser`، `whoami` و `app`.

۱. `bearer_token`: توکن را از مقدارِ هدرِ `Authorization` بیرون بکش.
۲. `issue_token`: ادعاهایِ HS256 را برایِ یک کاربر با ساعتِ پیکربندی امضا کن.
۳. `verify_token`: راستی‌آزماییِ کامل، با نتیجه‌ها به ترتیبی که توضیحِ مستند فهرست کرده.
۴. `require_auth`: میان‌افزار، که سه‌تایِ دیگر را به هم وصل می‌کند.

```sh
cargo test -p p3-07-03-jwt-and-tower-middleware --test jwt_test
```

توضیحِ مستندِ بالایِ هر تابع کلِ مشخصات است، از جمله ترتیبِ دقیقِ بررسی‌هایِ `verify_token`. هیچ‌وقت لازم نیست تست‌ها را باز کنی. تست‌ها هرجا لازم باشد توکن را با دست می‌سازند (یک پی‌لودِ جعلی، توکنِ بدونِ `exp`، هدرِ `alg: none`)، و همه ساعتِ ثابت دارند، پس هیچ‌کدام به زمانِ اجرا بستگی ندارند. تست‌ها با `oneshot` درخواست می‌فرستند، مثلِ ۳.۲.۱. import‌هایِ `jsonwebtoken` را باید خودت اضافه کنی. وقتی همه پاس شد، `04` و `08` کار می‌کنند.

### بساز

`admin_app`، در همان فایل: یک روتر با `GET /whoami` برایِ هر توکنِ معتبر و `GET /admin` فقط برایِ سوژه‌ای که آرگومانِ `admin` نام می‌برد. به یک میان‌افزارِ `from_fn_with_state`ِ دوم نیاز دارد که خودت می‌نویسی، میان‌افزاری که `AuthUser`ای را که `require_auth`ِ تو ذخیره کرده می‌خواند. توضیحِ مستند کدهایِ وضعیت را می‌گوید، از جمله اینکه وقتی هر دو بررسی شکست می‌خوردند کدام برنده است.

```sh
cargo test -p p3-07-03-jwt-and-tower-middleware --test jwt_test admin
```

پیش از نوشتن به ترتیبِ اضافه‌کردنِ دو لایه فکر کن، با پیازِ ۳.۲.۴. ترتیبِ غلط کامپایل می‌شود و با همان `500`ِ `04` شکست می‌خورد.

### چالش (اختیاری)

چرخشِ کلید. رازی که نشود عوضش کرد برایِ همیشه لو رفته می‌ماند، پس سامانه‌هایِ واقعی با یک شناسه‌یِ کلید (`kid` در هدر) امضا می‌کنند و کلیدهایِ قدیمی را فقط برایِ راستی‌آزمایی نگه می‌دارند. به `JwtConfig` فهرستی از رازهایِ بازنشسته بده و `verify_token` را طوری کن که کلیدی را امتحان کند که `kid`ِ توکن نام می‌برد. هیچ تستی این را نمی‌سنجد. تست‌هایِ خودت را با ساعتِ ثابت بنویس، و تصمیم بگیر توکنی با `kid`ِ ناشناخته چه باشد، و چرا نباید برگردد و همه‌یِ کلیدها را امتحان کند. این به اینکه رازها چطور بار می‌شوند در [۳.۴.۱](../../04-configuration-and-app-structure/01-config-and-secrets/README.fa.md) اشاره می‌کند.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| JWT | توکنِ امضاشده: هدر، پی‌لود و امضایِ base64url که با نقطه به هم وصل‌اند | ورودِ بی‌حالت برایِ یک API |
| ادعا (claims) | فیلدهایِ پی‌لود: `sub`، `iat`، `exp` و مالِ خودت | تصمیم‌گرفتن درخواست از طرفِ کیست |
| امضاشده، نه رمزگذاری‌شده | پی‌لود را هر کس می‌خواند، ولی دستکاری آشکار می‌شود | اینکه چه چیزی را مجاز هستی در توکن بگذاری |
| HS256 | HMAC-SHA256 با یک رازِ مشترک | الگوریتمِ امضایِ این درس |
| قفل‌کردنِ الگوریتم | فقط الگوریتمی را بپذیر که خودت صادر می‌کنی، نه آنکه هدر نام می‌برد | جلوگیری از `alg: none` و سردرگمیِ الگوریتم |
| مهلتِ تحمل (leeway) | ثانیه‌هایِ ارفاق بعد از `exp` برایِ اختلافِ ساعت | `Validation::leeway` و `JwtConfig::leeway_secs` |
| ساعتِ تزریق‌شده | «اکنون» به‌شکلِ یک تابع داده می‌شود، نه خوانده‌شده از سیستم | تستِ قطعی برایِ هر چیزِ وابسته به زمان |
| توکنِ bearer | اعتباری که به‌شکلِ `Authorization: Bearer <token>` فرستاده می‌شود | هر درخواستِ محافظت‌شده |
| `from_fn_with_state` | میان‌بُرِ `from_fn` با حالتی که به میان‌افزار داده می‌شود | احرازِ هویت، هر چیزی که پیکربندی می‌خواهد |
| افزونه‌هایِ درخواست (extensions) | نقشه‌ای رویِ درخواست با کلیدِ نوع؛ `Extension<T>` آن را می‌خواند | رساندنِ هویتِ راستی‌آزمایی‌شده به هندلرها |
| `401` / `403` | هویتِ معتبری نیست / هویتِ شناخته‌شده بدونِ مجوز | احرازِ هویت در برابرِ مجوزسنجی |

### الان می‌دانی

- JWT همان `header.payload.signature` است، هر تکه base64url، و فقط امضا به راز نیاز دارد. پی‌لود را هر کس می‌خواند.
- `jsonwebtoken` نسخه‌ی ۹ با یک `Validation` راستی‌آزمایی می‌کند: `exp` اجباری و بررسی‌شده، `leeway`ِ پیش‌فرضِ ۶۰ ثانیه، و فهرستِ `algorithms` که در ابتدا همان یک الگوریتمی است که نام بردی.
- الگوریتم را قفل می‌کنی تا کلاینت هیچ‌وقت انتخاب نکند چطور بررسی شود، و `alg: none` اصلاً پارس نمی‌شود.
- زمان یک ورودی است: ساعت را تزریق کن، بررسیِ `exp`ِ خودِ کتابخانه را خاموش کن و خودت مقایسه کن، تا هر حالتِ انقضا یک تستِ واحدِ ساده باشد.
- `from_fn_with_state` میان‌افزارِ احرازِ هویت را یک تابع می‌کند: با یک `Err` که `IntoResponse` دارد رد کن، یا یک `AuthUser` در افزونه‌هایِ درخواست بگذار و `next` را صدا بزن.
- یک لایه فقط مسیرهایِ پیش از خودش را می‌پیچد، و آخرین لایه‌یِ اضافه‌شده بیرونی‌ترین است، که دلیلِ آمدنِ `401` پیش از `403` است.
- پنج حالتِ خطایِ درونی می‌توانند هرکدام یک کدِ عمومی باشند، و امضا پیش از انقضا بررسی می‌شود تا توکنِ جعلی چیزی نفهمد.

### بعداً کامل‌تر می‌بینی

- **چرا یک JWTِ دزدیده‌شده را نمی‌شود ساده پس گرفت، توکن‌هایِ دسترسیِ کوتاه‌عمر، و چرخشِ توکنِ تازه‌سازی** — [۳.۷.۴ — چرخشِ refresh token و باطل‌سازیش](../04-refresh-token-rotation-and-revocation/README.fa.md)
- **یک مدلِ واقعیِ مجوز به‌جایِ `subject == "admin"`** — [۳.۷.۵ — مدل‌سازیِ RBAC و مجوزها](../05-modelling-rbac-and-permissions/README.fa.md)
- **مرورگر توکن را کجا نگه دارد، و مبادله‌یِ نشست در برابرِ JWT** — [۳.۷.۲ — Session در برابرِ JWT: مصالحه‌یِ واقعی](../02-sessions-vs-jwt/README.fa.md)
- **بارکردنِ رازِ امضا از پیکربندی به‌جایِ یک ثابت** — [۳.۴.۱ — پیکربندیِ ۱۲فاکتوری و سکرت‌ها](../../04-configuration-and-app-structure/01-config-and-secrets/README.fa.md)
- **یک شکلِ بدنه‌یِ خطا برایِ کلِ API، به‌جایِ سه کد فقط برایِ احرازِ هویت** — [۳.۸.۱ — پاکت‌هایِ خطایِ یکدست](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.fa.md)
- **اجازه‌دادن به مرورگری در مبدأیِ دیگر که هدرِ `Authorization` بفرستد** — [۳.۲.۵ — CORS و اتصال به فرانت‌اند](../../02-axum-and-rest-api-design/05-cors-and-frontend-integration/README.fa.md)

### می‌توانی توضیح بدهی؟

- سه تکه‌یِ یک JWT چیست، و خواندنِ کدام‌یک به راز نیاز دارد؟
- چرا «امضاشده، نه رمزگذاری‌شده» اولین چیزی است که باید بدانی، و چه چیزی هیچ‌وقت نباید در ادعاها برود؟
- `Validation::new(Algorithm::HS256)` به‌طورِ پیش‌فرض چه چیزهایی را بررسی می‌کند، و `leeway`اش چه چیزِ شگفت‌آوری دارد؟
- `alg: none` چیست، سردرگمیِ الگوریتم چیست، و کدام یک عادت جلویِ هر دو را می‌گیرد؟
- چرا این کریت ساعت را پارامتر می‌گیرد، و دقیقاً چه چیزی به‌خاطرش از کتابخانه به `verify_token` آمد؟
- هویت چطور از میان‌افزار به هندلر می‌رسد، و اگر چیزی آن را نگذاشته باشد چه می‌شود؟
- چرا `/admin` بدونِ توکن `401` می‌دهد و با کاربرِ غلط `403`، و ترتیبِ دو لایه چه ربطی به آن دارد؟
- چرا کلاینت سه کدِ خطا می‌بیند در حالی که کدِ تو پنج حالتِ خطا دارد؟

---

## بیشتر

- [RFC 7519: JSON Web Token](https://www.rfc-editor.org/rfc/rfc7519): ادعاها، از جمله آن‌هایی که این درس رد شد (`iss`، `aud`، `nbf`، `jti`).
- [RFC 8725: JSON Web Token Best Current Practices](https://www.rfc-editor.org/rfc/rfc8725): فهرستِ رسمیِ اشتباه‌هایِ جدولِ بالا، با نامِ حمله‌ها.
- [RFC 6750: Bearer Token Usage](https://www.rfc-editor.org/rfc/rfc6750): چرا `401` هدرِ `WWW-Authenticate: Bearer` را دارد.
- [`jsonwebtoken` نسخه‌ی 9.3.1 در docs.rs](https://docs.rs/jsonwebtoken/9.3.1/jsonwebtoken/): `Validation`، `Algorithm` و حالت‌هایِ `ErrorKind`.
- [`axum::middleware`](https://docs.rs/axum/0.8.9/axum/middleware/index.html): `from_fn`، `from_fn_with_state` و قاعده‌هایِ ترتیبِ لایه‌ها.
- [OWASP JWT Cheat Sheet for Java](https://cheatsheetseries.owasp.org/cheatsheets/JSON_Web_Token_for_Java_Cheat_Sheet.html): برایِ جاوا نوشته شده، ولی فهرستِ حمله‌هایِ توکن و جایِ نگه‌داریِ آن مستقل از زبان است.
