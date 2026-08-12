# ۰۴.۲ — migrationها

`makemigrations`/`migrate` در Django تغییرهای schema را فایل‌های versioned مرتب نگه می‌دارند، موردهای اجرا‌نشده را apply و اجراشده‌ها را ثبت می‌کنند. `sqlx::migrate!` همین کار را بدون تولید خودکار از model diff انجام می‌دهد: SQL را تو دستی می‌نویسی و sqlx ترتیب، tracking و اجرای امن دوباره را مدیریت می‌کند.

## خواندن `migrations/0001_create_widgets.sql`

```sql
CREATE TABLE p3_04_02_widgets (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

prefix عددی یا timestamp ترتیب apply را تعیین می‌کند. معمولاً `sqlx migrate add <name>` نام timestampدار درست را می‌سازد.

## `sqlx::migrate!("./migrations").run(pool)` چه می‌کند؟

۱. همه‌ی `.sql`های directory را در **compile time** می‌خواند و متنشان را در binary embed می‌کند.
۲. در runtime اگر لازم باشد table bookkeeping به نام `_sqlx_migrations` را با version، description، checksum و زمان اجرا می‌سازد.
۳. migrationها را به ترتیب می‌بیند: version ثبت‌نشده را در transaction اجرا و ثبت می‌کند؛ version ثبت‌شده را skip می‌کند.

پس اجرای دوم safe no-op است و خطای relation exists نمی‌دهد.

## چرا هر test `#[serial(p3_04_02_migrations_db)]` دارد؟

testهای Cargo به‌طور پیش‌فرض parallel‌اند اما هر سه test اینجا یک Postgres واقعی دارند و یکی table را `DROP` می‌کند. بدون `serial_test` یک test می‌تواند table را میان queryهای دیگری حذف کند و suite nondeterministic شود. tag یکسان، فقط همین group را serial می‌کند؛ نه همه testهای binary را.

## چرا SQL به `IF NOT EXISTS` نیاز ندارد؟

خود `_sqlx_migrations` idempotency را تضمین می‌کند. sqlx migration ثبت‌شده را دوباره اجرا نمی‌کند؛ SQL ساده‌تر و error واقعی double-application واضح‌تر می‌ماند.

## چرا database جدا؟

tableهای data را می‌توان prefix کرد، اما `_sqlx_migrations` یک table مشترک در هر database است که فقط version عددی را key می‌کند. `0001_create_widgets.sql` و migration version 1 capstone در `taskforge` checksum متفاوتی دارند؛ اجرای هر دو روی یک database به `VersionMismatch` می‌رسد و test reset ممکن است tracking capstone را خراب کند. برای همین یک بار database جدا بساز:

```sh
sudo -u postgres psql -c "CREATE DATABASE p3_04_02_migrations OWNER taskforge;"
```

```senpai-visual
{"kind":"database","labels":["0001.sql","_sqlx_migrations","transaction","schema جدید","اجرا دوم: skip"]}
```

history migration مثل دفتر ثبت تغییرات ساختمان است. مرز تشبیه: تغییر صفحه‌ی قدیمی دفتر harmless نیست؛ checksum از drift جلوگیری می‌کند.

## تمرین تو

`run_migrations`، `insert_widget` و `count_widgets` را کامل کن و test ignored را با database جدا اجرا کن.
