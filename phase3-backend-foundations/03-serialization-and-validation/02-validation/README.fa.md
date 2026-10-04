# ۳.۳.۲ — اعتبارسنجی

## در یک نگاه

بعد از این درس می‌توانی:

- با `#[derive(Validate)]` قاعده را رویِ خودِ نوع بگذاری (طول، بازه، ایمیل، تابعِ خودت، ساختارها و لیست‌هایِ تودرتو) و درختِ `ValidationErrors`ی را که `validate()` برمی‌گرداند بخوانی.
- آن درخت را به یک بدنه‌ی JSON با کلیدِ فیلد (`{"errors": {"reviewer.email": [...]}}`) تخت کنی و با `422` جواب بدهی، تا کلاینت بتواند هر پیام را کنارِ ورودیِ خودش نشان بدهد.
- بگویی کدام یک از دو گذر یک درخواست را رد کرده است: گذرِ شکل (`Json<T>`، `serde`) یا گذرِ قاعده (`validate()`)، و هر کدام چه کدِ وضعیتی می‌دهد.
- سه اشتباهی را که این crate راحت می‌سازد تشخیص بدهی: derive که فراموش شده (`E0599`)، قاعده‌ی دلخواه با امضایِ غلط (`E0308`)، و ساختارِ تودرتویی که قاعده‌هایش بی‌صدا هرگز اجرا نمی‌شوند.

**زمان:** حدود ۹۰ دقیقه · **پیش‌نیاز:**
[۳.۳.۱ — عمقِ serde](../01-serde-depth/README.fa.md)،
[۳.۲.۳ — عملیاتِ CRUD روی کاتالوگِ انیمه (در حافظه)](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.fa.md)،
[۳.۱.۳ — چیزهایی از HTTP که باید بدانی](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.fa.md)

---

## چرا اهمیت دارد

در ۳.۲.۳ رتبه‌ی `15` یک `422` گرفت. قاعده‌ی پشتش یک تابعِ دستی در ذخیره‌گاه بود، `validate_rating`، که از دو متد صدا زده می‌شد. برایِ یک فیلد کافی است. بدنه‌ی یک درخواستِ واقعی حدودِ دوازده فیلد دارد، بعضی‌شان تودرتو (یک reviewer درونِ یک review، یک لیست از یادداشتِ قسمت‌ها درونِ آن)، و هر قاعده‌ای که دستی بنویسی قاعده‌ای است که ممکن است صدا زدنش را فراموش کنی.

در DRF هرگز آن تابع را ننوشتی: `serializers.IntegerField(min_value=1, max_value=10)` خودِ قاعده است، `serializer.is_valid()` همه‌شان را اجرا می‌کند، و `serializer.errors` دیکشنری‌ای از `{"field": ["message", ...]}` است که مستقیم در یک `400` می‌رود. این درس همان سه قطعه را در Rust می‌سازد: قاعده‌هایی که رویِ نوع اعلام می‌شوند، یک فراخوانی که همه را اجرا می‌کند، و یک مقدارِ خطا که تو آن را به پاسخ تبدیل می‌کنی. نسخه‌ی Rust گامی جدا از پارس‌کردن است (نیمه‌ی پارس را ۳.۳.۱ دارد)، و فهمیدنِ همین جداییِ دو گام مهم‌ترین چیزِ این درس است.

crate، `validator` نسخه‌ی ۰٫۱۸ است. هر چیزی که پایین می‌بینی رویِ نسخه‌ی حل‌شده، `0.18.1`، اجرا شده است.

---

## مفهوم

### دو گذر، دو شکستِ متفاوت

بدنه‌ی یک درخواست از دو بررسی رد می‌شود، و این دو متفاوت شکست می‌خورند:

```senpai-visual
{"kind":"result","labels":["بدنه‌ی درخواست","Json از T: شکلش درست است؟","۴۰۰ یا ۴۲۲، متنِ سادهٔ axum","validate: مقدارها قاعده را رعایت می‌کنند؟","۴۲۲، شیء errorsِ خودت","هندلر اجرا می‌شود"]}
```

گذرِ اول `Json<T>` از ۳.۲.۱ است که از `serde` (۳.۳.۱) استفاده می‌کند: این JSON است، و فیلدهایی را که `T` لازم دارد، با نوع‌هایِ درست، دارد؟ اگر نه، `axum` خودش جواب می‌دهد (`400` برایِ JSONِ خراب، `422` برایِ شکلِ غلط، `415` برایِ نبودنِ `Content-Type`) و کدِ تو هرگز اجرا نمی‌شود. گذرِ دوم تازه است: بدنه حالا یک `T` است، پس همه‌ی نوع‌ها درست‌اند، ولی `rating: 15` یک `u8`ِ کاملاً درست است که باز هم قاعده را می‌شکند. نوع نمی‌تواند بگوید «یک `u8` از ۱ تا ۱۰»، پس یک گامِ دومِ صریح این کار را می‌کند.

DRF هر دو را درونِ `is_valid()` انجام می‌دهد. تشبیه کجا می‌شکند: اینجا خودت در کدِ خودت انتخاب می‌کنی گامِ دوم کجا اجرا شود و شکستش روی سیم چه شکلی باشد. هیچ‌چیز خودکار نیست.

### قاعده‌ها رویِ نوع می‌نشینند: `#[derive(Validate)]`

اول قاعده‌ها را ببین:

```rust
#[derive(Debug, Validate)]
struct Rating {
    #[validate(range(min = 1, max = 10, message = "rating must be between 1 and 10"))]
    score: u8,
    #[validate(length(min = 1, max = 20))]
    title: String,
    #[validate(email)]
    contact: String,
    #[validate(length(max = 5))]
    note: Option<String>,
}
```

`#[derive(Validate)]` یک متد تولید می‌کند، `validate(&self) -> Result<(), ValidationErrors>`. *همه‌ی* قاعده‌ها را اجرا می‌کند، نه فقط اولینی که شکست می‌خورد. `examples/01-derive-validate.rs` یک `Rating`ِ سالم می‌سازد، یکی که `note`اش بلند است، و یکی که سه قاعده را هم‌زمان می‌شکند:

```text
good: Ok(())
long note: true
contact: code=email message=None
score: code=range message=Some("rating must be between 1 and 10")
title: code=length message=None
```

از این خروجی دو چیز بخوان. قاعده‌ای که `message` ندارد باز هم یک `code` دارد (`email`، `length`، `range`): نامِ قاعده‌ای که شکست خورد. و `note: Option<String>` وقتی `None` بود قبول شد: قاعده‌ی رویِ یک `Option` فقط وقتی اجرا می‌شود که مقدار باشد، دقیقاً مثلِ `required=False`ِ DRF.

قاعده‌هایی که بیشتر می‌بینی:

| قاعده | رویِ | نمونه |
|---|---|---|
| `length` | `String`، `Vec`، ... | `length(min = 1, max = 100)` |
| `range` | عددها | `range(min = 1, max = 10)` |
| `email`، `url` | رشته‌ها | `email` |
| `custom` | هر چیز | `custom(function = "my_rule")` |
| `nested` | یک ساختار، یا یک `Vec` از آنها | `nested` |

هر قاعده یک `message = "..."`ِ اختیاری می‌گیرد. نامِ اتریبیوت‌ها همان‌هایی است که مستنداتِ ۰٫۱۸ فهرست می‌کند؛ در این نسخه `custom(function = "...")` نامِ تابع را *به‌شکلِ رشته* می‌گیرد.

### `validate()` چه برمی‌گرداند: یک درخت، نه یک فهرست

یک ساختارِ تخت نتیجه‌ی تخت می‌دهد. ساختاری که ساختارِ دیگری درونش است، یا یک `Vec` از آنها، درخت می‌دهد، و `ValidationErrors` برایِ هر سه شکل یک نوعِ گره دارد:

```rust
pub enum ValidationErrorsKind {
    Struct(Box<ValidationErrors>),
    List(BTreeMap<usize, Box<ValidationErrors>>),
    Field(Vec<ValidationError>),
}
```

خودِ `ValidationErrors` یک نقشه از نامِ فیلد به یکی از اینها را در خود دارد. `Field` برگ است: فهرستِ قاعده‌هایی که رویِ آن فیلد شکست خورده‌اند. `Struct` همان `ValidationErrors`ِ خودِ یک ساختارِ تودرتوست. `List` برایِ هر عنصرِ ناموفق یک `ValidationErrors` است، با کلیدِ اندیسِ آن عنصر (عنصرهایِ سالم اصلاً نیستند). `examples/02-nested-and-lists.rs` یک review را اعتبارسنجی می‌کند که `first_note`اش و دومین عنصرِ `notes`اش خالی‌اند، و درخت را چاپ می‌کند:

```text
first_note: Struct
  text: Field(1)
notes: List
  [1]
    text: Field(1)
title: Field(1)
```

برای همین است که `errors.field_errors()` وقتی تودرتو می‌شوی کافی نیست: فقط گره‌هایِ `Field` را برمی‌گرداند و هر گره‌ی `Struct` و `List` را بی‌صدا کنار می‌گذارد. برایِ گزارشِ `first_note` و `notes[1]` باید خودت درخت را پیمایش کنی، و همین پیمایش اولین تمرینِ پیاده‌سازیِ توست.

### قاعده‌ی خودت: تابعی که `Result<(), ValidationError>` برمی‌گرداند

وقتی هیچ قاعده‌ی آماده‌ای جور نیست، قاعده یک تابعِ ساده است. یک ارجاع به مقدارِ فیلد می‌گیرد و `Ok(())` یا خطایی که خودت می‌سازی برمی‌گرداند، با یک `code` و، اگر خواستی، یک `message`:

```rust
fn no_spaces(value: &str) -> Result<(), ValidationError> {
    if value.contains(' ') {
        return Err(ValidationError::new("no_spaces").with_message("no spaces allowed".into()));
    }
    Ok(())
}
// on the field: #[validate(length(min = 3), custom(function = "no_spaces"))]
```

`examples/03-custom-rule.rs` سه نام را امتحان می‌کند. آخری، `"m "`، هر دو قاعده را رویِ همان فیلد می‌شکند، پس آن فیلد دو خطا می‌گیرد، و `Debug` `params`ی را که هر قاعده ثبت کرده نشان می‌دهد:

```text
"matin" -> Ok(())
"ma tin" -> Err(ValidationErrors({"name": Field([ValidationError { code: "no_spaces", message: Some("no spaces allowed"), params: {"value": String("ma tin")} }])}))
"m " -> Err(ValidationErrors({"name": Field([ValidationError { code: "length", message: None, params: {"min": Number(3), "value": String("m ")} }, ValidationError { code: "no_spaces", message: Some("no spaces allowed"), params: {"value": String("m ")} }])}))
```

### از درخت تا یک `422` با بدنه‌ی کلیدشده با فیلد

کلاینت می‌خواهد هر پیام را کنارِ ورودیِ درستش بگذارد، پس بدنه با نامِ فیلد کلیدشده است. این همان شکلِ `serializer.errors`ِ DRF است، با یک انتخاب برایِ داده‌ی تودرتو: اینجا نقطه والد را به فرزند وصل می‌کند (`reviewer.email`) و کروشه اندیسِ لیست را می‌گوید (`notes[1].text`) (کوتاه‌شده از یک پاسخِ واقعی که در «دست‌به‌کد» کامل می‌بینی):

```json
{"errors":{"reviewer.email":["email"],"notes[1].text":["note must be 1 to 200 characters"]}}
```

هر فیلد به یک *فهرست* از پیام‌ها می‌رسد (یکی برایِ هر قاعده‌ی شکسته)، و پیام همان `message`ِ قاعده است اگر داشته باشد، وگرنه `code`اش. کدِ وضعیت `422` است، به قاعده‌ای که ۳.۱.۳ داد و ۳.۲.۳ رویِ `InvalidRating` به کار برد: درخواست درست پارس شد، محتوا قاعده را شکست.

تبدیلش به پاسخ همان حرکتِ `AnimeError`ِ ۳.۲.۳ است: یک نوعِ خطایِ کوچک که `IntoResponse` را پیاده می‌کند، به‌اضافه‌ی `From<ValidationErrors>` تا `?` خودش تبدیل کند. این هندلرِ `examples/07-rating-server.rs` است:

```rust
async fn create(Json(input): Json<NewRating>) -> Result<(StatusCode, String), ApiError> {
    input.validate()?;
    Ok((StatusCode::CREATED, format!("saved {}", input.score)))
}
```

`input.validate()?` اگر هر قاعده‌ای شکسته باشد زود با یک `ApiError` برمی‌گردد، و بدنه‌ی هندلر بعد از آن می‌تواند به داده اعتماد کند. سیستمِ نوع این را نمی‌داند: `NewRating` پیش و پس از آن فراخوانی همان نوع است، پس فراموش‌کردنِ آن خط بی‌مشکل کامپایل می‌شود. `ValidatedJson<T>`ِ بخشِ «بساز» این خطر را برمی‌دارد، چون فراخوانی را به یک اکسترکتور می‌برد، و هندلری که یکی را بخواهد هرگز با داده‌ی بررسی‌نشده اجرا نمی‌شود.

---

## دست‌به‌کد

سه تا از مثال‌ها در «مفهوم» بالا اجرا شدند (`01`، `02`، `03`). `06` عادی اجرا می‌شود و فقط غلط است؛ `04` و `05` عمداً خراب‌اند و پشتِ featureِ `broken` هستند («خطاهایی که خواهی دید» را ببین). آن بی‌صدا را اجرا کن:

```sh
cargo run -p p3-03-02-validation --example 06-forgot-nested-trap
```

```text
outer validate: Ok(())
inner has errors: true
```

`Reviewer` یک قاعده‌ی `email` دارد و ایمیل درست نیست، با این حال `validate()`ِ بیرونی `Ok` می‌گوید. پایین‌تر به آن برمی‌گردیم.

`07` یک سرورِ واقعی رویِ `127.0.0.1:3110` است. روشنش کن (`cargo run -p p3-03-02-validation --example 07-rating-server` در ترمینالِ خودش) و چهار بدنه برایش بفرست. `-w` بعد از هر پاسخ کدِ وضعیت و نوعِ محتوا را چاپ می‌کند:

```sh
curl -s -w '\n%{http_code} %{content_type}\n' -X POST -H 'content-type: application/json' -d '{"score":9}' 127.0.0.1:3110/ratings
curl -s -w '\n%{http_code} %{content_type}\n' -X POST -H 'content-type: application/json' -d '{"score":15}' 127.0.0.1:3110/ratings
curl -s -w '\n%{http_code} %{content_type}\n' -X POST -H 'content-type: application/json' -d '{"score":"nine"}' 127.0.0.1:3110/ratings
curl -s -w '\n%{http_code} %{content_type}\n' -X POST -H 'content-type: application/json' -d '{' 127.0.0.1:3110/ratings
```

```text
saved 9
201 text/plain; charset=utf-8
{"errors":{"score":["score must be between 1 and 10"]}}
422 application/json
Failed to deserialize the JSON body into the target type: score: invalid type: string "nine", expected u8 at line 1 column 15
422 text/plain; charset=utf-8
Failed to parse the request body as JSON: EOF while parsing an object at line 1 column 1
400 text/plain; charset=utf-8
```

هر دوی `15` و `"nine"` `422` می‌دهند، ولی دو گذرِ متفاوت‌اند. اولی `ApiError`ِ *خودت* است (JSON، کلیدشده با فیلد)؛ دومی ردِ شکلِ خودِ `axum` است (متنِ ساده)، و `validate()` اصلاً اجرا نشد. کلاینت فقط با کدِ وضعیت این دو را از هم تشخیص نمی‌دهد، و این یکی از چیزهایی است که ۳.۸.۱ درست می‌کند. وقتی کارت تمام شد سرور را متوقف کن.

تمرین‌ها قرمز شروع می‌شوند. `src/lib.rs` کلِ اسکلت را دارد، با یک `todo!()` و یک doc comment که مشخصاتِ کاملِ هر تابع است:

```sh
cargo test -p p3-03-02-validation --test validate_test 2>&1 | grep 'test result'
```

```text
test result: FAILED. 0 passed; 10 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

وقتی همه‌ی تست‌ها سبز شدند، API کاملِ review (`POST /reviews` و `GET /reviews`) در `solution/` است، که crateِ جداگانه‌ی خودش را با `main`ِ خودش دارد. اجرایش کن (`cd solution && cargo run` در ترمینالِ خودش؛ رویِ `127.0.0.1:3111` گوش می‌دهد) و یک review سالم، یکی با یک قاعده‌ی شکسته، یکی که پنج قاعده را هم‌زمان می‌شکند، و یکی که یک فیلدش نیست بفرست:

```sh
U=127.0.0.1:3111/reviews; C='content-type: application/json'
curl -s -X POST -H "$C" -d '{"title":"Frieren","rating":9,"reviewer":{"handle":"matin_01","email":"matin@example.com"}}' $U
curl -s -X POST -H "$C" -d '{"title":"Frieren","rating":15,"reviewer":{"handle":"matin_01","email":"matin@example.com"}}' $U
curl -s -X POST -H "$C" -d '{"title":"","rating":0,"reviewer":{"handle":"a!","email":"nope"},"notes":[{"episode":1,"text":"ok"},{"episode":0,"text":"x"}]}' $U
curl -s -X POST -H "$C" -d '{"title":"x","rating":5}' $U
```

```text
{"id":1,"title":"Frieren","rating":9,"reviewer":"matin_01"}
{"errors":{"rating":["rating must be between 1 and 10"]}}
{"errors":{"notes[1].episode":["range"],"rating":["rating must be between 1 and 10"],"reviewer.email":["email"],"reviewer.handle":["handle may only contain letters, digits and underscores","handle must be 3 to 20 characters"],"title":["title must be 1 to 100 characters"]}}
Failed to deserialize the JSON body into the target type: missing field `reviewer` at line 1 column 24
```

پاسخِ سوم نکته‌ی کلِ درس است: یک درخواست، همه‌ی قاعده‌هایِ شکسته هم‌زمان گزارش شده، هر کدام زیرِ مسیرِ ورودیِ مسبب. دقت کن `reviewer.handle` دو پیام دارد، مرتب‌شده، و `notes[1].episode` می‌گوید `range` چون آن قاعده پیام ندارد. سرور را متوقف کن و اینها را امتحان کن:

۱. رتبه‌ی `10` و بعد `11` بفرست. کدام قبول می‌شود، و این در کدام نقطه‌ی سورس نوشته شده؟
۲. `"notes": []` بفرست. معتبر است؟ چه چیزی نامعتبرش می‌کرد؟
۳. یک review سالم بفرست، بعد یکی با عنوانِ بد، بعد دوباره یکی سالم. درخواستِ سوم چه شناسه‌ای می‌گیرد؟ (تستِ `a_rejected_review_is_not_stored_and_uses_up_no_id` همین ایده را بررسی می‌کند.)

---

## خطاهایی که خواهی دید

### `E0599` — `validate` رویِ نوعی که derive نشده

```rust
use validator::Validate;

struct Rating {
    score: u8,
}
```

`examples/04-missing-derive-broken.rs` یک `main` اضافه می‌کند که `r.validate()` را صدا می‌زند. با feature اجرایش کن:

```text
error[E0599]: no method named `validate` found for struct `Rating` in the current scope
  --> phase3-backend-foundations\03-serialization-and-validation\02-validation\examples\04-missing-derive-broken.rs:13:24
   |
 7 | struct Rating {
   | ------------- method `validate` not found for this struct
...
13 |     println!("{:?}", r.validate());
   |                        ^^^^^^^^ method not found in `Rating`
   |
   = help: items from traits can only be used if the trait is implemented and in scope
   = note: the following trait defines an item `validate`, perhaps you need to implement it:
           candidate #1: `Validate`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `p3-03-02-validation` (example "04-missing-derive-broken") due to 1 previous error
```

**کامپایلر به چه ایراد می‌گیرد:** `validate` متدِ صفتِ `Validate` است و `Rating` آن را پیاده نکرده. import کردنِ صفت (`use validator::Validate;`) کافی نیست؛ چیزی باید آن را برایِ `Rating` پیاده کند. خطوطِ `help` و `note` کامپایلر است که تنها صفتی را که می‌شناسد و متدی با این نام دارد فهرست می‌کند.

**راه‌حل:** آن را derive کن:

```rust
#[derive(Validate)]
struct Rating {
    #[validate(range(min = 1, max = 10))]
    score: u8,
}
```

**چرا این راه‌حل است:** derive همان چیزی است که `impl Validate for Rating` را می‌نویسد، با کدِ هر `#[validate(...)]`.

### `E0308` — قاعده‌ی دلخواهی که `Result` برنمی‌گرداند

```rust
fn no_spaces(value: &str) -> bool {
    !value.contains(' ')
}
// on the field: #[validate(custom(function = "no_spaces"))]
```

`examples/05-custom-wrong-signature-broken.rs` اتریبیوتِ فیلد را رویِ یک ساختارِ `Handle` دارد:

```text
error[E0308]: mismatched types
  --> phase3-backend-foundations\03-serialization-and-validation\02-validation\examples\05-custom-wrong-signature-broken.rs:11:10
   |
11 | #[derive(Validate)]
   |          ^^^^^^^^ expected `bool`, found `Result<_, _>`
   |
   = note: expected type `bool`
              found enum `Result<_, _>`
   = note: this error originates in the derive macro `Validate` (in Nightly builds, run with -Z macro-backtrace for more info)

For more information about this error, try `rustc --explain E0308`.
error: could not compile `p3-03-02-validation` (example "05-custom-wrong-signature-broken") due to 1 previous error
```

**کامپایلر به چه ایراد می‌گیرد:** span رویِ `#[derive(Validate)]` است، نه رویِ تابعِ تو، چون derive همان چیزی است که کدی نوشته که `no_spaces` را صدا می‌زند، و آن کد یک `Result<(), ValidationError>` پس می‌خواهد. «expected `bool`» را زیادی تفسیر نکن: نکته فقط این است که *امضا*ی تابعی که اتریبیوت نامش را برده غلط است، و خطا رویِ derive گزارش می‌شود.

**راه‌حل:** `Result<(), ValidationError>` برگردان، مثلِ «مفهوم»: `Ok(())` برایِ قبول، `Err(ValidationError::new("code"))` برایِ شکست.

**چرا این راه‌حل است:** قاعده یک محمول (predicate) نیست؛ باید بتواند بگوید *چرا* شکست خورد. همان `code` (و `message`ِ اختیاری) است که به درخت و بعد به بدنه‌ی JSON می‌رسد.

### هیچ خطایی نیست: ساختارِ تودرتویی که قاعده‌هایش هرگز اجرا نمی‌شوند

```text
outer validate: Ok(())
inner has errors: true
```

**آنچه واقعاً خراب است:** `examples/06-forgot-nested-trap.rs` کامپایل می‌شود و اجرا می‌شود. `Review` یک فیلدِ `reviewer: Reviewer` دارد و `Reviewer` یک قاعده دارد، ولی فیلد با `#[validate(nested)]` علامت نخورده. derive رویِ `Review` فقط قاعده‌هایی را اجرا می‌کند که رویِ فیلدهایِ خودِ `Review` می‌بیند؛ مگر خودت بگویی، به نوع‌هایِ دیگر نمی‌رسد. ایمیلِ بد بی‌صدا قبول می‌شود.

**راه‌حل:**

```rust
#[derive(Validate)]
struct Review {
    #[validate(length(min = 1))]
    title: String,
    #[validate(nested)]
    reviewer: Reviewer,
}
```

**چرا این راه‌حل است:** `nested` به derive می‌گوید `reviewer.validate()` را صدا بزند و شکستش را زیرِ کلیدِ `reviewer` (یک گره‌ی `Struct`) نگه دارد. کامپایلر نمی‌تواند هشدار بدهد، چون «هیچ قاعده‌ای رویِ این فیلد نیست» حالتی مجاز است؛ فقط تستی که یک مقدارِ تودرتوی بد می‌فرستد و `422` انتظار دارد آن را می‌گیرد. برای همین `tests/api_test.rs`ِ این درس یکی دارد.

---

## تمرین

### گرم‌کردن

<details>
<summary>بدنه‌ی یک <code>POST</code> یک JSONِ معتبر با <code>"rating": 15</code> است. کدام گذر ردش می‌کند، و چه چیزی جواب می‌دهد؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

گذرِ دوم. `15` یک `u8`ِ درست است، پس `Json<T>` شکل را قبول می‌کند؛ بعد `validate()` قاعده‌ی `range` را شکست می‌دهد، و `ApiError`ِ تو با `422` و شیء `errors`ِ کلیدشده با فیلد جواب می‌دهد.

</details>

<details>
<summary>بدنه <code>"rating": "nine"</code> دارد. آیا <code>validate()</code>ِ تو اجرا می‌شود؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

نه. `"nine"` یک `u8` نیست، پس `Json<T>` بدنه را در گذرِ اول رد می‌کند، با `422`ِ متنِ سادهٔ خودِ `axum`. قاعده‌هایِ تو فقط مقدارهایی را می‌بینند که از قبل نوع‌هایِ درست دارند.

</details>

<details>
<summary>لیستِ <code>notes</code>ِ یک review دو عنصر دارد و فقط دومی <code>text</code>ِ خالی دارد. پیام زیرِ کدام کلید می‌آید؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

`notes[1].text`. اندیس‌ها از `0` شروع می‌شوند، لیست `[1]` را اضافه می‌کند، و فیلدِ درونِ عنصر با نقطه وصل می‌شود. عنصرِ اول سالم است و در درخت نیست.

</details>

<details>
<summary>فیلدِ <code>note: Option&lt;String&gt;</code> قاعده‌ی <code>length(max = 5)</code> دارد. آیا <code>None</code> خطاست؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

نه. قاعده‌ی رویِ یک `Option` فقط وقتی اجرا می‌شود که `Some` باشد. اگر مقدار باید حتماً باشد، آن یک سؤالِ شکل است: فیلد را یک `String`ِ ساده کن، و کلیدِ غایب در گذرِ اول شکست می‌خورد.

</details>

<details>
<summary>چرا <code>errors.field_errors()</code> مشکلاتِ یک ساختارِ تودرتو را نمی‌دهد؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

فقط ورودی‌هایِ `Field`ِ سطحِ بالا را برمی‌گرداند. ساختارِ تودرتو یک گره‌ی `Struct` است و لیستی از ساختارها یک گره‌ی `List`، و هر دو فیلتر می‌شوند. به‌جایش `errors.errors()` را پیمایش می‌کنی.

</details>

### تعمیر

هر سه مثال را درست کن:

۱. `examples/04-missing-derive-broken.rs` با `--features broken` کامپایل شود. به `Rating` یک قاعده بده تا `15`ی که دارد رد شود.
۲. `examples/05-custom-wrong-signature-broken.rs` با `--features broken` کامپایل شود، و `"ma tin"` با کدِ `no_spaces` شکست بخورد.
۳. `examples/06-forgot-nested-trap.rs` چاپ کند `outer validate: Err(...)`. خطا را چاپ کن و بخوان شکست زیرِ کدام کلید است.

### پیاده‌سازی

هر چیزی در `src/lib.rs` که `todo!()` است: `validate_handle`، `flatten_errors`، `ApiError::into_response`، `create_review`، `list_reviews`، و `app`. قاعده‌هایِ رویِ نوع‌ها (`Reviewer`، `EpisodeNote`، `NewReview`) از قبل نوشته شده‌اند و `ReviewStore` دستت داده شده. هر doc comment مشخصاتِ کامل است (قالبِ کلید، ترتیبِ پیام‌ها، کدِ وضعیت، بدنه‌ی دقیق)، پس هرگز لازم نیست تست‌ها را باز کنی. به این ترتیب پیش برو: `validate_handle`، بعد `flatten_errors`، بعد لبه‌ی HTTP.

```sh
cargo test -p p3-03-02-validation --test validate_test
cargo test -p p3-03-02-validation --test api_test
```

`tests/validate_test.rs` (۱۰ تست) قاعده‌ی دلخواه و تخت‌کردن را با فراخوانیِ ساده بررسی می‌کند: بدونِ `axum`، بدونِ runtime، بدونِ درخواست. `tests/api_test.rs` (۶ تست) کلِ پشته را از راهِ `oneshot` بررسی می‌کند، همان‌طور که ۳.۲.۱ کرد.

### بساز

`ValidatedJson<T>` را بنویس، اکسترکتوری که بدنه را دقیقاً مثلِ `Json<T>` می‌خواند و بعد قاعده‌هایِ `T` را اجرا می‌کند، تا هندلری که `ValidatedJson<NewReview>` بخواهد هرگز داده‌ی بررسی‌نشده نبیند. امضا در `src/lib.rs` هست (بدنه‌اش یک `todo!()` است که *چه* را در doc comment بالایش می‌گوید). باید برایِ *هر* `T` کار کند، نه فقط `NewReview`، و نباید ردهایی را که `Json<T>` از قبل می‌دهد تغییر دهد: بدنه‌ی خراب هنوز `400` است، نبودنِ `Content-Type` هنوز `415`. وقتی `tests/build_test.rs` سبز شد، `create_review` را عوض کن تا `ValidatedJson<NewReview>` بگیرد و ببین `tests/api_test.rs` بدونِ خطِ صریحِ `validate()?` هنوز می‌گذرد.

```sh
cargo test -p p3-03-02-validation --test build_test
```

### چالش (اختیاری)

قاعده‌هایی که دو فیلد را هم‌زمان می‌بینند. این قاعده را به `NewReview` اضافه کن: رتبه‌ی `1` یا `10` باید همراهِ یک `body` حداقل ۲۰ نویسه‌ای بیاید («رتبه‌ی افراطی دلیل می‌خواهد»). اتریبیوتِ یک فیلد نمی‌تواند فیلدِ دیگر را ببیند، پس از قاعده‌ی سطحِ ساختارِ `validator` استفاده کن، `#[validate(schema(function = "..."))]` رویِ ساختار، با تابعی که `&NewReview` می‌گیرد و `Result<(), ValidationError>` برمی‌گرداند. یک بار اجرایش کن و چاپ کن `flatten_errors` برایِ یک شکست چه می‌دهد: در ۰٫۱۸٫۱ خطایِ سطحِ ساختار زیرِ کلیدِ `__all__` ثبت می‌شود، که نامِ فیلد نیست، پس تصمیم بگیر کلاینت به‌جایش چه ببیند. تستِ آماده‌ای نیست: خودت بنویس.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| قاعده‌ی اعتبارسنجی (validation rule) | قیدی که یک مقدارِ درست‌نوع باز هم باید رعایتش کند (`1..=10`، یک طول، یک ایمیل) | هر بدنه‌ی درخواست و هر پیکربندی |
| `#[derive(Validate)]` | از اتریبیوت‌هایِ `#[validate(...)]` متدِ `validate(&self) -> Result<(), ValidationErrors>` می‌سازد | نوع‌هایِ درخواست |
| درخت `ValidationErrors` | نقشه‌ی نامِ فیلد به گره‌هایِ `Field`، `Struct` یا `List` | تبدیلِ شکست‌ها به پاسخ |
| قاعده‌ی دلخواه (custom rule) | تابعی `fn(&T) -> Result<(), ValidationError>` که در `custom(function = "...")` نامش می‌آید | هر چیزی که قاعده‌ی آماده نمی‌تواند بگوید |
| `#[validate(nested)]` | اختیاری و صریح: قاعده‌هایِ یک ساختار، یا هر عنصرِ یک `Vec`، را هم اجرا کن | بدنه‌هایِ تودرتو |
| بدنه‌ی خطا با کلیدِ فیلد | `{"errors": {"path": ["message"]}}` با نقطه برایِ تودرتو و `[i]` برایِ عنصرِ لیست | هر `422`ی که API می‌فرستد |
| `ValidatedJson<T>` | اکسترکتوری که هم پارس می‌کند هم اعتبارسنجی، تا هندلرها فقط داده‌ی بررسی‌شده بگیرند | هر هندلری که بدنه می‌گیرد |

### الان می‌دانی

- پارس‌کردن و اعتبارسنجی دو گذرِ جدا هستند: `Json<T>` شکل را تصمیم می‌گیرد (`400`/`415`/`422`، متنِ ساده، پیش از کدِ تو)، `validate()` قاعده‌ها را (`422`، در بدنه‌ای که خودت طراحی می‌کنی).
- `#[derive(Validate)]` همه‌ی قاعده‌ها را اجرا می‌کند و همه‌ی شکست‌ها را هم‌زمان گزارش می‌دهد؛ قاعده‌ی رویِ `Option` فقط رویِ `Some` اجرا می‌شود.
- نتیجه درختی با سه نوعِ گره است، و `field_errors()` فقط برگ‌ها را نشان می‌دهد، پس داده‌ی تودرتو پیمایشِ خودت را می‌خواهد.
- قاعده‌ی دلخواه تابعی است که `Result<(), ValidationError>` برمی‌گرداند، و `nested` باید نوشته شود تا قاعده‌هایِ یک ساختار از راهِ والد اجرا شوند.
- بدنه‌ی `422`ِ کلیدشده با فیلد یک بار، در یک `impl IntoResponse`، ساخته می‌شود و از هر هندلری با `?` به آن می‌رسی.

### بعداً کامل‌تر می‌بینی

- **گذاشتنِ همین قیدها در یک سندِ API که خودکار تولید می‌شود، تا کلاینت و مستندات از هم جدا نیفتند**: [۳.۳.۳ — قراردادهایِ API و OpenAPI (`utoipa`)](../03-api-contracts-and-openapi/README.fa.md)
- **یک قالبِ خطایِ یکدست برایِ همه‌ی شکست‌ها، از جمله ردهایِ متنِ سادهٔ `axum` که امروز دیدی**: [۳.۸.۱ — پاکت‌هایِ خطایِ یکدست](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.fa.md)
- **ذخیره‌ی review اعتبارسنجی‌شده جایی که با راه‌اندازیِ دوباره از بین نرود**: [۳.۵.۳ — کاتالوگ انیمه، این‌بار متصل به Postgres](../../05-postgres-and-sqlx/03-anime-catalog-postgres-backed/README.fa.md)
- **تغییرِ یک قاعده بدونِ شکستنِ هر کلاینتی که هنوز شکلِ قدیمی را می‌فرستد**: [۳.۳.۴ — نسخه‌بندیِ API و تکاملش](../04-api-versioning-and-evolution/README.fa.md)
- **صفت‌هایِ اکسترکتوری که `ValidatedJson<T>` پیاده می‌کند، یک سطح بالاتر**: [۳.۲.۲ — نوشتنِ اکسترکتورِ خودت (`FromRequestParts`)](../../02-axum-and-rest-api-design/02-writing-your-own-extractor/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا `{"rating": 15}` را بخشی دیگر از برنامه رد می‌کند تا `{"rating": "nine"}`، و هر کدام چه جوابی می‌دهد؟
- `#[derive(Validate)]` چه تولید می‌کند، و چرا همه‌ی قاعده‌هایِ شکسته را گزارش می‌دهد و در اولی نمی‌ایستد؟
- چرا `errors.field_errors()` برایِ reviewی که یک `reviewer` درونش است کافی نیست؟
- چرا قاعده‌ی یک فیلدِ `Option` برایِ `None` می‌گذرد، و «این فیلد لازم است» را کجا می‌گویی؟
- وقتی `#[validate(nested)]` را فراموش می‌کنی چه اشتباه می‌شود، و چرا کامپایلر نمی‌تواند هشدار بدهد؟
- `ValidatedJson<T>` چه چیزی به هندلر می‌دهد که `input.validate()?` نمی‌دهد؟

---

## بیشتر

- [`validator` 0.18.1 در docs.rs](https://docs.rs/validator/0.18.1/validator/): فهرستِ کاملِ قاعده‌ها (`contains`، `must_match`، `regex`، ...) و APIِ `ValidationErrors`.
- [`Json` در مستنداتِ `axum` 0.8.9](https://docs.rs/axum/0.8.9/axum/struct.Json.html): جدولِ ردهایی که اعتبارسنجیِ تو پشتِ آن‌ها می‌نشیند.
- [RFC 9110 §15.5.21 — 422 Unprocessable Content](https://www.rfc-editor.org/rfc/rfc9110.html#name-422-unprocessable-content): کدِ وضعیت، به زبانِ خودِ استاندارد.
- [DRF — Serializer validation](https://www.django-rest-framework.org/api-guide/serializers/#validation): `validate_<field>()` و `validate()`، نسخه‌هایِ پایتونیِ قاعده‌ی دلخواه و قاعده‌ی سطحِ ساختار.
