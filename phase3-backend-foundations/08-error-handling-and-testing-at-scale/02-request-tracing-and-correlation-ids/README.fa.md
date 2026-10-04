# ۳.۸.۲ — ردیابیِ درخواست و correlation ID

## در یک نگاه

بعد از این درس می‌توانی:

- به هر درخواست یک شناسه بدهی: از هدرِ `x-request-id`ِ فراخواننده اگر امن بود، وگرنه خودت بسازی؛ و آن را در extensionهای درخواست، رویِ یک بازه‌یِ `tracing`، رویِ هدرِ پاسخ و در بدنه‌یِ خطا بگذاری.
- لاگی را بخوانی که درخواست‌های زیادی در آن به‌هم‌ریخته‌اند، با فیلتر کردن رویِ یک `request_id`، و توضیح بدهی چرا یک `tokio::spawn` داخلِ هندلر شناسه را گم می‌کند.
- خروجیِ لاگ را تست کنی: یک subscriber بگذاری که در حافظه می‌نویسد، و ادعا کنی شناسه در خطِ لاگ هست.
- تصمیم بگیری با شناسه‌یِ ورودیِ خالی، خراب یا بی‌اندازه بلند چه کنی، و بگویی چرا «به فراخواننده اعتماد کن» جوابِ غلط است.
- میان‌افزارِ دست‌نویس را با `SetRequestIdLayer` و `TraceLayer` و `PropagateRequestIdLayer` عوض کنی، و بگویی نسخه‌یِ آماده چه چیزی را به تو واگذار می‌کند.

**زمان:** حدود ۱۰۰ دقیقه · **پیش‌نیاز:**
[۳.۲.۳ — عملیاتِ CRUD روی کاتالوگِ انیمه (در حافظه)](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.fa.md)،
[۳.۲.۴ — `tower::Service` و `Layer`: میان‌افزار با دست](../../02-axum-and-rest-api-design/04-tower-service-and-layer-middleware/README.fa.md)،
[۳.۸.۱ — پاکت‌هایِ خطایِ یکدست](../01-consistent-error-envelopes/README.fa.md)

---

## چرا اهمیت دارد

ساعتِ سه بامداد یک کاربر می‌نویسد: «حدودِ ساعتِ سه صفحه خراب شد، خطا گرفتم.» سرورت در همان دقیقه چهارصد درخواست را جواب داده، و هرکدام یک خطِ مثلِ `loading from the database` لاگ کرده‌اند. فقط ساعت را داری و هیچِ چیزِ دیگری برایِ جست‌وجو نداری. کدام‌یک از آن چهارصد تا مالِ او بود؟

**شناسه‌ی هم‌بستگی (correlation ID)**، که در این درس همان شناسه‌یِ درخواست است، همین را جواب می‌دهد. سرور به هر درخواست یک رشته‌یِ کوتاه می‌دهد، آن را رویِ هر خطِ لاگی که آن درخواست می‌سازد می‌نویسد، و در یک هدر و در بدنه‌یِ خطا پس می‌فرستد. کاربر `request_id` را از خطا کپی می‌کند، تو لاگ را رویِ آن فیلتر می‌کنی، و کلِ داستانِ همان یک درخواست را می‌بینی: چه آمد، هندلر چه کرد، چطور تمام شد. توسعه‌دهنده‌یِ فرانت‌اند هم همین دستگیره را بی‌هزینه می‌گیرد: شناسه در همان پاسخی است که جلویِ چشمش است.

در جنگو سراغِ `django-guid` می‌روی یا یک logging filter می‌نویسی، و هر دو دو کارِ یکسان می‌کنند: شناسه را جایی می‌گذارند که لاگ ببیندش، و رویِ پاسخ پژواکش می‌دهند. این درس هر دو کار را با دست می‌سازد، رویِ میان‌افزاری که در [۳.۲.۴](../../02-axum-and-rest-api-design/04-tower-service-and-layer-middleware/README.fa.md) نوشتی، چون خودِ شناسه بخشِ سخت نیست. بخشِ سخت بقیه است: شناسه وقتی درخواست از میانِ کدِ async رد می‌شود کجا زندگی می‌کند، وقتی فراخواننده مقدارِ بی‌معنی بفرستد چه باید کرد، و یک خطِ لاگ را چطور باید تست کرد.

---

## مفهوم

### درخواست‌های زیاد، یک لاگ

`examples/01-logs-without-an-id.rs` دو درخواست را هم‌زمان به هندلری می‌فرستد که دو خط لاگ می‌کند. لاگ این است:

```text
 INFO loading from the database
 INFO loading from the database
 WARN the query was slow
 WARN the query was slow
200 OK and 200 OK
```

چهار خط، دو درخواست، و هیچ‌چیز نمی‌گوید کدام `WARN` مالِ کدام `INFO` است. خطوطِ هندلر به‌تنهایی اشکالی ندارند. چیزی که کم است چیزی است که **هر** خط با بقیه‌یِ خطوطِ درخواستِ خودش مشترک داشته باشد و با هیچ درخواستِ دیگری نه. (subscriber اینجا `tracing_subscriber::fmt()` است با رنگِ ANSI و زمان و مسیرِ ماژول خاموش، تا transcriptهای این درس خوانا و پایدار بمانند. `LogBuffer`ِ crate که پایین‌تر می‌آید هم همین کار را می‌کند.)

### یک شناسه، پنج جا

نقشه پنج قدم دارد و میان‌افزار همه‌شان را انجام می‌دهد، پس هیچ هندلری لازم نیست از شناسه خبر داشته باشد:

```senpai-visual
{"kind":"network","labels":["درخواست می‌رسد، شاید با x-request-id","میان‌افزار: قبولش کن یا شناسه بساز","شناسه به extensionهای درخواست می‌رود","هندلر داخلِ بازه‌ای اجرا می‌شود که شناسه را دارد","پاسخ x-request-id می‌گیرد و بدنه‌یِ خطا request_id"]}
```

۱. **انتخاب** شناسه: شناسه‌یِ فراخواننده اگر قابلِ‌قبول بود، وگرنه یک UUIDِ تازه.
۲. **ذخیره** در *extensionهای* درخواست، یعنی جیبِ تایپ‌دارِ کناریِ یک `http::Request` (`Parts`ای که یک اکسترکتورِ `FromRequestParts` از [۳.۲.۲](../../02-axum-and-rest-api-design/02-writing-your-own-extractor/README.fa.md) می‌گیرد آن را دارد)، تا هندلر بتواند بخواهدش.
۳. **برچسب‌زدنِ** کار با آن: یک بازه (span) باز کن که شناسه یکی از فیلدهایش باشد، و بقیه‌یِ درخواست را داخلِ همان بازه اجرا کن.
۴. **پژواک**: `x-request-id` را رویِ پاسخ بگذار.
۵. **جاسازی** در بدنه‌یِ خطا، کنارِ code و message.

قدم‌هایِ ۱ و ۲ و ۴ Rustِ ساده‌اند. قدمِ ۳ تازه است، پس اول می‌آید.

### رویداد و بازه

`tracing` دو چیز دارد که ثبتشان می‌کنی. **رویداد (event)** چیزی است که در یک لحظه اتفاق افتاده: `tracing::info!("loading from the database")`. همان است که قبلاً «خطِ لاگ» صدایش می‌کردی. **بازه (span)** یک تکه از زمان است با اسم و فیلد: شروع و پایان دارد، و هر رویدادی که وقتی بازه *وارد شده* (entered) ثبت شود با فیلدهایِ بازه مهر می‌خورد. یک **گیرنده (subscriber)** هر دو را می‌گیرد و تصمیم می‌گیرد چه کار کند؛ `tracing_subscriber::fmt()` چاپشان می‌کند.

میان‌افزار در `examples/02-id-in-the-span.rs` برایِ هر درخواست یک بازه باز می‌کند و بقیه‌یِ اپ را داخلِ آن اجرا می‌کند:

```rust
async fn with_request_id(request: Request, next: Next) -> Response {
    let id = request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("none")
        .to_string();
    let span = tracing::info_span!("request", request_id = %id);
    let mut response = next.run(request).instrument(span).await;
    response
        .headers_mut()
        .insert("x-request-id", HeaderValue::from_str(&id).unwrap());
    response
}
```

هندلر همان هندلرِ مثالِ ۱ است. هیچ شناسه‌ای در آن نیست. دو درخواست با شناسه‌هایِ `req-a` و `req-b`:

```text
 INFO request{request_id=req-a}: loading from the database
 INFO request{request_id=req-b}: loading from the database
 WARN request{request_id=req-b}: the query was slow
 WARN request{request_id=req-a}: the query was slow
echoed: "req-a" "req-b"
```

(ترتیبِ دو خطِ `WARN` ممکن است در اجراهایِ مختلف فرق کند، که دقیقاً همان موقعیتی است که شناسه برایش هست.) حالا هر خط با `request{request_id=...}:` شروع می‌شود، یعنی اسم و فیلدهایِ بازه، و هندلر هیچ کاری نکرد که این را به دست بیاورد.

دو نکته را آهسته بخوان. `request_id = %id` فیلد را با `Display` ثبت می‌کند، پس بدونِ گیومه چاپ می‌شود. یک `request_id = id`ِ ساده رویِ یک `&str` با `Debug` ثبت می‌شود و در گیومه چاپ می‌شود، که در `examples/05-ready-made-layers.rs` می‌بینی. و `.instrument(span)` کلِ ترفند است: یک **future** را می‌پیچد تا بازه هر بار که future `poll` می‌شود وارد شود و وقتی `poll` برمی‌گردد خارج شود. به future چسبیده است، نه به ریسمان، و برای همین از نقطه‌هایِ `.await` جان به‌در می‌برد، حتی وقتی `tokio` تسک را به ریسمانِ دیگری ببرد.

```senpai-visual
{"kind":"async","labels":["future poll می‌شود: بازه وارد می‌شود","هندلر به .await می‌رسد: بازه خارج می‌شود","تسک ممکن است به ریسمانِ دیگری برود","future دوباره poll می‌شود: بازه دوباره وارد می‌شود","هر رویدادِ میانِ این‌ها request_id دارد"]}
```

جایی که تصویرِ جنگو دقیق نیست: `django-guid` و یک logging filter شناسه را در یک `contextvars.ContextVar` نگه می‌دارند، و پایتون آن را در هر تسکِ `asyncio` که بسازی کپی می‌کند. `tracing` برایِ تسکی که با `tokio::spawn` ساخته شود هیچ‌چیز را خودکار کپی نمی‌کند. «خطاهایی که خواهی دید» خطی را نشان می‌دهد که شناسه‌اش را گم می‌کند.

### شناسه ورودیِ کاربر است

مثالِ ۲ به هدر اعتماد کرد. `unwrap_or("none")` حتی نقشه هم نیست: هر درخواستِ بدونِ هدر یک شناسه‌یِ مشترک به نامِ `none` می‌گیرد، که هدفِ کار را از بین می‌برد. و آخرین درخواستِ همان مثال نشان می‌دهد اعتماد به هدر چه هزینه‌ای دارد:

```text
--- a caller who sends a crafted id
 INFO request{request_id=x level=ERROR forged=true}: loading from the database
 WARN request{request_id=x level=ERROR forged=true}: the query was slow
```

فراخواننده یک مقدارِ هدر انتخاب کرد که فاصله و علامتِ `=` دارد، و لاگِ ما حالا فیلدهایی دارد که ما ننوشته‌ایم. هرکسی که این لاگ را بخواند، آدم باشد یا ابزاری که با فاصله می‌شکندش، `level=ERROR forged=true` را می‌بیند. این **تزریقِ لاگ (log injection)** است، و هدر دَرِ آن است. راهِ حل سه بخش دارد، و قاعده‌هایِ این درس همین‌هاست:

- **نبود:** شناسه بساز. یک UUIDِ نسخه‌ی ۴ (`67e55044-10b1-426f-9247-bb680e5fe0c8`) ۳۶ نویسه است و به هماهنگی میانِ سرورها نیازی ندارد.
- **قابلِ‌قبول:** نگهش دار. شاید یک gateway یا فرانت‌اند گذاشته باشد، و نگه‌داشتنش اصلِ کارِ یک شناسه‌یِ *هم‌بستگی* است: یک شناسه در چند سامانه.
- **هر چیزِ دیگر: عوضش کن، تعمیرش نکن.** قابلِ‌قبول یعنی ۱ تا ۶۴ بایتِ حرفِ ASCII، رقم، `-`، `_` و `.`. بلندتر، خالی، یا هر نویسه‌یِ دیگر، و سرور مقدارِ فراخواننده را نادیده می‌گیرد و خودش یکی می‌سازد. نمی‌برد، کوتاه نمی‌کند و نویسه‌ای حذف نمی‌کند: شناسه‌یِ تعمیرشده ممکن است با مالِ کسِ دیگری برخورد کند، و عوض‌کردنِ بی‌صدایِ چیزی که فراخواننده فرستاده غافلگیرکننده است.

سقف دلبخواهی نیست. یک شناسه در هر خطِ لاگِ درخواست، در یک هدرِ پاسخ و در یک بدنه کپی می‌شود. ۵٬۰۰۰ بایتِ شناسه که در بیست خط تکرار شود برایِ یک درخواست ۱۰۰ کیلوبایت لاگ است، و اندازه را فراخواننده انتخاب می‌کند. در `src/lib.rs` تابع‌هایِ `is_valid_request_id` و `resolve_request_id` و `error_body` تمرینِ اول‌اند، چون این سه تصمیم تابع‌هایِ خالص‌اند که بدونِ سرور تستشان می‌کنی.

`examples/03-server-with-an-id.rs` یک سرورِ واقعی است رویِ `127.0.0.1:3230` با همان میان‌افزار به‌علاوه‌یِ بررسی. در یک ترمینال راهش بینداز:

```sh
cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 03-server-with-an-id
```

```text
listening on http://127.0.0.1:3230
```

بعد در ترمینالِ دیگر، یک شناسه، بدونِ شناسه، و یک شناسه‌یِ خراب بفرست (`curl -i` هدرهایِ پاسخ را هم چاپ می‌کند):

```sh
curl -i -H 'x-request-id: abc-123' http://127.0.0.1:3230/anime/1
curl -i http://127.0.0.1:3230/anime/1
curl -i -H 'x-request-id: not valid!' http://127.0.0.1:3230/anime/1
```

```text
HTTP/1.1 200 OK
content-type: application/json
x-request-id: abc-123
content-length: 31
date: Sun, 04 Oct 2026 11:13:46 GMT

{"id":1,"title":"Cowboy Bebop"}
HTTP/1.1 200 OK
content-type: application/json
x-request-id: 7b2effea-0bf6-4cb9-aa49-14543468845a
content-length: 31
date: Sun, 04 Oct 2026 11:13:46 GMT

{"id":1,"title":"Cowboy Bebop"}
HTTP/1.1 200 OK
content-type: application/json
x-request-id: 7d1d0ab0-02c5-4523-8d8a-3a746090d303
content-length: 31
date: Sun, 04 Oct 2026 11:13:46 GMT

{"id":1,"title":"Cowboy Bebop"}
```

(شناسه‌هایِ ساخته‌شده و هدرِ `date` در هر اجرا فرق می‌کنند.) شناسه‌یِ معتبر بدونِ تغییر برمی‌گردد. دو تایِ دیگر یک UUIDِ تازه می‌گیرند. درخواست با شناسه‌یِ ۵٬۰۰۰نویسه‌ای (`-H "x-request-id: $(printf 'a%.0s' $(seq 1 5000))"`) هم یکی می‌گیرد. لاگِ خودِ سرور، که همان چیزی است که وقتی کاربر شناسه‌ای دستت می‌دهد فیلتر می‌کنی:

```text
 INFO request{request_id=abc-123 method=GET path=/anime/1}: looking up anime id=1
 INFO request{request_id=7b2effea-0bf6-4cb9-aa49-14543468845a method=GET path=/anime/1}: looking up anime id=1
 WARN ignoring a malformed x-request-id
 INFO request{request_id=7d1d0ab0-02c5-4523-8d8a-3a746090d303 method=GET path=/anime/1}: looking up anime id=1
```

به `WARN` دقت کن: پیشوندِ `request{...}` ندارد. در آن لحظه میان‌افزار هنوز شناسه‌ای انتخاب نکرده بود، پس هیچ بازه‌ای باز نبود. رویدادِ بیرونِ بازه شناسه ندارد، پس هرچه می‌خواهی با شناسه پیدا کنی باید داخلِ بازه لاگ شود. وقتی کارت تمام شد سرور را با Ctrl+C ببند.

### شناسه در extensionهای درخواست

میان‌افزار `request.extensions_mut().insert(RequestId(id.clone()))` را هم انجام می‌دهد، که `RequestId` یک structِ تک‌فیلدی در `src/lib.rs` است. هندلر با اکسترکتورِ `Extension` آن را می‌خواهد:

```rust
async fn whoami(Extension(id): Extension<RequestId>) -> String {
    id.0
}
```

```text
c1c82f17-6d11-4ac2-ba5e-e6f3511ea514
```

این بدنه‌یِ `curl http://127.0.0.1:3230/whoami` است. بیشترِ هندلرها هیچ‌وقت به این نیاز ندارند، چون بازه همین حالا خطوطِ لاگشان را برچسب می‌زند. به extension وقتی نیاز داری که شناسه باید جایی برود که خطِ لاگ نیست: در فراخوانی به سرویسِ دیگر (تا لاگِ *آن‌ها* هم همین شناسه را داشته باشد)، در کاری که صف می‌کنی، یا در بدنه‌یِ پاسخ. نوع `RequestId` است، نه `String`، چون extensionها با نوع کلید می‌خورند، و یک `String`ِ لخت آن‌جا با هر چیزِ دیگری که `String` ذخیره کند برخورد می‌کند.

### شناسه در بدنه‌یِ خطا

کاربر می‌تواند `x-request-id` را از تبِ network مرورگر کپی کند، اما بیشتر مردم هیچ‌وقت آن را باز نمی‌کنند. خطایِ داخلِ صفحه را می‌خوانند. پس شناسه در بدنه هم می‌رود، در همان شکلِ پاکتِ خطایی که [۳.۸.۱](../01-consistent-error-envelopes/README.fa.md) درباره‌اش است:

```text
{"error":{"code":"not_found","message":"anime 99 not found","request_id":"abc-404"}}
```

هندلر `Err(ApiError::not_found(...))` برمی‌گرداند و شناسه‌ای ندارد که در آن بگذارد. به‌جایِ اینکه شناسه را به هر هندلر بدهیم، `ApiError`ِ این درس بدنه‌یِ ساده را می‌سازد و code و message‌اش را رویِ extensionهایِ خودِ پاسخ می‌گذارد (`ErrorInfo`). میان‌افزار که شناسه را می‌داند آن را پیدا می‌کند و بدنه را با `error_body(...)` بازنویسی می‌کند. سرورِ بالا همین‌جا می‌ایستد: `404`اش هنوز این‌طور است، چون بازنویسی را در تمرینِ «بساز» اضافه می‌کنی:

```text
{"error":{"code":"not_found","message":"anime 99 not found"}}
```

شکست را هم لاگ کن: یک رویدادِ `WARN` با `code`ِ خطا برایِ ۴xx، و یک `ERROR` برایِ ۵xx، هر دو داخلِ بازه. آن‌وقت «کاربر `req-9f2` را چسباند» می‌شود یک فیلتر رویِ لاگ، و سطحِ ۴xx در برابرِ ۵xx باعث می‌شود وقتی لاگ را مرور می‌کنی ۵xx بیرون بزند.

### تستِ یک خطِ لاگ

هرچه بالا آمد در ترمینال چاپ می‌شود، و تست نمی‌تواند ترمینال بخواند. `tracing` اجازه می‌دهد برنامه با انتخابِ subscriber تعیین کند رویدادها کجا بروند، و `tracing_subscriber::fmt` اجازه می‌دهد آن subscriber در هر چیزی بنویسد که `MakeWriter` را پیاده کرده: صفتی (trait) با یک متد، `make_writer`، که هر بار رویدادی ثبت می‌شود یک `io::Write`ِ تازه تحویل می‌دهد. `LogBuffer` در `src/lib.rs` آماده است و کوچک است: یک دسته‌یِ `Clone`‌پذیر رویِ `Arc<Mutex<Vec<u8>>>` که هم `io::Write` را پیاده می‌کند (بایت‌ها را اضافه کن) و هم `MakeWriter` را (یک کپی از خودش را برگردان). `examples/04-capture-the-log.rs` از آن استفاده می‌کند:

```rust
let buffer = LogBuffer::new();
let guard = tracing::subscriber::set_default(buffer.subscriber());

let id = "abc-123";
let span = tracing::info_span!("request", request_id = %id);
span.in_scope(|| tracing::info!(status = 200, "finished"));
drop(guard);

println!("{:?}", buffer.contents());
println!("contains the id: {}", buffer.contents().contains("request_id=abc-123"));
```

```text
" INFO request{request_id=abc-123}: finished status=200\n"
contains the id: true
```

`set_default` subscriber را **فقط برایِ ریسمانِ فعلی** می‌گذارد و یک نگهبان (guard) برمی‌گرداند که وقتی drop شود برش می‌دارد. همین باعث می‌شود در تست امن باشد، چون `cargo test` تست‌ها را موازی و رویِ ریسمان‌هایِ جدا اجرا می‌کند: هر تست بافرِ خودش را دارد، و یک `#[tokio::test]` رویِ یک ریسمان اجرا می‌شود، پس یک guard کلِ درخواست را پوشش می‌دهد. `.init()`ِ سراسری که در مثال‌ها دیدی در هر پردازش فقط یک بار می‌شود صدایش زد، که برایِ `main` درست است و برایِ مجموعه‌یِ تست غلط.

### نسخه‌یِ آماده

چیزی را ساختی که `tower-http` در سه لایه می‌فرستد: `SetRequestIdLayer` و `TraceLayer` و `PropagateRequestIdLayer` (اینجا `tower-http` نسخه‌ی 0.6.11 حل شده، با featureهایِ `request-id` و `trace` که در `Cargo.toml`ِ این درس روشن‌اند). `examples/05-ready-made-layers.rs` آن‌ها را با `ServiceBuilder` کنارِ هم می‌چیند، که در [۳.۲.۴](../../02-axum-and-rest-api-design/04-tower-service-and-layer-middleware/README.fa.md) اولی‌اش بیرونی‌ترین لایه بود:

```rust
ServiceBuilder::new()
    .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
    .layer(trace)
    .layer(PropagateRequestIdLayer::x_request_id())
```

`SetRequestIdLayer` وقتی درخواست هدر ندارد آن را از یک تولیدکننده (`MakeRequestUuid`) پر می‌کند. `TraceLayer` برایِ هر درخواست یک بازه باز می‌کند و رویدادهایِ «started» و «finished» را لاگ می‌کند؛ `trace` بالا یک `TraceLayer` است که مثال تنظیمش می‌کند هدر را در فیلدِ `request_id`ِ بازه‌اش کپی کند. `PropagateRequestIdLayer` هدر را از درخواست به پاسخ کپی می‌کند. سه درخواست، همان‌هایِ قبل:

```text
 INFO request{request_id="abc-123" path=/anime}: started processing request
 INFO request{request_id="abc-123" path=/anime}: finished processing request latency=0 ms status=200
sent Some("abc-123") -> echoed "abc-123"
 INFO request{request_id="96e7832f-e45e-4386-bee1-9514143f7605" path=/anime}: started processing request
 INFO request{request_id="96e7832f-e45e-4386-bee1-9514143f7605" path=/anime}: finished processing request latency=0 ms status=200
sent None -> echoed "96e7832f-e45e-4386-bee1-9514143f7605"
 INFO request{request_id="not valid!" path=/anime}: started processing request
 INFO request{request_id="not valid!" path=/anime}: finished processing request latency=0 ms status=200
sent Some("not valid!") -> echoed "not valid!"
```

شناسه‌یِ ساخته‌شده، بازه، تأخیر و پژواک بی‌هزینه آمدند، و درخواستِ آخر نشان می‌دهد چه نیامد: `not valid!` مستقیم رد شد، هم به لاگ هم به پاسخ. `SetRequestIdLayer` هدرِ موجود را دست‌نخورده می‌گذارد، هرچه که باشد. پس لایه‌هایِ آماده قدم‌هایِ ۱ (تا حدی)، ۳ و ۴ را پوشش می‌دهند، و بقیه هنوز با کدِ توست: شناسه‌یِ ورودی را اعتبارسنجی کنی یا عوض کنی (سوراخِ تزریقِ لاگ باز است)، یک `RequestId`ِ تایپ‌دار برایِ هندلرها در extensionها بگذاری، و شناسه را در بدنه‌یِ خطا بنویسی. دو نسخه رقیب نیستند. یک سرویسِ واقعی اغلب لایه‌هایِ آماده را می‌گذارد و برایِ بررسی‌ها یک میان‌افزارِ کوچکِ خودش را اضافه می‌کند، که **چالش**ِ آخرِ درس است.

(پیش‌فرضِ `TraceLayer` در سطحِ `DEBUG` لاگ می‌کند، پس با یک subscriberِ پیش‌فرض چیزی نمی‌بینی. مثال سطح‌ها را رویِ `INFO` می‌گذارد تا خطوطِ بالا را بگیرد، و این یک دلیلِ رایجِ «`TraceLayer` گذاشتم و لاگم خالی است» است.)

### `x-request-id` یک قرارداد است

`x-request-id` در هیچ استانداردی نیست. پروکسی‌ها و ابرهایِ زیادی از آن استفاده می‌کنند، و همین آن را به چیزی تبدیل کرده که یک gateway می‌گذارد و سرویسِ تو نگه می‌دارد. برایِ ردیابی میانِ سرویس‌هایِ زیاد با زمان‌بندی یک استاندارد هست، W3C Trace Context (هدرِ `traceparent`)، و OpenTelemetry رویش ساخته شده. شناسه‌یِ درخواست نسخه‌یِ کوچکِ یک‌هدریِ همان ایده است، و هرچه اینجا یاد می‌گیری (بازه‌ای که شناسه را حمل می‌کند، میان‌افزاری که می‌سازدش، لاگی که فیلتر می‌کنی) به آن‌جا هم می‌رسد.

---

## دست‌به‌کد

```sh
cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 01-logs-without-an-id
cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 02-id-in-the-span
cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 04-capture-the-log
cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 05-ready-made-layers
```

بعد سرور رویِ پورتِ ۳۲۳۰، با `curl` از ترمینالِ دوم، همان‌طور که در «شناسه ورودیِ کاربر است» بود:

```sh
cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 03-server-with-an-id
```

و سه‌تایی که غلط می‌روند. `06` و `08` کامپایل می‌شوند و اجرا می‌شوند و در زمانِ اجرا غلط‌اند. `07` کامپایل نمی‌شود:

```sh
cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 06-spawn-loses-the-id
cargo build -p p3-08-02-request-tracing-and-correlation-ids --example 07-enter-across-await-broken --features broken
cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 08-extension-without-middleware
```

بعد این‌ها را امتحان کن:

۱. در `02-id-in-the-span` یک درخواستِ هم‌زمانِ سوم با شناسه‌یِ `req-c` اضافه کن. چند خط از `req-c` حرف می‌زنند؟
۲. وقتی سرورِ `03` بالاست `-H 'x-request-id: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'` (۶۴ تا `a`) و بعد ۶۵ تا بفرست. کدام نگه داشته می‌شود؟
۳. در `05-ready-made-layers` خطِ `.on_response(...)` را بردار و دوباره اجرا کن. کدام خطوط نیستند، و چرا؟

---

## خطاهایی که خواهی دید

### `E0277` — نگهبانِ بازه که `Send` نیست

`examples/07-enter-across-await-broken.rs` بازه را با `let _guard = span.entered();` واردشده نگه می‌دارد و بعد `await` می‌کند. خروجی، با اخطارهایِ `todo!()`ِ این درس حذف‌شده:

```text
error[E0277]: the trait bound `FromFn<..., (), ..., _>: Service<...>` is not satisfied
   --> phase3-backend-foundations\08-error-handling-and-testing-at-scale\02-request-tracing-and-correlation-ids\examples\07-enter-across-await-broken.rs:23:16
    |
 23 |         .layer(from_fn(with_request_id));
    |          ----- ^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
    |          |
    |          required by a bound introduced by this call
    |
    = help: the trait `tower_service::Service<axum::http::Request<Body>>` is not implemented for `FromFn<fn(Request<Body>, Next) -> ... {with_request_id}, (), ..., _>`
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
note: required by a bound in `Router::<S>::layer`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:306:21
    |
303 |     pub fn layer<L>(self, layer: L) -> Router<S>
    |            ----- required by a bound in this associated function
...
306 |         L::Service: Service<Request> + Clone + Send + Sync + 'static,
    |                     ^^^^^^^^^^^^^^^^ required by this bound in `Router::<S>::layer`
    = note: the full name for the type has been written to 'M:\SenPai-Rust-Journey\target\debug\examples\07_enter_across_await_broken.long-type-9151973810204884499.txt'
    = note: consider using `--verbose` to print the full type name to the console

error[E0277]: the trait bound `FromFn<..., (), ..., _>: Service<...>` is not satisfied
  --> phase3-backend-foundations\08-error-handling-and-testing-at-scale\02-request-tracing-and-correlation-ids\examples\07-enter-across-await-broken.rs:21:15
   |
21 |       let app = Router::new()
   |  _______________^
22 | |         .route("/", get(|| async { "ok" }))
23 | |         .layer(from_fn(with_request_id));
   | |________________________________________^ unsatisfied trait bound
   |
   = help: the trait `tower_service::Service<axum::http::Request<Body>>` is not implemented for `FromFn<fn(Request<Body>, Next) -> ... {with_request_id}, (), ..., _>`
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
   = note: the full name for the type has been written to 'M:\SenPai-Rust-Journey\target\debug\examples\07_enter_across_await_broken.long-type-9151973810204884499.txt'
   = note: consider using `--verbose` to print the full type name to the console

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-08-02-request-tracing-and-correlation-ids` (example "07-enter-across-await-broken") due to 2 previous errors
```

(عددِ نامِ فایلِ `long-type-` رویِ ماشینِ تو فرق دارد، و در هر بیلد هم عوض می‌شود.)

**کامپایلر به چه ایراد می‌گیرد:** پیام هیچ‌وقت `Send` را نمی‌گوید، و همین سختش می‌کند. می‌گوید `FromFn<...>` پیاده‌کننده‌یِ `Service` نیست، با فهرستی از نوع‌هایِ شبیه، و فراخوانیِ `.layer` به آن نیاز داشت. دلیل یک قدم دورتر است: `from_fn` فقط وقتی یک `Service` می‌سازد که futureِ میان‌افزار `Send` باشد، و futureای که مقداری را از یک `.await` عبور بدهد فقط وقتی `Send` است که آن مقدار باشد. `span.entered()` یک `EnteredSpan` برمی‌گرداند، که `tracing` عمداً `!Send` کرده (در سورسِ `tracing` نسخه‌ی 0.1.44 یک فیلدِ `PhantomNotSend` دارد). دو خطا گزارش می‌شود، یکی برایِ فراخوانیِ `.layer` و یکی برایِ عبارتِ `Router::new()` که آن را در خود دارد، و هر دو یک اشتباه‌اند.

**راهِ حل:** نگهبانِ بازه را از یک await عبور نده. به‌جایش future را بپیچ:

```rust
let span = tracing::info_span!("request", request_id = "abc-123");
next.run(request).instrument(span).await
```

**چرا این راهِ حل است:** نگهبان بازه را رویِ *همین ریسمان* وارد می‌کند و وقتی drop شود خارج می‌کند. در یک `.await` ممکن است تسک به ریسمانی برود که هیچ‌وقت وارد بازه نشده، یا تسک‌هایِ دیگر رویِ همین ریسمان اجرا شوند در حالی که نگهبان هنوز واردشده است، و رویدادها با شناسه‌یِ درخواستِ غلط مهر می‌خورند. `tracing` نگهبان را `!Send` می‌کند تا کامپایلر جلویت را بگیرد. `.instrument(span)` دورِ هر `poll`ِ future وارد و خارج می‌شود، مثلِ «رویداد و بازه». (یک `span.enter()`ِ ساده کامپایل می‌شود، چون نگهبانش بازه را قرض می‌گیرد و `Send` است، و به همان شکل غلط است. آن سخت‌تر به چشم می‌آید، چون هیچ‌چیز جلویت را نمی‌گیرد.)

### یک باگِ بی‌صدا: تسکِ spawnشده بازه ندارد

```text
 INFO request{request_id="abc-123"}: handler: queueing the welcome email
 INFO worker: sending the welcome email
```

**چه چیزی واقعاً خراب است:** `examples/06-spawn-loses-the-id.rs` بدونِ هیچ شکایتی کامپایل و اجرا می‌شود. خطِ هندلر شناسه دارد. خطِ تسکِ `tokio::spawn`شده ندارد، چون یک تسکِ spawnشده بدونِ بازه شروع می‌شود: futureای تازه است که هیچ‌چیز instrumentش نکرده. اگر این لاگ را رویِ `abc-123` فیلتر کنی، نیمی از ماجرا را می‌بینی، و نیمه‌ای که کم داری معمولاً همان است که شکست خورده.

**راهِ حل:** futureِ spawnشده را با بازه‌یِ فعلی instrument کن:

```rust
let worker = tokio::spawn(
    async { tracing::info!("worker: sending the welcome email") }
        .instrument(tracing::Span::current()),
);
```

**چرا این راهِ حل است:** `Span::current()` بازه‌ای است که هندلر همین الان در آن اجرا می‌شود، و `.instrument(...)` باعث می‌شود تسکِ تازه در هر `poll` وارد آن شود، درست مثلِ کاری که میان‌افزار برایِ کلِ درخواست می‌کند. با این تغییر مثال `INFO request{request_id="abc-123"}: worker: sending the welcome email` را چاپ می‌کند. یک بازه ممکن است این‌طور از درخواستش بیشتر عمر کند (یک تسکِ پس‌زمینه بازنگهش دارد)، که برایِ یک کارِ کوتاه اشکالی ندارد و برایِ کارِ بلند بد نیست بدانی. در پایتون `asyncio.create_task` متنِ `contextvars`ِ فعلی را در تسکِ تازه کپی می‌کند، پس همین اشتباه آنجا شناسه را گم نمی‌کرد. این یکی از جاهایی است که مقایسه می‌شکند.

### یک `500` در زمانِ اجرا: extension نیست

```text
status: 500 Internal Server Error
body: Missing request extension: Extension of type `p3_08_02_request_tracing_and_correlation_ids::RequestId` was not found. Perhaps you forgot to add it? See `axum::Extension`.
```

**چه چیزی واقعاً خراب است:** `examples/08-extension-without-middleware.rs` مسیرِ `/whoami` را، که هندلرش `Extension<RequestId>` می‌خواهد، از روتری سرو می‌کند که بدونِ میان‌افزاری که آن را بگذارد ساخته شده. نوع‌ها جور درمی‌آیند، پس کامپایلر راضی است. `axum` به هر چنین درخواستی با `500` و همین پیام جواب می‌دهد، که دقیق است، و شکست در هر درخواست رخ می‌دهد، نه موقعِ راه‌افتادن.

**راهِ حل:** مسیرها را از میان‌افزار سرو کن، همان‌طور که `app()` می‌کند: `routes().layer(from_fn(request_id_middleware))`. (این وقتی کار می‌کند که تمرینِ «بساز»ت تمام شده باشد؛ تا آن موقع `request_id_middleware` یک `todo!()` است. سرورِ مثالِ ۰۳ یک نسخه‌یِ کارکننده دارد.) اگر یک هندلر ممکن است هم با میان‌افزار هم بدونِ آن اجرا شود (مثلاً در تست)، به‌جایش `Option<Extension<RequestId>>` بگیر و تصمیم بگیر حالتِ نبودن چه معنایی دارد.

**چرا این راهِ حل است:** extension قولی است که چیزی زودتر در خط‌لوله داده، و اکسترکتورها نمی‌توانند قول‌ها را در زمانِ کامپایل بررسی کنند. آن `500` همان `axum` است که کارش را می‌کند: هندلر چیزی خواست که هیچ‌وقت گذاشته نشده بود. تستی که رویِ `app()` به هر مسیری که `Extension<RequestId>` استفاده می‌کند یک درخواست می‌فرستد، این را پیش از کاربر می‌گیرد.

---

## تمرین

### گرم‌کردن

<details>
<summary>کاربری خطایی گزارش می‌کند و <code>request_id</code>ِ بدنه‌یِ پاسخ را به تو می‌دهد. با آن چه می‌کنی، و چرا جواب می‌دهد؟</summary>

پیش از دیدنِ جواب خودت فکر کن.

</details>

<details>
<summary>جواب</summary>

لاگِ سرور را رویِ آن شناسه فیلتر می‌کنی. جواب می‌دهد چون بازه `request_id` را حمل می‌کند و هر رویدادی که داخلِ بازه ثبت شود، از میان‌افزار و هندلر و کدی که هندلر صدا می‌زند، با آن مهر می‌خورد. یک فیلتر خطوطِ آن درخواست را می‌دهد و هیچ درخواستِ دیگری را نه.

</details>

<details>
<summary>فراخواننده <code>x-request-id: ../../etc/passwd</code> می‌فرستد. سرور آن را نگه می‌دارد، و به‌جایش چه می‌کند؟</summary>

پیش از دیدنِ جواب خودت فکر کن.

</details>

<details>
<summary>جواب</summary>

نگهش نمی‌دارد: `/` بیرونِ نویسه‌هایِ مجاز است (حرف، رقم، `-`، `_`، `.`). سرور یک UUIDِ تازه می‌سازد و آن را در بازه، در extensionها، رویِ پاسخ و در بدنه‌یِ خطا به کار می‌برد. نویسه‌هایِ بد را حذف نمی‌کند و باقی را نگه نمی‌دارد: شناسه‌یِ عوض‌شده هیچ‌وقت حدسی درباره‌ی منظورِ فراخواننده نیست.

</details>

<details>
<summary>چرا یک تست <code>tracing::subscriber::set_default</code> را به‌جایِ <code>.init()</code> به کار می‌برد؟</summary>

پیش از دیدنِ جواب خودت فکر کن.

</details>

<details>
<summary>جواب</summary>

`.init()` یک subscriber برایِ کلِ پردازش می‌گذارد و فقط فراخوانیِ اول می‌تواند موفق شود، اما `cargo test` تست‌هایِ زیادی را هم‌زمان اجرا می‌کند. `set_default` یکی را برایِ ریسمانِ فعلی می‌گذارد و وقتی نگهبانش drop شود برمی‌دارد، پس هر تست بافرِ خودش را دارد و خطوطِ تستِ دیگر را نمی‌بیند.

</details>

<details>
<summary>هندلر <code>tokio::spawn</code> صدا می‌زند و تسکِ spawnشده لاگ می‌کند. آن خط <code>request_id</code> دارد؟</summary>

پیش از دیدنِ جواب خودت فکر کن.

</details>

<details>
<summary>جواب</summary>

نه. تسکِ تازه بدونِ بازه شروع می‌شود. futureاش را با `.instrument(tracing::Span::current())` بپیچ تا بازه‌یِ هندلر را بگیرد.

</details>

### تعمیر

دو تا از مثال‌ها را درست کن:

۱. `examples/06-spawn-loses-the-id.rs` خطِ worker را با `request_id="abc-123"` در ابتدایش چاپ کند.
۲. `examples/07-enter-across-await-broken.rs` کامپایل شود (با `--features broken` اجرایش کن) و باز هم خطوطِ لاگِ درخواست را با شناسه برچسب بزند.

### پیاده‌سازی

سه تابعِ کوچک در `src/lib.rs`، برایِ `tests/resolve_test.rs`: `is_valid_request_id` و `resolve_request_id` و `error_body`.

```sh
cargo test -p p3-08-02-request-tracing-and-correlation-ids --test resolve_test
```

کامنتِ مستنداتیِ بالایِ هرکدام کلِ مشخصات است: مجموعه‌یِ دقیقِ نویسه‌ها و سقفِ طول، کدام ورودی برنده است، چه وقت تولیدکننده صدا زده می‌شود و چه وقت نه، و شکلِ دقیقِ JSON. هیچ‌وقت لازم نیست تست‌ها را باز کنی. تابع‌هایِ خالص‌اند، پس نه سرور می‌خواهند نه runtime.

### بساز

`request_id_middleware` در `src/lib.rs`: میان‌افزارِ کامل، به شکلِ `Log` و `from_fn` از [۳.۲.۴](../../02-axum-and-rest-api-design/04-tower-service-and-layer-middleware/README.fa.md). شناسه را انتخاب می‌کند، در extensionها می‌گذارد، اپ را در یک بازه‌یِ `request` اجرا می‌کند، `finished` را لاگ می‌کند، هدرِ پاسخ را می‌گذارد، و بدنه‌یِ پاسخِ خطا را با شناسه بازنویسی می‌کند.

```sh
cargo test -p p3-08-02-request-tracing-and-correlation-ids --test middleware_test
```

باز هم کامنتِ مستنداتی مشخصات است، قدم‌به‌قدم. تست‌ها درخواست‌ها را با `oneshot` می‌فرستند، مثلِ ۳.۲.۱ به بعد، و لاگ را از راهِ `LogBuffer` می‌خوانند: یکی ادعا می‌کند شناسه در خطِ خودِ هندلر هست، یکی که شناسه‌یِ مخرب هیچ‌وقت به لاگ نمی‌رسد، یکی که یک ۴۰۴ در سطحِ `WARN` و یک ۵۰۰ در سطحِ `ERROR` لاگ می‌شود. وقتی سبز شد، `examples/03-server-with-an-id.rs` را با میان‌افزارِ خودت به‌جایِ کپیِ داخلش اجرا کن و درخواستِ `404` را دوباره بفرست.

### چالش (اختیاری)

همین رفتار را از لایه‌هایِ `tower-http`ِ `examples/05-ready-made-layers.rs` دوباره بساز، به‌علاوه‌یِ **یک** میان‌افزارِ `from_fn`ِ کوچکِ خودت که میانِ `SetRequestIdLayer` و بقیه می‌نشیند و شناسه‌یِ ورودیِ نامعتبر را عوض می‌کند. بنویس از پنج قدم کدام را لایه‌هایِ آماده بر عهده گرفتند و کدام را نتوانستی واگذار کنی. هیچ تستی ندارد. با درخواست‌هایِ `oneshot`ِ خودت بررسی کن: شناسه‌یِ هدرِ پژواک‌شده باید همان شناسه‌یِ لاگ باشد. اگر فرق داشتند، کدام ترتیبِ لایه‌ها توضیحش می‌دهد؟

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به کار می‌آید |
|---|---|---|
| correlation ID | یک رشته که یک درخواست را در هر خطِ لاگ، هدر و بدنه‌یِ خطا دنبال می‌کند | پیدا کردنِ یک درخواست در لاگِ شلوغ |
| `x-request-id` | هدرِ قراردادی که آن را حمل می‌کند، رفت و برگشت | gatewayها، فرانت‌اندها، سرویس‌هایِ دیگر |
| رویداد / بازه | رویداد یک لحظه است؛ بازه یک تکه از زمان با اسم و فیلد است | `tracing::info!` و `info_span!` |
| `.instrument(span)` | بازه را در هر `poll`ِ یک future وارد می‌کند | میان‌افزار، تسک‌هایِ spawnشده |
| subscriber | رویدادها و بازه‌ها را می‌گیرد و تصمیم می‌گیرد کجا بروند | `tracing_subscriber::fmt()` |
| `MakeWriter` | صفتی که به `fmt` می‌گوید کجا بنویسد | بافرِ حافظه برایِ تست‌ها |
| تزریقِ لاگ | مقداری که فراخواننده انتخاب می‌کند و در یک خطِ لاگ فیلدِ جعلی می‌سازد | اعتبارسنجیِ شناسه‌هایِ ورودی |
| extensionهای درخواست | جیبِ تایپ‌دارِ کناریِ درخواست | `Extension<RequestId>` |
| `SetRequestIdLayer` و بقیه | لایه‌هایِ آماده‌یِ `tower-http` | تولید، با اعتبارسنجیِ خودت |

### الان می‌دانی

- شناسه‌یِ درخواست وقتی امن باشد پذیرفته می‌شود، وقتی نباشد ساخته می‌شود، و وقتی خالی، خراب یا بلندتر از ۶۴ بایت باشد عوض می‌شود (هیچ‌وقت تعمیر نمی‌شود).
- میان‌افزار آن را در extensionها می‌گذارد، درخواست را داخلِ بازه‌ای که آن را حمل می‌کند اجرا می‌کند، پژواکش می‌دهد، و در بدنه‌یِ خطا می‌نویسدش.
- بازه به future می‌چسبد، نه به ریسمان: `.instrument` چطوری‌اش است، و `tokio::spawn` جایی است که گم می‌شود.
- `set_default` به‌علاوه‌یِ یک `MakeWriter` رویِ `Arc<Mutex<Vec<u8>>>` به تست اجازه می‌دهد لاگِ خودش را بخواند.
- `tower-http` بخش‌هایِ ساختن، ردیابی و پژواک را می‌دهد، و شناسه‌یِ ورودی را اعتبارسنجی نمی‌کند.

### بعداً کامل‌تر می‌بینی

- **تست‌هایی که اپ را رویِ ذخیره‌سازیِ واقعی می‌رانند** — [۳.۸.۳ — تستِ یکپارچگی با `testcontainers`](../03-integration-tests-with-testcontainers/README.fa.md)
- **داده‌یِ تستی که با دست نمی‌نویسی** — [۳.۸.۴ — factory و fixture برایِ داده‌یِ تست](../04-test-data-factories-and-fixtures/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا `request_id`ِ کاربر درخواستش را در لاگ پیدا می‌کند، و چه چیزی باعث می‌شود یک خط آن را داشته باشد؟
- سرور با یک `x-request-id`ِ ورودی کدام سه کار را می‌تواند بکند، و چرا «همان‌طور که هست نگهش دار» یکی از آن‌ها نیست؟
- چرا بازه به future چسبیده است و نه به ریسمان، و `tokio::spawn` با آن چه می‌کند؟
- یک تست چطور یک خطِ لاگ را می‌خواند، و چرا `set_default` را به‌جایِ `.init()` به کار می‌برد؟
- `SetRequestIdLayer` و `TraceLayer` و `PropagateRequestIdLayer` کدام بخش‌هایِ این درس را به تو می‌دهند، و کدام هنوز با خودت است؟
- `contextvars` در جنگو (`django-guid`) کجا مثلِ یک بازه رفتار می‌کند و کجا نه؟

---

## بیشتر

- [مستنداتِ span در `tracing`](https://docs.rs/tracing/0.1.44/tracing/span/index.html): وارد و خارج شدن، `Instrument`، و اینکه چرا نگهبان‌ها در برابرِ awaitها `!Send` هستند.
- [`tracing_subscriber::fmt::MakeWriter`](https://docs.rs/tracing-subscriber/0.3.23/tracing_subscriber/fmt/trait.MakeWriter.html): صفتی که این درس برایِ بافرِ حافظه پیاده می‌کند.
- [`tower_http::request_id`](https://docs.rs/tower-http/0.6.11/tower_http/request_id/index.html): سه لایه‌یِ شناسه‌یِ درخواست و قاعده‌هایِ ترتیبشان.
- [`tower_http::trace`](https://docs.rs/tower-http/0.6.11/tower_http/trace/index.html): `TraceLayer`، سطح‌هایِ پیش‌فرضش، و callbackهایی که می‌توانی عوض کنی.
- [W3C Trace Context](https://www.w3.org/TR/trace-context/): هدرِ استاندارد (`traceparent`) برایِ ردیابی میانِ سرویس‌ها، قدمِ بعد از شناسه‌یِ درخواست.
- [`django-guid`](https://github.com/snok/django-guid): بسته‌یِ جنگو که مقایسه‌یِ این درس درباره‌اش است؛ README‌اش را برایِ تنظیماتی که عرضه می‌کند بخوان.
