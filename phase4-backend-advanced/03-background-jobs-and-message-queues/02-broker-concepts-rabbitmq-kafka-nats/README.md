# 04.3.2 — Broker concepts: RabbitMQ, Kafka, NATS

*(Reading only — no crate. There's nothing to `cargo check` here; the
previous lesson's toy queue is where the hands-on practice lives.)*

Lesson 01 built a queue directly on top of Postgres, using `FOR UPDATE SKIP
LOCKED`. This lesson is a conceptual survey of the alternative: dedicated
message brokers — software whose entire job is moving messages between
producers and consumers, rather than a database repurposed for it. You
won't write code against any of these here; the goal is vocabulary and
trade-off judgment, so that when a real project's requirements outgrow a
Postgres-backed queue, you know what to reach for and why.

## Why not just always use a dedicated broker?

Because "dedicated" cuts both ways: a message broker is *better* at moving
messages, at the cost of being *another whole system* to run, monitor,
upgrade, and reason about failure modes for — on top of the database you
already operate. `capstone-taskforge`'s own
[ADR-0002](../../../capstone-taskforge/docs/adr/0002-postgres-backed-queue.md)
makes exactly this call explicitly: Postgres for v1, a broker only if a
specific, concrete need (multi-datacenter delivery, true pub/sub fan-out, or
throughput Postgres genuinely can't sustain) shows up later. Treat this
lesson as "know your options," not "always use the fanciest tool."

## RabbitMQ — a message broker (queues, routing, delivery guarantees)

RabbitMQ implements the **AMQP** model: producers publish messages to an
**exchange**, which routes them to one or more **queues** based on rules
(direct, topic, fanout — pattern-matching on a message's routing key),
consumers pull from queues. This routing layer is RabbitMQ's headline
feature: it's straightforward to fan one message out to several independent
consumers, or route messages to different queues based on content, entirely
in broker configuration rather than application code.

- **Delivery model**: at-least-once by default (a consumer must explicitly
  `ack` a message; an unacked message gets redelivered), work-queue-style —
  once a message is acked, it's gone.
- **Ordering**: guaranteed only within a single queue, not globally.
- **Good fit for**: task queues with complex routing needs (route by job
  type, priority, or tenant), RPC-style request/reply patterns, systems
  that want a message gone once it's processed (not kept around for replay).
- **Trade-off vs. this lesson's toy queue**: real routing topology and
  broker-managed retry/dead-lettering (RabbitMQ has built-in dead-letter
  exchanges) vs. Postgres's simplicity and "it's a table you can `SELECT`
  from with SQL you already know."

## Kafka — a distributed commit log (streaming, replay, high throughput)

Kafka is a different model entirely: not a queue that empties as messages
are consumed, but an append-only, partitioned, replicated **log**.
Producers append messages to a **topic** (split into ordered **partitions**
for parallelism); consumers track their own **offset** (position) into the
log and can replay from any earlier offset — messages aren't deleted on
read, only after a configured retention period.

- **Delivery model**: at-least-once (or exactly-once with more setup), but
  the defining feature is that consumption doesn't destroy data — many
  independent consumer groups can each read the same topic at their own
  pace, and a consumer can rewind and replay history.
- **Ordering**: guaranteed within a partition, not across partitions of the
  same topic — this is the throughput/ordering trade-off: more partitions
  means more parallelism but a looser total ordering guarantee.
- **Good fit for**: event streaming (an "an order was placed" event that
  many independent downstream systems — billing, shipping, analytics — each
  want to react to, at their own pace), audit logs, very high sustained
  throughput (Kafka is built for millions of messages/second across a
  cluster).
- **Trade-off vs. a job queue**: Kafka is not naturally a "claim one job,
  exactly one worker processes it" primitive the way RabbitMQ or this
  lesson's toy queue are — that requires layering consumer-group semantics
  and careful offset management on top. Reach for Kafka when the shape of
  the problem is "broadcast this event to N independent interested
  parties," not "distribute this work item to exactly one of N workers."

## NATS — lightweight pub/sub, with JetStream for persistence

NATS starts from core pub/sub: publish a message to a **subject**, every
currently-connected subscriber to that subject gets it — by default, no
persistence at all (a subscriber that wasn't listening at publish time
missed the message, full stop). **JetStream** (NATS's persistence layer)
adds durable streams, replay, and at-least-once delivery on top, closing
much of the gap with Kafka/RabbitMQ, but the core value proposition stays:
NATS is built to be small, fast, and operationally simple — a single
lightweight binary, sub-millisecond latency, minimal configuration.

- **Delivery model**: fire-and-forget by default (core NATS); durable,
  acknowledged, replayable with JetStream enabled.
- **Good fit for**: service-to-service request/reply in a microservices
  architecture, lightweight real-time notifications, systems that value
  operational simplicity and low latency over the heavier feature set of
  Kafka/RabbitMQ.
- **Trade-off**: less mature ecosystem and fewer of the advanced
  routing/streaming features RabbitMQ and Kafka have had for years — the
  appeal is explicitly "do less, but do it simply and fast."

## A comparison table

| | Model | Delivery | Ordering | Replay | Best at |
|---|---|---|---|---|---|
| **This lesson's toy queue (Postgres)** | Table + `SKIP LOCKED` | At-least-once | Roughly FIFO (per query) | No (rows are updated in place) | Simplicity, transactional consistency with other DB writes |
| **RabbitMQ** | Exchange → queue routing | At-least-once (ack-based) | Per-queue | No (message gone once acked) | Complex routing, work queues, RPC |
| **Kafka** | Partitioned commit log | At-least-once (configurable) | Per-partition | Yes (configurable retention) | High-throughput event streaming, multiple independent consumers |
| **NATS (+ JetStream)** | Pub/sub subjects (+ durable streams) | Fire-and-forget, or durable with JetStream | Per-stream (JetStream) | Yes with JetStream | Low-latency service mesh messaging, operational simplicity |

## Connecting back to what you've already built

- The "never double-claim" property `FOR UPDATE SKIP LOCKED` gives you
  (lesson 01) has an analogue in every one of these: RabbitMQ's per-message
  ack model, Kafka's consumer-group partition assignment (only one consumer
  in a group reads a given partition at a time), and JetStream's durable
  consumer acknowledgment all solve the identical "exactly one worker gets
  this unit of work" problem, just with different mechanics.
- The "cache stampede" idea from
  [`01-caching-with-redis`](../../01-caching-with-redis/01-cache-aside-ttl-invalidation/README.md)
  and the retry-with-jitter idea in
  [`capstone-taskforge`'s ADR-0004](../../../capstone-taskforge/docs/adr/0004-worker-failure-handling.md)
  both apply identically here: a broker outage that causes every consumer to
  reconnect and immediately retry at the same moment is the same
  "thundering herd" shape as a cache expiring under load.
- Module 06 ([system design fundamentals](../../06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking/README.md))
  revisits idempotency in more depth — every "at-least-once" delivery
  guarantee above means your consumer code must tolerate processing the
  same message twice without corrupting state, exactly the idempotency
  problem discussed there.

## Next

the recall questions — no code, just the trade-off reasoning.
