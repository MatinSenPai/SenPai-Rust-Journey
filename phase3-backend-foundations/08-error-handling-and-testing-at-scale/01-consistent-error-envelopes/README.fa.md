# ۳.۸.۱ — پاکت‌هایِ خطایِ یکدست

## در یک نگاه

بعد از این درس می‌توانی:

- یک شکلِ JSON برایِ همه‌ی شکست‌هایی که یک API می‌تواند بسازد تعریف کنی، و بگویی کلاینت روی کدام فیلد اجازه‌ی تصمیم‌گیری دارد و کدام را هرگز نباید پارس کند.
- آن را در یک جا بسازی: یک enumِ `ApiError` که `IntoResponse`ِ آن تنها کدی است که شکست را به کدِ وضعیت و بدنه تبدیل می‌کند، از جمله شکست‌هایی که `axum` پیش از رسیدنِ هندلر خودش می‌سازد (JSONِ خراب، `{id}`ِ بد، مسیرِ ناشناخته، متدِ غلط).
- نگذاری یک `5xx` جزئیاتِ درونی را فاش کند، در حالی که یک `4xx` دقیقاً می‌گوید کلاینت چه چیزی را باید درست کند، با خطایِ اعتبارسنجیِ هر فیلد در همان پاکت.
- پاکتِ خودت را با «problem details» در RFC 9457 (`application/problem+json`) مقایسه کنی و بگویی هر کدام چه چیزی می‌دهد.

**زمان:** حدود ۱۰۰ دقیقه · **پیش‌نیاز:**
[۳.۲.۳ — عملیاتِ CRUD روی کاتالوگِ انیمه (در حافظه)](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.fa.md)،
[۲.۵.۴ — طراحیِ رده‌بندیِ خطا برایِ یک سرویس](../../../phase2-intermediate/05-error-handling/04-error-taxonomy-for-a-service/README.fa.md)،
[۳.۲.۲ — نوشتنِ اکسترکتورِ خودت (`FromRequestParts`)](../../02-axum-and-rest-api-design/02-writing-your-own-extractor/README.fa.md)،
[۳.۱.۳ — چیزهایی از HTTP که باید بدانی](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.fa.md)

---

## چرا اهمیت دارد

۳.۲.۳ با یک قول تمام شد. هندلرش به یک انیمه‌یِ پیدانشده با `{"error":"anime not found"}` جواب می‌داد، و در همان transcript ردکننده‌هایِ (rejection) خودِ `axum` با متنِ ساده جواب می‌دادند: `Invalid URL: ...` و `Expected request with Content-Type: ...`. یک API، و بعد از فقط یک درس دو شکل. حالا سی endpoint و چهار نویسنده را تصور کن. frontend نمی‌تواند یک error handler بنویسد و باید برایِ هر endpoint حالتِ خاص بگذارد. generatorِ SDK نمی‌تواند schema را حدس بزند. مهندسِ پشتیبانی نمی‌تواند روی لاگ‌ها `jq '.error.code'` بزند و بشمارد.

در DRF بیشترِ این را از یک جا می‌گیری: `REST_FRAMEWORK["EXCEPTION_HANDLER"]` هر استثنایی را که viewها می‌اندازند می‌پیچد، پس هر خطا از یک تابع بیرون می‌آید. `axum` چنین تنظیمی ندارد. چیزی که دارد سیستمِ نوع است، و راه‌حل همان ایده است که با نوع‌ها ساخته می‌شود: شکل را یک‌بار تعیین کن، در یک `impl` بساز، و دورزدنش را از نظرِ ساختاری سخت کن.

این درسِ امنیت هم هست. اینکه یک `500` به یک غریبه چه می‌گوید یک تصمیم است، و ساده‌ترین راهِ غلط‌کردنش این است که بگذاری متنِ خطایِ خودِ سرور به کلاینت برسد. این تصمیم را یک‌بار همین‌جا می‌گیری تا هیچ هندلری دوباره نگیرد.

---

## مفهوم

### هفت شکست، سه شکل

این یک API کوچک بدونِ هیچ پاکتی است: یک هندلرِ ما که با `{"error": "..."}` شکست می‌خورد، و `axum` که بقیه‌ی کارها را به‌صورتِ پیش‌فرض می‌کند. `examples/01-four-shapes.rs` هفت درخواستِ بد برایش می‌فرستد و آنچه برمی‌گردد را چاپ می‌کند:

```text
GET /shows/9 -> 404 [application/json]
    {"error":"show 9 not found"}
GET /shows/abc -> 400 [text/plain; charset=utf-8]
    Invalid URL: Cannot parse `abc` to a `u64`
POST /shows -> 400 [text/plain; charset=utf-8]
    Failed to parse the request body as JSON: EOF while parsing an object at line 1 column 1
POST /shows -> 422 [text/plain; charset=utf-8]
    Failed to deserialize the JSON body into the target type: missing field `title` at line 1 column 2
POST /shows -> 415 [text/plain; charset=utf-8]
    Expected request with `Content-Type: application/json`
GET /nope -> 404 [-]
    (empty body)
DELETE /shows/1 -> 405 [-]
    (empty body)
```

شکل‌ها را بشمار: شیءِ JSONِ خودمان، متنِ ساده‌یِ انگلیسی، و یک پاسخِ خالی بدونِ `Content-Type` (دو بار). هیچ‌کدام به‌تنهایی غلط نیست: پیش‌فرض‌هایِ `axum` معقول‌اند. مشکل فقط این است که سه چیزِ متفاوت‌اند.

### شکل: یک پاکت با `code`ِ پایدار

هر بدنه‌یِ خطا در این درس همان لایه‌ی بیرونی و همان فیلدهایِ درونی را دارد:

```json
{"error": {"code": "not_found", "message": "show 999 not found"}}
```

**پاکتِ خطا (error envelope)** همین شکلِ ثابتِ بیرونی است. دو فیلد دو کارِ جدا دارند:

- **`code`** برایِ ماشین و پایدار است. کلاینت می‌تواند روی آن `switch` کند. بخشی از قراردادِ API است، مثلِ یک URL.
- **`message`** برایِ انسان است: یک خطِ لاگ، ترمینالِ یک توسعه‌دهنده، یک toast. می‌شود بازنویسی، اصلاح یا ترجمه‌اش کرد بی‌آنکه کسی خراب شود، *چون کسی اجازه ندارد پارسش کند*. **`code` قرارداد است، `message` ادبِ پاسخ.**

فیلدِ سوم، `fields`، فقط در خطاهایِ اعتبارسنجی (پایین‌تر) می‌آید و در بقیه غایب است. پاکت همان جایی است که ردهِ کدِ وضعیت از ۳.۱.۳ نتیجه می‌دهد: کدِ وضعیتِ HTTP می‌گوید کدام طرف مقصر است (`4xx` تو، `5xx` ما)، و `code` دقیقاً می‌گوید چه شد. کلاینت‌ها اول رویِ کدِ وضعیت تصمیم می‌گیرند؛ `code` آن را دقیق‌تر می‌کند.

### یک enum، یک `IntoResponse`

۳.۲.۳ `impl IntoResponse for AnimeError` را برایِ دو گونه یادت داد. اینجا همان حرکت به همه‌یِ شکست‌هایِ یک API گسترش می‌یابد، پس نکته‌ی جالب این است که چطور سازمان می‌گیرد. یک enum *همه‌ی* راه‌هایِ شکستنِ یک درخواست را فهرست می‌کند (با `#[derive(Debug, thiserror::Error)]` بالایش، از ۲.۵.۳):

```rust
pub enum ApiError {
    NotFound(String),
    MethodNotAllowed,
    BadRequest(String),
    UnsupportedMediaType(String),
    InvalidBody(String),
    Validation(Vec<FieldError>),
    Internal(String),
}
```

سپس سه `match`ِ کوچک، یکی برایِ هر پرسشی که یک پاسخ باید جواب بدهد، و یک `IntoResponse` که هر سه را می‌پرسد:

```senpai-visual
{"kind":"result","labels":["هر شکست","ApiError","status() و code() و message()","یک IntoResponse","همان پاکتِ JSON"]}
```

| گونه | `status()` | `code()` |
|---|---|---|
| `NotFound` | `404` | `not_found` |
| `MethodNotAllowed` | `405` | `method_not_allowed` |
| `BadRequest` | `400` | `bad_request` |
| `UnsupportedMediaType` | `415` | `unsupported_media_type` |
| `InvalidBody` | `422` | `invalid_body` |
| `Validation` | `422` | `validation_failed` |
| `Internal` | `500` | `internal_error` |

چرا سه متد و نه یک `match` که یک توپل برگرداند، مثلِ ۳.۲.۳؟ چون حالا هر جواب جداگانه به کار می‌آید: یک تست می‌تواند `status()` و `code()`ِ هر گونه را بدونِ ساختنِ پاسخ بسنجد، یک logger می‌تواند فقط `code()` را چاپ کند، و یک گونه‌ی تازه تو را وادار می‌کند هر سه پرسش را جواب بدهی. بخشِ آخر را کامپایلر اعمال می‌کند، و «خطاهایی که خواهی دید» پیامِ دقیقش را نشان می‌دهد.

### کلاینت چه می‌تواند بداند: `4xx` در برابرِ `5xx`

جدولِ ۳.۱.۳ گفت `4xx` یعنی «درخواست را درست کن» و `5xx` یعنی «تقصیرِ تو نیست». همین قاعده برایِ این هم هست که در `message` چه می‌آید:

- پیامِ یک `4xx` **به کلاینت می‌گوید چطور درستش کند**: `title must be 1 to 100 characters`، `show 9 not found`. کلاینت باعثش بوده و فقط با اطلاعات می‌تواند ترمیمش کند.
- پیامِ یک `5xx` **هیچ‌چیز به کلاینت نمی‌گوید**: همیشه همان جمله‌یِ ثابت، `something went wrong on our side`. *علت* (یک اتصالِ ردشده به دیتابیس، پیامِ پنیک، یک مسیرِ فایل) برایِ اپراتور است. به لاگِ سرور می‌رود، هرگز به بدنه. خواننده‌یِ بدنه یک غریبه در اینترنت است، و یک connection string برایش هدیه است.

پس `ApiError::Internal(String)` متن را نگه می‌دارد، و `message()`ِ آن متن را نادیده می‌گیرد. یک `IntoResponse`ِ واحد متن را در standard error چاپ می‌کند (جایگزینی برایِ لاگی که در ۳.۸.۲ درست خواهی کرد) و جمله‌یِ ثابت را می‌فرستد. کلاینتی که بخواهد مشکل را گزارش کند چیزی برایِ نقل‌کردن لازم دارد؛ [۳.۸.۲ — ردیابیِ درخواست و correlation ID](../02-request-tracing-and-correlation-ids/README.fa.md) جایی است که یک شناسه به‌ازایِ هر درخواست `500`ِ کلاینت را به خطِ لاگِ سرور وصل می‌کند.

### کشیدنِ شکست‌هایِ خودِ `axum` به پاکت

هندلرهایت فقط روی شکست‌هایی می‌توانند از پاکت استفاده کنند که به هندلر برسند. آن‌هایی که در جدولِ ابتدایی بودند هرگز نمی‌رسند: اکسترکتورها اول شکست می‌خورند و خودشان جواب می‌دهند، و یک مسیرِ ناشناخته هندلری ندارد. سه ابزار این شکاف‌ها را می‌بندند.

**یک `From` به‌ازایِ هر نوعِ rejection.** ۳.۲.۲ نشان داد که ردکننده پاسخی است که اکسترکتور هنگامِ امتناع می‌دهد تا هندلر اجرا نشود. اینجا یکی را به enumِ خودت تبدیل می‌کنی، با کدِ وضعیتش گونه را انتخاب می‌کنی و با `body_text()` پیام را برمی‌داری:

```rust
impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        let message = rejection.body_text();
        match rejection.status() {
            StatusCode::UNSUPPORTED_MEDIA_TYPE => ApiError::UnsupportedMediaType(message),
            StatusCode::UNPROCESSABLE_ENTITY => ApiError::InvalidBody(message),
            _ => ApiError::BadRequest(message),
        }
    }
}
```

**یک اکسترکتورِ پوششی با derive.** `axum` می‌تواند اکسترکتوری تولید کند که مثلِ `Json<T>` رفتار می‌کند ولی با نوعِ خطایِ *تو* شکست می‌خورد، پس هندلر `ApiJson<NewShow>` می‌نویسد و چیزِ دیگری عوض نمی‌شود:

```rust
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct ApiJson<T>(pub T);
```

`via(axum::Json)` می‌گوید «کار را با `Json` انجام بده»، و `rejection(ApiError)` می‌گوید «و اگر شکست خورد، با `From` تبدیل کن». (`#[derive(FromRequest)]` فیچرِ `macros`ِ `axum` را می‌خواهد که `Cargo.toml`ِ این درس روشنش کرده.) `ApiPath` همین است برایِ `Path<T>` با `FromRequestParts`. هندلری که هنوز `Path` یا `Json`ِ ساده می‌گیرد تنها سوراخِ باقی‌مانده است، و `examples/05-one-envelope.rs` آن را عمداً برایِ `Path` باز می‌گذارد: آن rejection دوباره متنِ ساده است.

**دو fallback برایِ روتر.** یک مسیرِ ناشناخته و یک مسیرِ شناخته با متدِ ثبت‌نشده در `axum` دو حالتِ متفاوت‌اند، و هرکدام قلابِ خودش را دارد:

```rust
Router::new()
    .route("/shows", post(create_show))
    .route("/shows/{id}", get(get_show))
    .fallback(route_not_found)
    .method_not_allowed_fallback(method_not_allowed)
    .with_state(store)
```

`.fallback(...)` به درخواست‌هایی جواب می‌دهد که با هیچ route نمی‌خوانند (همان `404`). `.method_not_allowed_fallback(...)` به درخواست‌هایی که مسیرشان می‌خواند ولی متدهایش نه (همان `405`). فراموش‌کردنِ دومی یک تله‌یِ کلاسیک است: `fallback` آن را پوشش نمی‌دهد و کلاینت یک `405`ِ خالی می‌گیرد، خطِ آخرِ transcriptِ اول. هر هندلر یک خط است: یک `ApiError` برمی‌گرداند و `IntoResponse` بقیه را انجام می‌دهد.

### اعتبارسنجی: یک پاکت، یک ورودی به‌ازایِ هر فیلدِ خراب

خطایِ اعتبارسنجی تنها `4xx`ای است که «به کلاینت بگو چطور درستش کند» بیش از یک جمله می‌خواهد، چون فرمی با سه فیلدِ بد سه چیز برایِ درست‌کردن دارد. پس `Validation` یک فهرست دارد و پاکت یک آرایه‌یِ اختیاریِ `fields` پیدا می‌کند، یک ورودی به‌ازایِ هر قاعده‌یِ شکسته:

```json
{"error": {"code": "validation_failed",
  "message": "the request body has invalid fields",
  "fields": [{"field": "title", "code": "length", "message": "title must be 1 to 100 characters"}]}}
```

هر ورودی همان تقسیمِ دو-کاره را یک سطح پایین‌تر دارد: `field` و `code` برایِ کدِ کلاینت‌اند («ورودیِ `title` را هایلایت کن»)، `message` برایِ انسان. `code`ِ بیرونی برایِ همه‌یِ آن‌ها `validation_failed` می‌ماند، پس کلاینتی که فقط می‌خواهد بداند *چیزی* غلط است روی یک رشته تصمیم می‌گیرد. گزارشِ *همه‌یِ* فیلدهایِ خراب در یک پاسخ، به‌جایِ اولین، سه رفت‌وبرگشت را برایِ کاربر صرفه‌جویی می‌کند. کدِ وضعیت `422` است، از قاعده‌یِ ۳.۲.۳: JSON درست‌شکل است و محتوا یک قاعده را می‌شکند. [۳.۳.۲ — اعتبارسنجی](../../03-serialization-and-validation/02-validation/README.fa.md) این خطاهایِ هر فیلد را با یک کتابخانه تولید می‌کند؛ در این درس `validate_new_show` دو قاعده را دستی می‌نویسد تا پاکت تنها چیزِ تازه باشد.

### یک استاندارد برایِ همین کار: RFC 9457

این ایده را تو اختراع نکردی. RFC 9457، «Problem Details for HTTP APIs» (جانشینِ RFC 7807)، یک بدنه‌یِ خطا را با نوعِ رسانه‌یِ `application/problem+json` استاندارد می‌کند. اعضایش: `type` (یک URI که نوعِ مشکل را شناسایی می‌کند و در نبودش پیش‌فرض `about:blank` است)، `title` (خلاصه‌یِ کوتاهِ نوعِ مشکل که نباید از رخداد به رخداد عوض شود)، `status` (کدِ وضعیتِ HTTP، مشورتی: کدِ واقعی همان پاسخ است)، `detail` (توضیحِ *همین* رخداد، که RFC می‌گوید باید به کلاینت در اصلاحِ مشکل کمک کند و کلاینت‌ها نباید پارسش کنند)، و `instance` (یک URI-reference که همین رخداد را شناسایی می‌کند). هر عضوِ دیگری به‌عنوانِ extension مجاز است، و کلاینت‌ها باید آن‌هایی را که نمی‌شناسند نادیده بگیرند. `examples/06-problem-json.rs` همان `404` را به هر دو شکل می‌سازد:

```text
envelope: 404 application/json
  {"error":{"code":"not_found","message":"show 9 not found"}}
problem : 404 application/problem+json
  {"detail":"show 9 not found","instance":"/shows/9","status":404,"title":"Not Found","type":"https://api.example.com/problems/not_found"}
```

(ترتیبِ کلیدها الفبایی است چون `json!`ِ `serde_json` شیء را در یک map مرتب نگه می‌دارد. اشیایِ JSON بی‌ترتیب‌اند.) نگاشتِ میانِ این دو نزدیک است: `code` نقشِ `type` را دارد، `message` نقشِ `detail`، و extensionِ شبیهِ `fields` در RFC هر اسمی باشد که بگذاری (مثالِ اعتبارسنجیِ خودش `errors` می‌گوید). چیزی که RFC می‌دهد شناخته‌شدن است: کلاینت‌ها، gatewayها و ابزارهایِ استاندارد از قبل `application/problem+json` را می‌فهمند، و `type` یک URI است که می‌شود مستندات را به آن وصل کرد. هزینه‌اش تشریفات است: یک URI به‌ازایِ هر نوعِ مشکل که باید طراحی و پایدار نگه داشته شود. برایِ API‌ای که مصرف‌کننده‌هایش frontendهایِ خودتان‌اند، پاکتِ کوچک کافی است. برایِ API‌یِ عمومی که بیرونی‌ها وصل می‌شوند، استاندارد تشریفاتش را می‌ارزد. عوض‌کردنِ بعدی آسان است *چون* یک `IntoResponse` هست: یک تابع را عوض می‌کنی، نه سی هندلر را. «چالش» دقیقاً همین را از تو می‌خواهد.

---

## دست‌به‌کد

مثال‌هایی را که کامپایل می‌شوند اجرا کن. (`02` و `03` عمداً خراب‌اند و پشتِ فیچرِ `broken` هستند؛ «خطاهایی که خواهی دید» نشانشان می‌دهد. `04` عادی اجرا می‌شود و فقط غلط است، و «خطاهایی که خواهی دید» خروجی‌اش را هم نشان می‌دهد.) خروجیِ `01-four-shapes` و `06-problem-json` در «مفهوم» است؛ خودت اجرا کن و مقایسه کن. حالا اصلاح در مینیاتور:

```sh
cargo run -p p3-08-01-consistent-error-envelopes --example 05-one-envelope
```

```text
GET /shows/9 -> 404 {"error":{"code":"not_found","message":"show 9 not found"}}
GET /shows/abc -> 400 Invalid URL: Cannot parse `abc` to a `u64`
POST /shows -> 400 {"error":{"code":"rejected","message":"Failed to parse the request body as JSON: EOF while parsing an object at line 1 column 1"}}
POST /shows -> 422 {"error":{"code":"rejected","message":"Failed to deserialize the JSON body into the target type: missing field `title` at line 1 column 2"}}
POST /shows -> 415 {"error":{"code":"rejected","message":"Expected request with `Content-Type: application/json`"}}
GET /nope -> 404 {"error":{"code":"not_found","message":"no route for /nope"}}
DELETE /shows/1 -> 405 
GET /boom -> 500 {"error":{"code":"internal_error","message":"something went wrong on our side"}}
```

```text
internal error: connect to postgres://anime:hunter2@db.internal refused
```

(بلوکِ آخر standard error است که ترمینال میانِ خطوطِ بالا نشان می‌دهد؛ بدنه‌یِ `/boom` که به کلاینت می‌رسد هیچ ردی از آن ندارد.) دو خط عمداً هنوز پاکت نیستند: `{id}`ِ بد (یک rejectionِ `Path` که کسی تبدیلش نکرده) و `DELETE` (بدونِ `method_not_allowed_fallback`). راه‌حل هر دو را می‌بندد.

تست‌ها قرمز شروع می‌شوند. `src/lib.rs` کلِ اسکلت را دارد، و هر تابعی که پیاده می‌کنی یک `todo!()` با یک doc comment است که مشخصاتِ کاملش است:

```sh
cargo test -p p3-08-01-consistent-error-envelopes --no-fail-fast 2>&1 | grep 'test result'
```

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: FAILED. 0 passed; 12 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: FAILED. 0 passed; 14 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

وقتی همه سبز شد، سرورِ واقعی را اجرا کن (روی `127.0.0.1:3220` گوش می‌دهد). این transcriptها روی solution گرفته شده‌اند. اول مسیرِ موفق و دو نوع شکستِ دامنه:

```sh
cargo run -p p3-08-01-consistent-error-envelopes &
curl -s -w '\n%{http_code}\n' -X POST -H 'content-type: application/json' -d '{"title":"Frieren","episodes":28}' http://127.0.0.1:3220/shows
curl -s -w '\n%{http_code}\n' http://127.0.0.1:3220/shows/999
curl -s -w '\n%{http_code}\n' -X POST -H 'content-type: application/json' -d '{"title":"","episodes":0}' http://127.0.0.1:3220/shows
```

```text
{"id":1,"title":"Frieren","episodes":28}
201
{"error":{"code":"not_found","message":"show 999 not found"}}
404
{"error":{"code":"validation_failed","message":"the request body has invalid fields","fields":[{"field":"title","code":"length","message":"title must be 1 to 100 characters"},{"field":"episodes","code":"range","message":"episodes must be 1 to 2000"}]}}
422
```

حالا شکست‌هایی که `axum` در همان transcriptِ اول متنِ ساده یا بدنه‌یِ خالی داد:

```sh
curl -s -w '\n%{http_code}\n' -X POST -H 'content-type: application/json' -d '{' http://127.0.0.1:3220/shows
curl -s -w '\n%{http_code}\n' -X POST -H 'content-type: application/json' -d '{"episodes":3}' http://127.0.0.1:3220/shows
curl -s -w '\n%{http_code}\n' -X POST -d '{"title":"x","episodes":1}' http://127.0.0.1:3220/shows
curl -s -w '\n%{http_code}\n' http://127.0.0.1:3220/shows/abc
curl -s -w '\n%{http_code}\n' http://127.0.0.1:3220/nope
curl -s -w '\n%{http_code}\n' -X DELETE http://127.0.0.1:3220/shows/1
```

```text
{"error":{"code":"bad_request","message":"Failed to parse the request body as JSON: EOF while parsing an object at line 1 column 1"}}
400
{"error":{"code":"invalid_body","message":"Failed to deserialize the JSON body into the target type: missing field `title` at line 1 column 14"}}
422
{"error":{"code":"unsupported_media_type","message":"Expected request with `Content-Type: application/json`"}}
415
{"error":{"code":"bad_request","message":"Invalid URL: Cannot parse `abc` to a `u64`"}}
400
{"error":{"code":"not_found","message":"no route for /nope"}}
404
{"error":{"code":"method_not_allowed","message":"method not allowed for this route"}}
405
```

هفت مسیرِ شکستِ مختلف، یک شکل. در آخر `500` و ترمینالِ خودِ سرور:

```sh
curl -s -w '\n%{http_code}\n' http://127.0.0.1:3220/simulate-failure
```

```text
{"error":{"code":"internal_error","message":"something went wrong on our side"}}
500
```

```text
internal error: connect to postgres://anime:hunter2@db.internal:5432 refused
```

بلوکِ دوم همان است که standard errorِ *سرور* برایِ آن درخواست چاپ کرد. کلاینت فقط اولی را دید. وقتی تمام شد سرور را متوقف کن (`kill %1` در همان shell). بعد این‌ها را امتحان کن:

۱. به `curl`ِ `/shows/999` گزینه‌یِ `-i` را اضافه کن. پاسخ چه `content-type`ای دارد، و چه کسی آن را گذاشته؟
۲. `{"title":"x","episodes":"many"}` را بفرست. چه `code`ای می‌گیری، و چرا `validation_failed` نیست؟
۳. در `examples/05-one-envelope.rs` یک `method_not_allowed_fallback` اضافه کن تا خطِ `DELETE` پاکت شود.

---

## خطاهایی که خواهی دید

### `E0004` — گونه‌یِ تازه‌ای که `status()` پوشش نمی‌دهد

برایِ عنوانِ تکراری `Conflict` را به enum اضافه می‌کنی و یکی از `match`ها را فراموش می‌کنی:

```rust
fn status(&self) -> u16 {
    match self {
        ApiError::NotFound(_) => 404,
        ApiError::Internal(_) => 500,
    }
}
```

`examples/02-missing-arm-broken.rs` با `--features broken`:

```sh
cargo run -p p3-08-01-consistent-error-envelopes --example 02-missing-arm-broken --features broken
```

```text
error[E0004]: non-exhaustive patterns: `&ApiError::Conflict(_)` not covered
  --> phase3-backend-foundations\08-error-handling-and-testing-at-scale\01-consistent-error-envelopes\examples\02-missing-arm-broken.rs:14:15
   |
14 |         match self {
   |               ^^^^ pattern `&ApiError::Conflict(_)` not covered
   |
note: `ApiError` defined here
  --> phase3-backend-foundations\08-error-handling-and-testing-at-scale\01-consistent-error-envelopes\examples\02-missing-arm-broken.rs:6:6
   |
 6 | enum ApiError {
   |      ^^^^^^^^
 7 |     NotFound(String),
 8 |     Conflict(String),
   |     -------- not covered
   = note: the matched value is of type `&ApiError`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
16 ~             ApiError::Internal(_) => 500,
17 ~             &ApiError::Conflict(_) => todo!(),
   |

For more information about this error, try `rustc --explain E0004`.
error: could not compile `p3-08-01-consistent-error-envelopes` (example "02-missing-arm-broken") due to 1 previous error
```

(پیش از خطا، فرمان سه هشدارِ `unused variable` هم از اسکلتِ خودِ این درس چاپ می‌کند که `todo!()` دارد. بخشی از خطا نیستند و بعد از پیاده‌کردنِ توابع از بین می‌روند.)

**کامپایلر به چه اعتراض دارد:** یک `match` باید هر گونه را پوشش دهد، و `Conflict` بازو ندارد. note به گونه‌ای اشاره می‌کند که پوشش داده نشده.

**رفع:** بازو را اضافه کن، `ApiError::Conflict(_) => 409`. نه یک wildcardِ `_ =>`، و نه `todo!()`ای که خطِ help پیشنهاد می‌کند.

**چرا این رفع است:** این خطا *دلیلِ* وجودِ سه `match`ِ کوچک است. هر بار حالتِ شکستِ تازه‌ای اضافه می‌کنی، کامپایلر تو را به هر جایی می‌برد که باید بگوید روی سیم چه معنایی دارد (status، code، message)، و بازویی که نوشته نشده نمی‌تواند منتشر شود. یک wildcardِ `_ => 500` آن را ساکت می‌کرد و به `409`ِ تازه‌ات بدونِ هیچ هشداری `500` جواب می‌داد.

### `E0277` — `?` رویِ یک rejection بدونِ `From`

یک هندلر خودش `Result`ِ اکسترکتور را می‌گیرد و `?` می‌زند، تا rejection به‌صورتِ `ApiError` برگردانده شود:

```rust
async fn create(body: Result<Json<NewShow>, JsonRejection>) -> Result<String, ApiError> {
    let Json(show) = body?;
    Ok(show.title)
}
```

`examples/03-question-mark-without-from-broken.rs`:

```text
error[E0277]: `?` couldn't convert the error to `ApiError`
  --> phase3-backend-foundations\08-error-handling-and-testing-at-scale\01-consistent-error-envelopes\examples\03-question-mark-without-from-broken.rs:26:26
   |
26 |     let Json(show) = body?;
   |                      ----^ the trait `From<JsonRejection>` is not implemented for `ApiError`
   |                      |
   |                      this can't be annotated with `?` because it has type `Result<_, JsonRejection>`
   |
note: `ApiError` needs to implement `From<JsonRejection>`
  --> phase3-backend-foundations\08-error-handling-and-testing-at-scale\01-consistent-error-envelopes\examples\03-question-mark-without-from-broken.rs:17:1
   |
17 | struct ApiError;
   | ^^^^^^^^^^^^^^^
   = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the `From` trait

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-08-01-consistent-error-envelopes` (example "03-question-mark-without-from-broken") due to 1 previous error
```

(همان سه هشدارِ اسکلت اول چاپ می‌شوند، حذف شده‌اند.)

**کامپایلر به چه اعتراض دارد:** `?` فقط خطا را برنمی‌گرداند، رویش `From::from` صدا می‌زند تا به نوعِ خطایِ تابع برسد (قاعده‌یِ ۱.۶.۵). کامپایلر می‌گوید کدام impl کم است و حتی نوعی را که به آن نیاز دارد علامت می‌زند.

**رفع:** `From<JsonRejection> for ApiError` را پیاده کن، مثلِ «مفهوم». با deriveِ `ApiJson`، همین impl چیزی است که `rejection(ApiError)` صدا می‌زند، و هندلر دیگر اصلاً `JsonRejection` را نمی‌نویسد.

**چرا این رفع است:** `From` راهی است که خطاها از لایه‌یِ پایین‌تر وارد enumِ واحدِ تو می‌شوند. هر منبعِ شکستِ تازه دقیقاً یک `impl From` هزینه دارد، یک‌بار نوشته می‌شود، و `?` بقیه را همه‌جا انجام می‌دهد.

### هیچ خطایی نیست: `500`ای که فاش می‌کند

```text
500 Internal Server Error
{"error":{"code":"internal_error","message":"connect to postgres://anime:hunter2@db.internal:5432 refused"}}
```

**چه چیزی واقعاً خراب است:** `examples/04-leaky-500-trap.rs` کامپایل می‌شود، اجرا می‌شود، و یک پاکتِ کاملاً یکدست دارد. `IntoResponse`ِ آن متنِ گونه‌یِ `Internal` را در `message` کپی می‌کند. شکل درست است و محتوا فاجعه: کلاینت حالا اسمِ هاستِ دیتابیس، یک کاربر و یک رمز را می‌داند. پاکتِ یکدست انجامِ تصادفیِ این کار را *آسان‌تر* می‌کند، چون `message` برایِ هر گونه همان‌جاست.

**رفع:** به `Internal` بازوی `message()`ِ خودش را بده که متن را نادیده می‌گیرد و یک جمله‌یِ ثابت برمی‌گرداند، و متن را به‌جایش در لاگِ سرور چاپ کن.

**چرا این رفع است:** نوع از قبل دو مخاطب را جدا کرده است. `Internal(String)` متنِ سرور را نگه می‌دارد، و تنها تابعی که تصمیم می‌گیرد کلاینت چه ببیند `message()` است. با این تست کن که یک پاسخ هرگز رازی را که در متن گذاشته‌ای نداشته باشد. `into_response_hides_the_internal_detail` در `tests/error_test.rs` دقیقاً همین را می‌کند، و هیچ کامپایلری این دسته از اشتباه را نمی‌گیرد.

---

## تمرین

### گرم‌کردن

<details>
<summary>یک اپ هر <code>message</code>ای را که یک درخواستِ ناموفق برگردانده به کاربر نشان می‌دهد و رویِ <code>code</code> تصمیم می‌گیرد. تیم پیامی را از "show 9 not found" به "We couldn't find show 9" بازنویسی می‌کند. چه چیزی خراب می‌شود؟</summary>

پیش از دیدنِ پاسخ خودت فکر کن.

</details>

<details>
<summary>پاسخ</summary>

هیچ‌چیز، و همین نکته است. اپ رویِ `code` (`not_found`) تصمیم می‌گیرد که عوض نشده، و `message` را فقط نمایش می‌دهد. اگر رویِ جمله match کرده بود، هر بازنویسی یک شکستنِ بی‌صدایِ سازگاری می‌شد.

</details>

<details>
<summary>چرا پاکتِ یک <code>5xx</code> می‌تواند هر بار همان <code>message</code>ِ ثابت را داشته باشد، ولی یک <code>4xx</code> نه؟</summary>

پیش از دیدنِ پاسخ خودت فکر کن.

</details>

<details>
<summary>پاسخ</summary>

ترمیمِ یک `4xx` با کلاینت است، پس بدنه باید چیزی را که باید درست شود حمل کند (`title must be 1 to 100 characters`). ترمیمِ یک `5xx` با ماست؛ کلاینت فقط می‌تواند دوباره امتحان کند یا گزارش بدهد، و هر چیزِ دقیق‌تر فاش‌کردنِ جزئیاتِ درونی است. ردهایِ ۳.۱.۳ این قاعده را می‌رانند.

</details>

<details>
<summary>یک route برایِ <code>GET /shows/{id}</code> ثبت شده و روتر <code>.fallback(f)</code> دارد. کلاینت <code>DELETE /shows/1</code> می‌فرستد. آیا <code>f</code> اجرا می‌شود؟</summary>

پیش از دیدنِ پاسخ خودت فکر کن.

</details>

<details>
<summary>پاسخ</summary>

نه. مسیر خواند، فقط متد نخواند، و آن حالت قلابِ خودش را دارد، `method_not_allowed_fallback`. بدونِ آن کلاینت یک `405`ِ خالی می‌گیرد، همان‌طور که خطِ `DELETE`ِ `examples/05-one-envelope.rs` نشان می‌دهد.

</details>

<details>
<summary>پاکت <code>code</code> دارد و هر ورودیِ اعتبارسنجی هم <code>code</code>ِ خودش را. چرا این تکرار نیست؟</summary>

پیش از دیدنِ پاسخ خودت فکر کن.

</details>

<details>
<summary>پاسخ</summary>

به پرسش‌هایِ متفاوت جواب می‌دهند. `code`ِ بیرونی (`validation_failed`) می‌گوید *درخواست* چه نوع شکستی بود. `code`ِ درونی (`length`، `range`) می‌گوید یک فیلد کدام *قاعده* را شکست. کلاینتی که فقط می‌خواهد بداند «اعتبارسنجی شکست خورد یا نه» یک رشته می‌خواند، و فرمی که باید ورودی‌ها را هایلایت کند فهرست را.

</details>

<details>
<summary>RFC 9457 می‌گوید <code>status</code> مشورتی است، و پاسخ باید همان کد را به‌هرحال بگذارد. پس چرا اصلاً در بدنه تکرارش کنیم؟</summary>

پیش از دیدنِ پاسخ خودت فکر کن.

</details>

<details>
<summary>پاسخ</summary>

تا اطلاعات وقتی بدنه از پاسخِ HTTPاش جدا می‌شود بماند: در یک لاگ ذخیره شود، از یک صف رد شود، یا یک واسط کدِ وضعیت را عوض کرده باشد. RFC دقیقاً همین حالت‌ها را نام می‌برد. نرم‌افزاری که پیامِ HTTP را می‌خواند از خطِ وضعیتِ واقعی استفاده می‌کند.

</details>

### تعمیر

سه مثالی را که اشتباهاتِ «خطاهایی که خواهی دید» را دارند درست کن:

۱. `examples/02-missing-arm-broken.rs` با `--features broken` کامپایل شود: بازوی گم‌شده را اضافه کن تا `Conflict` به `409` جواب بدهد. wildcard نگذار.
۲. `examples/03-question-mark-without-from-broken.rs` با `--features broken` کامپایل شود: impl گم‌شده را اضافه کن. `ApiError` باید برایِ هر rejection به `400` جواب بدهد.
۳. `examples/04-leaky-500-trap.rs` اجرا می‌شود ولی نباید `hunter2` چاپ کند. بدنه باید همان شکل را نگه دارد، با پیامِ ثابتِ `something went wrong on our side`.

### پیاده‌سازی

هر چیزِ `src/lib.rs` که `todo!()` است: `status`، `code` و `message`ِ `ApiError`، `IntoResponse`ِ آن، دو `From` برایِ rejectionهایِ `axum`، و `validate_new_show`. هر doc comment مشخصاتِ کاملش است (کدهایِ وضعیت، کدها، پیام‌هایِ دقیق، ترتیبِ ورودی‌هایِ اعتبارسنجی)، پس هرگز لازم نیست برایِ فهمیدنِ آنچه باید بسازی تست‌ها را باز کنی. ذخیره‌گاه، هندلرها، اکسترکتورهایِ پوششی و روتر داده شده‌اند: کارِ این درس لایه‌یِ خطاست.

```sh
cargo test -p p3-08-01-consistent-error-envelopes
```

`tests/error_test.rs` (۱۴ تست) لایه‌یِ خطا را با فراخوانیِ ساده‌یِ تابع می‌سنجد، بدونِ روتر و بدونِ درخواست. `tests/api_test.rs` (۱۲ تست) درخواست‌هایِ واقعی را از `oneshot` رد می‌کند و می‌سنجد هر شکست، از جمله آن‌هایی که خودِ `axum` می‌سازد، پاکت دارد.

### بساز

یک شکستِ خودت را اضافه کن: ساختنِ یک show که عنوانش از قبل هست (دقیقاً همان‌طور که فرستاده شده مقایسه می‌شود) یک `409 Conflict` با `code`ِ `conflict` است. `ApiError::Conflict(String)` را اضافه کن؛ کامپایلر تو را از میانِ هر `match`ای که بازو لازم دارد می‌گذراند. کاری کن که `POST /shows` پیش از درج ذخیره‌گاه را بررسی کند، و با پیامِ `a show titled "<title>" already exists` جواب بدهد. تست‌هایِ خودت را در یک `tests/conflict_test.rs`ِ تازه بنویس، یکی برایِ کدِ وضعیت و پاکت و یکی که همان `POST` را دو بار می‌فرستد و می‌سنجد دومی `409` جواب می‌دهد در حالی که اولی `201` می‌ماند.

### چالش (اختیاری)

همان `ApiError` را به‌صورتِ یک problem از RFC 9457 تولید کن. `ApiError::problem_response(&self, instance: &str) -> Response` را اضافه کن که با همان کدِ وضعیت و با `Content-Type: application/problem+json` جواب بدهد، و بدنه‌ای با `type` (`https://api.example.com/problems/<code>`)، `title` (عبارتِ استانداردِ دلیلِ کدِ وضعیت)، `status`، `detail` (همان متنِ `message()`)، `instance`، و برایِ `Validation` یک extensionِ `errors` از ورودی‌هایِ فیلدها. RFCِ «بیشتر» مشخصاتِ کامل است؛ چیزِ دیگری در دوره به این وابسته نیست. تست‌هایِ خودت را بنویس.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به کار می‌آید |
|---|---|---|
| پاکتِ خطا | یک شکلِ JSONِ ثابت (`{"error": {...}}`) برایِ هر شکستی که یک API برمی‌گرداند | هر API با بیش از یک endpoint |
| `code` در برابرِ `message` | `code` قراردادِ پایدارِ ماشین‌خوان است؛ `message` ادبِ انسانی که می‌تواند عوض شود | کلاینت‌ها، تست‌ها، تحلیلِ لاگ |
| پیامِ امن برایِ کلاینت | متنی که کلاینت می‌تواند ببیند: چطور درست کردنِ یک `4xx`، یک جمله‌یِ ثابت برایِ `5xx` | هر گونه‌یِ خطا |
| تبدیلِ rejection | `From<SomeRejection> for ApiError` به‌علاوه‌یِ یک اکسترکتورِ پوششیِ derive‌شده، تا شکست‌هایِ خودِ `axum` از پاکت استفاده کنند | `Json`، `Path`، `Query`، هر چیزی با rejection |
| `fallback` / `method_not_allowed_fallback` | دو قلابِ روتر برایِ «مسیر نیست» و «مسیرِ شناخته، متدِ غلط» | هر API عمومی |
| Problem details (RFC 9457) | بدنه‌یِ استانداردِ خطا، `application/problem+json`، با `type`، `title`، `status`، `detail`، `instance` | APIهایِ عمومی، gatewayها |

### الان می‌دانی

- یک API یکدست یک شکلِ خطا دارد، و آن شکل ارزان به دست می‌آید: یک enum، یک `IntoResponse`، یک جا که می‌داند یک شکست رویِ سیم چه شکلی است.
- `code` قرارداد است و `message` ادب، پس کلاینت‌ها رویِ اولی تصمیم می‌گیرند و دومی را فقط نشان می‌دهند.
- یک `5xx` یک جمله‌یِ ثابت می‌گوید و علتِ واقعی را لاگ می‌کند؛ یک `4xx` می‌گوید چطور درستش کنی. نوع این دو را جدا می‌کند تا هیچ هندلری نتواند قاطی‌شان کند.
- شکست‌هایی که `axum` پیش از هندلرت می‌سازد (JSONِ خراب، `{id}`ِ بد، نبودنِ مسیر، متدِ غلط) سه قلابِ جدا می‌خواهند: یک `From` به‌ازایِ هر rejection با یک اکسترکتورِ پوششیِ derive‌شده، `fallback`، و `method_not_allowed_fallback`.
- خطاهایِ اعتبارسنجی در همان پاکت جا می‌شوند، به‌صورتِ فهرستی با یک ورودی به‌ازایِ هر فیلدِ خراب، همه‌یِ آن‌ها یک‌جا گزارش می‌شوند.
- RFC 9457 همین کار را استاندارد می‌کند. پاکتِ این درس نزدیک به آن نگاشت می‌شود، و یک `IntoResponse` جابه‌جایی را ارزان می‌کند.

### بعداً کامل‌تر می‌بینی

- **دنبال‌کردنِ `500`ِ یک کلاینت تا خطِ لاگِ سرور**: [۳.۸.۲ — ردیابیِ درخواست و correlation ID](../02-request-tracing-and-correlation-ids/README.fa.md)
- **خطاهایِ اعتبارسنجیِ هر فیلد با یک کتابخانه‌یِ واقعیِ اعتبارسنجی**: [۳.۳.۲ — اعتبارسنجی](../../03-serialization-and-validation/02-validation/README.fa.md)
- **توصیفِ نوع‌دارِ این بدنه‌هایِ خطا در یک قراردادِ API**: [۳.۳.۳ — قراردادهایِ API و OpenAPI (`utoipa`)](../../03-serialization-and-validation/03-api-contracts-and-openapi/README.fa.md)
- **جایی که ذخیره‌گاهِ در حافظه جایش را به یک دیتابیس می‌دهد**: [۳.۵.۳ — کاتالوگ انیمه، این‌بار متصل به Postgres](../../05-postgres-and-sqlx/03-anime-catalog-postgres-backed/README.fa.md)
- **احرازِ هویت به‌صورتِ میان‌افزارِ `tower`**: [۳.۷.۳ — JWT و میان‌افزار در `tower`](../../07-auth-and-security/03-jwt-and-tower-middleware/README.fa.md)
- **تستِ یکپارچگی رویِ یک دیتابیسِ واقعیِ یک‌بارمصرف**: [۳.۸.۳ — تستِ یکپارچگی با `testcontainers`](../03-integration-tests-with-testcontainers/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا پاکت هم `code` دارد هم `message`، و کلاینت اجازه دارد رویِ کدام match کند؟
- چرا یک `5xx` هر بار همان جمله‌یِ ثابت را جواب می‌دهد، و علتِ واقعی کجا می‌رود؟
- چرا یک `400`ِ متنِ ساده‌یِ `axum` هنوز برایِ API تو مشکل است، با اینکه کدِ وضعیتش درست است؟
- چه چیزی به روتر اضافه می‌کنی تا یک `DELETE` رویِ مسیرِ فقط-`GET` پاکت بگیرد، و چرا `.fallback` آن را پوشش نمی‌دهد؟
- چرا خوب است که `status()`، `code()` و `message()` سه `match` هستند، و `E0004` وقتی گونه‌ای اضافه می‌کنی چه کاری برایت می‌کند؟
- با رفتن به `application/problem+json` چه چیزی به دست می‌آوری و چه می‌پردازی؟

---

## بیشتر

- [RFC 9457 — Problem Details for HTTP APIs](https://www.rfc-editor.org/rfc/rfc9457.html): کوتاه و خواندنی؛ §3.1 پنج عضو را تعریف می‌کند، §3.2 extensionها، §4.2.1 `about:blank`. مثالِ اعتبارسنجی در §3 است.
- [`axum::extract::rejection`](https://docs.rs/axum/0.8.9/axum/extract/rejection/index.html): همه‌یِ نوع‌هایِ rejection، با کدِ وضعیتی که هرکدام می‌دهد.
- [`axum::Router::method_not_allowed_fallback`](https://docs.rs/axum/0.8.9/axum/struct.Router.html#method.method_not_allowed_fallback): قلاب برایِ مسیری که می‌خواند ولی متدِ ثبت‌شده ندارد.
- [`#[derive(FromRequest)]`](https://docs.rs/axum/0.8.9/axum/extract/derive.FromRequest.html): اتریبیوت‌هایِ `via` و `rejection` که این درس استفاده کرد.
- [Django REST framework — custom exception handling](https://www.django-rest-framework.org/api-guide/exceptions/#custom-exception-handling): همتایِ یک‌تابعیِ آن چیزی که از قبل می‌شناسی.
