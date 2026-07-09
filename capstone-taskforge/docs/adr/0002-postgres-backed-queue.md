# ADR-0002: PostgreSQL as the queue backend for v1

## Status
Accepted

## Context

A job queue needs a durable place to store job state and a way for
multiple worker processes to safely claim jobs without two workers running
the same job at once. The obvious "real" options: a dedicated message
broker (RabbitMQ, Kafka, NATS — surveyed conceptually in
`phase4-backend-advanced/03-background-jobs-and-message-queues/
02-broker-concepts-rabbitmq-kafka-nats`), or the relational database the
system likely already has for everything else.

## Decision

Use PostgreSQL, via `SELECT ... FOR UPDATE SKIP LOCKED`, as TaskForge's
queue backend — the same pattern rehearsed at small scale in
`phase4-backend-advanced/03-background-jobs-and-message-queues/
01-postgres-skip-locked-toy-queue`, now built out into a real system.

Reasoning:
- **Operational simplicity.** Most backend systems (including everything
  built in Phase 3-4 of this curriculum) already run Postgres. Adding
  Kafka or RabbitMQ as a hard dependency for job queueing means operating
  a second stateful system, with its own failure modes, monitoring, and
  operational knowledge required — a real cost, not free.
- **Transactional consistency.** `SELECT ... FOR UPDATE SKIP LOCKED`
  inside a transaction that also does other business-logic writes gives
  you "enqueue this job in the same transaction as the row it's about"
  for free — genuinely hard to get with an external broker without a
  separate outbox pattern.
- **Good enough throughput for the vast majority of real systems.**
  Postgres-backed queues comfortably handle thousands of jobs/second on
  modest hardware — past that point, a dedicated broker becomes
  justified, and the `JobStore` trait boundary (ADR-0001) means that's an
  additive change later, not a rewrite.

## Consequences

- **Positive**: one fewer piece of infrastructure to run/operate/learn for
  v1; transactional guarantees "for free"; directly reuses Phase 3's
  sqlx/Postgres skills instead of introducing a whole new client library.
- **Negative**: Postgres wasn't *designed* to be a queue — polling
  (`claim_next`, called on an interval by each worker) has real latency
  overhead compared to a broker's push-based delivery, and very high
  throughput (tens of thousands of jobs/sec sustained) would eventually
  need a dedicated broker instead. This is an explicit, documented
  trade-off, not an oversight — see this ADR's title: "for v1."
- **Revisit trigger**: if TaskForge needed multi-datacenter delivery, true
  pub/sub fan-out to many independent consumers, or sustained throughput
  Postgres can't handle, that's the signal to introduce a `KafkaJobStore`
  or similar behind the same `JobStore` trait.
