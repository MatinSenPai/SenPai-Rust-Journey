# Phase 4 — Backend Advanced + System Design

Phase 3 gave you a working backend. Phase 4 gives you the vocabulary and tools to talk about — and build for — scale: caching, rate limiting, background jobs, service-to-service communication, observability, distributed data, and the system-design ideas (CAP, load balancing, idempotency, capacity estimation) that interviewers and real production incidents both care about. Each system-design idea is attached to the module it naturally belongs to, not left as an abstract lecture.

## 1. [Caching](01-caching/README.md)

1. [Redis from Rust](01-caching/01-redis-from-rust/README.md)
2. [Cache strategies and invalidation](01-caching/02-cache-aside-ttl-invalidation/README.md)
3. [Stampede protection, single-flight and TTL jitter](01-caching/03-stampede-protection-and-single-flight/README.md)
4. [In-process caching (moka) and the two-tier pattern](01-caching/04-in-process-caching-and-two-tier/README.md)

## 2. [Rate limiting & backpressure](02-rate-limiting-and-backpressure/README.md)

1. [Rate-limiting algorithms (token bucket, tower::limit)](02-rate-limiting-and-backpressure/01-token-bucket-and-tower-limit/README.md)
2. [Distributed rate limiting with Redis and Lua](02-rate-limiting-and-backpressure/02-distributed-rate-limiting-redis-lua/README.md)
3. [Backpressure, load shedding and queue depth](02-rate-limiting-and-backpressure/03-backpressure-and-load-shedding/README.md)

## 3. [Jobs & queues](03-jobs-and-queues/README.md)

1. [Job queue design (a Postgres SKIP LOCKED queue)](03-jobs-and-queues/01-postgres-skip-locked-toy-queue/README.md)
2. [Broker concepts: RabbitMQ, Kafka, NATS](03-jobs-and-queues/02-broker-concepts-rabbitmq-kafka-nats/README.md)
3. [At-least-once delivery, idempotency and deduplication](03-jobs-and-queues/03-at-least-once-delivery-and-idempotency/README.md)
4. [The transactional outbox pattern](03-jobs-and-queues/04-the-transactional-outbox/README.md)
5. [Retries, dead-letter queues and poison messages](03-jobs-and-queues/05-retries-dlq-and-poison-messages/README.md)

## 4. [Service-to-service communication](04-service-to-service/README.md)

1. [A gRPC service with tonic](04-service-to-service/01-tonic-grpc-service/README.md)
2. [async-graphql: an overview](04-service-to-service/02-async-graphql-overview/README.md)
3. [Choosing between REST, gRPC, GraphQL and events](04-service-to-service/03-choosing-rest-grpc-graphql-or-events/README.md)

## 5. [Observability](05-observability/README.md)

1. [Structured logging with tracing](05-observability/01-structured-logging-with-tracing/README.md)
2. [Metrics (RED and USE) and Prometheus](05-observability/02-metrics-and-prometheus/README.md)
3. [Distributed tracing with OpenTelemetry, end to end](05-observability/03-distributed-tracing-with-opentelemetry/README.md)
4. [SLOs, alerting and on-call reality](05-observability/04-slos-alerting-and-on-call/README.md)

## 6. [Distributed data patterns](06-distributed-data-patterns/README.md)

1. [Sagas and compensating transactions](06-distributed-data-patterns/01-sagas-and-compensating-transactions/README.md)
2. [Event sourcing and CQRS](06-distributed-data-patterns/02-event-sourcing-and-cqrs/README.md)
3. [Multi-tenancy models](06-distributed-data-patterns/03-multi-tenancy-models/README.md)

## 7. [System design fundamentals](07-system-design-fundamentals/README.md)

1. [CAP, scaling, load balancing, idempotency and locking](07-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking/README.md)
2. [Capacity estimation and back-of-the-envelope math](07-system-design-fundamentals/02-capacity-estimation/README.md)

## 8. [Deployment & operations](08-deployment-and-operations/README.md)

1. [Docker for Rust and Compose](08-deployment-and-operations/01-docker-compose-and-ci/README.md)
2. [CI/CD for Rust](08-deployment-and-operations/02-ci-cd-for-rust/README.md)
3. [Secrets, config and environments in production](08-deployment-and-operations/03-config-and-secrets/README.md)
4. [Zero-downtime deploys, migrations and rollback](08-deployment-and-operations/04-zero-downtime-deploys-and-rollback/README.md)

## 9. [Performance](09-performance/README.md)

1. [Profiling, benchmarks and flamegraphs](09-performance/01-criterion-benchmarks-and-flamegraphs/README.md)
2. [Allocation awareness and hot-path Rust](09-performance/02-allocation-awareness-and-hot-paths/README.md)
3. [Async performance traps](09-performance/03-async-performance-traps/README.md)
4. [Load testing and finding your real limits](09-performance/04-load-testing/README.md)

**A warm-up project:** [Side-quest 4 — Anime/Manga Aggregator API](../side-quests/sq-04-anime-manga-aggregator-api/README.md) combines caching, rate limiting, and observability in one lower-stakes project, as practice for the capstone.

When Phase 4 is fully checked off in [`PROGRESS.md`](../PROGRESS.md), move on to the [Capstone: TaskForge](../capstone-taskforge/README.md) — everything up to here has been building toward it.
