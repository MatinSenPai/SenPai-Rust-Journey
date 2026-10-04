# ۳.۲.۲ — نوشتنِ اکسترکتورِ خودت (`FromRequestParts`)

## در یک نگاه

بعد از این درس می‌توانی:

- توضیح بدهی چرا یک هندلر می‌تواند چند اکسترکتور داشته باشد ولی فقط یکی‌شان بدنه را بخواند، و بگویی هر نوع کدام صفت را پیاده می‌کند.
- با `FromRequestParts` یک اکسترکتور را با دست بنویسی: کلیدِ API از یک هدر، صفحه‌بندی از رشته‌ی کوئری، نسخه‌ی کلاینت از یک هدر.
- نوعِ ردِ درخواست (rejection) را طوری انتخاب کنی که `axum` از قبل بلد باشد بفرستدش، و چهار خطایِ کامپایلری را که با انتخابِ غلط می‌بینی بخوانی.
- در `axum` نسخه‌ی ۰٫۸ یک اکسترکتور را اختیاری کنی، جایی که `Option<T>` دیگر برایِ همه‌ی اکسترکتورها کار نمی‌کند.

**زمان:** حدود ۷۵ دقیقه · **پیش‌نیاز:**
[۳.۲.۱ — مسیریابی، هندلرها، اکسترکتورها](../01-routing-handlers-extractors/README.fa.md)،
[۲.۳.۱ — تعریف و پیاده‌سازیِ صفت‌ها](../../../phase2-intermediate/03-traits-and-generics/01-defining-and-implementing-traits/README.fa.md)،
[۲.۳.۵ — نوع‌هایِ وابسته در برابرِ پارامترهایِ جنریک](../../../phase2-intermediate/03-traits-and-generics/05-associated-types/README.fa.md)،
[۲.۹.۴ — صفت‌هایِ async و `spawn_blocking`](../../../phase2-intermediate/09-async-in-practice/04-async-traits-and-blocking/README.fa.md)

---

## چرا اهمیت دارد

[۳.۲.۱](../01-routing-handlers-extractors/README.fa.md) به تو `Path`، `Query`، `Json` و `State` را داد. این‌ها چیزهایی را پوشش می‌دهند که `axum` درباره‌ی هر APIای می‌تواند بداند. چیزی را که *APIِ تو* لازم دارد پوشش نمی‌دهند: یک هدرِ `x-api-key` رویِ همه‌ی مسیرها، یک جفتِ `?page=` و `?per_page=` با مقدارِ پیش‌فرض و سقف، نسخه‌ی اپِ کلاینتی که درخواست را فرستاده.

بدونِ اکسترکتورِ خودت، این منطق بالایِ هر هندلر کپی می‌شود. در جنگو هر دو راه‌حل را دیده‌ای. راهِ کپی‌پیست یک تابعِ کمکی مثلِ `get_pagination(request)` است که هر ویو اول صدایش می‌زند. راهِ مرتب یک کلاسِ احرازِ هویتِ DRF است: یک‌بار فهرستش می‌کنی، و تا ویو اجرا شود `request.user` از قبل پر شده، یا درخواست از قبل با `401` برگردانده شده. اکسترکتورِ `axum` از نوعِ دوم است، با یک فرق. هندلر صدایش نمی‌زند. اسمش را در امضایِ خودش می‌آورد، و `axum` پیش از اجرایِ هندلر صدایش می‌زند.

این درس به یک وعده‌ی ۳.۲.۱ هم عمل می‌کند. آن درس گفت `Json` باید آخرین آرگومان باشد. این‌جا می‌فهمی چرا، و دلیلش فقط انتخابِ یک صفت است.

---

## مفهوم

### یک درخواست دو تکه است

```rust
let (parts, body) = request.into_parts();
println!("method:        {}", parts.method);
println!("api key:       {:?}", parts.headers.get("x-api-key"));
println!("api key again: {:?}", parts.headers.get("x-api-key"));
let bytes = to_bytes(body, 1024).await.unwrap();
```

`examples/01-parts-and-body.rs` یک `POST` را به دو تکه‌اش می‌شکند و هر کدام را می‌خواند:

```text
method:        POST
path:          /anime
query:         Some("page=2")
api key:       Some("secret-123")
api key again: Some("secret-123")
body:          {"title":"Frieren"}
```

`Parts` یعنی متد، URI، هدرها و extensionها: داده‌ی ساده. هر چند بار و از هر چند جا که بخواهی می‌توانی نگاهش کنی. بدنه فرق دارد. یک جریانِ بایت است که شاید هنوز در حالِ رسیدن باشد، و `to_bytes(body, ...)` آن را *با مقدار* می‌گیرد. بعد از آن فراخوانی جریان تمام شده است. دو بار نمی‌شود خواندش، چون بایت‌ها هیچ‌جا نگه داشته نشدند.

```senpai-visual
{"kind":"concept","labels":["درخواست می‌رسد","Parts: متد، uri، هدرها، extensions","Body: جریانی که یک‌بار خوانده می‌شود","اکسترکتورهایِ Parts: هر تعداد، هر ترتیب","فقط آخرین اکسترکتور می‌تواند Body را بگیرد"]}
```

### دو صفت، به‌خاطرِ همین شکاف

`axum` برایِ هر تکه یک صفتِ اکسترکتور دارد (از `axum-core` نسخه‌ی ۰٫۵٫۶، همان نسخه‌ای که این دوره حل می‌کند، بدونِ کامنت‌هایِ مستندات):

```rust
pub trait FromRequestParts<S>: Sized {
    type Rejection: IntoResponse;
    fn from_request_parts(parts: &mut Parts, state: &S)
        -> impl Future<Output = Result<Self, Self::Rejection>> + Send;
}
pub trait FromRequest<S, M = private::ViaRequest>: Sized {
    type Rejection: IntoResponse;
    fn from_request(req: Request, state: &S)
        -> impl Future<Output = Result<Self, Self::Rejection>> + Send;
}
```

این تعریف از سورسِ `axum-core` است، پس چیزی برایِ اجرا نیست. دو آرگومان را مقایسه کن. `FromRequestParts` یک `&mut Parts` می‌گیرد: می‌تواند هدرها و URI را ببیند، ولی به بدنه نمی‌رسد، چون بدنه در `Parts` نیست. `FromRequest` کلِ `Request` را، با بدنه، *با مقدار* می‌گیرد. هر که آن را بگیرد صاحبِ تنها نسخه‌ی جریان است.

پس `Path`، `Query`، `State`، `HeaderMap` و `Method` صفتِ `FromRequestParts` را پیاده می‌کنند. `Json`، `String` و `Bytes` صفتِ `FromRequest` را. وقتی `axum` هندلرت را صدا می‌زند، همه‌ی آرگومان‌ها جز آخری را از `from_request_parts` رد می‌کند، چپ به راست، رویِ همان `Parts`. بعد `Parts` و بدنه را دوباره به یک `Request` وصل می‌کند و به `from_request`ِ آخرین آرگومان می‌دهد. درخواستِ دومی برایِ دادن وجود ندارد. برایِ همین فقط آخرین آرگومان می‌تواند بدنه را بخواند، و این دلیلِ واقعیِ پشتِ قاعده‌ی «`Json` باید آخر باشد» در ۳.۲.۱ است. کامپایلر این قاعده را با این اجرا می‌کند که از هر آرگومان جز آخری `FromRequestParts` می‌خواهد و از آخری `FromRequest`.

دو نکته‌ی ریز این را جور می‌کند. اول، هر چیزی که `FromRequestParts` را پیاده کرده در جایگاهِ آخر هم قبول می‌شود، پس اکسترکتورِ تو می‌تواند هر جا بنشیند. دوم، از ۰٫۸ هر دو صفت `async fn` ساده در صفت را به کار می‌برند، همان قابلیتی که [۲.۹.۴](../../../phase2-intermediate/09-async-in-practice/04-async-traits-and-blocking/README.fa.md) نشانت داد. ماکروی `#[async_trait]` که آموزش‌هایِ قدیمی رویِ هر `impl` می‌گذاشتند رفته است.

### اولین اکسترکتورت

```rust
struct UserAgent(String);

impl<S: Send + Sync> FromRequestParts<S> for UserAgent {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let rejection = (StatusCode::BAD_REQUEST, "missing or unreadable user-agent header");
        let value = parts.headers.get("user-agent").ok_or(rejection)?;
        let text = value.to_str().map_err(|_| rejection)?;
        Ok(UserAgent(text.to_string()))
    }
}
```

تابع داخلِ `parts` را نگاه می‌کند و یا یک `UserAgent` می‌سازد یا یک ردِ درخواست برمی‌گرداند. `Rejection` یک نوعِ وابسته است، مثلِ [۲.۳.۵](../../../phase2-intermediate/03-traits-and-generics/05-associated-types/README.fa.md): هر `impl` یک‌بار پرش می‌کند. `rejection` دو بار استفاده شده چون `(StatusCode, &'static str)` از نوعِ `Copy` است. `<S: Send + Sync>` می‌گوید اکسترکتور با هر حالتِ برنامه کار می‌کند؛ این یکی هیچ‌وقت به state نگاه نمی‌کند، پس برایش فرقی نمی‌کند کدام باشد.

حالا یک هندلر آن را در امضایش می‌خواهد، و بدنه‌ی تابع هیچ‌وقت به هدر دست نمی‌زند:

```rust
async fn hello(UserAgent(agent): UserAgent) -> String {
    println!("  (the handler ran)");
    format!("hello, {agent}")
}
```

`examples/02-user-agent-extractor.rs` دو درخواست را با `oneshot` از یک `Router` رد می‌کند، همان‌طور که ۳.۲.۱ کرد:

```text
GET / with a user-agent header
  (the handler ran)
  -> 200 OK: hello, curl/8.9.1
GET / without one
  -> 400 Bad Request: missing or unreadable user-agent header
```

در درخواستِ دوم `(the handler ran)` هیچ‌وقت چاپ نمی‌شود. ردِ درخواست همان پاسخ شد، و `hello` اصلاً صدا زده نشد. این دوباره همان تصویرِ DRF است: `AuthenticationFailed`ی که داخلِ یک کلاسِ احرازِ هویت پرتاب شود هم یعنی ویو شروع نمی‌شود. فرقی که باید یادت بماند *جایِ اعلام* است. یک ویوی DRF کلاس‌هایش را از یک صفتِ کلاس یا یک تنظیمِ سراسری می‌گیرد، و از امضایِ ویو نمی‌فهمی کدام‌ها اجرا می‌شوند. اکسترکتورهایِ یک هندلرِ `axum` همان آرگومان‌هایش هستند، پس امضا کلِ فهرست است، و کامپایلر بررسی‌اش می‌کند.

```senpai-visual
{"kind":"network","labels":["درخواست","ApiKey: هدرها را می‌خواند","Pagination: کوئری را می‌خواند","هندلر اجرا می‌شود","ردِ درخواست اینجا کار را تمام می‌کند"]}
```

### ردِ درخواست یک پاسخ است

هر ردِ درخواستِ بالا یک `(StatusCode, &'static str)` است. صفت `type Rejection: IntoResponse` را می‌خواهد، و `axum` از قبل `IntoResponse` را برایِ همین جفت پیاده کرده: کدِ وضعیت می‌شود خطِ وضعیت و رشته می‌شود بدنه. می‌توانی `StatusCode` تنها یا یک `&'static str` ساده هم بگذاری. نوشتنِ `IntoResponse` برایِ نوعِ خطایِ *خودت*، تا یک خطایِ غنی به بدنه‌ی JSON تبدیل شود، موضوعِ [۳.۲.۳](../03-anime-catalog-crud-in-memory/README.fa.md) است. تا آن موقع یک تاپل کافی است، و «خطاهایی که خواهی دید» نشان می‌دهد اگر از آن بگذری کامپایلر چه می‌گوید.

### صدا زدنِ اکسترکتورِ دیگر از داخلِ اکسترکتورِ خودت

اکسترکتورِ تو لازم نیست همه‌چیز را خودش پارس کند. می‌تواند یک اکسترکتورِ موجود را رویِ همان `parts` صدا بزند و بعد قاعده‌هایِ خودش را اضافه کند:

```rust
async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
    let Query(params) = Query::<SearchParams>::from_request_parts(parts, state)
        .await
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid query string"))?;
    let term = params.q.unwrap_or_default().trim().to_string();
    if term.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "missing search term: add ?q=..."));
    }
    Ok(SearchTerm(term))
}
```

این بدنه‌ی `from_request_parts`ِ `SearchTerm` در `examples/03-composing-query.rs` است. `Query` پارس می‌کند، `SearchTerm` «موجود و غیرخالی» را اضافه می‌کند. چهار درخواست:

```text
/search?q=frieren    -> 200 OK: searching for "frieren"
/search?q=%20%20     -> 400 Bad Request: missing search term: add ?q=...
/search              -> 400 Bad Request: missing search term: add ?q=...
/search?q=a&q=b      -> 400 Bad Request: invalid query string
```

خطِ آخر دو لایه را نشان می‌دهد. `q`ِ تکراری شکستِ خودِ `Query` است که `SearchTerm` با `map_err` به پیامِ خودش تبدیلش کرد. `q`ِ خالی از `Query` راحت رد می‌شود و به قاعده‌ی `SearchTerm` می‌خورد. دقیقاً همین کار را `Pagination` در تمرین‌هایِ پایین می‌کند، با مقدارِ پیش‌فرض و سقف به‌جایِ چکِ خالی‌بودن.

### اختیاری کردنِ یک اکسترکتور

گاهی مجاز است هدر نباشد. ابزاری که برایِ *هر* اکسترکتور کار می‌کند `Result<T, T::Rejection>` است: اکسترکتور هنوز اجرا می‌شود، ولی شکستش به‌جایِ تمام‌کردنِ درخواست، به‌شکلِ یک `Err` به هندلر می‌رسد.

```rust
async fn greet(agent: Result<UserAgent, (StatusCode, &'static str)>) -> String {
    match agent {
        Ok(UserAgent(agent)) => format!("hello, {agent}"),
        Err((_, reason)) => format!("hello, whoever you are ({reason})"),
    }
}
```

`examples/04-optional-with-result.rs` آن را با هدر و بدونِ هدر اجرا می‌کند:

```text
200 OK: hello, curl/8.9.1
200 OK: hello, whoever you are (missing or unreadable user-agent header)
```

`Option<UserAgent>` خواناتر می‌بود، و در `axum` نسخه‌ی ۰٫۷ برایِ هر اکسترکتور کار می‌کرد. در ۰٫۸ نمی‌کند. چنج‌لاگِ `axum-core` برایِ ۰٫۵٫۰ صریح می‌گوید: `Option<T>` به‌عنوانِ اکسترکتور حالا لازم دارد `T` یک صفتِ دوم را پیاده کند، `OptionalFromRequestParts` (یا `OptionalFromRequest` برایِ اکسترکتورِ بدنه). دلیلش این است که «استخراج شکست خورد» دو معنایِ متفاوت می‌تواند داشته باشد، «هدر نیست» و «هدر هست ولی نامعتبر است»، و یک `Option`ِ فراگیر مجبور بود هر دو را `None` حساب کند. حالا هر اکسترکتور خودش می‌گوید `None` برایِ او یعنی چه. «خطاهایی که خواهی دید» نشان می‌دهد اگر باز هم `Option<UserAgent>` بنویسی چه می‌شود. بعضی اکسترکتورهایِ آماده، مثلِ `MatchedPath`، صفتِ تازه را پیاده کرده‌اند. مالِ تو نه، مگر آن `impl` را بنویسی، و `Result` معمولاً راهِ سریع‌تر است.

### چرا `S: Send + Sync`

`S` در `FromRequestParts<S>` همان حالتِ برنامه است که با `.with_state(...)` می‌دهی. اکسترکتوری که از state استفاده نمی‌کند باید هر `S`ای را قبول کند، برایِ همین `impl` جنریک است. `FromRequestParts` می‌خواهد Futureای که برمی‌گردانی `Send` باشد، چون `axum` ممکن است آن را به ریسمانِ دیگری ببرد. یک `async fn` همه‌ی آرگومان‌هایش را داخلِ آن Future زنده نگه می‌دارد، از جمله `&S`، و `&S` فقط وقتی `Send` است که `S: Sync` باشد. کلِ دلیلِ این کران همین است. اگر نگذاریش کامپایلر در آخرین خطایِ پایین بهت می‌گوید. اکسترکتوری که *از* state استفاده می‌کند (مثلاً برایِ چکِ یک کلید در یک فهرست) کرانِ قوی‌تری رویِ `S` می‌گذارد. آن چالش است.

---

## دست‌به‌کد

```sh
cargo run -p p3-02-02-writing-your-own-extractor --example 01-parts-and-body
cargo run -p p3-02-02-writing-your-own-extractor --example 02-user-agent-extractor
cargo run -p p3-02-02-writing-your-own-extractor --example 03-composing-query
cargo run -p p3-02-02-writing-your-own-extractor --example 04-optional-with-result
```

بعد چهار مثالِ خراب، هرکدام پشتِ فیچرِ `broken`:

```sh
cargo build -p p3-02-02-writing-your-own-extractor --example 05-rejection-not-into-response-broken --features broken
cargo build -p p3-02-02-writing-your-own-extractor --example 06-option-extractor-broken --features broken
cargo build -p p3-02-02-writing-your-own-extractor --example 07-missing-send-sync-bound-broken --features broken
cargo build -p p3-02-02-writing-your-own-extractor --example 08-body-extractor-not-last-broken --features broken
```

بعد این‌ها را امتحان کن:

۱. در `02-user-agent-extractor` یک درخواستِ سوم اضافه کن که `user-agent`اش رشته‌ی خالی است. این یک ردِ درخواست است؟ اکسترکتور را طوری عوض کن که باشد.
۲. در `03-composing-query` اکسترکتور را طوری کن که عبارت را به حروفِ کوچک تبدیل کند. کدام خط‌هایِ خروجی عوض می‌شوند؟
۳. در `04-optional-with-result` کدِ وضعیتی را که ردِ درخواست حمل می‌کرد چاپ کن. چند است؟

---

## خطاهایی که خواهی دید

تا وقتی تمرینِ «پیاده‌سازی» را تمام نکرده‌ای، `cargo` بالایِ هر خطایِ زیر هشدارهایِ `unused` را هم (از بدنه‌هایِ `todo!()` در `src/lib.rs`) چاپ می‌کند. این‌جا حذف شده‌اند، پس خروجی از خودِ خطا شروع می‌شود.

### `E0277` — نوعِ ردِ درخواست یک پاسخ نیست

```text
error[E0277]: the trait bound `UserAgentError: IntoResponse` is not satisfied
  --> phase3-backend-foundations\02-axum-and-rest-api-design\02-writing-your-own-extractor\examples\05-rejection-not-into-response-broken.rs:20:22
   |
20 |     type Rejection = UserAgentError;
   |                      ^^^^^^^^^^^^^^ unsatisfied trait bound
   |
help: the trait `IntoResponse` is not implemented for `UserAgentError`
  --> phase3-backend-foundations\02-axum-and-rest-api-design\02-writing-your-own-extractor\examples\05-rejection-not-into-response-broken.rs:12:1
   |
12 | enum UserAgentError {
   | ^^^^^^^^^^^^^^^^^^^
   = help: the following other types implement trait `IntoResponse`:
             &'static [u8; N]
             &'static [u8]
             &'static str
             ()
             (R,)
             (Response<()>, R)
             (Response<()>, T1, R)
             (Response<()>, T1, T2, R)
           and 120 others
note: required by a bound in `axum::extract::FromRequestParts::Rejection`
  --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-core-0.5.6\src\extract\mod.rs:56:21
   |
56 |     type Rejection: IntoResponse;
   |                     ^^^^^^^^^^^^ required by this bound in `FromRequestParts::Rejection`
   = note: the full name for the type has been written to 'M:\SenPai-Rust-Journey\target\debug\examples\05_rejection_not_into_response_broken.long-type-10887280357418551112.txt'
   = note: consider using `--verbose` to print the full type name to the console

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-02-02-writing-your-own-extractor` (example "05-rejection-not-into-response-broken") due to 1 previous error
```

(عددِ نامِ فایلِ `long-type-` رویِ ماشینِ تو فرق دارد، و در هر بیلد هم عوض می‌شود.)

**کامپایلر به چه ایراد می‌گیرد:** `Rejection`ِ مثال یک `enum`ِ ساده‌ی خودساخته است. خط‌هایِ آخر به قاعده اشاره می‌کنند: صفت `type Rejection: IntoResponse` را اعلام کرده، پس هر چه انتخاب کنی باید چیزی باشد که `axum` بتواند به پاسخ تبدیلش کند. فهرستِ نوع‌هایِ واجدِ شرایط بلند است، و `enum`ِ تو در آن نیست.

**راه‌حل:** نوعی را به کار ببر که از قبل واجدِ شرایط است:

```rust
type Rejection = (StatusCode, &'static str);
```

**چرا این راه‌حل است:** وقتی یک اکسترکتور شکست می‌خورد `axum` باید *چیزی* پس بفرستد، و این را فقط برایِ نوعی می‌تواند که بلد باشد تبدیلش کند. یک تاپلِ کدِ وضعیت و متن از قبل در آن فهرست است. دادنِ `IntoResponse` به enumِ خطایِ خودت راه‌حلِ بهترِ بلندمدت است، و ۳.۲.۳ یادش می‌دهد.

### `E0277` — `Option<UserAgent>` در `axum` نسخه‌ی ۰٫۸

```text
error[E0277]: the trait bound `fn(Option<UserAgent>) -> impl Future<Output = String> {greet}: Handler<_, _>` is not satisfied
   --> phase3-backend-foundations\02-axum-and-rest-api-design\02-writing-your-own-extractor\examples\06-option-extractor-broken.rs:38:53
    |
 38 |     let _app: Router = Router::new().route("/", get(greet));
    |                                                 --- ^^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(Option<UserAgent>) -> impl Future<Output = String> {greet}`
    |                                                 |
    |                                                 required by a bound introduced by this call
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
error: could not compile `p3-02-02-writing-your-own-extractor` (example "06-option-extractor-broken") due to 1 previous error
```

**کامپایلر به چه ایراد می‌گیرد:** این خطا اسمِ صفتِ گم‌شده را نمی‌آورد. می‌گوید تابعِ `greet` یک `Handler` نیست، و به خطی اشاره می‌کند که آن را به `get` دادی. تابع فقط وقتی هندلر است که همه‌ی آرگومان‌هایش اکسترکتور باشند، و `Option<UserAgent>` اکسترکتور نیست: پیاده‌سازیِ `Option<T>` در `axum` لازم دارد `T: OptionalFromRequestParts`، و `UserAgent` فقط `FromRequestParts` دارد. این را از رویِ صفحه نمی‌خوانی. از این می‌فهمی که تنها چیزِ تازه در هندلر همان `Option` است.

**راه‌حل:** `Result` بخواه، که برایِ هر اکسترکتور کار می‌کند، مثلِ `examples/04-optional-with-result.rs`:

```rust
async fn greet(agent: Result<UserAgent, (StatusCode, &'static str)>) -> String {
```

**چرا این راه‌حل است:** `Result<T, T::Rejection>` یک پیاده‌سازیِ فراگیر دارد، پس چیزی از `UserAgent` نمی‌خواهد. اگر شکلِ `Option` را می‌خواهی، `OptionalFromRequestParts` را برایِ `UserAgent` پیاده کن و خودت تصمیم بگیر «بدونِ هدر» یعنی چه.

### `E0277` — اکسترکتورِ بدنه که آخر نیست

```text
error[E0277]: the trait bound `fn(String, Method) -> impl Future<Output = String> {upload}: Handler<_, _>` is not satisfied
   --> phase3-backend-foundations\02-axum-and-rest-api-design\02-writing-your-own-extractor\examples\08-body-extractor-not-last-broken.rs:17:54
    |
 17 |     let _app: Router = Router::new().route("/", post(upload));
    |                                                 ---- ^^^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(String, Method) -> impl Future<Output = String> {upload}`
    |                                                 |
    |                                                 required by a bound introduced by this call
    |
    = note: Consider using `#[axum::debug_handler]` to improve the error message
note: required by a bound in `post`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\method_routing.rs:167:16
    |
167 |             H: Handler<T, S>,
    |                ^^^^^^^^^^^^^ required by this bound in `post`
...
445 | top_level_handler_fn!(post, POST);
    | ---------------------------------
    | |                     |
    | |                     required by a bound in this function
    | in this macro invocation
    = note: this error originates in the macro `top_level_handler_fn` (in Nightly builds, run with -Z macro-backtrace for more info)

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-02-02-writing-your-own-extractor` (example "08-body-extractor-not-last-broken") due to 1 previous error
```

**کامپایلر به چه ایراد می‌گیرد:** همان پیامِ خطایِ قبلی، با علتی دیگر. `String` بدنه را می‌خواند، پس فقط `FromRequest` دارد. در پیاده‌سازیِ `Handler` همه‌ی آرگومان‌ها جز آخری باید `FromRequestParts` باشند، و `String` اول است.

**راه‌حل:** اکسترکتورِ بدنه را بیاور آخر:

```rust
async fn upload(method: Method, body: String) -> String {
```

**چرا این راه‌حل است:** آرگومان‌هایِ قبلی تکه‌شان را از `Parts` می‌گیرند، بعد آخری کلِ درخواست را، با بدنه، می‌گیرد. این همان قاعده‌ی ۳.۲.۱ است، و حالا می‌دانی چه چیزی اجرایش می‌کند.

### بدونِ کدِ خطا — Future را نمی‌شود بینِ ریسمان‌ها فرستاد

```text
error: future cannot be sent between threads safely
  --> phase3-backend-foundations\02-axum-and-rest-api-design\02-writing-your-own-extractor\examples\07-missing-send-sync-bound-broken.rs:18:67
   |
18 |     async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
   |                                                                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ future returned by `from_request_parts` is not `Send`
   |
note: captured value is not `Send` because `&` references cannot be sent unless their referent is `Sync`
  --> phase3-backend-foundations\02-axum-and-rest-api-design\02-writing-your-own-extractor\examples\07-missing-send-sync-bound-broken.rs:18:52
   |
18 |     async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
   |                                                    ^^^^^^ has type `&S` which is not `Send`, because `S` is not `Sync`
note: required by a bound in `FromRequestParts::from_request_parts::{anon_assoc#0}`
  --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-core-0.5.6\src\extract\mod.rs:62:64
   |
62 |     ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send;
   |                                                                ^^^^ required by this bound in `FromRequestParts::from_request_parts::{anon_assoc#0}`
help: consider restricting type parameter `S` with trait `Sync`
   |
15 | impl<S: std::marker::Sync> FromRequestParts<S> for UserAgent {
   |       +++++++++++++++++++

error: could not compile `p3-02-02-writing-your-own-extractor` (example "07-missing-send-sync-bound-broken") due to 1 previous error
```

این‌جا کدِ خطا نیست، پس در خطِ `expected:`ِ مثال چیزی جز خودِ جمله نمی‌توانست بیاید. هدرِ مثال همین را می‌گوید.

**کامپایلر به چه ایراد می‌گیرد:** صفت یک Futureِ `+ Send` قول داده است. یک `async fn` همه‌ی آرگومان‌هایش را نگه می‌دارد، پس Future یک `&S` نگه می‌دارد، و `&S` فقط اگر `S: Sync` باشد `Send` است. این `impl<S>` درباره‌ی `S` هیچ‌چیز نمی‌گوید.

**راه‌حل:** `rustc` پیشنهاد می‌کند `S: Sync`. `Send + Sync` را بگذار، همان کرانی که مستنداتِ خودِ `axum` رویِ اکسترکتورهایِ سفارشی‌اش می‌گذارد:

```rust
impl<S: Send + Sync> FromRequestParts<S> for UserAgent {
```

**چرا این راه‌حل است:** تنها چیزی را که Future لازم دارد می‌گوید: اینکه به‌اشتراک‌گذاشتنِ یک ارجاع به state بینِ ریسمان‌ها امن است. برایِ فراخواننده هزینه‌ای ندارد، چون هر stateِ واقعیِ برنامه از قبل این را برآورده می‌کند.

---

## تمرین

### گرم‌کردن

<details>
<summary>آیا یک هندلر می‌تواند دو اکسترکتور داشته باشد که هر دو بدنه‌ی درخواست را بخوانند؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

نه. بدنه جریانی است که یک‌بار خوانده می‌شود. هر اکسترکتوری که آن را می‌خواند `FromRequest` را پیاده می‌کند و کلِ درخواست را با مقدار می‌گیرد، پس حداکثر یکی می‌تواند وجود داشته باشد، و `axum` اصرار دارد آخرین آرگومان باشد. وگرنه خطایِ کامپایل می‌گیری.

</details>

<details>
<summary>یک اکسترکتورِ <code>ApiKey</code> هدرِ <code>x-api-key</code> را پیدا نمی‌کند و با <code>401</code> رد می‌کند. هندلر اجرا می‌شود؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

نه. `axum` ردِ درخواست را به پاسخ تبدیل می‌کند و برمی‌گرداند. هندلر هیچ‌وقت صدا زده نمی‌شود، و اکسترکتورهایِ بعد از `ApiKey` در فهرستِ آرگومان‌ها هم هیچ‌وقت اجرا نمی‌شوند.

</details>

<details>
<summary>یک اکسترکتور فقط هدرِ <code>authorization</code> را می‌خواند. <code>FromRequestParts</code> یا <code>FromRequest</code>؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

`FromRequestParts`. هدرها در `Parts` هستند، و این‌جا چیزی به بدنه نیاز ندارد. انتخابِ `FromRequest` بی‌دلیل مجبورش می‌کرد آخرین آرگومان باشد.

</details>

<details>
<summary>در <code>axum</code> نسخه‌ی ۰٫۸، آیا <code>async fn f(key: Option&lt;ApiKey&gt;)</code> کامپایل می‌شود وقتی <code>ApiKey</code> فقط <code>FromRequestParts</code> را پیاده کرده؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

نه. `Option<T>` به‌عنوانِ اکسترکتور `T: OptionalFromRequestParts` لازم دارد. به‌جایش `Result<ApiKey, (StatusCode, &'static str)>` بنویس، یا صفتِ اختیاری را پیاده کن.

</details>

### تعمیر

چهار مثالِ خراب را درست کن. همچنان با `--features broken` بیلدشان کن؛ هرکدام بعد از درست‌شدن باید کامپایل شود، و هر راه‌حل یکی دو خط است:

۱. `examples/05-rejection-not-into-response-broken.rs`: یک نوعِ ردِ درخواست بگذار که `axum` بتواند بفرستد.
۲. `examples/06-option-extractor-broken.rs`: هندلر را کامپایل‌شدنی کن، و همچنان نبودنِ هدر را «بدونِ user agent» حساب کن.
۳. `examples/07-missing-send-sync-bound-broken.rs`: سرِ `impl` را درست کن.
۴. `examples/08-body-extractor-not-last-broken.rs`: ترتیبِ آرگومان‌ها را درست کن.

### پیاده‌سازی

دو اکسترکتور در `src/lib.rs`:

```sh
cargo test -p p3-02-02-writing-your-own-extractor
```

هر `impl` در کامنتِ مستنداتِ خودش کامل مشخص شده، پس لازم نیست تست‌ها را بخوانی:

- `ApiKey`: هدرِ `x-api-key`، با حذفِ فاصله‌هایِ دورش، یا `401`.
- `Pagination`: `page` و `per_page` از رشته‌ی کوئری، با مقدارِ پیش‌فرض، سقف، و دو نوع `400`. قرار است `Query` را صدا بزند، همان‌طور که `SearchTerm` زد.

روتر `app()` و دو هندلرش از قبل نوشته شده‌اند. پیش از شروع، هر چهارده تست با `not yet implemented` شکست می‌خورند. `ApiKey` و `Pagination` یازده‌تایشان را پاس می‌کنند؛ سه‌تایِ آخر مالِ پله‌ی بعدی‌اند. یک تست، `extractors_run_left_to_right_so_the_key_is_checked_first`، ترتیبِ اجرایِ اکسترکتورها را چک می‌کند، همان که بالا دیدی.

### بساز

`ClientVersion` در همان فایل: نسخه‌ی `MAJOR.MINOR` از هدرِ `x-client-version`، مثلِ `1.4`. کامنتِ مستنداتِ خودش را دارد، با دو پیامِ `400`ِ متفاوت، و تست‌هایش سه‌تایِ `client_version` هستند. پیش از شروع فکر کن «دقیقاً دو عددِ صحیح» یعنی چه: `1.2.3`، `1.` و `-1.2` همه رد می‌شوند.

### چالش (اختیاری)

`ApiKey` هر کلیدی را قبول می‌کند. یک اکسترکتورِ `Authorized` بنویس که کلید را با مجموعه‌ای از کلیدهایِ شناخته‌شده که در stateِ برنامه نگه‌داری می‌شود چک کند، و کلیدِ ناشناخته را با `403` رد کند، در حالی که کلیدِ غایب `401` می‌ماند. باید با هر stateای کار کند که بتواند یک `KeyStore` *تولید* کند، یعنی کرانِ `KeyStore: FromRef<S>` رویِ `impl`، نه یک نوعِ stateِ مشخص. `FromRef` در `axum::extract` است. ساده‌ترین راه این است که اکسترکتورِ خودِ `ApiKey` را از داخلِ مالِ خودت صدا بزنی. `State` و جایی که زندگی می‌کند در [۳.۴.۲](../../04-configuration-and-app-structure/02-app-state-and-dependency-wiring/README.fa.md) درست‌وحسابی بررسی می‌شود، پس این درس فقط می‌خواهد امتحانش کنی.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| `FromRequestParts` | صفتِ اکسترکتورهایی که فقط متد، URI، هدرها و extensions را می‌خوانند | هر اکسترکتوری که بدنه نمی‌خواند |
| `FromRequest` | صفتِ اکسترکتورهایی که کلِ درخواست را، با بدنه، می‌گیرند | `Json`، `String`، `Bytes`؛ حداکثر یکی، و آخر |
| `Parts` | درخواست بدونِ بدنه: داده‌ی ساده و قابلِ‌اشتراک | آرگومانِ اولِ `from_request_parts` |
| ردِ درخواست (rejection) | نوعِ خطایی که اکسترکتور برمی‌گرداند؛ باید یک پاسخ باشد | `(StatusCode, &'static str)` تا ۳.۲.۳ |
| ترکیبِ اکسترکتورها | صدا زدنِ یک اکسترکتور از داخلِ دیگری رویِ همان `parts` | `Pagination` رویِ `Query` |
| `Result<T, T::Rejection>` | اکسترکتوری که شکستش به‌جایِ تمام‌کردنِ درخواست به هندلر می‌رسد | هدرهایِ اختیاری، صفحه‌هایِ خطایِ سفارشی |
| `OptionalFromRequestParts` | صفتِ ۰٫۸ که `Option<T>` را برایِ `T` معنا می‌دهد | اکسترکتورهایِ اختیاری که `Parts` می‌خوانند |

### الان می‌دانی

- یک درخواست `Parts` است (هر چند بار قابلِ‌خواندن) و یک بدنه (یک‌بار قابلِ‌خواندن). `FromRequestParts` برایِ اولی است، `FromRequest` برایِ دومی.
- «`Json` باید آخر باشد» یک قاعده‌ی سلیقه‌ای نیست. بدنه را هر که بخواند مصرفش می‌کند، پس فقط یک اکسترکتور می‌تواند، و آخری همان است که درخواست را می‌گیرد.
- نوشتنِ یک اکسترکتور یعنی انتخابِ یک `Rejection`، نوشتنِ یک `async fn from_request_parts`، و نگه‌داشتنِ کرانِ `S: Send + Sync`.
- ردِ درخواست به پاسخ تبدیل می‌شود و هندلر اجرا نمی‌شود. اکسترکتورها چپ به راست اجرا می‌شوند.
- در ۰٫۸، `Option<T>` به `OptionalFromRequestParts` نیاز دارد. `Result<T, T::Rejection>` برایِ همه‌چیز کار می‌کند.

### بعداً کامل‌تر می‌بینی

- **تبدیلِ نوعِ خطایِ خودت به پاسخِ HTTP با `IntoResponse`** — [۳.۲.۳ — عملیاتِ CRUD رویِ کاتالوگِ انیمه (داخل حافظه)](../03-anime-catalog-crud-in-memory/README.fa.md)
- **کدی که پیش از *هر* مسیر اجرا می‌شود، نه فقط مسیرهایی که اکسترکتور می‌خواهند** — [۳.۲.۴ — `tower::Service` / `Layer`: میان‌افزار با دست](../04-tower-service-and-layer-middleware/README.fa.md)
- **حالتِ برنامه و سیم‌کشیِ `FromRef` پشتِ یک اکسترکتورِ آگاه از state** — [۳.۴.۲ — حالتِ برنامه و سیم‌کشیِ وابستگی‌ها](../../04-configuration-and-app-structure/02-app-state-and-dependency-wiring/README.fa.md)
- **صفحه‌بندیِ offset در برابرِ keyset، طراحیِ واقعیِ پشتِ `?page=`** — [۳.۶.۲ — صفحه‌بندی: offset در برابرِ keyset](../../06-database-design-and-query-performance/02-pagination/README.fa.md)
- **احرازِ هویتِ یک درخواست با JWT به‌جایِ یک کلیدِ ثابت** — [۳.۷.۳ — JWT و میان‌افزارِ `tower`](../../07-auth-and-security/03-jwt-and-tower-middleware/README.fa.md)
- **یک قالب برایِ بدنه‌ی هر خطایی که APIات می‌فرستد** — [۳.۸.۱ — پوشش‌هایِ خطایِ یکدست](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا یک هندلر می‌تواند شش اکسترکتورِ `FromRequestParts` داشته باشد ولی فقط یک اکسترکتورِ `FromRequest`؟
- `axum` با `Rejection`ِ یک اکسترکتور چه می‌کند، و هندلر اجرا می‌شود؟
- چرا `impl` به `S: Send + Sync` نیاز دارد، حتی وقتی اکسترکتور هیچ‌وقت به state دست نمی‌زند؟
- `Option<T>` به‌عنوانِ اکسترکتور بینِ `axum` نسخه‌ی ۰٫۷ و ۰٫۸ چه تغییری کرد، و راهِ سریعِ دور زدنش چیست؟
- اکسترکتورِ `axum` چه شباهتی به کلاسِ احرازِ هویتِ DRF دارد، و مقایسه کجا تمام می‌شود؟

---

## بیشتر

- [`axum::extract`](https://docs.rs/axum/0.8/axum/extract/index.html): صفحه‌ی رسمیِ اکسترکتورها، با بخش‌هایِ «Defining custom extractors» و «Optional extractors».
- [`FromRequestParts`](https://docs.rs/axum/0.8/axum/extract/trait.FromRequestParts.html) و [`OptionalFromRequestParts`](https://docs.rs/axum/0.8/axum/extract/trait.OptionalFromRequestParts.html): دو صفتِ این درس، همان‌طور که مستند شده‌اند.
- [چنج‌لاگِ `axum-core`](https://docs.rs/crate/axum-core/0.5.6/source/CHANGELOG.md): مدخلِ ۰٫۵٫۰ که `#[async_trait]` در آن برداشته شد و `Option<T>` عوض شد.
- [DRF — Authentication](https://www.django-rest-framework.org/api-guide/authentication/): تصویرِ سمتِ جنگو که این درس با آن مقایسه کرد.
