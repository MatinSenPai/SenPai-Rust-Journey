# ۰۵.۲ — صفحه‌بندی: offset در برابر keyset

`Paginator` در Django و `PageNumberPagination` در DRF، روی API خوش‌دستِ `OFFSET`/`LIMIT` سوار شده‌اند. DRF برای همین مسئله `CursorPagination` هم دارد: صفحه‌بندی offset وقتی جدول بزرگ و پررفت‌وآمد شود، دو ایراد پنهان دارد. این درس هر دو الگو را با SQL خام می‌سازد تا دقیق ببینی کجا و چرا می‌شکنند.

```senpai-visual
{"kind":"database","labels":["ردیف‌های تازه","cursor آخرین ردیف دیده‌شده","صفحهٔ بعد"]}
```

## مشکل اول: `OFFSET` کارِ ردشده‌ها را هم انجام می‌دهد

```sql
SELECT id, title, created_at FROM articles
ORDER BY created_at DESC, id DESC
LIMIT 20 OFFSET 100000;
```

Postgres نمی‌تواند مستقیم به ردیف ۱۰۰٬۰۰۱ بپرد. باید ترتیب را طی کند، صد هزار ردیف اول را تولید کند و دور بریزد، بعد ۲۰ ردیف نگه دارد. ایندکس هر ردیفِ ردشده را ارزان‌تر می‌کند، نه رایگان: صفحهٔ اول `O(20)` است و صفحهٔ ۵٬۰۰۰، `O(100020)`. هرچه عمیق‌تر بروی کندتر می‌شود.

## مشکل دوم: صفحه‌ها هنگام نوشتن جابه‌جا می‌شوند

offset یعنی «از ابتدای *وضعیت فعلی جدول* N ردیف رد شو». کاربر صفحهٔ اول را می‌بیند، ردیف تازه‌ای بالای جدول درج می‌شود، سپس صفحهٔ دوم را می‌خواهد: آخرین ردیف صفحهٔ اول دوباره برمی‌گردد. حذف هم اثر معکوس دارد و یک ردیف بی‌صدا جا می‌ماند. تست `offset_pages_drift_when_a_row_lands_between_requests_keyset_pages_do_not` همین تکرار را بازتولید می‌کند.

## صفحه‌بندی keyset یا cursor

به‌جای «N ردیف رد کن»، کلید مرتب‌سازی آخرین ردیف دیده‌شده را نگه دار و دقیقاً بعد از آن ادامه بده:

```sql
SELECT id, title, created_at FROM articles
WHERE (created_at, id) < ($1, $2)
ORDER BY created_at DESC, id DESC
LIMIT $3;
```

- `WHERE` به نقطهٔ ادامه *seek* می‌کند؛ با ایندکس `(created_at, id)` هر صفحه `O(limit)` است، چه صفحهٔ اول چه صفحهٔ ۵٬۰۰۰.
- درج ردیف بالاتر از cursor چیزی را جابه‌جا نمی‌کند: «قدیمی‌تر از چیزی که دیده‌ام» یک رابطهٔ داده‌ای است، نه یک شمارش.
- `(created_at, id) < ($1, $2)` مقایسهٔ tuple در Postgres است: همان `created_at < $1 OR (created_at = $1 AND id < $2)`.

چرا کلید مرکب؟ `created_at` یکتا نیست. با زمان‌های یکسان، `<` بقیهٔ ردیف‌های هم‌زمان را رد می‌کند و `<=` ردیف‌های دیده‌شده را تکرار می‌کند. `id` یکتا و non-null است؛ پس می‌فهمیم دقیقاً «بعد از این ردیف» کجاست.

## tokenِ cursor

کلاینت نباید زوج `(timestamp, id)` را بفهمد یا بسازد؛ یک token کدر می‌گیرد و برمی‌گرداند. این درس `"<timestamp_micros>:<id>"` را encode می‌کند. microsecond انتخاب شده چون `TIMESTAMPTZ` در Postgres همین دقت را نگه می‌دارد. در production معمولاً payload را base64 و با HMAC امضا می‌کنند تا کلاینت به قالب وابسته نشود یا cursor جعلی نسازد. decode مرز اعتماد است: `decode_cursor` برای دادهٔ خراب خطای تایپ‌شده می‌دهد، نه panic.

```json
{ "articles": [ ... ], "next_cursor": "1750000000000000:42" }
```

`next_cursor` همان کلید آخرین ردیف صفحه است؛ صفحهٔ خالی یعنی `None`. بهای keyset روشن است: پرش ارزان به «صفحهٔ ۳۷» و شمارش کل صفحه‌ها ندارد. برای infinite scroll و endpointهای فهرستی API، معمولاً همین معامله درست است.

## تمرین

در `src/lib.rs` چهار `todo!()` داری:

1. `encode_cursor` و `decode_cursor`، تابع‌های خالص و بدون database.
2. `page_by_offset` و `page_by_keyset`، با `sqlx::query_as` برای tupleهای `(i64, String, DateTime<Utc>)` و سپس `rows_to_articles`.

تست‌های `#[ignore]` را روی database مشترک اجرا کن؛ جدول `p3_05_02_articles` را پاک و seed می‌کنند:

```sh
DATABASE_URL=postgres://taskforge:taskforge@localhost:5432/taskforge \
  cargo test -p p3-05-02-pagination -- --ignored
```

## Checkpoint

اول `cargo test -p p3-05-02-pagination`، بعد `CHECKPOINT.fa.md` و سپس `solution/SOLUTION.fa.md`.
