# پیشرفت

**در حال کار روی:** فاز صفر، درس یک — `phase0-setup/01-what-is-a-compiled-language`

> checkbox هر درس را در همان commit پایان درس ثبت کن. وقتی همه‌ی درس‌های یک فاز تمام شد tag مربوط به `phaseN-complete` را بساز.
>
> **این tracker فرعی است.** منبع اصلی UI وب با فرمان `cargo run -p course-ui` و فایل gitignored به نام `.course-progress.json` است. UI این فایل Markdown را تغییر نمی‌دهد؛ اگر هر دو را دستی استفاده کنی ممکن است از هم فاصله بگیرند. دلیل معماری در [`docs/adr/0001-web-ui-progress-state.md`](docs/adr/0001-web-ui-progress-state.md) آمده است.

<details open>
<summary><b>فاز صفر — آماده‌سازی و جهت‌یابی</b></summary>

- [ ] ۰۱ — زبان کامپایل‌شده چیست؟
- [ ] ۰۲ — نصب Rust و toolchainها
- [ ] ۰۳ — مبانی Cargo
- [ ] ۰۴ — ابزار‌ها: clippy، fmt و rust-analyzer
- [ ] ۰۵ — سلام Rust
- [ ] ۰۶ — گردش کار Git و مخزن

</details>

<details>
<summary><b>فاز یک — مبانی اصلی</b></summary>

**۰۱ — متغیر، نوع و control flow**
- [ ] ۰۱ — متغیر، mutability و shadowing
- [ ] ۰۲ — نوع‌های scalar و compound
- [ ] ۰۳ — شرط، loop و آشنایی با match

**۰۲ — مالکیت و حافظه**
- [ ] ۰۱ — معنای انتقال
- [ ] ۰۲ — `Clone` و `Copy`
- [ ] ۰۳ — `Drop` و RAII

**۰۳ — قرض‌گیری و ارجاع**
- [ ] ۰۱ — ارجاع اشتراکی و تغییرپذیر
- [ ] ۰۲ — قواعد borrow checker

**۰۴ — رشته و slice**
- [ ] ۰۱ — `String` در برابر `&str`
- [ ] ۰۲ — sliceها

**۰۵ — struct، enum و pattern matching**
- [ ] ۰۱ — struct و متد
- [ ] ۰۲ — enum و match
- [ ] ۰۳ — `if let` و `while let`

**۰۶ — مبانی `Option`، `Result` و خطا**
- [ ] ۰۱ — `Option` و ایمنی null
- [ ] ۰۲ — `Result` و operator `?`
- [ ] ۰۳ — `panic!` در برابر `Result`

- [ ] **ماموریت جانبی یک** — [CLI نقل‌قول انیمه](side-quests/sq-01-anime-quote-cli)
</details>

<details>
<summary><b>فاز دو — Rust میانی و اصطلاحی</b></summary>

**۰۱ — مجموعه‌ها**
- [ ] ۰۱ — `Vec` و `HashMap`
- [ ] ۰۲ — `BTreeMap`، `HashSet` و `VecDeque`

**۰۲ — iterator و closure**
- [ ] ۰۱ — closure و traitهای `Fn`
- [ ] ۰۲ — adapterهای iterator

**۰۳ — generic و trait**
- [ ] ۰۱ — تابع و struct generic
- [ ] ۰۲ — تعریف و پیاده‌سازی trait
- [ ] ۰۳ — شیء trait در برابر ارسال ایستا

**۰۴ — مدیریت خطا و طول عمر**
- [ ] ۰۱ — نوع‌های خطای سفارشی
- [ ] ۰۲ — `thiserror` و `anyhow`
- [ ] ۰۳ — مبانی طول عمر و elision

**۰۵ — اشاره‌گر هوشمند‌ها**
- [ ] ۰۱ — `Box` و تخصیص حافظه روی heap
- [ ] ۰۲ — `Rc` و `Arc`
- [ ] ۰۳ — `RefCell` و interior mutability

**۰۶ — سازمان‌دهی پروژه و آزمون**
- [ ] ۰۱ — module، visibility و workspace
- [ ] ۰۲ — unit، integration و doc آزمون
- [ ] ۰۳ — آشنایی با Rust دارای `unsafe`

**۰۷ — concurrency و async**
- [ ] ۰۱ — thread، `Mutex` و `Arc`
- [ ] ۰۲ — channel و message passing
- [ ] ۰۳ — Future و runtime
- [ ] ۰۴ — مبانی Tokio

**۰۸ — جعبه‌ابزار Rust**
- [ ] ۰۱ — pattern matching عمیق
- [ ] ۰۲ — مبانی `macro_rules!`
- [ ] ۰۳ — featureهای Cargo
- [ ] ۰۴ — `TryFrom` و تبدیل خطاپذیر

- [ ] **ماموریت جانبی دو** — [ربات مسابقه‌ی تلگرام](side-quests/sq-02-telegram-quiz-bot)
</details>

<details>
<summary><b>فاز سه — مبانی بک‌اند</b></summary>

**۰۱ — شبکه و HTTP از صفر**
- [ ] ۰۱ — TCP echo سرور
- [ ] ۰۲ — HTTP parser دست‌ساز

**۰۲ — `axum` و طراحی REST API**
- [ ] ۰۱ — route، پردازشگر و extractor
- [ ] ۰۲ — CRUD کاتالوگ انیمه در حافظه
- [ ] ۰۳ — CORS و اتصال frontend

**۰۳ — serialization و validation**
- [ ] ۰۱ — `serde_json` و `validator`

**۰۴ — PostgreSQL و `sqlx`**
- [ ] ۰۱ — اتصال و pool
- [ ] ۰۲ — migrationها
- [ ] ۰۳ — کاتالوگ انیمه با Postgres
- [ ] ۰۴ — transactionها

**۰۵ — طراحی database و کارایی query**
- [ ] ۰۱ — index، `EXPLAIN ANALYZE` و مسئله‌ی N+1
- [ ] ۰۲ — pagination: offset در برابر keyset

**۰۶ — authentication و security**
- [ ] ۰۱ — hash کردن password با `argon2`
- [ ] ۰۲ — JWT و middlewareهای `tower`

**۰۷ — خطا و آزمون در مقیاس**
- [ ] ۰۱ — envelope یکدست خطا
- [ ] ۰۲ — integration آزمون با `testcontainers`

- [ ] **ماموریت جانبی سه** — [سرویس اعلان وب‌تون](side-quests/sq-03-webtoon-notifier-service)
</details>

<details>
<summary><b>فاز چهار — بک‌اند پیشرفته و طراحی سیستم</b></summary>

- [ ] cache-aside، TTL و invalidation با Redis
- [ ] token bucket و `tower::limit`
- [ ] صف آزمایشی Postgres با `SKIP LOCKED`
- [ ] مفهوم broker در RabbitMQ، Kafka و NATS
- [ ] سرویس gRPC با `tonic`
- [ ] آشنایی با `async-graphql`
- [ ] logging ساختاریافته با `tracing`
- [ ] metric و Prometheus
- [ ] CAP، scaling، load balancing، idempotency و distributed locking
- [ ] Docker Compose و CI
- [ ] پیکربندی و secret
- [ ] benchmark با Criterion و flamegraph
- [ ] **ماموریت جانبی چهار** — [API تجمیع‌کننده‌ی انیمه/مانگا](side-quests/sq-04-anime-manga-aggregator-api)
</details>

<details>
<summary><b>Capstone — TaskForge</b></summary>

_نیمه‌ی یک: ارجاع core را مطالعه کن؛ ADRها و کد را بخوان و آزمون هر crate را اجرا کن._
- [ ] مطالعه‌ی ADRهای `capstone-taskforge/docs/adr/`
- [ ] `taskforge-core` — نوع‌های دامنه و ماشین حالت کار
- [ ] `taskforge-storage` — مخزن در Postgres و claim با `SKIP LOCKED`
- [ ] `taskforge-worker` — پردازشگر pool، retry/backoff و graceful shutdown
- [ ] `taskforge-scheduler` — کارهای دوره‌ای و صف dead-letter
- [ ] `taskforge-api` — REST، OpenAPI و metric
- [ ] `taskforge-admin-bot` — client تلگرام ChatOps
- [ ] `taskforge-cli` — client خط فرمان

_نیمه‌ی دو: لایه‌ی عملیات و `todo!()`ها را خودت بساز._
- [ ] `taskforge-api/src/main.rs` — pool، migration و serve با graceful shutdown
- [ ] `taskforge-worker/src/main.rs` — pool و تخلیه‌ی watch-channel هنگام shutdown
- [ ] `taskforge-scheduler/src/main.rs` — اجرای scheduleها تا shutdown
- [ ] `docker compose up --build` کل stack را پایدار بالا می‌آورد
- [ ] load آزمون در `loadtest/` سبز است و تحلیل تغییر‌های لازم در مقیاس ۱۰ برابر نوشته شده
</details>

<details>
<summary><b>فاز پنج — تسلط بر طراحی سیستم</b></summary>

**۰۱ — شبکه و protocol**
- [ ] تکامل HTTP و status کد‌ها
- [ ] ارتباط real-time: polling، SSE، WebSocket و webhook
- [ ] proxy، gateway و load balancer
- [ ] مقایسه‌ی REST، GraphQL، gRPC، SOAP و RPC

**۰۲ — database و storage در مقیاس**
- [ ] SQL در برابر NoSQL و انتخاب database
- [ ] transaction، ACID و isolation level
- [ ] sharding، partitioning و consistent hashing
- [ ] replication و read replica
- [ ] index عمیق: B-tree در برابر LSM-tree
- [ ] locking خوش‌بینانه در برابر بدبینانه
- [ ] message صف و event streaming

**۰۳ — cache و performance**
- [ ] strategyهای cache و eviction policy
- [ ] CDN و عدد‌های تأخیر مهم

**۰۴ — الگوهای سیستم توزیع‌شده**
- [ ] CAP theorem و consistency model
- [ ] idempotency و distributed locking
- [ ] retry، circuit breaker و timeout
- [ ] تولید ID یکتا در مقیاس
- [ ] scaling عمودی/افقی و partitioning

**۰۵ — security و authentication در مقیاس**
- [ ] encoding در برابر encryption و hashing
- [ ] HTTPS و TLS handshake
- [ ] مقایسه‌ی session، JWT، OAuth2 و SSO

**۰۶ — مبانی DevOps و cloud**
- [ ] Docker و container
- [ ] مبانی Kubernetes
- [ ] deployment آبی-سبز، canary و rolling
- [ ] Twelve-Factor App

**۰۷ — طراحی سیستم کاربردی**
- [ ] طراحی URL shortener
- [ ] طراحی distributed rate limiter
- [ ] طراحی chat system
- [ ] طراحی notification system
- [ ] طراحی distributed کار scheduler با بازگشت به TaskForge
</details>
