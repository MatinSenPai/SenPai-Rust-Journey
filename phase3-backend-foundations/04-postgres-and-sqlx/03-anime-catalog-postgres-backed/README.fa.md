# ۰۴.۳ — catalog انیمه متصل به PostgreSQL

همان domain درس CRUD درون‌حافظه‌ای—`Anime`، `WatchStatus`، `CreateAnime`، `UpdateAnime` و `AnimeError`—این بار با PostgreSQL واقعی. دو درس را کنار هم بخوان تا دقیقاً ببینی persistence چه کدی اضافه می‌کند و چه قراردادهایی را تغییر نمی‌دهد.

## چه چیزهایی مانند نسخه‌ی درون‌حافظه‌ای است؟

شکل عمومی `Anime`، inputها، errorها، signature همه handlerها و routeهای `app()` ثابت‌اند. frontend، test یا `curl` از سطح HTTP نمی‌فهمد `AnimeStore` پشتش `HashMap` است یا Postgres. storage implementation detail پشت methodهای store است، نه بخشی از API contract.

## چه چیزهایی عوض می‌شود؟

- همه‌ی methodهای `AnimeStore` اکنون `async`اند و error storage از Postgres را در `AnimeError::Storage(String)` می‌گذارند.
- `id` از `u64` به `i64` تغییر کرده؛ `BIGSERIAL` PostgreSQL auto-increment signed 64-bit است و unsigned counterpart ندارد.
- `WatchStatus` در Postgres به‌صورت `TEXT` نگه‌داری می‌شود؛ `as_str()` هنگام نوشتن و `WatchStatus::parse()` هنگام خواندن boundary conversion هستند. native `ENUM` type ایمنی بیشتری می‌دهد، اما برای variant تازه migration می‌خواهد و `TEXT` شروع ساده‌تری است.
- `update` و `delete` ابتدا `self.get(id).await?` را صدا می‌زنند تا existence check و `NotFound` تکرار نشود.

## setup و test هم‌زمان

```sh
DATABASE_URL=postgres://taskforge:taskforge@localhost:5432/taskforge \
  cargo test -p p3-04-03-anime-catalog-postgres-backed -- --ignored
```

testها table را create و پاک می‌کنند، اما همه باید `#[serial(p3_04_03_anime_db)]` یکسان داشته باشند؛ `store_test.rs` و `api_test.rs` دو binary جدا هستند و بدون tag مشترک یک test می‌تواند وسط query دیگری `DELETE` بزند.

```senpai-visual
{"kind":"database","labels":["CreateAnime","sqlx bind","PostgreSQL TEXT row","WatchStatus::parse","API ثابت"]}
```

این مانند تعویض دفتر موقت با آرشیو دائمی است. مرز تشبیه: database persistence می‌دهد، اما read-then-write مستقل هنوز race update دارد و خودکار حل نمی‌شود.

## تمرین تو

methodهای async CRUD، پنج handler و `app()` را پیاده کن و testهای ignored را اجرا کن.
