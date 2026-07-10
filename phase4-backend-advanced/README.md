# Phase 4 — Backend Advanced + System Design

Phase 3 gave you a working backend. Phase 4 gives you the vocabulary and
tools to talk about — and build for — scale: caching, rate limiting,
background jobs, observability, and the system-design ideas (CAP theorem,
load balancing, idempotency, distributed locking) that interviewers and real
production incidents both care about. Each system-design idea is attached to
the module it naturally belongs to, not left as an abstract lecture.

1. **Caching with Redis**
   - [01 — Cache-aside, TTL, invalidation](01-caching-with-redis/01-cache-aside-ttl-invalidation/README.md)
2. **Rate limiting & backpressure**
   - [01 — Token bucket and `tower::limit`](02-rate-limiting-and-backpressure/01-token-bucket-and-tower-limit/README.md)
3. **Background jobs & message queues**
   - [01 — Postgres `SKIP LOCKED` toy queue](03-background-jobs-and-message-queues/01-postgres-skip-locked-toy-queue/README.md)
   - [02 — Broker concepts: RabbitMQ, Kafka, NATS](03-background-jobs-and-message-queues/02-broker-concepts-rabbitmq-kafka-nats/README.md) *(reading only — no crate)*
4. **gRPC & GraphQL**
   - [01 — `tonic` gRPC service](04-grpc-and-graphql/01-tonic-grpc-service/README.md)
   - [02 — `async-graphql` overview](04-grpc-and-graphql/02-async-graphql-overview/README.md)
5. **Observability**
   - [01 — Structured logging with `tracing`](05-observability/01-structured-logging-with-tracing/README.md)
   - [02 — Metrics and Prometheus](05-observability/02-metrics-and-prometheus/README.md)
6. **System design fundamentals**
   - [01 — CAP, scaling, load balancing, idempotency, distributed locking](06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking/README.md) *(reading + worked examples — no crate)*
7. **Deployment & operations**
   - [01 — Docker Compose and CI](07-deployment-and-operations/01-docker-compose-and-ci/README.md)
8. **Performance & profiling**
   - [01 — Criterion benchmarks and flamegraphs](08-performance-and-profiling/01-criterion-benchmarks-and-flamegraphs/README.md)

**Motivational checkpoint:** [Side-quest 4 — Anime/Manga Aggregator API](../side-quests/sq-04-anime-manga-aggregator-api/README.md)
— combines caching, rate limiting, and observability in one lower-stakes
project, as a warm-up for the capstone.

When Phase 4 is fully checked off in [`PROGRESS.md`](../PROGRESS.md), move on
to the [Capstone: TaskForge](../capstone-taskforge/README.md) — everything
up to here has been building toward it.
