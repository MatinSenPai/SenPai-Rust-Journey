# ۰۴.۱ — اتصال و connection pool

`DATABASES` در Django connection یا pool کوچک را پنهان مدیریت می‌کند. این درس همان مدیریت را با `sqlx` دستی می‌سازد تا connection pool یک setting کپی‌شده از tutorial نباشد.

## چرا pool، نه یک connection؟

ساخت TCP connection تازه برای هر request درست است اما handshake، TLS در صورت فعال‌بودن و authentication را پیش از query تکرار می‌کند. یک connection مشترک هم با دو task async هم‌زمان جواب نمی‌دهد؛ connection Postgres را نمی‌توان این‌گونه multiplex کرد.

pool میان این دو است: `N` connection واقعی را یک بار باز می‌کند، برای مدت query به task می‌دهد و پس می‌گیرد. `sqlx::PgPool` clone ارزان و `Send + Sync` است؛ یک بار در `main` یا test بساز و handle آن را پخش کن.

```rust
let pool = PgPoolOptions::new()
    .max_connections(5)
    .acquire_timeout(Duration::from_secs(3))
    .connect(database_url)
    .await?;
```

- `max_connections(5)` سقف connectionهای واقعی pool است. caller ششم منتظر آزادشدن یکی از پنج connection می‌ماند. کم‌بودن آن requestها را queue و زیادبودنش سقف Postgres را میان replicaها تمام می‌کند.
- `acquire_timeout(3s)` انتظار برای connection اولیه یا آزادشدن pool را محدود می‌کند؛ database unreachable یا overload درخواست را برای همیشه hang نمی‌کند.

## `.connect()` fail-fast است

`PgPoolOptions::connect(...)` پیش از return دست‌کم یک connection واقعی می‌سازد. host/password/database غلط همان‌جا `sqlx::Error` می‌دهد، نه روی نخستین query واقعی.

## test بدون credential hardcode

testها `DATABASE_URL` را از environment می‌گیرند، درست مانند settings امن Django. اجرا:

```sh
DATABASE_URL=postgres://taskforge:taskforge@localhost:5432/taskforge \
  cargo test -p p3-04-01-connecting-and-pooling -- --ignored
```

```senpai-visual
{"kind":"database","labels":["task ۱..۲۰","PgPool: ۵ connection","صف انتظار","PostgreSQL","بازگشت connection"]}
```

pool را مثل پنج کارت ورود به database ببین. مرز تشبیه: کارت بیشتر throughput نامحدود نمی‌دهد؛ server-side memory و `max_connections` هم سقف دارند.

## تمرین تو

`connect_pool` و `health_check` را پیاده کن، سپس testهای ignored را با `DATABASE_URL` اجرا کن.
