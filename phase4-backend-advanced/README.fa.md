# فاز ۴ — بک‌اندِ پیشرفته و طراحیِ سیستم

فاز ۳ یک بک‌اندِ کارا به تو داد. فاز ۴ واژه‌ها و ابزارهایی می‌دهد که بتوانی درباره‌یِ مقیاس حرف بزنی — و برایش بسازی: کش، محدودسازیِ نرخ، کارهایِ پس‌زمینه، ارتباطِ سرویس‌به‌سرویس، مشاهده‌پذیری، دادهِ توزیع‌شده، و ایده‌هایِ طراحیِ سیستم (CAP، load balancing، خودتوانی، تخمینِ ظرفیت) که هم مصاحبه‌کننده‌ها و هم حادثه‌هایِ واقعیِ پروداکشن به آن‌ها اهمیت می‌دهند. هر ایده‌یِ طراحیِ سیستم به ماژولی وصل شده که طبیعتاً به آن تعلق دارد، نه اینکه یک سخنرانیِ انتزاعی بماند.

## ۱. [کش (Caching)](01-caching/README.fa.md)

۱. [Redis از Rust](01-caching/01-redis-from-rust/README.fa.md)
۲. [راهبردهایِ کش و باطل‌سازی](01-caching/02-cache-aside-ttl-invalidation/README.fa.md)
۳. [محافظت در برابرِ stampede، single-flight و jitterِ TTL](01-caching/03-stampede-protection-and-single-flight/README.fa.md)
۴. [کشِ درون‌پردازه‌ای (moka) و الگویِ دولایه](01-caching/04-in-process-caching-and-two-tier/README.fa.md)

## ۲. [محدودسازیِ نرخ و فشارِ برگشتی](02-rate-limiting-and-backpressure/README.fa.md)

۱. [الگوریتم‌هایِ محدودسازیِ نرخ (token bucket و tower::limit)](02-rate-limiting-and-backpressure/01-token-bucket-and-tower-limit/README.fa.md)
۲. [محدودسازیِ نرخِ توزیع‌شده با Redis و Lua](02-rate-limiting-and-backpressure/02-distributed-rate-limiting-redis-lua/README.fa.md)
۳. [فشارِ برگشتی، load shedding و عمقِ صف](02-rate-limiting-and-backpressure/03-backpressure-and-load-shedding/README.fa.md)

## ۳. [کار و صف](03-jobs-and-queues/README.fa.md)

۱. [طراحیِ صفِ کار (یک صفِ SKIP LOCKED در Postgres)](03-jobs-and-queues/01-postgres-skip-locked-toy-queue/README.fa.md)
۲. [مفاهیمِ بروکر: RabbitMQ، Kafka، NATS](03-jobs-and-queues/02-broker-concepts-rabbitmq-kafka-nats/README.fa.md)
۳. [تحویلِ حداقل‌یک‌بار، خودتوانی و حذفِ تکراری‌ها](03-jobs-and-queues/03-at-least-once-delivery-and-idempotency/README.fa.md)
۴. [الگویِ transactional outbox](03-jobs-and-queues/04-the-transactional-outbox/README.fa.md)
۵. [تلاشِ دوباره، DLQ و پیام‌هایِ سمی](03-jobs-and-queues/05-retries-dlq-and-poison-messages/README.fa.md)

## ۴. [ارتباطِ سرویس‌به‌سرویس](04-service-to-service/README.fa.md)

۱. [سرویسِ gRPC با tonic](04-service-to-service/01-tonic-grpc-service/README.fa.md)
۲. [async-graphql: یک مرور](04-service-to-service/02-async-graphql-overview/README.fa.md)
۳. [انتخاب بینِ REST، gRPC، GraphQL و رویداد](04-service-to-service/03-choosing-rest-grpc-graphql-or-events/README.fa.md)

## ۵. [مشاهده‌پذیری (Observability)](05-observability/README.fa.md)

۱. [لاگِ ساخت‌یافته با tracing](05-observability/01-structured-logging-with-tracing/README.fa.md)
۲. [متریک‌ها (RED و USE) و Prometheus](05-observability/02-metrics-and-prometheus/README.fa.md)
۳. [ردیابیِ توزیع‌شده با OpenTelemetry، سرتاسری](05-observability/03-distributed-tracing-with-opentelemetry/README.fa.md)
۴. [SLO، آلارم و واقعیتِ on-call](05-observability/04-slos-alerting-and-on-call/README.fa.md)

## ۶. [الگوهایِ دادهِ توزیع‌شده](06-distributed-data-patterns/README.fa.md)

۱. [ساگا و تراکنش‌هایِ جبرانی](06-distributed-data-patterns/01-sagas-and-compensating-transactions/README.fa.md)
۲. [Event sourcing و CQRS](06-distributed-data-patterns/02-event-sourcing-and-cqrs/README.fa.md)
۳. [مدل‌هایِ چندمستاجری (multi-tenancy)](06-distributed-data-patterns/03-multi-tenancy-models/README.fa.md)

## ۷. [مبانیِ طراحیِ سیستم](07-system-design-fundamentals/README.fa.md)

۱. [CAP، مقیاس‌پذیری، load balancing، خودتوانی و قفل](07-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking/README.fa.md)
۲. [تخمینِ ظرفیت و حساب‌وکتابِ پشتِ پاکت](07-system-design-fundamentals/02-capacity-estimation/README.fa.md)

## ۸. [استقرار و عملیات](08-deployment-and-operations/README.fa.md)

۱. [Docker برایِ Rust و Compose](08-deployment-and-operations/01-docker-compose-and-ci/README.fa.md)
۲. [CI/CD برایِ Rust](08-deployment-and-operations/02-ci-cd-for-rust/README.fa.md)
۳. [سکرت‌ها، پیکربندی و محیط‌ها در پروداکشن](08-deployment-and-operations/03-config-and-secrets/README.fa.md)
۴. [استقرارِ بدونِ توقف، مایگریشن و rollback](08-deployment-and-operations/04-zero-downtime-deploys-and-rollback/README.fa.md)

## ۹. [کارایی (Performance)](09-performance/README.fa.md)

۱. [پروفایل‌گیری، بنچمارک و flamegraph](09-performance/01-criterion-benchmarks-and-flamegraphs/README.fa.md)
۲. [آگاهی از تخصیصِ حافظه و Rustِ مسیرِ داغ](09-performance/02-allocation-awareness-and-hot-paths/README.fa.md)
۳. [تله‌هایِ کارایی در async](09-performance/03-async-performance-traps/README.fa.md)
۴. [تستِ بار و پیدا کردنِ حدِّ واقعی](09-performance/04-load-testing/README.fa.md)

**یک پروژه‌یِ گرم‌کردن:** [مأموریتِ جانبی ۴ — API تجمیع‌کننده‌یِ انیمه و مانگا](../side-quests/sq-04-anime-manga-aggregator-api/README.fa.md) کش، محدودسازیِ نرخ و مشاهده‌پذیری را در یک پروژه‌یِ کم‌ریسک‌تر کنار هم می‌گذارد، به‌عنوانِ تمرین برایِ پروژه‌یِ پایانی.

وقتی فاز ۴ در [`PROGRESS.md`](../PROGRESS.md) کامل تیک خورد، برو سراغِ [پروژه‌یِ پایانی (Capstone): TaskForge](../capstone-taskforge/README.fa.md) — هر چیزی که تا اینجا خواندی داشت برایِ همین آماده‌ات می‌کرد.
