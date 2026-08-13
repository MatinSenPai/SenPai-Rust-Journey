# فاز ۴ — مباحث پیشرفته‌ی بک‌اند + طراحی سیستم (System Design)

تو فاز ۳ تونستی یه بک‌اند کاملاً کارآمد و قابل استفاده بسازی. حالا فاز ۴ قراره واژه‌ها و ابزارهایی رو بهت بده که بتونی باهاشون در مورد مبحث مقیاس‌پذیری (scale) حرف بزنی — و البته برای اون مقیاس‌ها کد بنویسی: چیزایی مثل کش کردن (caching)، محدودسازی نرخ درخواست‌ها (rate limiting)، کارهای پس‌زمینه (background jobs)، نظارت‌پذیری (observability)، و اون ایده‌های مهم طراحی سیستم (مثل قضیه CAP، متعادل‌کننده‌های بار (load balancing)، اصلِ بی‌اثر بودنِ تکرار (idempotency)، و قفل‌های توزیع‌شده (distributed locking)) که هم برای مصاحبه‌کننده‌ها خیلی مهمن و هم تو حوادث و خرابی‌های واقعی تو پروداکشن به دادت می‌رسن. هر ایده‌ی مربوط به طراحی سیستم، به جای اینکه به عنوان یه سخنرانی و نظریه‌ی خشک‌وخالی و انتزاعی رها بشه، دقیقاً به همون ماژولی وصل شده که به طور طبیعی و منطقی بهش تعلق داره.

1. **کش کردن (Caching) با Redis**
   - [۰۱ — الگوی Cache-aside، زمان انقضا (TTL)، و باطل کردن کَش (invalidation)](01-caching-with-redis/01-cache-aside-ttl-invalidation/README.fa.md)
2. **محدودسازی نرخ درخواست (Rate limiting) و فشار متقابل (backpressure)**
   - [۰۱ — الگوی Token bucket و ابزار `tower::limit`](02-rate-limiting-and-backpressure/01-token-bucket-and-tower-limit/README.fa.md)
3. **کارهای پس‌زمینه (Background jobs) و صف پیام‌ها (message queues)**
   - [۰۱ — ساخت یه صف تمرینی با ویژگیِ `SKIP LOCKED` تو Postgres](03-background-jobs-and-message-queues/01-postgres-skip-locked-toy-queue/README.fa.md)
   - [۰۲ — مفاهیم مربوط به Brokerها: RabbitMQ، Kafka، NATS](03-background-jobs-and-message-queues/02-broker-concepts-rabbitmq-kafka-nats/README.fa.md) *(فقط برای مطالعه — هیچ کدی نداره)*
4. **قراردادهای gRPC و GraphQL**
   - [۰۱ — ساخت سرویس gRPC با کریت `tonic`](04-grpc-and-graphql/01-tonic-grpc-service/README.fa.md)
   - [۰۲ — مروری بر `async-graphql`](04-grpc-and-graphql/02-async-graphql-overview/README.fa.md)
5. **نظارت‌پذیری (Observability)**
   - [۰۱ — لاگ‌زنی ساخت‌یافته (Structured logging) با کریت `tracing`](05-observability/01-structured-logging-with-tracing/README.fa.md)
   - [۰۲ — متریک‌ها (Metrics) و ابزار Prometheus](05-observability/02-metrics-and-prometheus/README.fa.md)
6. **مبانی طراحی سیستم (System design fundamentals)**
   - [۰۱ — قضیه CAP، مقیاس‌پذیری، load balancing، ویژگی idempotency، و قفل‌های توزیع‌شده](06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking/README.fa.md) *(فقط برای مطالعه + همراه با مثال‌های حل‌شده — هیچ کدی نداره)*
7. **استقرار (Deployment) و عملیات**
   - [۰۱ — ابزار Docker Compose و CI](07-deployment-and-operations/01-docker-compose-and-ci/README.fa.md)
8. **پرفورمنس (Performance) و پروفایل‌گیری (profiling)**
   - [۰۱ — بنچمارک با Criterion و استفاده از flamegraphها](08-performance-and-profiling/01-criterion-benchmarks-and-flamegraphs/README.fa.md)

**یه چک‌پوینت انگیزشی:** [مأموریت جانبی ۴ — ساخت API تجمیع‌کننده‌ی انیمه و مانگا](../side-quests/sq-04-anime-manga-aggregator-api/README.fa.md)
— این پروژه میاد مفاهیم کش کردن، محدودسازی نرخ، و نظارت‌پذیری رو تو یه پروژه‌یِ کم‌ریسک‌تر و راحت‌تر، به عنوان یه دست‌گرمی واسه پروژه‌یِ پایانی (capstone) با هم ترکیب می‌کنه.

وقتی تمام گزینه‌های فاز ۴ تو فایلِ [`PROGRESS.md`](../PROGRESS.md) تیک خورد، اون موقع می‌تونی بری سراغ [پروژه‌ی پایانی (Capstone): TaskForge](../capstone-taskforge/README.fa.md) — هر چیزی که تا الان خوندی، در واقع داشت تو رو برای رسیدن به این نقطه آماده می‌کرد.