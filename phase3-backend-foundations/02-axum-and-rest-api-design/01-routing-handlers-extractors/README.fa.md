# ۳.۲.۱ — مسیریابی، هندلرها، اکسترکتورها

## در یک نگاه

بعد از این درس می‌توانی:

- با هندلرهایِ async یک `Router` در `axum` بسازی، و بگویی هر پارامترِ هندلر (`Path`، `Query`، `Json`، `State`) کدام بخشِ درخواست را می‌خواند.
- وقتی یک اکسترکتور درخواست را رد می‌کند، کدِ وضعیتی را که خودِ `axum` برمی‌گرداند پیش‌بینی کنی (`400`، `415`، `422`، `404`، `405`)، و بگویی چرا بدنه‌ی هندلر اصلاً اجرا نمی‌شود.
- یک روتر را از مسیریابی تا هندلرِ خودت تا پاسخِ سریالایزشده، با `oneshot` و بدونِ سوکت تست کنی، و همان را با `curl` واقعی امتحان کنی.
- خطایِ `E0277` با متنِ `Handler<_, _>` را، که وقتی اکسترکتورِ بدنه آخر نباشد می‌گیری، بخوانی و درستش کنی.

**زمان:** حدود ۹۰ دقیقه · **پیش‌نیاز:**
[۳.۱.۲ — پارسرِ دست‌سازِ HTTP](../../01-networking-and-http-from-scratch/02-hand-rolled-http-parser/README.fa.md)،
[۳.۱.۳ — چیزهایی از HTTP که باید بدونی](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.fa.md)،
[۲.۸.۶ — مبانیِ `tokio`](../../../phase2-intermediate/08-concurrency/06-tokio-basics/README.fa.md)

---

## چرا اهمیت دارد

ماژولِ ۱ HTTP را با دست ساخت: در ۳.۱.۱ یک حلقه‌ی `TcpListener`، و در ۳.۱.۲ یک `parse_request` که بایت را به `HttpRequest` تبدیل می‌کرد و برایِ هر شکلِ خرابیِ ورودی یک خطایِ نام‌دار برمی‌گرداند. `axum` همان کار است، برایِ هر درخواست، به دستِ یک کتابخانه؛ و آنچه برایِ تو می‌ماند همان است که جنگو هم برایِ تو می‌گذارد: **مسیرها** (`urls.py`) و **هندلرها** (ویوها).

بخشِ ارزشمند، پیوندِ این دو است. وقتی ۳.۱.۲ گفت «`axum` وقتی `Json<T>` یا `Path<T>` جور نباشد یک ۴۰۰ِ خودکار برمی‌گرداند»، منظورش این بود که یک *اکسترکتور* همان `parse_request` است که کسِ دیگری نوشته و پیش از تابعِ تو اجرا می‌شود. در این درس دقیقاً می‌بینی چه جوابی می‌دهد، و می‌بینی که جواب همیشه `400` نیست.

| Django / DRF | `axum` |
|---|---|
| `urls.py`: `path("greet/<name>/", views.greet)` | `Router::new().route("/greet/{name}", get(greet))` |
| ویوی `def greet(request, name)` | هندلرِ `async fn greet(Path(name): Path<String>) -> String` |
| `request.GET["q"]` | `Query<T>` |
| سریالایزرِ DRF که `request.data` را می‌خواند | `Json<T>` |
| kwargهایِ URL | `Path<T>` |
| فهرستِ میان‌افزارها | لایه‌هایِ `tower`، که در [۳.۲.۴](../04-tower-service-and-layer-middleware/README.fa.md) با دست می‌سازی |

این جدول یک حد دارد. ویویِ جنگو *یک* شیءِ `request` می‌گیرد و خودش در آن می‌گردد. هندلرِ `axum` فقط چیزی را می‌گیرد که اعلام کرده، آماده و پارس‌شده، یا اصلاً صدا زده نمی‌شود.

---

## مفهوم

### هندلر یک تابعِ async است که پارامترهایش همان درخواست‌اند

```rust
async fn greet(Path(name): Path<String>) -> String {
    format!("Hello, {name}!")
}
```

وقتی رویِ سوکتِ واقعی سرو شود (`examples/05-serve-on-a-real-port.rs` همین هندلر را سرو می‌کند)، `curl -i http://127.0.0.1:3021/greet/senpai` این را چاپ می‌کند:

```text
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 14
date: Thu, 24 Sep 2026 22:49:24 GMT

Hello, senpai!
```

(مقدارِ `date` همان لحظه‌ای است که اجرا می‌کنی.) سه کار انجام شد که تو ننوشتی: `axum` درخواست را خواند، بخشِ `senpai` را در `name` گذاشت، و `String`ِ برگشتی را به پاسخی با خطِ وضعیت، `content-type` و `content-length`ِ درست تبدیل کرد. آخری همان کاری است که `HttpResponse::to_bytes`ِ ۳.۱.۲ با دست می‌کرد.

دو قاعده را همین‌جا بگیر:

- **فهرستِ پارامترها همان درخواست است.** `axum` به *نوعِ* هر پارامتر نگاه می‌کند و از آن نوع می‌خواهد خودش را از درخواست بیرون بکشد. در امضا اعلام می‌کنی چه می‌خواهی، و پارس‌شده می‌رسد.
- **نوعِ خروجی همان پاسخ است.** هر چیزی که `IntoResponse` را پیاده کرده باشد کار می‌کند: `&'static str`، `String`، `Json<T>`، یک جفتِ `(StatusCode, body)`. ۳.۲.۳ یاد می‌دهد چطور نوعِ خطایِ خودت هم این‌طور شود.

تابع `async fn` است چون `axum` از هندلر همین را می‌خواهد. هنوز چیزی برایِ `.await` کردن نداری؛ آن با دیتابیس در ماژولِ ۵ می‌آید.

### `Router` جفتِ (متد، مسیر) را به یک هندلر وصل می‌کند

```rust
let app = Router::new()
    .route("/", get(hello))
    .route("/greet/{name}", get(greet));
```

هر `.route(path, method_router)` یک مسیر را ثبت می‌کند، و `get(handler)` و `post(handler)` متدِ HTTP را انتخاب می‌کنند. پارامترِ مسیر با آکولاد نوشته می‌شود: `{name}`. `examples/01-first-router.rs` همین روتر را می‌سازد و چهار درخواست برایش می‌فرستد:

```text
GET  /              -> 200 OK  body: "Hello, world!"
GET  /greet/senpai  -> 200 OK  body: "Hello, senpai!"
GET  /nope          -> 404 Not Found  body: ""
POST /greet/senpai  -> 405 Method Not Allowed  body: ""  allow: "GET,HEAD"
```

برایِ دو خطِ آخر هیچ کدی ننوشتی. مسیری که با هیچ‌چیز جور نیست `404` است. مسیری که هست ولی فقط برایِ متدهایِ دیگر ثبت شده `405` است، با یک هدرِ `allow` که متدهایِ ممکن را فهرست می‌کند. این همان معناشناسیِ متدهایِ ۳.۱.۳ است که روتر اعمال می‌کند.

```senpai-visual
{"kind":"network","labels":["درخواست می‌رسد: متد و مسیر","Router هندلر را انتخاب می‌کند","اکسترکتورها از چپ به راست اجرا می‌شوند","اگر یکی رد کند: axum خودش ۴۰۰ یا ۴۱۵ یا ۴۲۲ می‌دهد و هندلر رد می‌شود","هندلر اجرا می‌شود و مقدار برمی‌گرداند","IntoResponse آن را به پاسخِ HTTP تبدیل می‌کند"]}
```

### اکسترکتورها: `Path`، `Query`، `Json`، `State`

چهار اکسترکتور بیشترِ هندلرها را پوشش می‌دهند:

- **`Path<T>`** بخش‌هایِ `{name}` را از الگویِ مسیر می‌خواند.
- **`Query<T>`** رشته‌ی کوئری را می‌خواند. در ۳.۱.۲ `?status=watching` را با دست رویِ `&` و `=` تکه کردی و جفت‌ها را در یک `Vec` نگه داشتی. `Query<T>` همان تکه‌کردن را، با decodeِ درصدی، انجام می‌دهد و یک ساختار را پر می‌کند:

```rust
#[derive(Debug, Deserialize)]
struct AnimeFilter {
    status: String,
    page: Option<u32>,
}

async fn list_anime(Query(filter): Query<AnimeFilter>) -> String {
    format!("{filter:?}")
}
```

```text
GET /anime?status=watching
    -> 200 OK: AnimeFilter { status: "watching", page: None }
GET /anime?status=watching&page=2
    -> 200 OK: AnimeFilter { status: "watching", page: Some(2) }
GET /anime?status=plan%20to%20watch
    -> 200 OK: AnimeFilter { status: "plan to watch", page: None }
GET /anime?page=2
    -> 400 Bad Request: Failed to deserialize query string: missing field `status`
GET /anime?status=watching&page=two
    -> 400 Bad Request: Failed to deserialize query string: page: invalid digit found in string
```

این `examples/03-query-extractor.rs` است. فیلدی از نوعِ `Option<u32>` اختیاری است؛ فیلدی از نوعِ `String` اجباری. فرقِ `Path` و `Query` همین است: بخشِ مسیر می‌گوید *کدام* منبع (`/anime/7`)، و پارامترِ کوئری می‌گوید *چطور* آن را می‌خواهی (`?page=2`) و معمولاً اختیاری است.

- **`Json<T>`** بدنه‌ی درخواست را به‌عنوانِ JSON در `T` پارس می‌کند؛ `T` باید `Deserialize` داشته باشد. اگر `Json<T>` را برگردانی، `T` سریالایز می‌شود و `content-type: application/json` گذاشته می‌شود. همان سریالایزرِ DRF است، با اعتبارسنجیِ توکار.
- **`State<S>`** یک کپی از مقداری را می‌دهد که به `.with_state(...)` داده‌ای:

```rust
#[derive(Clone, Default)]
struct AppState {
    visits: Arc<Mutex<u64>>,
}

async fn visit(State(state): State<AppState>) -> String {
    let mut visits = state.visits.lock().unwrap();
    *visits += 1;
    format!("visit #{}", *visits)
}
```

```text
clone of the same router -> visit #1
clone of the same router -> visit #2
clone of the same router -> visit #3
a brand-new AppState     -> visit #1
```

(`examples/04-shared-state.rs`.) کلون‌کردنِ `AppState` خودِ `Arc` را کلون می‌کند، نه عدد را؛ پس هر درخواست همان شمارنده را می‌بیند، و یک `AppState`ِ تازه از اول شروع می‌کند. این همان `Arc<Mutex<T>>`ِ [۲.۶.۳](../../../phase2-intermediate/06-smart-pointers/03-rc-and-arc/README.fa.md) است، با درخواست‌هایِ HTTP به‌جایِ ریسمان‌ها، و `Mutex` دلیلِ امن‌بودنش وقتی دو درخواست هم‌زمان می‌رسند.

### وقتی اکسترکتور رد می‌کند: `axum` چه جوابی خودکار می‌دهد

`parse_request`ِ ۳.۱.۲ برایِ هر نوع ورودیِ بد یک `Err(HttpParseError::...)` برمی‌گرداند. اکسترکتورِ `axum` همین کار را می‌کند و در حالتِ `Err`، به‌جایِ صدا زدنِ هندلرت یک پاسخ می‌فرستد. `examples/02-what-axum-answers-for-you.rs` برایِ یک هندلرِ `Json<EchoRequest>` و یکی با `Path<u32>` هفت درخواست می‌فرستد:

```text
POST /echo {"message":"hi"}
    -> 200 OK: echo: hi
POST /echo not json
    -> 400 Bad Request: Failed to parse the request body as JSON: expected ident at line 1 column 2
POST /echo {"msg":"hi"}
    -> 422 Unprocessable Entity: Failed to deserialize the JSON body into the target type: missing field `message` at line 1 column 12
POST /echo {"message":42}
    -> 422 Unprocessable Entity: Failed to deserialize the JSON body into the target type: message: invalid type: integer `42`, expected a string at line 1 column 13
POST /echo {"message":"hi"}   (no Content-Type header)
    -> 415 Unsupported Media Type: Expected request with `Content-Type: application/json`
GET /anime/7
    -> 200 OK: anime #7
GET /anime/seven
    -> 400 Bad Request: Invalid URL: Cannot parse `seven` to a `u32`
```

پس «۴۰۰ِ خودکار» یک ساده‌سازی بود. جوابِ واقعی، برایِ `axum` نسخه‌ی ۰٫۸٫۹:

| چه رسید | کدِ وضعیت | چرا |
|---|---|---|
| بدنه‌ای که اصلاً JSON نیست | `400` | بدشکل است: نحوش خراب است |
| JSONِ معتبر با شکلِ غلط (فیلدِ گم‌شده، نوعِ غلط) | `422` | پارس شد، ولی محتوا نامعتبر است |
| بدنه‌ی JSON بدونِ `Content-Type: application/json` | `415` | نوعِ رسانه غلط است، بدنه هر چه باشد |
| مقدارِ `Path` یا `Query` که به نوع پارس نمی‌شود | `400` | خودِ خطِ درخواست غلط است |

دو ردیفِ اولْ همان قاعده‌ی `400` در برابرِ `422`ِ ۳.۱.۳ است که اکسترکتور اعمال می‌کند: `400` یعنی درخواست بدشکل است، `422` یعنی درست پارس شد ولی محتوایش نامعتبر است. `415` کدی است که ۳.۱.۳ فهرست نکرد؛ سرور می‌گوید *فرمت* را نمی‌پذیرد، که همان شکستِ مذاکره‌ی محتواست. ردیفِ آخر نشان می‌دهد این قاعده یکدست اعمال نمی‌شود: `Path` و `Query` `400` می‌دهند، با اینکه `?page=two` به «پارس شد ولی محتوا نامعتبر است» نزدیک‌تر است. این انتخابِ `axum` است؛ امتحانش کن، فرضش نکن.

دو نتیجه می‌گیری. بدنه‌ی هندلرت می‌تواند فرض کند پارامترهایش معتبرند، چون درخواستِ نامعتبر داخل نمی‌شود. و متنِ خطا از `serde` و `axum` می‌آید، متنِ ساده است نه JSON؛ یک بدنه‌ی خطایِ JSONِ یکدست کارِ [۳.۸.۱](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.fa.md) است.

### ترتیبِ اکسترکتورها: بدنه آخر می‌آید

اکسترکتورها به ترتیبِ پارامترها، از چپ به راست اجرا می‌شوند. بدنه‌ی درخواست جریانی است که فقط یک‌بار خوانده می‌شود، پس `Json<T>` (و `String` و `Bytes`) فقط می‌تواند **آخرین** پارامتر باشد. `axum` این را در نوع‌ها اعمال می‌کند: هر پارامتر جز آخری باید `FromRequestParts` باشد، و آخری باید `FromRequest`. `Path`، `Query` و `State` از نوعِ اولند؛ `Json` از نوعِ دوم. اگر `Json` را اول بگذاری تابع اصلاً هندلر نیست، که همان `E0277`ِ بخشِ «خطاهایی که خواهی دید» است.

### زیرِ `Router`: تسکِ `tokio`، نه ریسمان

۳.۱.۱ برایِ هر اتصال یک ریسمانِ سیستم‌عامل ساخت و گفت `tokio` جایش را می‌گیرد. `axum::serve(listener, app)` رویِ `tokio` اجرا می‌شود: اتصال‌ها را در یک حلقه قبول می‌کند و برایِ هر کدام با `tokio::spawn` یک تسک می‌سازد که آن اتصال را پیش می‌برد (در ماژولِ `serve`ِ `axum` ۰٫۸٫۹ بررسی شد). تسک خیلی ارزان‌تر از ریسمانِ OS است، همان‌طور که [۲.۸.۶](../../../phase2-intermediate/08-concurrency/06-tokio-basics/README.fa.md) نشان داد، و برایِ همین است که شکلِ هندلر `async fn` است: وقتی یک درخواست منتظر می‌ماند، ریسمان به درخواستِ دیگری می‌رسد. تسک به‌ازایِ اتصال است، نه به‌ازایِ درخواست: درخواست‌هایی که پشتِ هم رویِ یک اتصالِ keep-alive (۳.۱.۳) می‌آیند، با همان تسک سرو می‌شوند.

### تست بدونِ سوکت: `oneshot`

`Router` یک `tower::Service` است: چیزی که درخواست می‌گیرد و پاسخ برمی‌گرداند. پس یک تست می‌تواند با `tower::ServiceExt::oneshot` یک `Request` را مستقیم به آن بدهد، و مسیریابی، استخراج، هندلرِ تو و سریالایز همه اجرا می‌شوند، بدونِ پورتِ باز و بدونِ شبکه:

```rust
let response = app(AppState::default())
    .oneshot(
        Request::builder()
            .uri("/echo") // a GET, but only POST is registered
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();

assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
```

این همان ایده‌ی «I/O در لبه»ی `run_echo`ِ ۳.۱.۱ است، یک لایه بالاتر. در راه‌حلِ کامل، `cargo test --test routes_test` این را چاپ می‌کند (ترتیبِ خطوط عوض می‌شود):

```text
running 7 tests
test wrong_method_on_a_known_route_returns_405 ... ok
test unknown_route_returns_404 ... ok
test root_returns_hello_world ... ok
test greet_uses_the_path_segment ... ok
test echo_rejects_a_malformed_json_body_before_the_handler_runs ... ok
test counter_starts_at_zero_and_increments ... ok
test echo_round_trips_json_and_reports_length ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

---

## دست‌به‌کد

پنج مثال را اجرا کن. چهارتای اول از `oneshot` استفاده می‌کنند؛ پنجمی یک پورتِ واقعی را می‌گیرد.

```sh
cargo run -p p3-02-01-routing-handlers-extractors --example 01-first-router
cargo run -p p3-02-01-routing-handlers-extractors --example 02-what-axum-answers-for-you
cargo run -p p3-02-01-routing-handlers-extractors --example 03-query-extractor
cargo run -p p3-02-01-routing-handlers-extractors --example 04-shared-state
cargo run -p p3-02-01-routing-handlers-extractors --example 05-serve-on-a-real-port
```

پنجمی را روشن بگذار و در یک ترمینالِ دوم واقعی امتحانش کن. پورتِ `3021` انتخاب شده تا با سرورهایِ دیگر تداخل نکند:

```sh
curl -si -X POST -H 'content-type: application/json' -d '{"message":"hi"}' http://127.0.0.1:3021/shout
curl -si -X POST -H 'content-type: application/json' -d '{"message":' http://127.0.0.1:3021/shout
curl -si http://127.0.0.1:3021/shout
```

```text
HTTP/1.1 200 OK
content-type: application/json
content-length: 16
date: Thu, 24 Sep 2026 22:49:24 GMT

{"shouted":"HI"}
HTTP/1.1 400 Bad Request
content-type: text/plain; charset=utf-8
content-length: 96
date: Thu, 24 Sep 2026 22:49:24 GMT

Failed to parse the request body as JSON: message: EOF while parsing a value at line 1 column 11
HTTP/1.1 405 Method Not Allowed
allow: POST
content-length: 0
date: Thu, 24 Sep 2026 22:49:24 GMT

```

وقتی تمام شد سرور را با Ctrl+C ببند. بعد این‌ها را امتحان کن:

۱. در `03-query-extractor` نوعِ `page` را از `Option<u32>` به `u32`ِ ساده تبدیل کن. کدام‌یک از پنج درخواست عوض می‌شود، و به چه؟
۲. در `02-what-axum-answers-for-you` نوعِ `Path<u32>` را به `Path<String>` عوض کن. حالا `/anime/seven` چه برمی‌گرداند؟
۳. در `04-shared-state` از `.clone()`ِ فراخوانیِ اول بردار و `shared` را دو بار به‌کار ببر. کامپایلر چه می‌گوید، و چرا؟

---

## خطاهایی که خواهی دید

### `E0277` — `Handler<_, _>` برقرار نیست

```text
error[E0277]: the trait bound `fn(Json<NewNote>, State<AppState>) -> impl Future<Output = String> {add_note}: Handler<_, _>` is not satisfied
   --> phase3-backend-foundations\02-axum-and-rest-api-design\01-routing-handlers-extractors\examples\06-body-extractor-not-last-broken.rs:33:31
    |
 33 |         .route("/notes", post(add_note))
    |                          ---- ^^^^^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(Json<NewNote>, State<AppState>) -> impl Future<Output = String> {add_note}`
    |                          |
    |                          required by a bound introduced by this call
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
```

**کامپایلر به چه ایراد می‌گیرد:** خطا رویِ `post(add_note)` است، نه داخلِ `add_note`. `post` می‌خواهد آرگومانش `Handler` را پیاده کرده باشد، و `axum` برایِ یک `async fn` فقط وقتی `Handler` را پیاده می‌کند که همه‌ی پارامترها جز آخری `FromRequestParts` باشند و آخری `FromRequest`. اینجا `Json<NewNote>` اول آمده، و `Json` بدنه را می‌خواهد، پس فقط می‌تواند `FromRequest` باشد. پیام می‌گوید *کدام تابع* مشکل دارد، نه *کدام پارامتر*؛ برایِ همین بی‌کمک به نظر می‌رسد. خطِ اول را برایِ شکلِ تابعت بخوان: `fn(Json<NewNote>, State<AppState>)`، و ببین اکسترکتورِ بدنه آخر نیست.

**راه‌حل:** `Json<NewNote>` را به جایگاهِ آخر ببر:

```rust
async fn add_note(State(state): State<AppState>, Json(note): Json<NewNote>) -> String {
```

**چرا این راه‌حل است:** بدنه فقط یک‌بار خوانده می‌شود، پس باید بعد از هر چیزی بیاید که فقط سرِ درخواست را می‌بیند (متد، مسیر، هدرها، state). یادداشتِ خروجی `#[axum::debug_handler]` را نام می‌برد، اتریبیوتی (attribute) که این خطا را به خطایی تبدیل می‌کند که پارامترِ مشکل‌دار را نشان می‌دهد. پشتِ فیچرِ اختیاریِ `macros`ِ `axum` است که این workspace روشنش نکرده، پس اینجا مثالی برایش نیست؛ وقتی فیچر روشن باشد، اتریبیوت را رویِ هندلر بگذار.

### یک پنیکِ زمانِ اجرا — نحوِ قدیمیِ `/:id`

```text
building the router...
thread 'main' (13560) panicked at phase3-backend-foundations\02-axum-and-rest-api-design\01-routing-handlers-extractors\examples\07-colon-path-syntax-broken.rs:18:38:
Path segments must not start with `:`. For capture groups, use `{capture}`. If you meant to literally match a segment starting with a colon, call `without_v07_checks` on the router.
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(عددِ داخلِ پرانتز شناسه‌ی ریسمان است و هر بار عوض می‌شود.)

**واقعاً چه چیزی خراب است:** `axum` تا نسخه‌ی ۰٫۷ پارامترِ مسیر را `/:id` می‌نوشت. کد و آموزش‌هایِ زیادی برایِ آن هست. در ۰٫۸ نحو `/{id}` است، و `.route` همین‌که بخشی را با `:` شروع‌شده ببیند پنیک می‌کند. پنیکِ زمانِ اجراست، نه خطایِ کامپایل، چون یک مسیر فقط یک رشته است. خبرِ خوب اینکه موقعِ *ساختنِ* روتر، یعنی هنگامِ راه‌اندازی، رخ می‌دهد، نه در اولین درخواست.

**راه‌حل:** `.route("/anime/{id}", get(anime))`.

**چرا این راه‌حل است:** `{...}` املایِ ۰٫۸ است، و می‌گذارد `:`ِ تحت‌اللفظی هم در مسیر بیاید. به شماره‌ی خطِ پنیک نگاه کن: به فراخوانیِ `.route`ِ خودت اشاره می‌کند، همان خطی که باید درست کنی.

### `E0308` — فراموش‌کردنِ `.with_state(...)`

```text
error[E0308]: mismatched types
   --> phase3-backend-foundations\02-axum-and-rest-api-design\01-routing-handlers-extractors\examples\08-missing-with-state-broken.rs:24:36
    |
 24 |     Router::new().route("/visits", get(visits))
    |                   -----            ^^^^^^^^^^^ expected `MethodRouter`, found `MethodRouter<AppState>`
    |                   |
    |                   arguments to this method are incorrect
    |
    = note: expected struct `MethodRouter<()>`
               found struct `MethodRouter<AppState>`
note: method defined here
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:178:12
    |
178 |     pub fn route(self, path: &str, method_router: MethodRouter<S>) -> Self {
    |            ^^^^^

For more information about this error, try `rustc --explain E0308`.
error: could not compile `p3-02-01-routing-handlers-extractors` (example "08-missing-with-state-broken") due to 1 previous error
```

**کامپایلر به چه ایراد می‌گیرد:** `Router<S>` و `MethodRouter<S>` نوعِ stateای را حمل می‌کنند که هنوز *کم دارند*. `Router::new()` اینجا یک `Router<()>` است که state نمی‌خواهد، ولی `get(visits)` یک `MethodRouter<AppState>` است چون `visits` `State<AppState>` می‌خواهد. این دو با هم نمی‌خوانند. `.with_state(state)` است که `Router<AppState>` را به `Router<()>` تبدیل می‌کند، و `fn app() -> Router` یعنی `Router<()>`.

**راه‌حل:** به روتر state بده:

```rust
Router::new().route("/visits", get(visits)).with_state(AppState::default())
```

**چرا این راه‌حل است:** نوعِ state از هندلرها به روتر جاری می‌شود و `.with_state` مقدارش را می‌دهد، پس دیگر چیزی کم نیست. دقت کن خطا رویِ *آرگومانِ هندلر* گزارش شده، نه رویِ فراخوانیِ جاافتاده؛ این نوعی است که «هنوز چیزی کم دارد»، و راه‌حلش تقریباً همیشه `.with_state` است.

---

## تمرین

### گرم‌کردن

<details>
<summary>هندلری هم <code>Json&lt;T&gt;</code> می‌گیرد هم <code>State&lt;AppState&gt;</code>. به چه ترتیبی باید نوشته شوند، و چرا؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

اول `State`، آخر `Json`. بدنه فقط یک‌بار خوانده می‌شود، پس اکسترکتوری که آن را مصرف می‌کند باید آخرین پارامتر باشد؛ `axum` این را با این شرط اعمال می‌کند که همه‌ی پارامترهایِ قبلی `FromRequestParts` باشند.

</details>

<details>
<summary>کلاینتی <code>{"msg":"hi"}</code> را با <code>Content-Type: application/json</code> به هندلری می‌فرستد که <code>Json&lt;EchoRequest&gt;</code> می‌گیرد و <code>EchoRequest</code> فیلدِ <code>message</code> دارد. چه کدی می‌گیرد، و هندلر اجرا می‌شود؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

`422 Unprocessable Entity`، و هندلر اجرا نمی‌شود. بدنه JSONِ معتبر است، پس `400` نیست؛ شکلش غلط است، یعنی همان حالتِ «درست پارس شد ولی نامعتبر است» از ۳.۱.۳.

</details>

<details>
<summary>شناسه‌ی انیمه را در <code>/anime/7</code> و شماره‌ی صفحه را در <code>/anime?page=2</code> می‌خواهی. برایِ هر کدام کدام اکسترکتور؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

`Path<u32>` برایِ شناسه (منبع را مشخص می‌کند، و مسیر `/anime/{id}` است)، و `Query<...>` برایِ صفحه (فهرست را تنظیم می‌کند و معمولاً اختیاری است).

</details>

<details>
<summary>یک <code>GET</code> به مسیری می‌رسد که فقط <code>post(...)</code> برایش ثبت شده. روتر چه جواب می‌دهد؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

`405 Method Not Allowed` با هدرِ `allow: POST`، بدونِ هیچ کدی از تو. `404` برایِ مسیری است که با هیچ routeای جور نیست.

</details>

### تعمیر

هر سه مثالِ خراب را درست کن (دستورها در «خطاهایی که خواهی دید» است):

۱. `examples/06-body-extractor-not-last-broken.rs` با `--features broken` کامپایل شود.
۲. `examples/07-colon-path-syntax-broken.rs` عبارتِ `router built` را چاپ کند.
۳. `examples/08-missing-with-state-broken.rs` کامپایل شود.

### پیاده‌سازی

پنج تابع در `src/lib.rs`: `greet`، `echo`، `get_counter`، `increment_counter` و `app`.

```sh
cargo test -p p3-02-01-routing-handlers-extractors --test routes_test
```

کامنتِ مستنداتِ هر کدام مشخصاتِ کامل است (مسیرها، متدها، کدهایِ وضعیت، شکلِ JSON)، پس لازم نیست تست‌ها را باز کنی. `hello` به‌عنوانِ نمونه داده شده.

### بساز

`GET /search?q=...&limit=...` را به روتر اضافه کن. در `src/lib.rs` یک هندلرِ `search` بنویس که `Query<T>` بگیرد و برایِ `/search?q=naruto&limit=3` با `200` و JSONِ `{"q":"naruto","limit":3}` جواب بدهد.

مشخصات: `q` اجباری است؛ `limit` اختیاری است و پیش‌فرضش `10` است (`/search?q=naruto` جوابِ `{"q":"naruto","limit":10}` را می‌دهد). `q` decode‌شده برمی‌گردد (`q=one%20piece` می‌شود `"one piece"`). درخواستِ بدونِ `q`، یا با `limit`ی که عدد نیست، پیش از هندلرت با `400` توسطِ اکسترکتور رد می‌شود، پس برایش کدی نمی‌نویسی. مسیر را در `app` ثبت کن و اجرا کن:

```sh
cargo test -p p3-02-01-routing-handlers-extractors --test search_test
```

### چالش (اختیاری)

کاری کن `echo` خطایِ خودش را گزارش کند. پارامترش را به یک `Result` از اکسترکتور و ردکننده‌اش (`axum::extract::rejection::JsonRejection`) تبدیل کن. در حالتِ `Err` با کدِ وضعیتِ خودِ ردکننده (`rejection.status()`) و متنِ `bad echo request` جواب بده؛ در حالتِ `Ok` مثلِ قبل. دو بازو باید یک نوع برگردانند، پس هر دو را با `.into_response()` (از `axum::response::IntoResponse`) تبدیل کن و هندلر را `Response` برگردان. تست‌هایِ فعلی باید همچنان پاس شوند. این یک چالشِ رو‌به‌جلوست: داستانِ کامل تبدیلِ خطایِ *خودت* به پاسخ کارِ ۳.۲.۳ است.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| هندلر | تابعی async که پارامترهایش اکسترکتورند و مقدارِ برگشتی‌اش پاسخ می‌شود | هر route |
| `Router` | جدولی که (متد، مسیر) را به هندلر وصل می‌کند | کلِ API |
| اکسترکتور (extractor) | نوعی که یک تکه را از درخواست بیرون می‌کشد، یا ردش می‌کند | امضایِ هندلرها |
| `Path<T>` / `Query<T>` | بخشِ مسیر / رشته‌ی کوئری، پارس‌شده در `T` | مشخص‌کردن در برابرِ تنظیم‌کردنِ یک منبع |
| `Json<T>` | بدنه‌ای که در `T` پارس می‌شود؛ در خروجی، `T` سریالایز‌شده | بدنه‌یِ درخواست و پاسخ |
| `State<S>` | مقداری که به `.with_state` داده‌ای، به‌ازایِ هر درخواست کلون‌شده | داده‌یِ مشترک مثلِ شمارنده یا pool |
| ردکننده (rejection) | پاسخی که `axum` وقتی اکسترکتور شکست بخورد می‌فرستد: `400`، `415` یا `422` | هر هندلرِ دارایِ اکسترکتور |
| `oneshot` | فرستادنِ یک درخواست از میانِ `Router` بدونِ سوکت | تستِ همه‌ی routeها |

### الان می‌دانی

- هندلر در نوعِ پارامترهایش اعلام می‌کند چه می‌خواهد، و روتر پیش از اجرایِ بدنه اکسترکتورها را از چپ به راست اجرا می‌کند.
- اکسترکتوری که درخواست را رد کند خودش جواب می‌دهد. برایِ `Json`: `400` برایِ نحوِ خراب، `422` برایِ شکلِ غلط، `415` برایِ `Content-Type`ِ غلط. برایِ `Path` و `Query`: `400`.
- اکسترکتورِ بدنه باید آخر باشد، و `E0277` با `Handler<_, _>` شکلِ نمایشِ اشتباه در این مورد است.
- در `axum` ۰٫۸ پارامترِ مسیر `/{id}` است، و `/:id`ِ قدیمی موقعِ راه‌اندازی پنیک می‌کند.
- `axum::serve` رویِ `tokio` اجرا می‌شود و برایِ هر اتصال یک تسک می‌سازد، نه یک ریسمانِ OS.
- می‌توانی کلِ یک روتر را با `oneshot` تست کنی و با `curl` واقعی امتحانش کنی.

### بعداً کامل‌تر می‌بینی

- **نوشتنِ اکسترکتورِ خودت** — [۳.۲.۲ — نوشتنِ اکسترکتورِ خودت](../02-writing-your-own-extractor/README.fa.md)
- **انتخابِ کدِ وضعیت و تبدیلِ خطایِ خودت به پاسخ، رویِ یک منبعِ CRUDِ کامل** — [۳.۲.۳ — کاتالوگِ انیمه با CRUD (داخلِ حافظه)](../03-anime-catalog-crud-in-memory/README.fa.md)
- **`.layer(...)` زیرِ پوست چیست، و `tower::Service`** — [۳.۲.۴ — `tower::Service` و `Layer`](../04-tower-service-and-layer-middleware/README.fa.md)
- **`State`ای که pool دیتابیس را نگه می‌دارد، و پرسشِ قفل در حینِ `.await`** — [۳.۵.۱ — اتصال به دیتابیس و pooling](../../05-postgres-and-sqlx/01-connecting-and-pooling/README.fa.md)
- **یک شکلِ JSONِ یکدست برایِ همه‌ی ردکننده‌ها** — [۳.۸.۱ — پاکت‌هایِ خطایِ یکدست](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.fa.md)

### می‌توانی توضیح بدهی؟

- `axum` برایِ بدنه‌ای که JSON نیست، برایِ JSONِ معتبر با شکلِ غلط، و برایِ `Content-Type`ِ گم‌شده چه جواب می‌دهد، و این چطور به `400` در برابرِ `422`ِ ۳.۱.۳ وصل می‌شود؟
- چرا `Json<T>` باید آخرین پارامتر باشد، و وقتی نباشد خطا چه شکلی دارد؟
- کِی `Path<T>` و کِی `Query<T>`؟
- چرا کلون‌کردنِ `AppState` برایِ هر درخواست یک شمارنده‌ی جدا نمی‌سازد؟
- `axum::serve` به‌ازایِ هر اتصال چه می‌سازد، و این با ۳.۱.۱ چه فرقی دارد؟
- چطور یک تست می‌تواند بدونِ باز کردنِ پورت درخواستی را از کلِ روتر رد کند؟

---

## بیشتر

- [مستنداتِ `axum`: اکسترکتورها](https://docs.rs/axum/0.8.9/axum/extract/index.html) — فهرستِ کامل، قاعده‌یِ ترتیب، و اینکه چطور خودت ردکننده‌ها را مدیریت کنی.
- [مستنداتِ `axum`: خطایِ نوعِ هندلر](https://docs.rs/axum/0.8.9/axum/handler/index.html) — شرطِ `Handler` و `#[debug_handler]`.
- [اعلامِ `axum` ۰٫۸](https://tokio.rs/blog/2025-01-01-announcing-axum-0-8-0) — چرا مسیرها از `/:id` به `/{id}` رفتند.
- [`tower::ServiceExt::oneshot`](https://docs.rs/tower/0.5.3/tower/trait.ServiceExt.html#method.oneshot) — متدی که تست‌ها به‌کار می‌برند.
