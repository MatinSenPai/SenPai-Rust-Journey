# ۳.۲.۳ — عملیاتِ CRUD روی کاتالوگِ انیمه (در حافظه)

## در یک نگاه

بعد از این درس می‌توانی:

- یک منبعِ کاملِ ساخت/خواندن/به‌روزرسانی/حذف در `axum` بسازی: یک ذخیره‌گاهِ ساده که هیچ HTTP‌ای در آن نیست و جداگانه تست می‌شود، و زیرِ آن یک لایه‌ی نازکِ هندلر.
- خطایِ دامنه‌ی خودت را با پیاده‌کردنِ `IntoResponse` به پاسخِ HTTP تبدیل کنی، تا هندلر بتواند `Result<Json<Anime>, AnimeError>` برگرداند و `?` کار کند.
- برایِ هر نتیجه کدِ وضعیتش را (`201`، `204`، `404`، `422`، `400`) انتخاب کنی و با چیزی که ۳.۱.۳ درباره‌ی متدها گفت ازش دفاع کنی.
- وقتی future یک هندلر به‌خاطرِ یک `std::sync::MutexGuard` که از یک `.await` عبور می‌کند `Send` نیست، خطایِ `E0277` را بخوانی و رفعش کنی.

**زمان:** حدود ۱۰۰ دقیقه · **پیش‌نیاز:**
[۳.۲.۱ — مسیریابی، هندلرها، اکسترکتورها](../01-routing-handlers-extractors/README.fa.md)،
[۳.۲.۲ — نوشتنِ اکسترکتورِ خودت (`FromRequestParts`)](../02-writing-your-own-extractor/README.fa.md)،
[۳.۱.۳ — چیزهایی از HTTP که باید بدانی](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.fa.md)

---

## چرا اهمیت دارد

۳.۲.۱ و ۳.۲.۲ قطعه‌ها را دستت دادند: یک `Router`، هندلرها، اکسترکتورها. مسیرهایشان دموهایِ جدا بودند. این درس شکلی است که تقریباً هر REST API در عمل می‌گیرد: یک منبع، پنج عملیات، یک جا که داده را نگه می‌دارد، و برایِ هر نتیجه یک تصمیم درباره‌ی اینکه به کلاینت چه گفته شود.

در DRF بیشترِ این را از `ModelViewSet` می‌گیری: خودش `201` را انتخاب می‌کند، `Http404` را به پاسخ تبدیل می‌کند، و `ValidationError`ِ سریالایزر `400` می‌شود. در `axum` هیچ‌چیز برایت انتخاب نمی‌شود. هندلر یک *چیزی* برمی‌گرداند و تو تصمیم می‌گیری چه. این تصمیم، «کدام کد، کدام بدنه، برایِ کدام شکست»، همان بخشی از API است که کلاینت واقعاً به آن تکیه می‌کند، و در این درس تمرینش می‌کنی. ۳.۱.۳ کدها را فهرست کرد و گفت «از ماژولِ بعد به بعد هنوز خودت انتخابشان می‌کنی». اینجا همان‌جاست.

ذخیره‌گاه عمداً در حافظه است و با بسته‌شدنِ پردازش از بین می‌رود. [۳.۵.۳ — کاتالوگ انیمه، این‌بار متصل به Postgres](../../05-postgres-and-sqlx/03-anime-catalog-postgres-backed/README.fa.md) همین منبع را رویِ دیتابیس از نو می‌سازد، تا ببینی کدام بخش‌هایِ یک CRUD API طراحیِ HTTP است (این درس) و کدام ماندگاری (آن درس).

---

## مفهوم

### یک منبع، پنج عملیات، یک جدولِ مسیر

| DRF (`ModelViewSet` رویِ `Anime`) | این درس | موفقیت |
|---|---|---|
| `GET /anime/` → `list()` | `GET /anime` → `list_anime` | `200` + آرایه‌ی JSON |
| `POST /anime/` → `create()` | `POST /anime` → `create_anime` | `201` + `Location` + JSON |
| `GET /anime/{id}/` → `retrieve()` | `GET /anime/{id}` → `get_anime` | `200` + JSON |
| `PATCH /anime/{id}/` → `partial_update()` | `PATCH /anime/{id}` → `update_anime` | `200` + JSON |
| `DELETE /anime/{id}/` → `destroy()` | `DELETE /anime/{id}` → `delete_anime` | `204`، بدنه‌ی خالی |
| `Anime.objects` | `AnimeStore` | |

تشبیه کجا می‌شکند: `ModelViewSet` این جدول را از یک کلاس تولید می‌کند. اینجا جدول، پنج هندلر، و ذخیره‌گاه را خودت دستی می‌نویسی، پس هر کدِ وضعیتِ داخلش را خودت انتخاب کرده‌ای.

### ذخیره‌گاهِ خالص، لبه‌ی نازکِ HTTP

دو لایه، عمداً جدا از هم:

```senpai-visual
{"kind":"result","labels":["درخواست","هندلرِ نازک","ذخیره‌گاه: Arc و Mutex","Result از Anime یا AnimeError","IntoResponse","کدِ وضعیت و JSON"]}
```

`AnimeStore` ذخیره‌گاه (store) است و Rustِ ساده. `axum` را import نمی‌کند، هرگز `StatusCode` نمی‌بیند، و متدهایش `Result<Anime, AnimeError>` برمی‌گردانند. هندلر سه کارِ کوچک دارد: درخواست را با اکسترکتورها باز می‌کند، یک متدِ ذخیره‌گاه را صدا می‌زند، و `Result` را به `axum` پس می‌دهد. چون قاعده‌ها («رتبه `1..=10` است»، «شناسه‌ها از ۱ شروع می‌شوند»، «لیست بر اساسِ شناسه مرتب است») در ذخیره‌گاه‌اند، `tests/store_test.rs` با صدازدنِ ساده‌ی تابع بررسی‌شان می‌کند: بدونِ runtime، بدونِ درخواست. `tests/api_test.rs` فقط لبه را از راهِ `oneshot` تست می‌کند، همان‌طور که ۳.۲.۱ کرد.

### به اشتراک‌گذاشتنِ یک ذخیره‌گاه: `Arc` و `Mutex`

هر درخواست یک هندلر اجرا می‌کند، شاید رویِ ریسمانی دیگر، و همه باید یک کاتالوگ را ببینند. پس `main` یک `Arc<AnimeStore>` می‌سازد و از راهِ `State` به هر هندلر یک کلون می‌دهد:

```rust
#[derive(Default)]
struct StoreInner {
    next_id: u64,
    items: HashMap<u64, Anime>,
}

#[derive(Default)]
pub struct AnimeStore {
    inner: Mutex<StoreInner>,
}
```

متدهایِ ذخیره‌گاه `&self` می‌گیرند، نه `&mut self`: چند هندلر هم‌زمان یک ارجاعِ اشتراکی دارند. تغییر پشتِ آن اتفاق می‌افتد، یعنی **تغییرپذیریِ درونی (interior mutability)** (۲.۶.۵ `RefCell` را نشان داد؛ `Mutex` نسخه‌ی ریسمان‌امنِ آن است، از ۲.۸.۱). شمارنده و نقشه عمداً *یک* `Mutex` مشترک دارند. با دو قفل، دو create هم‌زمان می‌توانستند هر دو یک `next_id` را بخوانند و درج دومی اولی را رونویسی می‌کرد.

هر متدِ ذخیره‌گاه قفل می‌کند، کار می‌کند و برمی‌گردد؛ گارد (`MutexGuard`ِ ۲.۸.۱) با برگشتنِ متد drop می‌شود. ذخیره‌گاه همگام (synchronous) است، پس هیچ گاردی نمی‌تواند از یک `.await` عبور کند. «خطاهایی که خواهی دید» نشان می‌دهد وقتی عبور کند چه می‌شود.

### یک مسیر، چند متد

`.route(path, ...)` یک `MethodRouter` می‌گیرد، و `get(handler)` یکی برمی‌گرداند که می‌توانی زنجیره‌اش کنی. یک مسیر، چند متد، یک فراخوانیِ `.route`:

```rust
Router::new()
    .route("/ping", get(ping).post(pong))
    .route("/ping/{id}", get(ping_one).delete(drop_one))
    .with_state(state)
```

متدی که کسی روی آن مسیر ثبت نکرده، خودش `405 Method Not Allowed` با هدرِ `Allow` جواب می‌دهد. پارامترِ مسیر در `axum` نسخه‌ی ۰٫۸ به شکلِ `/{id}` است؛ `/:id`ِ قدیمی هنگامِ ساختنِ روتر پنیک می‌کند.

### نوعِ خطایِ خودت به‌عنوانِ پاسخ: `IntoResponse`

۳.۲.۲ اجازه داد یک رد (rejection) فقط `(StatusCode, &'static str)` باشد، چون `axum` از قبل می‌داند آن توپل را چطور به پاسخ تبدیل کند. برایِ نوعِ خطایِ خودت، خودت به `axum` یاد می‌دهی. صفتش `IntoResponse` است و یک متد دارد:

```rust
impl IntoResponse for CardError {
    fn into_response(self) -> Response {
        match self {
            CardError::Declined => (StatusCode::PAYMENT_REQUIRED, "card declined"),
            CardError::Expired => (StatusCode::UNPROCESSABLE_ENTITY, "card expired"),
        }
        .into_response()
    }
}
```

بخوانش به‌صورتِ «یک `CardError` رویِ سیم چه شکلی است؟». هر دو بازو توپلِ `(StatusCode, &'static str)` می‌سازند، و آن توپل خودش می‌داند چطور پاسخ شود، پس آخرش متد را رویش صدا می‌زنی. بدنه می‌تواند هر چیزی باشد که خودش `IntoResponse` است: `Json(...)`، یک `String`، توپلی دیگر.

وقتی نوعِ خطایت این impl را داشت، `axum` یک قاعده برایِ نوعِ نتیجه دارد: `Result<T, E>` هر وقت هم `T` و هم `E` آن را داشته باشند `IntoResponse` است. حالا هندلر می‌تواند `Result<Json<Anime>, AnimeError>` برگرداند، و `?` رویِ یک فراخوانیِ ذخیره‌گاه، `Err(AnimeError)`ِ ذخیره‌گاه را به `Err`ِ هندلر تبدیل می‌کند. `axum` رویِ هر طرفی که برگردد `.into_response()` را صدا می‌زند. هندلر هیچ `match`ی ندارد. `examples/01-what-a-handler-return-becomes.rs` همین را با دست رویِ سه شکلی که هندلرهایت برمی‌گردانند صدا می‌زند:

```text
--- Json(anime)
HTTP/1.1 200 OK
content-type: application/json

{"id":1,"title":"Frieren"}
--- (CREATED, [(LOCATION, ..)], Json(anime))
HTTP/1.1 201 Created
content-type: application/json
location: /anime/1

{"id":1,"title":"Frieren"}
--- StatusCode::NO_CONTENT
HTTP/1.1 204 No Content


```

یک `Json`ِ تنها `200` است. توپلی که با `StatusCode` شروع شود آن را عوض می‌کند، و آرایه‌ی هدر در وسط هدر اضافه می‌کند. یک `StatusCode`ِ تنها پاسخی با بدنه‌ی خالی است.

چرا اصلاً یک خطایِ دامنه تعریف کنیم، به‌جایِ اینکه ذخیره‌گاه `(StatusCode, String)` برگرداند؟ چون ذخیره‌گاه نباید چیزی از HTTP بداند. `AnimeError::NotFound` برایِ نسخه‌ی Postgres، یک CLI و یک تست هم درست است؛ «404» فقط در لبه درست است. این یک `impl` تنها جایی است که این دو واژگان به هم می‌رسند. (جایگاهش در DRF همان `exception_handler`ِ جنگوست. یک بار دستی ساختنش راهِ دیدنِ این است که شکلِ JSONِ خطا از کجا می‌آید. ۳.۸.۱ آن شکل را برایِ همه‌ی خطاهایِ یک API یکی می‌کند.)

### کدام شکست کدام کد را می‌گیرد

درس دو شکستِ دامنه دارد و `axum` چند تای خودش را هم پیش از رسیدنِ هندلر اضافه می‌کند:

```senpai-visual
{"kind":"concept","labels":["404: درخواستِ سالم، ولی این شناسه وجود ندارد","422: JSONِ درست، ولی رتبه قاعده را می‌شکند","400: اصلاً JSON نیست، یا شناسه عدد نیست","415: Content-Type: application/json ندارد"]}
```

- **`NotFound` → `404`.** درخواست سالم بود. منبعی که اسم می‌برد وجود ندارد.
- **`InvalidRating(r)` → `422`.** قاعده‌ی ۳.۱.۳: `400` یعنی خودِ درخواست بدشکل است؛ `422` یعنی درست پارس شد ولی *محتوا* نامعتبر است. رتبه‌ی `15` یک عددِ کاملاً درست است و JSON هم JSONِ کاملاً درست. فقط یک قاعده‌ی کسب‌وکار را می‌شکند، پس `422`.
- **`400` و `422` و `415` از اکسترکتورهایِ `axum`** پیش از اجرایِ کدت: JSONِ خراب `400` است؛ JSONی که درست است ولی در ساختار نمی‌گنجد (`title` ندارد) `422`؛ نبودنِ `Content-Type: application/json` `415`؛ و `{id}`ی که `u64` نیست `400`. هر چهار را در «دست‌به‌کد» می‌بینی.

### `201`، `Location`، `PATCH` در برابرِ `PUT`، و `DELETE` دو بار

- **`POST` خودتوان نیست، پس پاسخ `201 Created` است**، نه `200`. جدولِ ۳.۱.۳ `POST` را نه ایمن و نه خودتوان می‌داند: دو بار بفرستی دو انیمه می‌گیری. هدرِ `Location` (`Location: /anime/{id}`) به کلاینت می‌گوید منبعِ تازه کجاست، و `full_crud_lifecycle` آن را چک می‌کند.
- **`PATCH` برایِ به‌روزرسانیِ جزئی است**: فقط فیلدهایی را که می‌خواهی عوض شوند بفرست، بقیه می‌مانند (`UpdateAnime`، هر فیلد اختیاری). **`PUT` کلِ بازنمایی را جایگزین می‌کند**: همه‌ی فیلدها را بفرست، و هر چه نفرستی ریست می‌شود. جدولِ ۳.۱.۳ `PUT` را میانِ متدهایِ خودتوان می‌گذارد (همان `PUT` دو بار، همان حالت) و `PATCH` را کنارِ `POST`، دو متدی که چنین قولی ندارند. دلیلش این است که یک patch *می‌تواند* بگوید «۱ به رتبه اضافه کن»، و تکرارش دوباره چیزی را عوض می‌کند. patchِ این API فقط فیلد را مقدار می‌دهد، پس عملاً خودتوان است، ولی قرارداد قولش را نمی‌دهد، پس منطقِ retryِ کلاینت نباید روی آن حساب کند. «بساز» یک `PUT` اضافه می‌کند تا فرق را حس کنی.
- **`DELETE` خودتوان است، با اینکه دومی `404` جواب می‌دهد.** بعد از `DELETE`ِ اول، انیمه‌ی ۱ رفته است. بعد از دومی هم رفته است. حالتِ سرور همان است، و خودتوانی همین را می‌سنجد. *پاسخ* می‌تواند فرق کند. `204` و بعد `404` درست است، و تستِ `deleting_twice_answers_differently_but_leaves_the_same_state` هر دو نیمه را چک می‌کند.

---

## دست‌به‌کد

سه مثالی را که کامپایل می‌شوند اجرا کن. (`02`، `03` و `04` عمداً خراب‌اند و پشتِ فیچرِ `broken` هستند؛ «خطاهایی که خواهی دید» نشانشان می‌دهد. `05` معمولی اجرا می‌شود و فقط غلط است.)

```sh
cargo run -p p3-02-03-anime-catalog-crud-in-memory --example 01-what-a-handler-return-becomes
cargo run -p p3-02-03-anime-catalog-crud-in-memory --example 06-guard-dropped-before-await-fix
cargo run -p p3-02-03-anime-catalog-crud-in-memory --example 05-created-answers-200-trap
```

```text
GET /count -> 1
GET /count -> 2
POST /anime -> 200 OK  (should be 201 Created)
```

(خروجیِ دستورِ اول همان بلوکِ «مفهوم» است؛ این‌ها خروجیِ دوتایِ دیگرند.)

تست‌ها قرمز شروع می‌شوند. `src/lib.rs` کلِ اسکلت را دارد و هر تابع یک `todo!()` با یک کامنتِ مستندات است که دقیقاً می‌گوید چه باید بکند. از ذخیره‌گاه شروع کن:

```sh
cargo test -p p3-02-03-anime-catalog-crud-in-memory --test store_test 2>&1 | grep 'test result'
```

```text
test result: FAILED. 0 passed; 15 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

وقتی همه‌چیز سبز شد، سرورِ واقعی را اجرا کن (رویِ `127.0.0.1:3001` گوش می‌دهد) و با `curl` با آن حرف بزن. این خروجی‌ها رویِ solution گرفته شده‌اند:

```sh
cargo run -p p3-02-03-anime-catalog-crud-in-memory &
curl -si -X POST -H 'content-type: application/json' -d '{"title":"Frieren","status":"watching","rating":9}' http://127.0.0.1:3001/anime | head -n 3
curl -s -X POST -H 'content-type: application/json' -d '{"title":"Dandadan","status":"plan_to_watch"}' http://127.0.0.1:3001/anime
curl -s http://127.0.0.1:3001/anime
```

```text
HTTP/1.1 201 Created
content-type: application/json
location: /anime/1
{"id":2,"title":"Dandadan","status":"plan_to_watch","rating":null}[{"id":1,"title":"Frieren","status":"watching","rating":9},{"id":2,"title":"Dandadan","status":"plan_to_watch","rating":null}]
```

(`curl -s` بدنه را بدونِ خطِ جدیدِ آخر چاپ می‌کند، پس خروجیِ بعدی درست بعدِ آن شروع می‌شود.) حالا شکست‌ها، هرکدام با کدِ وضعیتش در خطِ آخر:

```sh
curl -s -w '\n%{http_code}\n' -X POST -H 'content-type: application/json' -d '{"title":"Bad","status":"watching","rating":15}' http://127.0.0.1:3001/anime
curl -s -w '\n%{http_code}\n' -X POST -H 'content-type: application/json' -d '{' http://127.0.0.1:3001/anime
curl -s -w '\n%{http_code}\n' -X POST -H 'content-type: application/json' -d '{"status":"watching"}' http://127.0.0.1:3001/anime
curl -s -w '\n%{http_code}\n' -X POST -d '{"title":"x","status":"watching"}' http://127.0.0.1:3001/anime
curl -s -w '\n%{http_code}\n' http://127.0.0.1:3001/anime/abc
```

```text
{"error":"rating must be between 1 and 10, got 15"}
422
Failed to parse the request body as JSON: EOF while parsing an object at line 1 column 1
400
Failed to deserialize the JSON body into the target type: missing field `title` at line 1 column 21
422
Expected request with `Content-Type: application/json`
415
Invalid URL: Cannot parse `abc` to a `u64`
400
```

خطِ اول *`AnimeError`ِ خودت* است از راهِ *`IntoResponse`ِ خودت*. چهارتایِ دیگر ردهایِ خودِ `axum` هستند: متنِ ساده، نه شکلِ JSONِ تو. ۳.۸.۱ جایی است که این ناهمخوانی درست می‌شود. در آخر `DELETE` دو بار، با لیست بعد از هر کدام:

```sh
curl -s -o /dev/null -w '%{http_code}\n' -X DELETE http://127.0.0.1:3001/anime/1
curl -s -w '\n%{http_code}\n' -X DELETE http://127.0.0.1:3001/anime/1
curl -s http://127.0.0.1:3001/anime
```

```text
204
{"error":"anime not found"}
404
[{"id":2,"title":"Dandadan","status":"plan_to_watch","rating":null}]
```

وقتی کارت تمام شد سرور را متوقف کن (`kill %1` در همان شل). بعد این‌ها را امتحان کن:

۱. دو انیمه بساز، اولی را حذف کن، سومی را بساز. چه شناسه‌ای می‌گیرد، و چرا نمی‌تواند دوباره `1` باشد؟
۲. `PATCH /anime/2` را با `{"rating": null}` بفرست. رتبه پاک می‌شود؟ (کامنتِ مستنداتِ `UpdateAnime` را بخوان.)
۳. در `examples/05-created-answers-200-trap.rs` هندلر را طوری عوض کن که `201` بدهد. کدام `use` لازم داری؟

---

## خطاهایی که خواهی دید

### `E0277` — یک `MutexGuard` که از `.await` عبور می‌کند

این همان است که در سرویس‌هایِ واقعی بهش می‌خوری. یک هندلر قفل می‌کند، کارِ async می‌کند، و هنوز گارد را دارد:

```rust
async fn count(State(counter): State<Arc<Mutex<u32>>>) -> String {
    let mut guard = counter.lock().unwrap();
    *guard += 1;
    audit_log().await;
    format!("{}", *guard)
}
```

`examples/02-guard-across-await-broken.rs` آن را با `get(count)` ثبت می‌کند:

```text
error[E0277]: the trait bound `fn(State<Arc<std::sync::Mutex<u32>>>) -> impl Future<Output = String> {count}: Handler<_, _>` is not satisfied
   --> phase3-backend-foundations\02-axum-and-rest-api-design\03-anime-catalog-crud-in-memory\examples\02-guard-across-await-broken.rs:26:30
    |
 26 |         .route("/count", get(count))
    |                          --- ^^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(State<Arc<std::sync::Mutex<u32>>>) -> impl Future<Output = String> {count}`
    |                          |
    |                          required by a bound introduced by this call
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
error: could not compile `p3-02-03-anime-catalog-crud-in-memory` (example "02-guard-across-await-broken") due to 1 previous error
```

**کامپایلر به چه ایراد می‌گیرد:** هیچ‌جایِ این پیام `Send` یا `MutexGuard` نیست. فقط می‌گوید `count` یک `Handler` نیست، و تنها سرنخش این است که `#[axum::debug_handler]` را امتحان کنی. این خطا همیشه همین شکل را دارد، و برایِ همین این‌قدر گیج‌کننده است. دلیلش در صفتِ `Handler` است: futureای که هندلرت برمی‌گرداند باید `Send` باشد. یک runtimeِ چندریسمانی ممکن است بعد از هر `.await` تسک را رویِ ریسمانِ کارگرِ دیگری بردارد، پس هر چیزی که از یک `.await` زنده عبور می‌کند باید بین ریسمان‌ها قابلِ انتقال باشد، که همان `Send`ِ ۲.۸.۴ است. `std::sync::MutexGuard` `Send` نیست.

اتریبیوت را رویِ هندلر بگذار (فیچرِ `macros`ِ `axum` را می‌خواهد، که `Cargo.toml`ِ این درس روشنش کرده) و کامپایلر می‌گوید واقعاً چه شده. این `examples/04-guard-across-await-debug-handler-broken.rs` است:

```text
error: future cannot be sent between threads safely
  --> phase3-backend-foundations\02-axum-and-rest-api-design\03-anime-catalog-crud-in-memory\examples\04-guard-across-await-debug-handler-broken.rs:17:1
   |
17 | #[axum::debug_handler]
   | ^^^^^^^^^^^^^^^^^^^^^^ future returned by `count` is not `Send`
   |
   = help: within `impl Future<Output = String>`, the trait `Send` is not implemented for `std::sync::MutexGuard<'_, u32>`
note: future is not `Send` as this value is used across an await
  --> phase3-backend-foundations\02-axum-and-rest-api-design\03-anime-catalog-crud-in-memory\examples\04-guard-across-await-debug-handler-broken.rs:21:17
   |
19 |     let mut guard = counter.lock().unwrap();
   |         --------- has type `std::sync::MutexGuard<'_, u32>` which is not `Send`
20 |     *guard += 1;
21 |     audit_log().await;
   |                 ^^^^^ await occurs here, with `mut guard` maybe used later
note: required by a bound in `__axum_macros_check_count_future::check`
  --> phase3-backend-foundations\02-axum-and-rest-api-design\03-anime-catalog-crud-in-memory\examples\04-guard-across-await-debug-handler-broken.rs:17:1
   |
17 | #[axum::debug_handler]
   | ^^^^^^^^^^^^^^^^^^^^^^ required by this bound in `check`
   = note: this error originates in the attribute macro `axum::debug_handler` (in Nightly builds, run with -Z macro-backtrace for more info)
```

(خروجیِ دستور ادامه دارد: همان `E0277`ِ قبلی بار دوم در فراخوانیِ `.route(...)` چاپ می‌شود و با `could not compile ... due to 2 previous errors` تمام می‌شود.)

حالا روشن است: `guard` نوعی دارد که `Send` نیست، و یک `.await` در حالی رخ می‌دهد که `guard` زنده است.

**راه‌حل:** مطمئن شو گارد پیش از `.await` رفته. کوچک‌ترین راه یک بلوک است که عمرِ قفل را تمام می‌کند:

```rust
let now = {
    let mut guard = counter.lock().unwrap();
    *guard += 1;
    *guard
}; // the guard is dropped here
audit_log().await;
format!("{now}")
```

`examples/06-guard-dropped-before-await-fix.rs` همان هندلر با همین تغییر است و `GET /count -> 1` و بعد `GET /count -> 2` چاپ می‌کند.

**چرا این راه‌حل است:** dropِ گارد (۲.۸.۱) قفل را آزاد می‌کند، و پایانِ بلوک همان زمانی است که drop می‌شود. وقتی گاردی در لحظه‌ی `.await` زنده نیست، future فقط چیزهایِ `Send` نگه می‌دارد و کرانِ `Handler` برقرار است. دو راهِ دیگر هم هست. حالتِ مشترک را پشتِ یک ذخیره‌گاهِ همگام نگه دار، که شکلِ همین درس است، تا قفل هرگز به `.await` نرسد؛ یا، برایِ موردِ نادری که باید قفل را از `.await` عبور بدهی، `Mutex`ِ async خودِ `tokio` (`tokio::sync::Mutex`) را بگیر که گاردش `Send` است. نگه‌داشتنِ گاردِ `std` از یک `.await` حتی وقتی کامپایل می‌شود بد است (با runtimeِ تک‌ریسمانی می‌تواند deadlock شود: تسکِ دوم منتظرِ قفلی می‌ماند که همان ریسمانِ خودش نگهش داشته).

### `E0277` — نوعِ خطایی که `IntoResponse` نیست

```text
error[E0277]: the trait bound `fn() -> impl Future<Output = Result<&'static str, ShowError>> {show}: Handler<_, _>` is not satisfied
   --> phase3-backend-foundations\02-axum-and-rest-api-design\03-anime-catalog-crud-in-memory\examples\03-error-without-into-response-broken.rs:20:57
    |
 20 |     let _app: Router = Router::new().route("/show", get(show));
    |                                                     --- ^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn() -> impl Future<Output = Result<&'static str, ShowError>> {show}`
    |                                                     |
    |                                                     required by a bound introduced by this call
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
error: could not compile `p3-02-03-anime-catalog-crud-in-memory` (example "03-error-without-into-response-broken") due to 1 previous error
```

**کامپایلر به چه ایراد می‌گیرد:** همان عبارت، به دلیلی دیگر. نوعِ بازگشتیِ هندلر باید `IntoResponse` باشد، و `Result<&str, ShowError>` فقط وقتی این است که `ShowError` هم باشد. `ShowError` یک enumِ ساده است که `axum` چیزی از آن نمی‌داند. خطا اسمِ نوعِ مقصر را نمی‌آورد، و `#[axum::debug_handler]` به آن اشاره می‌کرد.

**راه‌حل:** صفت را برایِ نوعِ خطایت پیاده کن:

```rust
impl IntoResponse for ShowError {
    fn into_response(self) -> Response {
        (StatusCode::NOT_FOUND, "no such show").into_response()
    }
}
```

**چرا این راه‌حل است:** `Result<T, E>` یک impl از `IntoResponse` دارد که هر دو نیمه را لازم دارد. نیمه‌ی گم‌شده را تو می‌دهی، و همان یک impl جایی است که برایِ هر واریانت کدِ وضعیت و بدنه را انتخاب می‌کنی.

### هیچ خطایی نیست: `200` جایی که `201` باید باشد

```text
POST /anime -> 200 OK  (should be 201 Created)
```

**واقعاً چه چیزی خراب است:** `examples/05-created-answers-200-trap.rs` کامپایل می‌شود، اجرا می‌شود، و با بدنه‌ی درست جواب می‌دهد. هندلرش یک `Json(...)`ِ تنها برمی‌گرداند، و `Json`ِ تنها `200` است. کلاینت‌هایی که از رویِ کدِ وضعیت تصمیم می‌گیرند (سیاستِ retry، کش، یک API gateway، یک تست در مخزنِ دیگران) برایِ درخواستی که همین الان چیزی ساخته سیگنالِ غلط می‌گیرند.

**راه‌حل:** توپلی برگردان که با کدِ وضعیت شروع شود:

```rust
Ok((StatusCode::CREATED, Json(anime)))
```

**چرا این راه‌حل است:** عنصرِ اولِ توپل `200`ِ پیش‌فرض را عوض می‌کند. کامپایلر کدِ وضعیتِ غلط‌اما‌معتبر را نمی‌گیرد، و فقط تستی که کدِ وضعیت را چک کند می‌گیردش. برایِ همین هر تست در `tests/api_test.rs` پیش از بدنه `response.status()` را چک می‌کند.

---

## تمرین

### گرم‌کردن

<details>
<summary>کلاینت یک <code>POST /anime</code>ِ معتبر می‌فرستد. کدام کدِ وضعیت، و کدام هدر باید همراهش باشد؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

`201 Created`، به‌علاوه‌ی `Location: /anime/{id}` که به منبعِ تازه اشاره می‌کند. `POST` خودتوان نیست (۳.۱.۳): هر بار یک انیمه‌ی دیگر می‌سازد، پس `200` («این چیزی است که از قبل بود») پیامِ غلطی است.

</details>

<details>
<summary><code>DELETE /anime/1</code>ِ دوم <code>404</code> جواب می‌دهد. <code>DELETE</code> هنوز خودتوان است؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

بله. خودتوانی درباره‌ی *حالتِ* سرور است، نه پاسخ. بعد از یک `DELETE` یا پنج‌تا، انیمه‌ی ۱ رفته و کاتالوگ همان است. پاسخ می‌تواند فرق کند (`204`، بعد `404`) بدونِ اینکه قول بشکند، که همان نکته‌ای است که ۳.۱.۳ گفت.

</details>

<details>
<summary>یک <code>POST</code> با <code>"rating": 15</code> در JSONِ درست می‌رسد. <code>400</code> یا <code>422</code>؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

`422`. درخواست بدشکل نیست: JSON پارس می‌شود و هر فیلد نوعِ درست دارد. *محتوا* یک قاعده را می‌شکند. `400` برایِ درخواستی است که اصلاً فهمیده نمی‌شود (JSONِ خراب، `{id}`ی که عدد نیست).

</details>

<details>
<summary>چرا <code>get_anime</code> می‌تواند <code>Result&lt;Json&lt;Anime&gt;, AnimeError&gt;</code> برگرداند؟ چه دو چیز باید درست باشد؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

`Json<Anime>` `IntoResponse` را دارد، و `AnimeError` هم دارد چون خودت پیاده‌اش کردی. `axum` برایِ `Result<T, E>` دقیقاً وقتی `IntoResponse` را پیاده کرده که هر دو داشته باشند، و رویِ هر کدام که برگردد `.into_response()` را صدا می‌زند.

</details>

<details>
<summary>آیا اسپک می‌گوید <code>PATCH</code> خودتوان است؟ و <code>PUT</code>؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

`PUT` بله، `PATCH` نه. `PUT` کلِ بازنماییِ جدید را می‌گوید، پس تکرارش چیزِ بیشتری را عوض نمی‌کند. یک patch می‌تواند یک *تغییر* را توصیف کند («۱ اضافه کن»)، پس اسپک قول نمی‌دهد تکرارش بی‌ضرر است، حتی اگر patchِ یک APIِ خاص (مثلِ این یکی) فقط فیلد را مقدار بدهد.

</details>

### تعمیر

هر چهار مثالِ خراب را درست کن:

۱. `examples/02-guard-across-await-broken.rs` کامپایل شود: future را `Send` کن. برایِ چک کردن با `--features broken` بیلدش کن.
۲. `examples/04-guard-across-await-debug-handler-broken.rs` کامپایل شود. همان باگ است با خطایِ روشن‌تر. فقط اتریبیوت را پاک نکن.
۳. `examples/03-error-without-into-response-broken.rs` کامپایل شود، با پاسخِ `NOT_FOUND` برایِ `ShowError::NotFound`.
۴. `examples/05-created-answers-200-trap.rs` عبارتِ `201 Created` را چاپ کند.

### پیاده‌سازی

همه‌ی `todo!()`هایِ `src/lib.rs`: `into_response`ِ `AnimeError`، پنج متدِ ذخیره‌گاه، پنج هندلر، و `app`. هر کامنتِ مستندات مشخصاتِ کاملِ خودش است (کدهایِ وضعیت، شکلِ JSON، ترتیب، قاعده‌ی اعتبارسنجی)، پس هیچ‌وقت لازم نیست برایِ فهمیدنِ اینکه چه بسازی تست‌ها را باز کنی. از پایینِ پشته به بالا کار کن: اول ذخیره‌گاه، بعد خطا، بعد هندلرها و جدولِ مسیر.

```sh
cargo test -p p3-02-03-anime-catalog-crud-in-memory
```

`tests/store_test.rs` (۱۵ تست) ذخیره‌گاه را با فراخوانیِ ساده چک می‌کند. `tests/api_test.rs` (۷ تست) کلِ پشته را از راهِ `oneshot` چک می‌کند.

### بساز

`PUT /anime/{id}` را اضافه کن، یک جایگزینیِ کامل. بدنه‌اش همان شکلِ `POST` (`CreateAnime`) است، با `200` و انیمه‌ی جدید جواب می‌دهد، شناسه را نگه می‌دارد، و برایِ شناسه‌ای که وجود ندارد `404` می‌دهد (هرگز نمی‌سازد؛ دادنِ شناسه با سرور است). رتبه‌ی خارج از بازه مثلِ همه‌جا `422` است. آن را به‌صورتِ `AnimeStore::replace` به‌علاوه‌ی یک هندلر اضافه کن، بعد `.put(...)` را به مسیرِ `/anime/{id}` زنجیر کن. تست‌هایِ خودت را در یک `tests/put_test.rs`ِ تازه بنویس. یکی‌شان همان `PUT`ِ یکسان را دو بار بفرستد و ادعا کند دو پاسخ و حالتِ نهایی یکی‌اند: این خودتوانی است، به‌صورتِ تست.

### چالش (اختیاری)

هم‌روندیِ خوش‌بینانه (optimistic concurrency)، کوچک نگه‌داشته. دو کلاینت از نسخه‌هایِ کهنه یک انیمه را `PATCH` می‌کنند، و دومی بی‌صدا اولی را رونویسی می‌کند. به هر `Anime` یک `version: u64` بده که از `1` شروع شود و با هر تغییرِ موفق یکی زیاد شود. `PATCH` را مجبور کن هدرِ `If-Match: <version>` بخواهد. ناهمخوانی `412 Precondition Failed` جواب بدهد (یک واریانتِ تازه در `AnimeError` و یک بازوی تازه در `IntoResponse`ِ تو). این تنها جایی است که درس به جلو دست می‌برد: درخواست‌هایِ شرطیِ HTTP و `ETag` مکانیزمِ واقعی‌اند، و در دیتابیس همین ایده به شکلِ یک بندِ `WHERE version = $2` درمی‌آید. تستِ آماده‌ای نیست: برایِ حالتِ مطابق، کهنه، و نبودنِ هدر تست‌هایِ خودت را بنویس.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| خطایِ دامنه (domain error) | خطا در واژگانِ خودِ مسئله (`NotFound`)، بدونِ هیچ HTTP | هر ذخیره‌گاه، سرویس و repository |
| `IntoResponse` | صفتی که می‌گوید یک مقدار چطور پاسخِ HTTP می‌شود | نوعِ بازگشتیِ هر هندلر، هر نوعِ خطا |
| تغییرپذیریِ درونی پشتِ `State` | یک `Arc<Store>` که همه‌ی هندلرها شریکند و از راهِ `Mutex`ِ داخلش تغییر می‌کند | حالتِ مشترکِ برنامه، کش، شمارنده |
| زنجیرکردنِ متد رویِ یک مسیر | `get(a).post(b)` چند متد را رویِ یک مسیر ثبت می‌کند | هر منبعی که بیش از یک فعل دارد |
| `#[axum::debug_handler]` | اتریبیوتی که `E0277`ِ هندلر را وادار می‌کند بگوید واقعاً چه خراب است | هر وقت «این یک Handler نیست» می‌بینی |

### الان می‌دانی

- ذخیره‌گاه Rustِ ساده‌ای است که `Result<_, YourError>` برمی‌گرداند. هندلرها لایه‌ی نازکی‌اند که آن را به HTTP تبدیل می‌کند، و هر نیمه جداگانه تست می‌شود.
- `impl IntoResponse for YourError` تنها جایی است که یک شکستِ دامنه به کدِ وضعیت و بدنه تبدیل می‌شود، و همان است که `Result<_, YourError>` را یک بازگشتِ مجازِ هندلر می‌کند.
- کدام کد برایِ کدام نتیجه: `201` + `Location` برایِ `POST`، `204` برایِ `DELETE`، `404` برایِ شناسه‌ی گم، `422` برایِ محتوایِ درست‌اما‌نامعتبر، `400` برایِ درخواستِ بدشکل.
- `PATCH` به‌روزرسانیِ جزئی بدونِ قولِ خودتوانی است، `PUT` جایگزینیِ کامل و خودتوان است، و `DELETE` خودتوان است با اینکه پاسخِ دومش `404` است.
- futureِ یک هندلر باید `Send` باشد. یک `MutexGuard`ِ `std` که از `.await` عبور کند این را می‌شکند، و راه‌حل این است که گارد را پیش از آن drop کنی.

### بعداً کامل‌تر می‌بینی

- **جایگزین‌کردنِ `HashMap` با دیتابیسِ واقعی**: [۳.۵.۳ — کاتالوگ انیمه، این‌بار متصل به Postgres](../../05-postgres-and-sqlx/03-anime-catalog-postgres-backed/README.fa.md)
- **یک قالبِ خطایِ یکدست برایِ همه‌ی شکست‌ها، از جمله ردهایِ متنِ‌ساده‌ای که امروز دیدی**: [۳.۸.۱ — پاکت‌هایِ خطایِ یکدست](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.fa.md)
- **یک لایه‌ی اعتبارسنجیِ واقعی به‌جایِ یک قاعده‌ی دستی**: [۳.۳.۲ — اعتبارسنجی](../../03-serialization-and-validation/02-validation/README.fa.md)
- **جایگاهِ یک ذخیره‌گاهِ مشترکِ مثلِ این در یک برنامه‌ی بزرگ‌تر**: [۳.۴.۲ — وضعیتِ اپلیکیشن و سیم‌کشیِ وابستگی‌ها](../../04-configuration-and-app-structure/02-app-state-and-dependency-wiring/README.fa.md)
- **`.layer(...)` دورِ هندلرهایی مثلِ این چه می‌کند**: [۳.۲.۴ — `tower::Service` و `Layer`: میان‌افزار با دست](../04-tower-service-and-layer-middleware/README.fa.md)
- **صفحه‌صفحه‌کردنِ یک لیستِ بلند به‌جایِ برگرداندنِ همه‌اش**: [۳.۶.۲ — صفحه‌بندی: offset در برابرِ keyset](../../06-database-design-and-query-performance/02-pagination/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا ذخیره‌گاه `AnimeError` برمی‌گرداند و نه یک کدِ وضعیتِ HTTP؟
- `impl IntoResponse for AnimeError` چه چیزی به هندلر می‌دهد، و چرا `?` رویِ یک فراخوانیِ ذخیره‌گاه کار می‌کند؟
- چرا رتبه‌ی `15` یک `422` است و نه `400`، و فرقش با بدنه‌ای که JSON نیست چیست؟
- چرا `DELETE` خودتوان است وقتی فراخوانیِ دومش `404` می‌دهد؟
- چرا `PATCH` قولِ خودتوانی ندارد ولی `PUT` دارد؟
- چرا کامپایلر می‌گوید یک هندلر «`Handler` نیست» وقتی مشکلِ واقعی یک `MutexGuard` از میانِ یک `.await` است؟

---

## بیشتر

- [`IntoResponse` در مستنداتِ `axum`](https://docs.rs/axum/0.8.9/axum/response/trait.IntoResponse.html): هر نوعی که از قبل آن را دارد، از جمله شکل‌هایِ توپلی که امروز استفاده کردی.
- [`axum::error_handling`](https://docs.rs/axum/0.8.9/axum/error_handling/index.html): `axum` چطور به خطا فکر می‌کند، و چرا هندلرها عمداً خطاناپذیرند.
- [RFC 9110 — HTTP Semantics](https://www.rfc-editor.org/rfc/rfc9110.html): بخشِ ۱۵.۳.۲ (`201 Created`)، ۱۰.۲.۲ (`Location`)، ۹.۳.۴ (`PUT`)، ۱۳.۱.۱ (`If-Match`)، ۱۵.۵.۲۱ (`422`).
- [RFC 5789 — PATCH Method for HTTP](https://www.rfc-editor.org/rfc/rfc5789.html): چرا `PATCH` کنارِ `PUT` وجود دارد، و چرا طبقِ تعریف خودتوان نیست.
- [۲.۸.۴ — `Send` و `Sync`](../../../phase2-intermediate/08-concurrency/04-send-and-sync/README.fa.md): قاعده‌ی پشتِ `E0277`ِ امروز.
