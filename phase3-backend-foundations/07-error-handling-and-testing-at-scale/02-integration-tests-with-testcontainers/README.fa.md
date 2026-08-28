# ۰۷.۲ — تست یکپارچه با `testcontainers`

تا اینجا درس‌های database با Postgres مشترک و بلندمدتِ `postgres://taskforge:taskforge@localhost:5432/taskforge` تست می‌شدند؛ سریع و ساده است اما فرض می‌کند database از قبل وجود دارد و schema درست است. `testcontainers` برای هر test یک Postgres واقعی و disposable در Docker بالا می‌آورد، تست را اجرا می‌کند و با پایان test آن container را نابود می‌کند: state مشترک و setup دستی از بین می‌رود.

```senpai-visual
{"kind":"database","labels":["test آغاز می‌شود","Postgres container تازه","SQL واقعی","Drop و پاک‌سازی"]}
```

## چرا همه‌جا test double درون‌حافظه‌ای نه؟

`Mutex<HashMap<...>>` برای unit testهای سریعِ منطق business عالی است، اما SQL واقعی نمی‌فرستد. typo در column، migration ناقص یا تفاوت Postgres development و production را نمی‌بیند. پروژهٔ backend سالم هر دو را دارد: تعداد زیاد unit test سریع و تعداد کمتر integration test کند با infrastructure واقعی.

## `testcontainers` چه می‌کند؟

```rust
let container = Postgres::default().start().await?;
let host = container.get_host().await?;
let port = container.get_host_port_ipv4(5432).await?;
let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");
```

`start()` image واقعی Postgres را pull/run می‌کند و تا آماده‌شدن صبر می‌کند. داخل container همیشه ۵۴۳۲ است، اما Docker روی host یک port آزادِ پویا می‌دهد؛ پس چند test موازی با هم collision ندارند. عمر container به `ContainerAsync` گره خورده است و هنگام Drop پاک می‌شود.

## چرا testها `#[ignore]` هستند؟

راه‌اندازی container به Docker daemon نیاز دارد. ممکن است CLI نصب باشد اما daemon اجرا نباشد، مانند این sandbox یا CI بدون Docker-in-Docker. `#[ignore]` باعث می‌شود `cargo test` بدون infrastructure سبز باشد و دارندهٔ Docker با `cargo test -- --ignored` اجرا کند. مهم: ignore فقط **اجرا** را رد می‌کند؛ همهٔ کد و typeها همچنان compile و type-check می‌شوند.

`WidgetRepository` همان `Widget` قبلی است اما با `sqlx::query_as` واقعی کار می‌کند و `#[derive(sqlx::FromRow)]` row را بر پایهٔ نام ستون‌ها decode می‌کند.

## تمرین

`run_schema`، `WidgetRepository::create`، `get` و `list` را در `src/lib.rs` کامل کن.

```sh
cargo test -p p3-07-02-integration-tests-with-testcontainers
cargo test -p p3-07-02-integration-tests-with-testcontainers -- --ignored
```

دستور دوم Docker در حال اجرا می‌خواهد. سپس `solution/SOLUTION.fa.md` را بخوان.
