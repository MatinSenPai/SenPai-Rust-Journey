# ۰۳ — Background job و message queue

هر کار در چرخهٔ request/response جا ندارد. email فرستادن، resize تصویر و تماس با API کندِ بیرونی را بعداً و بیرون مسیر request انجام بده تا caller سریع پاسخ بگیرد. این کار به **queue** نیاز دارد: محل durable برای «بعداً انجام بده» و روشی امن که چند worker یک item را دوبار انجام ندهند.

1. [صف toy با Postgres و `SKIP LOCKED`](01-postgres-skip-locked-toy-queue/README.md)
2. [مفاهیم broker: RabbitMQ، Kafka و NATS](02-broker-concepts-rabbitmq-kafka-nats/README.fa.md)
