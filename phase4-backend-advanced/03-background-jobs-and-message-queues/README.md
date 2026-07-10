# 03 — Background jobs & message queues

Not every unit of work belongs in the request/response cycle. Sending an
email, resizing an image, calling a slow third-party API — do those
*later*, off the request path, and the caller gets a fast response while
the work happens asynchronously. That requires a **queue**: somewhere
durable to put "do this later" work items, and a safe way for one or more
worker processes to pick them up without two workers ever doing the same
item twice.

1. [Postgres `SKIP LOCKED` toy queue](01-postgres-skip-locked-toy-queue/README.md)
2. [Broker concepts: RabbitMQ, Kafka, NATS](02-broker-concepts-rabbitmq-kafka-nats/README.md) *(reading only — no crate)*
