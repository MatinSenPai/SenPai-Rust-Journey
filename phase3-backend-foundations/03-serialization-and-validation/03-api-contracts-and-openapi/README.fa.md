# ۳.۳.۳ — قراردادهایِ API و OpenAPI (`utoipa`)

## در یک نگاه

بعد از این درس می‌توانی:

- از نوع‌ها و هندلرهایِ خودت یک سندِ OpenAPI 3.1 تولید کنی، با `#[derive(ToSchema)]`، `#[utoipa::path]` و `#[derive(OpenApi)]`، و آن را از یک مسیرِ `axum` سرو کنی.
- قرارداد را تست کنی: ثابت کنی سند همان مسیرها، اسکیم‌ها، کدهایِ وضعیت و انواعِ محتوایی را فهرست می‌کند که قول داده‌ای، و روتر واقعاً همان‌ها را جواب می‌دهد.
- سه خطایِ کامپایلی که `utoipa` می‌دهد را بخوانی، و دو اشتباهی را که هیچ خطایی نمی‌دهد (مسیری که کسی مستند نکرده، کدِ وضعیتی که هندلر هرگز برنمی‌گرداند) تشخیص بدهی.

**زمان:** حدود ۹۰ دقیقه · **پیش‌نیاز:**
[۳.۱.۳ — چیزهایی از HTTP که باید بدانی](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.fa.md)،
[۳.۲.۳ — عملیاتِ CRUD روی کاتالوگِ انیمه (در حافظه)](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.fa.md)،
[۳.۳.۱ — عمقِ serde](../01-serde-depth/README.fa.md)،
[۳.۳.۲ — اعتبارسنجی](../02-validation/README.fa.md)

---

## چرا اهمیت دارد

۳.۲.۳ یک API کاتالوگ ساخت و برایِ هر نتیجه یک کدِ وضعیت انتخاب کرد. این انتخاب‌ها در کد هستند، و توسعه‌دهنده‌ی کلاینت نمی‌تواند کدِ تو را بخواند. او یک *قرارداد* می‌خواهد: سندی که بگوید «`POST /anime` با `201` و این JSON جواب می‌دهد، یا با `422` و آن JSON». قالبی که همه روی آن توافق کرده‌اند **OpenAPI** است. تولیدکننده‌هایِ کدِ کلاینت، gatewayها، Postman و سرورهایِ mock همه آن را می‌خوانند.

می‌توانی این سند را دستی بنویسی. روزی که می‌نویسی درست است. یک هفته بعد کسی یک فیلد اضافه می‌کند، یا مسیری جدید، یا `201` را `200` می‌کند، و هیچ‌کس YAML را باز نمی‌کند. مشخصاتِ دست‌نویس از کد جدا می‌شود (drift)، و قراردادِ غلط از نبودنِ قرارداد بدتر است، چون مردم به آن اعتماد می‌کنند.

اگر در Django `drf-spectacular` را دیده‌ای: سریالایزرها و ویوهایت را می‌خواند و اسکیما را تولید می‌کند، پس سند دنبالِ کد می‌آید. `utoipa` همین ایده برایِ Rust است، با یک تفاوتِ بزرگ در *چگونگی*. پایتون می‌تواند وقتی برنامه اجرا می‌شود کلاس‌هایت را ببیند. Rust نمی‌تواند، پس `utoipa` در زمانِ کامپایل کار می‌کند: ماکروهایِ derive رویِ نوع‌ها و یک attribute رویِ هندلرها سند را برایت می‌نویسند. در عوض باید فهرست کنی چه چیزی داخلش برود. این درس دقیقاً نشان می‌دهد این چه چیزی می‌خرد، و دو جایی که هنوز اجازه می‌دهد سند دروغ بگوید.

---

## مفهوم

### قرارداد یک سندِ JSON است

```senpai-visual
{"kind":"concept","labels":["نوع‌هایِ Rust: ToSchema","هندلرها: utoipa::path","ApiDoc: مشتقِ OpenApi","ApiDoc::openapi()","مسیرِ JSON: /api-docs/openapi.json","کلاینت‌ها، تولیدکننده‌هایِ کد، gatewayها"]}
```

سندِ OpenAPI یک شیءِ JSON با شکلِ ثابت است. این طرح، بخشی از آن است که این درس به کار می‌برد (یک طرح است، نه خروجیِ برنامه):

```text
openapi: "3.1.0"
info:        title, version
paths
  "/anime/{id}"
    get -> responses -> "200" -> content -> "application/json" -> schema
components
  schemas -> Anime, ApiError, CreateAnime, WatchStatus
```

هر عملیات (یک متد رویِ یک مسیر) `responses`اش را به تفکیکِ کدِ وضعیت فهرست می‌کند. هر پاسخ انواعِ رسانه‌ای (media type) را فهرست می‌کند که ممکن است برگردد، و هر نوعِ رسانه‌ای به یک اسکیما اشاره می‌کند. اسکیماهایی که چند عملیات مشترکاً به کار می‌برند یک بار زیرِ `components` می‌آیند و با `$ref` به آن‌ها ارجاع داده می‌شود.

### نوع‌ها اسکیما می‌شوند: `ToSchema`

```rust
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WatchStatus { Watching, Completed, PlanToWatch, Dropped }

/// One catalog entry.
#[derive(Serialize, Deserialize, ToSchema)]
pub struct Anime {
    #[schema(example = 1)]
    pub id: u64,
    pub title: String,
    pub status: WatchStatus,
    /// `1..=10`, or `null` when the show is not rated yet.
    #[schema(minimum = 1, maximum = 10)]
    pub rating: Option<u8>,
}
```

`examples/08-print-a-schema.rs` چیزی را که derive برایِ این دو نوشته چاپ می‌کند:

```sh
cargo run -p p3-03-03-api-contracts-and-openapi --example 08-print-a-schema
```

```text
--- WatchStatus
{
  "description": "Where you are with a show. On the wire it is snake_case:\n`\"watching\"`, `\"completed\"`, `\"plan_to_watch\"`, `\"dropped\"`.",
  "enum": [
    "watching",
    "completed",
    "plan_to_watch",
    "dropped"
  ],
  "type": "string"
}
--- Anime
{
  "description": "One catalog entry.",
  "properties": {
    "id": {
      "example": 1,
      "format": "int64",
      "minimum": 0,
      "type": "integer"
    },
    "rating": {
      "description": "`1..=10`, or `null` when the show is not rated yet.",
      "format": "int32",
      "maximum": 10,
      "minimum": 1,
      "type": [
        "integer",
        "null"
      ]
    },
    "status": {
      "$ref": "#/components/schemas/WatchStatus"
    },
    "title": {
      "example": "Frieren",
      "type": "string"
    }
  },
  "required": [
    "id",
    "title",
    "status"
  ],
  "type": "object"
}
```

با کدِ Rust مقایسه‌اش کن. derive به attributeهایِ `serde`ات (۳.۳.۱) احترام می‌گذارد: `rename_all = "snake_case"` دلیلِ آن است که مقدارهایِ enum `plan_to_watch` هستند. `Option<u8>` می‌شود `["integer", "null"]` و در `required` نیست. یک `u64` یک `int64` با کمینه‌ی ۰ است. کامنتِ مستندات می‌شود `description`. `#[schema(...)]` چیزی را اضافه می‌کند که نوع‌هایِ Rust نمی‌توانند بگویند: یک مثال، یک بازه.

یک محدودیتِ صادقانه: `minimum = 1, maximum = 10` *مستندسازی* است. هیچ‌چیز در `utoipa` رتبه‌ی ۱۵ را رد نمی‌کند. بررسی یک `if` در هندلر است (۳.۳.۲ جایِ درستیِ چنین قاعده‌هایی است). اگر یکی را عوض کنی و دیگری را فراموش کنی، دوباره drift داری، این بار داخلِ یک فایل.

### هندلرها عملیات می‌شوند: `#[utoipa::path]`

```rust
/// `GET /anime/{id}`: `200 OK` with the anime, or `404`.
#[utoipa::path(
    get,
    path = "/anime/{id}",
    params(("id" = u64, Path, description = "the anime's id")),
    responses(
        (status = 200, description = "found", body = Anime),
        (status = 404, description = "no anime has this id", body = ApiError),
    ),
    tag = "anime"
)]
pub async fn get_anime(/* extractors */) -> ApiResult<Json<Anime>> { /* ... */ }
```

این attribute به خودِ تابع دست نمی‌زند. کنارش می‌نشیند و چیزی را ثبت می‌کند که *تو* در آن می‌نویسی: متد، مسیر (با `{id}` به همان شکلِ `axum` 0.8)، پارامترها، و یک ورودی برایِ هر کدِ وضعیت. خطِ اولِ کامنتِ مستندات می‌شود `summary` عملیات و بقیه `description`.

چند نوعِ رسانه‌ای برایِ یک کدِ وضعیت از `content(...)` استفاده می‌کند. در `utoipa` 5 هر ورودی `(Schema = "media/type")` است:

```rust
(status = 422, description = "rating out of range, or wrong JSON shape",
    content((ApiError = "application/json"), (String = "text/plain"))),
```

به همین شکل `POST /anime` دربارهٔ ۳.۲.۳ راست می‌گوید: `422`ِ *خودت* JSON است، ولی `400`، `415` و `422`ِ دیگر را اکسترکتورِ `Json`ِ `axum` پیش از رسیدن به تابعت و به‌شکلِ `text/plain` برمی‌گرداند. قراردادی که فقط خطاهایِ JSON را فهرست کند غلط است. (۳.۸.۱ همه‌ی خطاها را یک شکل می‌کند؛ تا آن موقع سند همان چیزی را می‌گوید که واقعی است.)

تشبیهِ DRF کجا می‌شکند: `drf-spectacular` ویو را بررسی می‌کند و کدهایِ وضعیت و سریالایزرها را حدس می‌زند، پس می‌تواند بی‌صدا غلط باشد. `utoipa` هیچ‌چیز را حدس نمی‌زند. فهرستِ `responses(...)` یک *قولِ تایپ‌شده* است و هرگز با بدنه‌ی تابع مقایسه نمی‌شود. این جمله را برایِ «خطاهایی که خواهی دید» در ذهن نگه دار.

### جمع‌کردن: `#[derive(OpenApi)]` و یک مسیر

```rust
#[derive(OpenApi)]
#[openapi(
    info(title = "Anime catalog", version = "1.0.0"),
    paths(list_anime, create_anime, get_anime, delete_anime),
)]
pub struct ApiDoc;

pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
```

`paths(...)` فهرستِ هندلرهایِ annotate‌شده است؛ اسکیمایی که از آن‌ها در دسترس باشد (`Anime`، `ApiError`، ...) خودکار به `components` اضافه می‌شود. `ApiDoc::openapi()` سند را در حافظه می‌سازد، و چون `Serialize` دارد، `Json(...)` می‌تواند سرویسش بدهد. `examples/01-print-the-document.rs` بخش‌هایی از نتیجه را نشان می‌دهد:

```sh
cargo run -p p3-03-03-api-contracts-and-openapi --example 01-print-the-document
```

```text
openapi version: "3.1.0"
paths: ["/anime", "/anime/{id}"]
schemas: ["Anime", "ApiError", "CreateAnime", "WatchStatus"]
--- GET /anime/{id} responses
{
  "200": {
    "content": {
      "application/json": {
        "schema": {
          "$ref": "#/components/schemas/Anime"
        }
      }
    },
    "description": "found"
  },
  "404": {
    "content": {
      "application/json": {
        "schema": {
          "$ref": "#/components/schemas/ApiError"
        }
      }
    },
    "description": "no anime has this id"
  }
}
```

(کلیدها به ترتیبِ الفبایی درمی‌آیند چون سند از `serde_json::Value` رد شده است.)

### تستِ قرارداد

چون سند JSON ساده است، تست می‌تواند از آن سؤال بپرسد: بدونِ شبکه، بدونِ مرورگر. در پله‌ی «پیاده‌سازی» چهار خواننده‌ی کوچک رویِ یک `serde_json::Value` می‌نویسی، و تست‌ها این‌طور به کارشان می‌برند:

| سؤال | خواننده | جوابِ این API |
|---|---|---|
| چه اسکیماهایی هست؟ | `schema_names` | `Anime`، `ApiError`، `CreateAnime`، `WatchStatus` |
| `POST /anime` چه کدهایی را قول می‌دهد؟ | `operation_statuses` | `201`، `400`، `415`، `422` |
| آن `422` چه نوعِ رسانه‌ای می‌تواند باشد؟ | `response_content_types` | `application/json`، `text/plain` |
| کدام مسیرها را کسی مستند نکرده؟ | `undocumented` | هیچ‌کدام |

`tests/api_test.rs` نیمه‌ی دیگر است: درخواست‌هایِ واقعی را با `oneshot` (۳.۲.۱) می‌فرستد و بررسی می‌کند روتر دقیقاً همین کدها و انواعِ محتوا را جواب می‌دهد. تستِ قرارداد *هر دو* را می‌سنجد: آنچه سند می‌گوید، و اینکه سرور همان را انجام می‌دهد.

### دو راهی که قرارداد هنوز دروغ می‌گوید

```senpai-visual
{"kind":"result","labels":["مسیر به روتر اضافه شد","در paths(...) نیست","سند دربارهٔ آن ساکت است","attribute می‌گوید 201","هندلر 200 جواب می‌دهد","سند چیزِ غلطی قول می‌دهد"]}
```

- **غایب از سند.** روتر یک مسیر دارد؛ کسی آن را در `paths(...)` نگذاشته. هیچ‌چیز خراب نمی‌شود. `undocumented` آن را می‌گیرد، به شرطی که فهرستِ دقیقی از مسیرها بدهی. `axum` 0.8 راهی برایِ فهرست‌کردنِ مسیرهایِ یک روتر نمی‌دهد، پس آن فهرست (`ROUTES` اینجا) دستی نوشته می‌شود: یک چیزِ دوم که باید هم‌قدم نگهش داری. باز هم نوشتنش خیلی ارزان‌تر از یک مشخصاتِ کامل است.
- **قولی که بدنه انجام نمی‌دهد.** attribute می‌گوید `201`، تابع یک `Json`ِ خالی برمی‌گرداند که `200` است (تله‌ی ۳.۲.۳). فقط تستی که درخواست را می‌فرستد متوجه می‌شود.

---

## دست‌به‌کد

دو مثالی را اجرا کن که یک دروغ نشان می‌دهند. هیچ‌کدام خطا نیستند؛ هر دو کامپایل می‌شوند.

```sh
cargo run -p p3-03-03-api-contracts-and-openapi --example 03-forgotten-route-trap
cargo run -p p3-03-03-api-contracts-and-openapi --example 06-documented-201-answers-200-trap
```

```text
GET /ping   -> 200 OK   documented: true
GET /health -> 200 OK   documented: false
contract says POST /anime answers ["201"]
the handler answers 200 OK
```

تست‌ها نیمه‌قرمز شروع می‌شوند. `src/lib.rs` کلِ API را دارد، از قبل مستندشده، و چهار خوانده‌ی `todo!()`:

```sh
cargo test -p p3-03-03-api-contracts-and-openapi 2>&1 | grep 'test result'
```

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: FAILED. 0 passed; 12 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

آن ۷ تستِ `api_test` همین حالا سبزند: API داده‌شده را با سندِ داده‌شده می‌سنجند. آن ۱۲ تستِ `contract_test` مالِ توست.

سرورِ واقعی (روی `127.0.0.1:3120` گوش می‌دهد) قراردادِ خودش را سرو می‌کند. اجرایش کن و از آن بپرس:

```sh
cargo run -p p3-03-03-api-contracts-and-openapi --example 02-serve-the-contract &
curl -si http://127.0.0.1:3120/api-docs/openapi.json | head -n 2
curl -s http://127.0.0.1:3120/api-docs/openapi.json | head -c 120
```

```text
listening on http://127.0.0.1:3120
HTTP/1.1 200 OK
content-type: application/json
{"openapi":"3.1.0","info":{"title":"Anime catalog","description":"A tiny catalog API, documented from its own code.","li
```

(خطِ اول از سرور می‌آید؛ `head -c` بدنه را بدونِ newline می‌برد.) حالا ادعاهایِ قرارداد دربارهٔ `POST /anime` و `GET /anime/{id}` را بسنج، هرکدام با کدِ وضعیت و نوعِ محتوایش:

```sh
curl -s -w '\n%{http_code} %{content_type}\n' -X POST -H 'content-type: application/json' -d '{"title":"Frieren","status":"watching","rating":9}' http://127.0.0.1:3120/anime
curl -s -w '\n%{http_code} %{content_type}\n' -X POST -H 'content-type: application/json' -d '{"title":"Bad","status":"watching","rating":15}' http://127.0.0.1:3120/anime
curl -s -w '\n%{http_code} %{content_type}\n' -X POST -H 'content-type: application/json' -d '{' http://127.0.0.1:3120/anime
curl -s -w '\n%{http_code} %{content_type}\n' -X POST -d '{"title":"x","status":"watching"}' http://127.0.0.1:3120/anime
curl -s -w '\n%{http_code} %{content_type}\n' http://127.0.0.1:3120/anime/9
```

```text
{"id":1,"title":"Frieren","status":"watching","rating":9}
201 application/json
{"error":"rating must be between 1 and 10, got 15"}
422 application/json
Failed to parse the request body as JSON: EOF while parsing an object at line 1 column 1
400 text/plain; charset=utf-8
Expected request with `Content-Type: application/json`
415 text/plain; charset=utf-8
{"error":"anime not found"}
404 application/json
```

همه‌ی کدها و انواعِ رسانه‌ای اینجا در سند هستند. سرور را متوقف کن (`kill %1` در همان شل). بعد این‌ها را امتحان کن:

۱. JSON را باز کن و `422` زیرِ `POST /anime` را پیدا کن. چرا دو نوعِ رسانه‌ای فهرست کرده؟
۲. در attributeِ `get_anime`، `status = 404` را به `status = 410` عوض کن و `api_test` را اجرا کن. چیزی خراب می‌شود؟ این دربارهٔ چیزی که `utoipa` بررسی می‌کند چه می‌گوید؟
۳. `examples/01-print-the-document.rs` را اجرا کن و به `204`ِ `DELETE /anime/{id}` نگاه کن. چرا `content` ندارد؟

---

## خطاهایی که خواهی دید

هر مثالِ خراب پشتِ فیچرِ `broken` است. در ترنسکریپت‌هایِ زیر، وقتی رویِ اسکلت build می‌کنی `cargo` اول برایِ چهار خواننده‌ی `todo!()` هشدارِ `unused variable` چاپ می‌کند. آن‌ها اینجا حذف شده‌اند و وقتی خواننده‌ها را پیاده کنی از بین می‌روند.

### `E0277` — نوعِ بدنه‌ای که `ToSchema` ندارد

```rust
#[derive(Serialize)]
struct Genre { name: String }

#[utoipa::path(get, path = "/genre", responses((status = 200, body = Genre)))]
async fn genre() -> Json<Genre> { /* ... */ }
```

`examples/04-missing-to-schema-broken.rs`:

```text
error[E0277]: the trait bound `Genre: ToSchema` is not satisfied
  --> phase3-backend-foundations\03-serialization-and-validation\03-api-contracts-and-openapi\examples\04-missing-to-schema-broken.rs:14:70
   |
14 | #[utoipa::path(get, path = "/genre", responses((status = 200, body = Genre)))]
   |                                                                      ^^^^^ unsatisfied trait bound
   |
help: the trait `ToSchema` is not implemented for `Genre`
  --> phase3-backend-foundations\03-serialization-and-validation\03-api-contracts-and-openapi\examples\04-missing-to-schema-broken.rs:10:1
   |
10 | struct Genre {
   | ^^^^^^^^^^^^
   = help: the following other types implement trait `ToSchema`:
             &'t [T]
             &'t mut [T]
             &str
             ()
             BTreeMap<K, T>
             BTreeSet<K>
             Box<T>
             Cow<'a, T>
           and 26 others

error[E0277]: the trait bound `Genre: PartialSchema` is not satisfied
   --> phase3-backend-foundations\03-serialization-and-validation\03-api-contracts-and-openapi\examples\04-missing-to-schema-broken.rs:14:70
    |
 14 | #[utoipa::path(get, path = "/genre", responses((status = 200, body = Genre)))]
    |                                                                      ^^^^^ unsatisfied trait bound
    |
help: the trait `utoipa::__dev::ComposeSchema` is not implemented for `Genre`
   --> phase3-backend-foundations\03-serialization-and-validation\03-api-contracts-and-openapi\examples\04-missing-to-schema-broken.rs:10:1
    |
 10 | struct Genre {
    | ^^^^^^^^^^^^
    = help: the following other types implement trait `utoipa::__dev::ComposeSchema`:
              &[T]
              &mut [T]
              &str
              BTreeMap<K, T>
              BTreeSet<K>
              Box<T>
              Cow<'a, T>
              HashMap<K, T, S>
            and 24 others
    = note: required for `Genre` to implement `PartialSchema`
note: required by a bound in `name`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\utoipa-5.5.0\src\lib.rs:374:21
    |
374 | pub trait ToSchema: PartialSchema {
    |                     ^^^^^^^^^^^^^ required by this bound in `ToSchema::name`
...
405 |     fn name() -> Cow<'static, str> {
    |        ---- required by a bound in this associated function

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-03-03-api-contracts-and-openapi` (example "04-missing-to-schema-broken") due to 4 previous errors
```

**کامپایلر به چه اعتراض دارد:** `body = Genre` باعث می‌شود ماکرو از `Genre` اسکیمایش را بخواهد. `Genre` یک `Serialize` دارد که به `serde` می‌گوید چطور نوشته شود، ولی هیچ‌چیز به `utoipa` نمی‌گوید در سند چه شکلی است. خطایِ دوم همان impl نبودن است، دیده‌شده از راهِ trait‌ای که `ToSchema` رویش بنا شده.

**رفع:** derive کن.

```rust
#[derive(Serialize, ToSchema)]
struct Genre { name: String }
```

**چرا این رفع است:** `ToSchema` تنها جایی است که یک نوع خودش را به سند معرفی می‌کند. هر نوعی که در `body = ...` نام می‌بری، یا در یکی از آن‌ها هست، به آن نیاز دارد.

### `E0425` — `paths(...)` تابعی را نام می‌برد که attribute ندارد

```rust
async fn health() -> &'static str { "ok" }

#[derive(OpenApi)]
#[openapi(paths(health))]
struct Doc;
```

`examples/05-path-without-attribute-broken.rs`:

```text
error[E0425]: cannot find type `__path_health` in this scope
  --> phase3-backend-foundations\03-serialization-and-validation\03-api-contracts-and-openapi\examples\05-path-without-attribute-broken.rs:11:10
   |
11 | #[derive(OpenApi)]
   |          ^^^^^^^ not found in this scope
   |
   = note: this error originates in the derive macro `OpenApi` (in Nightly builds, run with -Z macro-backtrace for more info)

error[E0433]: cannot find module or crate `__path_health` in this scope
  --> phase3-backend-foundations\03-serialization-and-validation\03-api-contracts-and-openapi\examples\05-path-without-attribute-broken.rs:11:10
   |
11 | #[derive(OpenApi)]
   |          ^^^^^^^ use of unresolved module or unlinked crate `__path_health`
   |
   = help: if you wanted to use a crate named `__path_health`, use `cargo add __path_health` to add it to your `Cargo.toml`
   = note: this error originates in the derive macro `OpenApi` (in Nightly builds, run with -Z macro-backtrace for more info)

Some errors have detailed explanations: E0425, E0433.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `p3-03-03-api-contracts-and-openapi` (example "05-path-without-attribute-broken") due to 2 previous errors
```

**کامپایلر به چه اعتراض دارد:** `#[utoipa::path]` یک نوعِ کمکیِ پنهان به نامِ `__path_<function>` کنارِ هندلر تولید می‌کند. `paths(health)` دنبالِ همان کمکی می‌گردد. نامِ `__path_health` برایت معنایی ندارد، ولی سرنخ همین است: `health` هرگز attribute نگرفته.

**رفع:** `#[utoipa::path(get, path = "/health", responses((status = 200, description = "ok")))]` را رویِ `health` بگذار.

**چرا این رفع است:** `paths(...)` تابعِ تو را نمی‌خواند، فقط کمکی‌ای را که attribute جا می‌گذارد. بدونِ attribute، کمکی نیست؛ بدونِ کمکی، ورودی نیست.

### یک خطایِ پارسِ ماکرو — املایِ `utoipa` 4 برایِ `content(...)`

```rust
responses((status = 200, content(("text/plain" = String))))
```

اینجا کدِ خطا نیست، چون ماکرو ورودیِ خودش را پیش از آنکه `rustc` نوعی برایِ شکایت داشته باشد رد می‌کند. `examples/07-v4-content-syntax-broken.rs`:

```text
error: expected `,`
  --> phase3-backend-foundations\03-serialization-and-validation\03-api-contracts-and-openapi\examples\07-v4-content-syntax-broken.rs:10:52
   |
10 |     responses((status = 200, content(("text/plain" = String))))
   |                                                    ^

error: could not compile `p3-03-03-api-contracts-and-openapi` (example "07-v4-content-syntax-broken") due to 1 previous error
```

**کامپایلر به چه اعتراض دارد:** علامت زیرِ `=` است. ماکرو `"text/plain"` را اسکیما خواند، انتظارِ پایانِ ورودی داشت، و `=` دید. مثال‌ها و پست‌هایی که برایِ `utoipa` 4 نوشته شده‌اند نوعِ رسانه‌ای را اول می‌گذارند. در نسخه‌ی ۵ (این درس 5.5.0 را بررسی کرده، همان که در `Cargo.lock` است) ترتیب `(Schema = "media/type")` است.

**رفع:**

```rust
responses((status = 200, content((String = "text/plain"))))
```

**چرا این رفع است:** دستورِ ماکرو برایِ یک ورودی «اسکیما، و بعد اختیاری `= نوعِ رسانه‌ای`» است. وقتی یک attribute پارس نمی‌شود، نسخه‌ی چیزی را که از آن کپی کرده‌ای بررسی کن.

### هیچ خطایی نیست: مسیری که سند هرگز نمی‌گوید

```text
GET /ping   -> 200 OK   documented: true
GET /health -> 200 OK   documented: false
```

**چه چیزی واقعاً خراب است:** `examples/03-forgotten-route-trap.rs` کار می‌کند. مسیر `200` جواب می‌دهد. فقط سند نمی‌داند وجود دارد، پس کلاینتی که از سند تولید شده راهی برایِ صدا زدنش ندارد.

**رفع:** هندلر را annotate کن و به `paths(...)` اضافه‌اش کن، بعد کاری کن که اگر فراموش شد یک تست شکست بخورد:

```rust
assert!(undocumented(&doc, ROUTES).is_empty());
```

**چرا این رفع است:** یک ورودیِ غایب نمی‌تواند خطایِ کامپایل باشد، چون روتر و سند دو فهرستِ جدا هستند. تست آن‌ها را مقایسه می‌کند. فقط به اندازه‌ی `ROUTES` خوب است که آن را دستی نگه می‌داری.

### هیچ خطایی نیست: `201`ِ مستندشده، `200`ِ جواب‌داده‌شده

```text
contract says POST /anime answers ["201"]
the handler answers 200 OK
```

**چه چیزی واقعاً خراب است:** `examples/06-documented-201-answers-200-trap.rs` در attribute `status = 201` دارد و هندلری که یک `Json`ِ خالی برمی‌گرداند، که `200` است. سند کدی را توصیف می‌کند که *قصد داشتی*.

**رفع:** هندلر را با آن یکی کن و بگذار یک تست درخواست را بفرستد:

```rust
Ok((StatusCode::CREATED, Json(anime)))
```

**چرا این رفع است:** عنصرِ اولِ تاپل کدِ وضعیت را تعیین می‌کند (۳.۲.۳). `utoipa` هرگز آن را نمی‌خواند، پس `created_answers_201_with_the_new_anime` در `tests/api_test.rs` چیزی است که قول را به رفتار گره می‌زند.

---

## تمرین

### گرم‌کردن

<details>
<summary>در اسکیما رویِ <code>rating</code> نوشته <code>#[schema(minimum = 1, maximum = 10)]</code>. کلاینت <code>15</code> می‌فرستد. <code>utoipa</code> ردش می‌کند؟</summary>

قبل از دیدنِ جواب خودت فکر کن.

</details>

<details>
<summary>جواب</summary>

نه. attribute فقط `minimum` و `maximum` را در سند می‌نویسد. ردکردن (`422`) همان `if` در هندلر است. این دو را خودت و یک تست باید هم‌قدم نگه دارید.

</details>

<details>
<summary>کدِ <code>422</code>ِ <code>POST /anime</code> چه انواعِ رسانه‌ای دارد، و چرا بیش از یکی؟</summary>

قبل از دیدنِ جواب خودت فکر کن.

</details>

<details>
<summary>جواب</summary>

`application/json` و `text/plain`. بررسیِ رتبه در خودِ هندلر یک `ApiError`ِ JSON جواب می‌دهد. یک بدنه‌ی JSON با شکلِ غلط را اول اکسترکتورِ `Json`ِ `axum` به‌شکلِ متنِ ساده رد می‌کند (۳.۲.۳).

</details>

<details>
<summary>یک <code>.route("/health", get(health))</code> اضافه می‌کنی و attribute را فراموش می‌کنی. خطایِ کامپایل؟ سند چه می‌گوید؟</summary>

قبل از دیدنِ جواب خودت فکر کن.

</details>

<details>
<summary>جواب</summary>

خطایِ کامپایل نیست، و سند دربارهٔ `/health` هیچ نمی‌گوید. روتر و `paths(...)` فهرست‌هایِ جدا هستند.

</details>

<details>
<summary>آیا <code>rating: Option&lt;u8&gt;</code> در <code>required</code> است؟ <code>type</code>اش چیست؟</summary>

قبل از دیدنِ جواب خودت فکر کن.

</details>

<details>
<summary>جواب</summary>

لازم نیست. نوعش `["integer", "null"]` است، یعنی راهِ OpenAPI 3.1 برایِ گفتنِ «یک عددِ صحیح یا null».

</details>

<details>
<summary>چرا <code>204</code>ِ <code>DELETE /anime/{id}</code> <code>content</code> ندارد؟</summary>

قبل از دیدنِ جواب خودت فکر کن.

</details>

<details>
<summary>جواب</summary>

`204 No Content` بدنه ندارد، و attribute هم `body`ای نام نمی‌برد، پس چیزی برایِ توصیف نیست.

</details>

### تعمیر

هر پنج را درست کن. سه‌تایِ اول را با `--features broken` build کن تا خطا را ببینی، بعد کاری کن کامپایل شوند و اجرا شوند:

۱. `examples/04-missing-to-schema-broken.rs`.
۲. `examples/05-path-without-attribute-broken.rs`: attribute را اضافه کن تا `/health` در سند بیاید.
۳. `examples/07-v4-content-syntax-broken.rs`.
۴. `examples/03-forgotten-route-trap.rs`: کاری کن برایِ هر دو مسیر `documented: true` چاپ شود.
۵. `examples/06-documented-201-answers-200-trap.rs`: attribute را همان‌طور نگه دار و هندلر را عوض کن تا `201 Created` جواب بدهد.

### پیاده‌سازی

چهار خوانده‌ی `todo!()` در `src/lib.rs`: `schema_names`، `operation_statuses`، `response_content_types`، `undocumented`. هر کامنتِ مستندات کلِ مشخصات است (مرتب‌سازی، حروفِ متد، چه چیزی برایِ موردِ غایب برگردد).

```sh
cargo test -p p3-03-03-api-contracts-and-openapi
```

`tests/contract_test.rs` (۱۲ تست) خواننده‌ها را رویِ سندِ واقعی و رویِ سندهایِ دست‌ساز و کوچک می‌سنجد. `tests/api_test.rs` (۷ تست، از قبل سبز) API را با سندش می‌سنجد.

### بساز

`PUT /anime/{id}` را اضافه کن: جایگزینیِ کامل. بدنه `CreateAnime` است؛ با `200` و `Anime`ِ جدید (همان شناسه) جواب می‌دهد، با `404` برایِ شناسه‌ی ناشناخته (هرگز نمی‌سازد)، و با `422` برایِ رتبه‌ی بیرون از `1..=10`. با `#[utoipa::path]` مستندش کن، هندلر را به `paths(...)` اضافه کن، `.put(...)` را به مسیر زنجیر کن، و `("PUT", "/anime/{id}")` را به `ROUTES` اضافه کن. در یک `tests/put_test.rs`ِ جدید یک `PUT` بفرست، و ثابت کن سند برایِ آن `200`، `404` و `422` را فهرست می‌کند و `undocumented(&doc, ROUTES)` خالی است.

### چالش (اختیاری)

یک فیلترِ `?status=watching` به `GET /anime` اضافه کن و با `#[derive(IntoParams)]` رویِ structِ کوئری (همراه با `#[into_params(parameter_in = Query)]`) و `params(YourQuery)` در attributeِ path مستندش کن. `parameters`ِ تولیدشده را در یک تست بررسی کن. یک چیز را دنبال کن: در یک بررسیِ آزمایشی، یک structِ کوئری که به `WatchStatus` اشاره می‌کرد `$ref`ای داشت که به اسکیمایِ غایب از `components` اشاره می‌کرد، تا وقتی `WatchStatus` از یک بدنه در دسترس باشد یا با `#[openapi(components(schemas(...)))]` ثبت شود. سندت را خودکفا کن. تستِ آماده‌ای نیست.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به کار می‌بری |
|---|---|---|
| سندِ OpenAPI | توصیفِ JSONِ مسیرها، پاسخ‌ها و اسکیماهایِ یک API | تولیدکننده‌هایِ کلاینت، gatewayها، مستندات |
| قرارداد (contract) | آنچه API قول می‌دهد: کدها، انواعِ رسانه‌ای، شکل‌ها | تست‌ها، نسخه‌بندی |
| `ToSchema` | derive‌ای که یک نوع را به سند معرفی می‌کند | هر نوعِ درخواست و پاسخ |
| `#[utoipa::path]` | attribute‌ای که یک عملیات را ثبت می‌کند | هر هندلر |
| `#[derive(OpenApi)]` | مسیرها و اسکیماها را در `ApiDoc::openapi()` جمع می‌کند | یکی برایِ هر API |
| تستِ قرارداد | تستی که هم سند و هم پاسخ‌هایِ واقعی را می‌سنجد | هر API عمومی |

### الان می‌دانی

- سندِ تولیدشده از کد مثلِ سندِ دست‌نویس کهنه نمی‌شود، ولی باز هم فقط چیزی را ثبت می‌کند که تایپ کرده‌ای.
- `ToSchema` از attributeهایِ `serde`ات پیروی می‌کند. بازه‌هایِ `#[schema(...)]` مستندسازی‌اند، نه اعتبارسنجی.
- پاسخ‌هایِ هر endpoint به تفکیکِ کدِ وضعیت و نوعِ رسانه‌ای فهرست می‌شود، از جمله ردهایِ `text/plain`ِ خودِ `axum`.
- `utoipa` 5 چند نوعِ رسانه‌ای را به‌شکلِ `(Schema = "media/type")` می‌نویسد.
- دو دروغِ بی‌صدا (مسیرِ مستندنشده، کدِ وضعیتِ قول‌داده‌شده‌ی غلط) تست می‌خواهند، نه کامپایلر.

### بعداً کامل‌تر می‌بینی

- **یک‌شکل‌کردنِ همه‌ی خطاها به یک JSON تا سند بتواند ورودی‌هایِ `text/plain` را کنار بگذارد**: [۳.۸.۱ — پاکت‌هایِ خطایِ یکدست](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.fa.md)
- **قاعده‌هایِ واقعیِ سطحِ فیلد به‌جایِ یک `if`ِ دست‌نویس**: [۳.۳.۲ — اعتبارسنجی](../02-validation/README.fa.md)
- **عوض‌کردنِ قرارداد بدونِ شکستنِ مصرف‌کننده‌ها**: [۳.۳.۴ — نسخه‌بندیِ API و تکاملش](../04-api-versioning-and-evolution/README.fa.md)
- **ذخیره‌ی کاتالوگ بعداً کجا می‌رود**: [۳.۵.۳ — کاتالوگ انیمه، این‌بار متصل به Postgres](../../05-postgres-and-sqlx/03-anime-catalog-postgres-backed/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا مشخصاتِ دست‌نویس drift می‌کند، و تولید از نوع‌ها چه چیزی را درست می‌کند و چه چیزی را نه؟
- چرا `utoipa` می‌تواند از ماکرو تولید کند در حالی که `drf-spectacular` در زمانِ اجرا بررسی می‌کند؟
- `#[schema(minimum = 1)]` چه می‌کند، و چه نمی‌کند؟
- چرا `422`ِ `POST /anime` دو نوعِ رسانه‌ای فهرست می‌کند؟
- دو اشتباهی را که `utoipa` برایشان خطایِ کامپایل نمی‌دهد نام ببر، و تستی که هرکدام را می‌گیرد.

---

## بیشتر

- [`utoipa` در docs.rs](https://docs.rs/utoipa/5.5.0/utoipa/): مرجعِ attributeها برایِ همین نسخه‌ای که اینجا به کار رفته.
- [OpenAPI Specification 3.1.0](https://spec.openapis.org/oas/v3.1.0): خودِ قالب.
- [`drf-spectacular`](https://drf-spectacular.readthedocs.io/): همتایِ Django، برایِ مقایسه‌ی دو رویکرد.
- یک crateِ همراه، `utoipa-axum`، یک مسیر و مستنداتش را در یک فراخوانی ثبت می‌کند تا آن دو فهرست از هم جدا نشوند. این دوره از آن استفاده نمی‌کند؛ پیش از به‌کاربردنش مستنداتش را برایِ نسخه‌ی سازگار با `axum` 0.8 بررسی کن.
