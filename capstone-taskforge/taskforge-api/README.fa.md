# کریت `taskforge-api`

پوسته‌ی REST، ساخته شده روی فریم‌ورک `axum`.

| مسیر (Route) | احراز هویت | توضیحات |
|---|---|---|
| `GET /health` | ندارد | چکِ زنده بودن (liveness check) |
| `GET /metrics` | ندارد | خروجیِ متنی فرمتِ Prometheus |
| `GET /api-docs/openapi.json` | ندارد | سندِ OpenAPI 3.1 (پایین رو ببینید) |
| `POST /jobs` | با توکن (bearer) | ثبت جاب جدید — بدنه: `{"job_type": "...", "payload": {...}, "max_attempts": 5}` |
| `GET /jobs` | با توکن (bearer) | لیست جاب‌ها — `?job_type=...&limit=...` |
| `GET /jobs/{id}` | با توکن (bearer) | دریافت جزئیات یک جاب |
| `POST /jobs/{id}/cancel` | با توکن (bearer) | کنسل کردن یه جاب با وضعیت `Pending` یا `Retrying` |

## داکیومنتِ خودکارِ OpenAPI

کُلِ این API خودش، خودشو توصیف می‌کنه. کتابخونه‌ی `utoipa` یه داکیومنت OpenAPI 3.1 رو تو زمانِ کامپایل از روی صفت‌های `#[utoipa::path(...)]` روی هر هندلر و صفتِ `#[derive(ToSchema)]` روی هر نوعِ درخواست/پاسخ می‌سازه، و روتِر هم اونو بدون نیاز به احراز هویت تو مسیرِ `GET /api-docs/openapi.json` برمی‌گردونه. هر ابزارِ OpenAPIای که داری رو (مثل Swagger UI، واردکننده Postman، یا کدسازهای SDK) متصل کن به این مسیر تا یه داکیومنتِ تعاملی و همیشه به‌روز داشته باشی بدون اینکه نیاز باشه یه داکیومنتِ دستی رو نگه‌داری که بعداً از کدها فاصله بگیره. فایل `tests/api_test.rs` چک می‌کنه که این مسیر حتماً یه سندِ معتبر برگردونه که شامل مسیرهای `/jobs` باشه، تا داکیومنت با تغییرِ کدها به مرور زمان خراب و فاسد نشه.

تک‌تکِ پاسخ‌های خطا از یه قالبِ JSON یکسان پیروی می‌شن: `{"error": "message"}`، که توش نوعِ `JobError` (از کریت `taskforge-core`) به کدِ وضعیتِ مناسبِ HTTP تو فایل `src/error.rs` مپ شده — واسه دیدنِ درسی که این الگو ازش اومده فایلِ [`phase3-backend-foundations/07-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.fa.md`](../../phase3-backend-foundations/07-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.fa.md) رو ببین.

احراز هویت کلاً یه توکنِ Bearerِ مشترکه (`AppState::auth_token`) — که واسه یه API داخلی/ادمین که توسط `taskforge-admin-bot` و `taskforge-cli` استفاده می‌شه کاملاً کافیه، نه اینکه یه سیستمِ احراز هویت چند-مستاجره‌ی (multi-tenant) کامل باشه (دیدگاه‌ها و جایگزینِ JWT رو تو داکیومنتِ `src/auth.rs` ببين).

## ران کردن تست‌ها

```sh
cargo test -p taskforge-api
```

تک‌تکِ تست‌های `tests/api_test.rs` کُلِ روتِر رو تو خودِ پروسه (از طریق `tower::ServiceExt::oneshot` — بدون بایند کردن پورت TCP و بدون خطاهای اشغال بودن پورت) روی `InMemoryJobStore` ران می‌کنه. بدون دیتابیس، بدون Redis، بدون نیاز به استارت کردنِ هیچی از قبل.

## ران کردن واقعی

تابع `build_router` بالا در واقع درزِ کتابخونه‌ایه؛ فایل `src/main.rs` خودِ سرویسِ قابل‌اجراست که تنظیمات، دیتابیس Postgres، مایگریشن‌ها، و خاموش شدنِ مسالمت‌آمیز (graceful shutdown) رو دورش متصل می‌کنه. این باینری یه **تمرینِ اصلی از پروژه‌ی پایانی** هستش — با یک‌سری `todo!()` ارائه شده تا خودت کاملش کنی (بخش "چیزایی که تو می‌سازی" تو READMEِ capstone رو ببین). شکلی که تو باید بسازیش:

```rust,ignore
let store: Arc<dyn JobStore> = Arc::new(PostgresJobStore::connect(&database_url).await?);
let app = taskforge_api::build_router(store, auth_token);
let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await?;
```

وقتی تموم بشه، زدنِ `docker compose up --build` از دایرکتوری `capstone-taskforge/` اونو کنار worker، scheduler، و Postgres ران می‌کنه.
