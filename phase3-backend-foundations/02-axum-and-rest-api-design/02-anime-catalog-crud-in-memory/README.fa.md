# ۰۲.۲ — عملیات CRUD روی کاتالوگ انیمه (داخل مموری - in-memory)

## کنار هم قرار دادنِ قطعاتِ درس قبلی برای ساختن یه منبعِ (resource) واقعی

مسیرهایی (routes) که تو درس قبلی ساختیم اکثراً دموهای مستقل بودن (`/{greet/{name}`، `/echo`، `/counter`). اما این درس، دقیقاً همون قالب و شکلیه که هر منبعِ REST تو دنیای واقعی به خودش می‌گیره: **یه دونه ساختار (struct)، پنج تا عملیات اصلی، و یه جدولِ مسیر (route table)** — دقیقاً همون چیزی که تو DRF با `ModelViewSet` داربست‌بندی می‌کردی، که اینجا قراره همه رو دستی و از پایه بسازی تا با گوشت و پوستت حسشون کنی.

| تو دنیای DRF (یه `ModelViewSet` روی مدل `Anime`) | تو این درس می‌شه |
|---|---|
| مسیر `GET /anime/` ← وصل می‌شه به `()list.` | مسیر `GET /anime` ← وصل می‌شه به `list_anime` |
| مسیر `POST /anime/` ← وصل می‌شه به `()create.` | مسیر `POST /anime` ← وصل می‌شه به `create_anime` |
| مسیر `/{GET /anime/{id` ← وصل می‌شه به `()retrieve.` | مسیر `/{GET /anime/{id` ← وصل می‌شه به `get_anime` |
| مسیر `/{PATCH /anime/{id` ← وصل می‌شه به `()partial_update.` | مسیر `/{PATCH /anime/{id` ← وصل می‌شه به `update_anime` |
| مسیر `/{DELETE /anime/{id` ← وصل می‌شه به `()destroy.` | مسیر `/{DELETE /anime/{id` ← وصل می‌شه به `delete_anime` |
| ابزار `Anime.objects` از جنگو ORM | ساختارِ `AnimeStore` (امروز تو حافظه‌ست، تو ماژول ۴ می‌شه Postgres) |

تو این درس ذخیره‌گاهِ (store) ما واقعاً داخل حافظه‌ست (یعنی یه `HashMap` که پشتِ یه `Mutex` قفل شده، و همون ثانیه‌ای که اجرایِ برنامه متوقف بشه همه‌چیز از بین میره) — این یه انتخابِ کاملاً عمدیه. تو ماژول ۴ قراره دقیقاً همین قالب و پوسته رو برداری و بذاری روی یه دیتابیسِ Postgresِ واقعی، تا دقیقاً و مو به مو متوجه بشی کدوم بخش‌های یه APIِ CRUD مربوط به *طراحیِ HTTP و REST* (همین درس) هستن، و کدوم بخش‌هاش مربوط به بحث *ذخیره‌سازیِ مانا (persistence)* (ماژول بعدی). خود جدولِ مسیر (route table)، شکلِ ظاهریِ درخواست/پاسخ‌ها (request/response shapes) و حتی نحوه‌ی مدیریت خطاها، تو گذار به ماژول بعدی تقریباً هیچ تغییری نمی‌کنن.

## منطقِ ناب، و یه لایه‌یِ خیلی نازک واسه HTTP — باز هم همون الگو

ساختار `AnimeStore` که تو فایل `src/lib.rs` هست اصلاً نمی‌دونه `axum`، یا `Json` یا حتی کدهای وضعیتِ HTTP (status codes) چی هستن — این بخش کلاً کدِ خالصِ زبان Rustئه: یه `<HashMap<u64, Anime>` که با یه `Mutex` محافظت می‌شه، و دارای متدهای `create`/`get`/`list`/`update`/`delete` هستش که همگی خروجی‌شون از نوعِ `<Result<Anime, AnimeError>` در میاد. فایلِ تستِ `tests/store_test.rs` تمامِ این منطق رو مستقیماً بدون اینکه اصلاً پاشو تو دنیای HTTP بذاره تست می‌کنه — یعنی دقیقاً همون استراتژیِ «هسته‌یِ منطق رو بدون دست زدن به I/O تست کن» که از اولِ فاز ۳ تا الان تو تک‌تکِ درسا تکرار شده، حالا داره به جایِ اینکه رو یه تابع پیاده بشه، رو کُلِ یه موجودیتِ CRUD پیاده می‌شه.

در عوض **هندلرها (handlers)** فقط یه لایه‌ی نازکِ ترجمه هستن: میان و آرگومان‌ها رو به کمک استخراج‌کننده‌ها (extractors) از تو دلِ درخواست (request) در میارن، متدهای مربوط به store رو صدا می‌زنن، و در نهایت نتیجه‌یِ `Result` رو می‌گیرن و ترجمه‌اش می‌کنن به یه پاسخِ HTTP. فایلِ تستِ `tests/api_test.rs` هم میاد *کُلِ* پشته‌یِ سرورِ ما رو — از روتر بگیر تا استخراج‌کننده‌ها، هندلرها و خودِ store — با کمکِ `tower::ServiceExt::oneshot` اجرا می‌کنه؛ یعنی همون تکنیکی که تو درسِ قبلی هم دیدی.

## یک دونه مسیر، با چند تا متد

```rust
Router::new()
    .route("/anime", get(list_anime).post(create_anime))
    .route("/anime/{id}", get(get_anime).patch(update_anime).delete(delete_anime))
```

ویژگیِ جدید نسبت به درس قبلی اینه: تابعِ `(route(path, method_router.` تو ورودیش یدونه `MethodRouter` می‌گیره، و توابعی مثل `(get(handler` و امثالش می‌تونن به همدیگه **زنجیر بشن (chained)** — مثلاً `(get(list_anime).post(create_anime` هر دو تا متد رو فقط روی یک دونه مسیرِ واحد ثبت می‌کنه، به جای اینکه مجبور باشی دو بار متدِ `(...)route.` رو با همون مسیر ولی واسه متدهای مختلف صدا بزنی. این دقیقاً معادلِ همون کاریه که `ModelViewSet` تو DRF می‌کرد که از طریق یه روتر چندین فعلِ HTTP رو روی یه دونه الگویِ URL ثبت می‌کرد، منتهی اینجا به جای اینکه یه کلاس بیاد و پشت‌پرده برات تولیدش کنه، خودت به صورت صریح و خط‌به‌خط دارى می‌نویسیش.

## تبدیل کردنِ خطاهایِ مربوط به دامنه‌ی نرم‌افزار (domain error) به یه پاسخِ HTTP

```rust
pub enum AnimeError {
    NotFound,
    InvalidRating(u8),
}

impl IntoResponse for AnimeError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AnimeError::NotFound => (StatusCode::NOT_FOUND, "anime not found".to_string()),
            AnimeError::InvalidRating(r) => (
                StatusCode::BAD_REQUEST,
                format!("rating must be between 1 and 10, got {r}"),
            ),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
```

پیاده‌سازیِ خصیصه‌یِ `axum::response::IntoResponse` برای نوعِ خطایِ شخصی‌سازه‌شده‌ی خودت، دقیقاً همون چیزیه که به هندلرت اجازه می‌ده تا مستقیماً مقدارِ `<Result<Json<Anime>, AnimeError>` رو به عنوان خروجی برگردونه — اونوقت فریم‌ورک `axum` میاد و رو هر کدوم از اون گونه‌ها که برگرده، چه `Ok` و چه `Err`، خودش متدِ `()into_response.` رو صدا می‌زنه، و دیگه هیچ نیازی به نوشتنِ یه `match`ِ خسته‌کننده داخلِ خودِ هندلر نیست. این در واقع یه پیش‌نمایش کوچیک از همون مفهومِ «قالب‌های خطایِ یکدست (consistent error envelopes)» هست که تو ماژول ۷ قراره ببینیش — تو این درس، هر خطایی که رخ می‌ده تو خروجی به شکلِ `{"... :"error"}` درمیاد، که البته همون رویه و عادتیه که متد `exception_handler` تو DRF می‌خواست تو رو به سمتش ببره، اما اینجا ما فقط یه بار با دستِ خودمون ساختیمش تا دقیقاً ببینی این شکل و شمایلِ JSON از کجا میاد و چطور ساخته می‌شه.

## وظیفه‌ی تو

جاهای خالیِ `!()todo` تو فایل `src/lib.rs` رو پر کن:

- متدهایِ `create` / `get` / `list` / `update` / `delete` رویِ ساختارِ `AnimeStore` — که منطقِ خالص و بی‌نیاز از HTTPِ عملیات‌های CRUD رو شامل می‌شن.
- هندلرهای `create_anime` / `list_anime` / `get_anime` / `update_anime` / `delete_anime` — که هندلرهایِ مربوط به `axum` هستن، یعنی همون پوسته‌یِ نازکی که رو کدهای store کشیده شده.
- تابعِ `app` — یعنی همون جدولِ مسیری (route table) که وظیفه داره دو تا متد رو روی آدرس `/anime` و سه تا متدِ دیگه رو روی آدرسِ `/{anime/{id/` سیم‌کشی کنه.

## تو دنیای واقعی امتحانش کن

```sh
cargo run -p p3-02-02-anime-catalog-crud-in-memory &
curl -X POST -H 'content-type: application/json' \
  -d '{"title":"Frieren","status":"watching","rating":9}' http://127.0.0.1:3001/anime
curl http://127.0.0.1:3001/anime
curl -X PATCH -H 'content-type: application/json' -d '{"status":"completed"}' http://127.0.0.1:3001/anime/1
curl -X DELETE http://127.0.0.1:3001/anime/1
curl http://127.0.0.1:3001/anime/1   # الان دیگه باید 404 بده
```

## چک‌پوینت

اول `CHECKPOINT.md` رو بخون و جواب بده، بعد هم برو سراغ `solution/SOLUTION.md`.