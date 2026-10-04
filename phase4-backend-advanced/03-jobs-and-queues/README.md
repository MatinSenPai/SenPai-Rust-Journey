# Module 3 — Jobs & queues

Some work should not happen inside a request. This module builds a job queue from a database table up, surveys the real brokers, and then spends most of its time on what actually goes wrong: messages delivered twice, a database write and a message that must happen together, and the job that fails forever.

1. [01 — Job queue design (a Postgres SKIP LOCKED queue)](01-postgres-skip-locked-toy-queue/README.md)
   — A job queue built from a table, and the design choices it forces.
2. [02 — Broker concepts: RabbitMQ, Kafka, NATS](02-broker-concepts-rabbitmq-kafka-nats/README.md)
   — What the real brokers are for and how they differ (reading only).
3. [03 — At-least-once delivery, idempotency and deduplication](03-at-least-once-delivery-and-idempotency/README.md)
   — Why a message arrives twice, and how a consumer survives it.
4. [04 — The transactional outbox pattern](04-the-transactional-outbox/README.md)
   — Writing to the database and publishing a message, without losing either.
5. [05 — Retries, dead-letter queues and poison messages](05-retries-dlq-and-poison-messages/README.md)
   — Backoff, giving up, and the message that crashes every consumer.
