# پیشرفت تو این دوره

**در حال کار روی:** فاز صفر، درس یک — `phase0-setup/01-what-is-a-compiled-language`

> چک‌باکسِ مربوط به هر درس رو دقیقاً تو همون کامیت (commit)ِ مربوط به پایانِ همون درس ثبت کن. وقتی که تموم درس‌های یه فاز به پایان رسید، تگِ مربوط به `phaseN-complete` رو بساز.
>
> **این چک‌لیست صرفاً یه ابزارِ پیگیریِ فرعیه.** منبعِ اصلی و واقعی، رابطِ وبِ سیستم هستش که با دستورِ `cargo run -p course-ui` میاد بالا و دیتاش رو تو فایلِ `.course-progress.json` ذخیره می‌کنه که تو گیت نادیده گرفته شده. رابطِ وب اصلاً این فایلِ Markdown رو دستکاری نمی‌کنه؛ اگه از هر دوتای اینا به صورت دستی استفاده کنی ممکنه اطلاعاتشون با هم هماهنگ نباظرن. دلیلِ معماریِ این تصمیم تو فایلِ [`docs/adr/0001-web-ui-progress-state.fa.md`](docs/adr/0001-web-ui-progress-state.fa.md) اومده.

<details open>
<summary><b>فاز صفر — آماده‌سازی و جهت‌یابی</b></summary>

- [ ] ۰۱ — زبان کامپایلی یعنی چه؟
- [ ] ۰۲ — نصبِ Rust و زنجیره‌ابزارها
- [ ] ۰۳ — سلام Rust
- [ ] ۰۴ — مبانی Cargo
- [ ] ۰۵ — خواندنِ خطاهای کامپایلر
- [ ] ۰۶ — ابزارها: clippy، fmt و rust-analyzer
- [ ] ۰۷ — گیت و گردشِ کارِ مخزن

</details>

<details>
<summary><b>فاز یک — مفاهیم بنیادین</b></summary>

**۰۱ — متغیرها، نوع‌ها و کنترل جریان**
- [ ] ۰۱ — متغیر، mutability و shadowing
- [ ] ۰۲ — نوع‌های scalar و compound
- [ ] ۰۳ — شرط، loop و آشنایی با match

**۰۲ — مالکیت و حافظه**
- [ ] ۰۱ — معنای انتقال (Move)
- [ ] ۰۲ — `Clone` و `Copy`
- [ ] ۰۳ — `Drop` و RAII

**۰۳ — قرض‌گیری و ارجاع**
- [ ] ۰۱ — ارجاع اشتراکی و تغییرپذیر
- [ ] ۰۲ — قواعد borrow checker

**۰۴ — رشته و slice**
- [ ] ۰۱ — `String` در برابر `&str`
- [ ] ۰۲ — اسلایس‌ها (slice)

**۰۵ — struct، enum و pattern matching**
- [ ] ۰۱ — struct و متدها
- [ ] ۰۲ — enum و match
- [ ] ۰۳ — `if let` و `while let`

**۰۶ — مبانی `Option`، `Result` و خطا**
- [ ] ۰۱ — `Option` و ایمنی null
- [ ] ۰۲ — `Result` و عملگر `?`
- [ ] ۰۳ — `panic!` در برابر `Result`

- [ ] **مأموریت جانبی یک** — [CLI نقل‌قول انیمه](side-quests/sq-01-anime-quote-cli)
</details>

<details>
<summary><b>فاز دو — Rust میانی و اصطلاحی</b></summary>

**۰۱ — مجموعه‌ها**
- [ ] ۰۱ — `Vec` و `HashMap`
- [ ] ۰۲ — `BTreeMap`، `HashSet` و `VecDeque`

**۰۲ — iterator و closure**
- [ ] ۰۱ — closure و traitهای `Fn`
- [ ] ۰۲ — آداپتورهای iterator

**۰۳ — generic و trait**
- [ ] ۰۱ — توابع و structهای generic
- [ ] ۰۲ — تعریف و پیاده‌سازی trait
- [ ] ۰۳ — شیء trait در برابر ارسال ایستا

**۰۴ — مدیریت خطا و طول عمر**
- [ ] ۰۱ — نوع‌های خطای سفارشی
- [ ] ۰۲ — `thiserror` و `anyhow`
- [ ] ۰۳ — مبانی طول عمر و elision

**۰۵ — اشاره‌گرهای هوشمند**
- [ ] ۰۱ — `Box` و تخصیص حافظه روی heap
- [ ] ۰۲ — `Rc` و `Arc`
- [ ] ۰۳ — `RefCell` و interior mutability

**۰۶ — سازمان‌دهی پروژه و تست‌نویسی**
- [ ] ۰۱ — module، visibility و workspace
- [ ] ۰۲ — تست‌های unit، integration و doc
- [ ] ۰۳ — آشنایی با Rust دارای `unsafe`

**۰۷ — همروندی و async**
- [ ] ۰۱ — thread، `Mutex` و `Arc`
- [ ] ۰۲ — channel و message passing
- [ ] ۰۳ — Future و runtime
- [ ] ۰۴ — مبانی Tokio

**۰۸ — جعبه‌ابزار Rust**
- [ ] ۰۱ — pattern matching عمیق
- [ ] ۰۲ — مبانی `macro_rules!`
- [ ] ۰۳ — featureهای Cargo
- [ ] ۰۴ — `TryFrom` و تبدیل‌های خطاپذیر

- [ ] **مأموریت جانبی دو** — [ربات مسابقه‌ی تلگرام](side-quests/sq-02-telegram-quiz-bot)
</details>

<details>
<summary><b>فاز سه — مبانی پایه‌ای بک‌اند</b></summary>

**۰۱ — شبکه و HTTP از صفر**
- [ ] ۰۱ — سرور TCP echo
- [ ] ۰۲ — پارسر HTTP دست‌ساز

**۰۲ — `axum` و طراحی REST API**
- [ ] ۰۱ — route، هندلر و extractor
- [ ] ۰۲ — CRUD کاتالوگ انیمه تو حافظه
- [ ] ۰۳ — CORS و اتصال به فرانت‌اند

**۰۳ — سریالایز و اعتبارسنجی**
- [ ] ۰۱ — `serde_json` و `validator`

**۰۴ — PostgreSQL و `sqlx`**
- [ ] ۰۱ — اتصال و پُول (pool)
- [ ] ۰۲ — مایگریشن‌ها
- [ ] ۰۳ — کاتالوگ انیمه با Postgres
- [ ] ۰۴ — تراکنش‌ها (transactions)

**۰۵ — طراحی دیتابیس و پرفورمنس کوئری**
- [ ] ۰۱ — ایندکس، `EXPLAIN ANALYZE` و مسئله‌ی N+1
- [ ] ۰۲ — صفحه‌بندی: offset در برابر keyset

**۰۶ — احراز هویت و امنیت**
- [ ] ۰۱ — هش کردن پسورد با `argon2`
- [ ] ۰۲ — JWT و میدل‌ورهای `tower`

**۰۷ — مدیریت خطا و تست تو مقیاس بالا**
- [ ] ۰۱ — پاکتِ یکدستِ خطا (consistent envelopes)
- [ ] ۰۲ — تست‌های یکپارچگی با `testcontainers`

- [ ] **مأموریت جانبی سه** — [سرویس اعلان وب‌تون](side-quests/sq-03-webtoon-notifier-service)
</details>

<details>
<summary><b>فاز چهار — بک‌اند پیشرفته</b></summary>

- [ ] الگوی cache-aside، TTL و انقضا با Redis
- [ ] محدودکننده نرخ token bucket و `tower::limit`
- [ ] صف اسباب‌بازی Postgres با `SKIP LOCKED`
- [ ] مفاهیم broker تو RabbitMQ، Kafka و NATS
- [ ] سرویس gRPC با `tonic`
- [ ] آشنایی با `async-graphql`
- [ ] لاگ‌زنی ساخت‌یافته با `tracing`
- [ ] متریک‌ها و Prometheus
- [ ] قضیه CAP، مقیاس‌پذیری، متعادل‌سازی بار، idempotency و قفل‌های توزیع‌شده
- [ ] داکر کامپوز و CI
- [ ] تنظیمات و سکرت‌ها
- [ ] بنچمارک با Criterion و flamegraph
- [ ] **مأموریت جانبی چهار** — [API تجمیع‌کننده‌ی انیمه/مانگا](side-quests/sq-04-anime-manga-aggregator-api)
</details>

<details>
<summary><b>پروژه‌ی پایانی — TaskForge</b></summary>

_نیمه‌ی اول: کدهای اصلی مرجع رو مطالعه کن؛ ADRها و کدهای پروژه رو بخون و تست‌های هر کریت رو اجرا کن._
- [ ] مطالعه‌ی ADRهای موجود تو مسیر `capstone-taskforge/docs/adr/`
- [ ] `taskforge-core` — نوع‌های دامنه و ماشین حالتِ کارها
- [ ] `taskforge-storage` — ذخیره‌سازی تو Postgres و برداش با `SKIP LOCKED`
- [ ] `taskforge-worker` — استخر پردازش‌گرها، retry/backoff و graceful shutdown
- [ ] `taskforge-scheduler` — کارهای دوره‌ای و صف کارهای مرده (dead-letter)
- [ ] `taskforge-api` — اندپوینت‌های REST، OpenAPI و متریک‌ها
- [ ] `taskforge-admin-bot` — کلاینتِ تلگرامیِ ChatOps
- [ ] `taskforge-cli` — کلاینتِ خطِ فرمان

_نیمه‌ی دوم: لایه‌ی عملیاتی و کدهای مربوط به `!()todo`ها رو خودت بساز._
- [ ] `taskforge-api/src/main.rs` — پُول، مایگریشن و serve با graceful shutdown
- [ ] `taskforge-worker/src/main.rs` — پُول و تخلیه‌ی watch-channel موقع خاموش شدن
- [ ] `taskforge-scheduler/src/main.rs` — اجرای زمان‌بندی‌ها تا زمان خاموش شدن
- [ ] دستور `docker compose up --build` کُلِ سیستم رو به صورت پایدار بالا میاره
- [ ] تست فشار تو مسیر `loadtest/` سبزه و تحلیلِ تغییراتِ لازم واسه مقیاس ۱۰ برابر نوشته شده
</details>

<details>
<summary><b>فاز پنج — تسلط بر طراحی سیستم</b></summary>

**۰۱ — شبکه و پروتکل‌ها**
- [ ] تکامل HTTP و کدهای وضعیت
- [ ] ارتباطات همزمان (real-time): polling، SSE، WebSocket و webhook
- [ ] پروکسی، API Gateway و متعادل‌کننده بار (load balancer)
- [ ] مقایسه‌ی REST، GraphQL، gRPC، SOAP و RPC

**۰۲ — دیتابیس و ذخیره‌سازی تو مقیاس بالا**
- [ ] مقایسه‌ی SQL و NoSQL و انتخاب دیتابیس مناسب
- [ ] تراکنش‌ها، ACID و سطوح Isolation
- [ ] sharding، پارتیشن‌بندی و consistent hashing
- [ ] رپلیکیشن و Read Replicaها
- [ ] ایندکس‌گذاری عمیق: B-tree تو مقایسه با LSM-tree
- [ ] قفل‌گذاری خوش‌بینانه تو مقایسه با بدبینانه
- [ ] صف‌های پیام و event streaming

**۰۳ — کشینگ و پرفورمنس**
- [ ] استراتژی‌های کشینگ و سیاست‌های انقضا (eviction policies)
- [ ] شبکه‌های CDN و اعدادی که هر مهندس بک‌اندی باید بدونه

**۰۴ — الگوهای سیستم توزیع‌شده**
- [ ] قضیه CAP و مدل‌های Consistency
- [ ] ویژگی idempotency و قفل‌های توزیع‌شده
- [ ] الگوی تلاش مجدد، کلیدهای قطع‌کننده (circuit breaker) و تایم‌اوت
- [ ] تولید شناسه (ID) یکتا تو مقیاس بالا
- [ ] مقیاس‌پذیری عمودی/افقی و پارتیشن‌بندی

**۰۵ — امنیت و احراز هویت تو مقیاس بالا**
- [ ] فرق Encoding، Encryption و Hashing
- [ ] پروتکل HTTPS و فرآیند TLS handshake
- [ ] مقایسه‌ی session، JWT، OAuth2 و SSO

**۰۶ — مبانی DevOps و کلود**
- [ ] داکر و کانتینرها
- [ ] مبانی کوبرنیتیز (Kubernetes)
- [ ] استراتژی‌های استقرار (آبی-سبز، کاناری و رولینگ)
- [ ] قوانین ۱۲ عاملی اپلیکیشن (Twelve-Factor App)

**۰۷ — طراحی سیستم کاربردی**
- [ ] طراحی کوتاه کننده لینک (URL shortener)
- [ ] طراحی Rate Limiter توزیع‌شده
- [ ] طراحی سیستم چت (Chat System)
- [ ] طراحی سیستم ارسال اعلان (Notification System)
- [ ] طراحی زمان‌بندی جاب توزیع‌شده (با نگاه مجدد به TaskForge)
</details>
