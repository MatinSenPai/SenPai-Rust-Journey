# ۰۴.۳.۲ — مفاهیم broker: RabbitMQ، Kafka و NATS

*(فقط مطالعه است؛ تمرین عملی در صف Postgresِ درس قبل است.)*

درس قبل queue را با `FOR UPDATE SKIP LOCKED` روی Postgres ساخت. اینجا جایگزینش را می‌شناسی: **message broker**، نرم‌افزاری که کارش رساندن پیام بین producer و consumer است. هدف این نیست که همیشه ابزار پرزرق‌وبرق‌تر را انتخاب کنی؛ وقتی نیاز واقعی از queueِ Postgres گذشت، بفهمی چرا و به چه چیزی مهاجرت کنی. TaskForge هم در ADR-0002 برای v1 عمداً Postgres را برگزیده و broker را برای نیاز مشخص بعدی گذاشته است.

```senpai-visual
{"kind":"queue","labels":["producer","broker یا log","consumer group","ack/replay"]}
```

## RabbitMQ: queue و routing

RabbitMQ در مدل AMQP، پیام را به **exchange** می‌فرستد و exchange بر پایهٔ ruleهای direct/topic/fanout آن را به یک یا چند **queue** route می‌کند. consumer از queue می‌خواند و با `ack` پیام را نهایی می‌کند؛ بدون ack دوباره تحویل می‌گیرد. delivery معمولاً at-least-once و ordering فقط در یک queue است. برای job queue با priority، tenant یا نوع job و dead-lettering مناسب است. بهایش یک سیستم عملیاتی تازه در کنار database است.

## Kafka: log توزیع‌شده و قابل replay

Kafka queue خالی‌شونده نیست؛ یک **append-only log** partition‌شده و replicate است. producer به topic می‌نویسد، consumer offset خودش را نگه می‌دارد و می‌تواند از گذشته replay کند؛ خواندن پیام را حذف نمی‌کند، retention حذف می‌کند. ordering فقط در یک partition است. برای eventی مانند «سفارش ثبت شد» که billing، shipping، analytics و fraud هرکدام با سرعت خودشان می‌خواهند بخوانند عالی است. برای «یک job دقیقاً به یک worker» به‌تنهایی primitive طبیعی نیست.

## NATS و JetStream

NATS پایه‌اش pub/sub سبک است: فقط subscriber حاضر پیام را می‌گیرد. JetStream persistence، stream durable، ack و replay اضافه می‌کند. مزیت اصلی سادگی عملیاتی، latency بسیار کم و binary سبک است؛ برای request/reply بین microserviceها و notification لحظه‌ای خوش‌دست است، با featureهای routing/streaming بالغِ کمتر از RabbitMQ/Kafka.

| ابزار | مدل | replay | مناسب برای |
|---|---|---|---|
| Postgres + `SKIP LOCKED` | table و claim | خیر | سادگی و transaction با writeهای DB |
| RabbitMQ | exchange → queue | خیر | work queue، routing پیچیده، RPC |
| Kafka | partitioned commit log | بله | event stream پُرحجم و consumer مستقل |
| NATS + JetStream | pub/sub و stream durable | بله | latency کم و عملیات ساده |

همهٔ deliveryهای at-least-once یعنی consumer باید **idempotent** باشد: retry ممکن است یک پیام را دوبار تحویل دهد. outageِ broker و reconnect هم همان شکل thundering herdِ cache stampede را دارد؛ retry با jitter فراموش نشود.

## Checkpoint

`CHECKPOINT.fa.md` را پاسخ بده؛ کدی در این درس نیست.
