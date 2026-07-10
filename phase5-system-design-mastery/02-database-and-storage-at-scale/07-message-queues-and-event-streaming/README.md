# 02.7 — Message queues and event streaming

No code in this lesson. `phase4-backend-advanced/03-background-jobs-and-message-queues/02-broker-concepts-rabbitmq-kafka-nats`
already surveyed RabbitMQ, Kafka, and NATS at a "what is each one, when
would you reach for it" level — read that lesson first if you haven't;
this one doesn't repeat that ground. This lesson goes one level deeper on
two things that lesson only touched briefly: the structural
queue-vs-log distinction underneath "which product do I pick," and
delivery semantics — what "at-least-once" and "exactly-once" actually mean,
precisely, rather than as marketing copy on a broker's homepage.

## Message queue vs. event stream: not just different products

It's tempting to treat RabbitMQ/SQS and Kafka as "two options for the same
job, pick based on scale." They're not — they implement genuinely
different **delivery models**, and picking the wrong one for your access
pattern doesn't just cost you performance, it makes the thing you're
trying to build harder or impossible to express cleanly.

**A message queue is point-to-point.** A message is produced once,
delivered to (conceptually) one consumer, and then it's *gone* — consumed,
acked, removed from the queue. If you need three different downstream
systems to each react to the same event, a plain queue doesn't give you
that for free; you'd need to publish the same message to three separate
queues (which is what RabbitMQ's fanout exchange does under the hood — it
duplicates the message into multiple queues at publish time, one per
subscribing queue). The mental model: a work ticket that gets picked up
and consumed by exactly one worker, then thrown away.

**An event stream (log) is pub-sub, with replay.** A message (usually
called an *event* in this context) is appended to a durable, ordered log
and *stays there* (until a retention period expires) — many independent
consumer groups can each read the same log at their own pace, each
tracking their own position (offset), and a consumer can rewind and
re-read history it's already seen before. Reading doesn't destroy data;
only retention does. The mental model: a shared, append-only diary that
any number of readers can read from any point, as many times as they want.

The concrete difference this creates: "one worker processes this job" is
what a **queue** is built for — TaskForge's `claim_next` is exactly this
shape, and it's why TaskForge doesn't need Kafka at all. "N independent
systems each need to react to the same event, at their own pace, possibly
replaying it later for a new system that didn't exist yet when the event
first happened" is what a **log** is built for — an "order placed" event
that billing, shipping, *and* a not-yet-built fraud-detection service (added
six months later, and able to replay the last 90 days of events to build
its initial model) all need to see. Reaching for a queue when you actually
need a log means bolting fan-out onto something not built for it; reaching
for a log when you actually need "distribute this work to exactly one of N
workers" means layering consumer-group/offset-management complexity on top
of a tool whose core abstraction doesn't natively give you that (as
`02-broker-concepts-rabbitmq-kafka-nats` already notes about Kafka).

## Delivery semantics, precisely

Three terms get thrown around, and only two of them describe something a
real system can actually promise:

- **At-most-once** — a message is delivered zero or one times, never more.
  Achieved by *not* retrying: if delivery fails or is unconfirmed, the
  message is just gone. Simple, and sometimes fine (a live metrics sample
  where losing one data point occasionally doesn't matter) — but silently
  dropping a message is unacceptable for almost anything with business
  consequences ("send this email," "process this payment").
- **At-least-once** — a message is delivered one or more times, never
  zero. Achieved by retrying on any doubt: if the sender isn't sure a
  message was received/processed (a timeout, a dropped ack), it resends.
  This guarantees you'll never silently lose a message — but it means your
  consumer *will*, eventually, see the same message more than once, and
  has to be written to tolerate that.
- **"Exactly-once"** — delivered exactly one time, no more, no less. This
  is, in the general case (an arbitrary producer, an arbitrary network, an
  arbitrary consumer that might crash mid-processing), **not actually
  achievable as a standalone primitive** — it's usually a marketing claim
  built on top of at-least-once delivery *plus* idempotent consumers, not
  a real, independent guarantee the transport layer alone provides. The
  reasoning: at some layer, the sender always has to decide "did the
  receiver get this?" without a perfectly reliable way to know for certain
  (the network can lose the message *or* lose the acknowledgment — those
  two failures are indistinguishable to the sender). The only way to be
  safe against both losing a message *and* duplicating it is to keep
  retrying (at-least-once) and make the *processing itself* safe to run
  more than once (idempotent) — at which point "exactly-once" is really
  "at-least-once delivery, with duplicates absorbed harmlessly by the
  consumer," not a magic transport-layer trick. (Kafka's "exactly-once
  semantics," to be fair, get closer than most — transactional producers
  plus idempotent producer IDs really do dedupe at the broker level for
  Kafka-to-Kafka pipelines — but the moment your consumer does anything
  external, like calling a payment API or sending a real email, you're
  back to needing idempotency at that boundary regardless of what the
  broker promises internally.)

This is exactly the idempotency problem `phase4-backend-advanced/06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking`
already introduced for `taskforge-api`'s `POST /jobs` (a client retry after
a dropped connection can enqueue the same job twice) — at-least-once
delivery is the *same shape* of problem, just happening at the
message-transport layer instead of the HTTP layer. The fix pattern is
identical: give each unit of work a stable identity, and make the
"has this already been done?" check happen before doing the work again.

## What guarantee does the toy queue actually provide?

`phase4-backend-advanced/03-background-jobs-and-message-queues/01-postgres-skip-locked-toy-queue`
gives you hands-on practice with the `FOR UPDATE SKIP LOCKED` claiming
pattern, and it's worth being precise about which delivery guarantee it
actually implements as built — because the honest answer has a real gap in
it. Once a worker's `claim_next` marks a job `running`, that job is no
longer visible to any other worker's `claim_next` (it's not `pending`
anymore) — so far, so good, exactly one worker has it. But what happens if
*that* worker crashes, or is killed, or loses its network connection,
*after* claiming the job but *before* calling `complete`? As the toy queue
is actually built in that lesson, **nothing** — there's no timeout, no
heartbeat, no background sweep that notices a job has been sitting in
`running` for suspiciously long and puts it back up for grabs. That job is
stuck `running` forever, silently never retried, never completed. That's
not at-least-once delivery — it's closer to at-most-once for exactly the
crash-after-claim case, which is the opposite of what you generally want
from a job queue (losing work silently is usually worse than occasionally
doing it twice).

This is a real, honest gap in the toy queue as built, called out explicitly
here rather than glossed over — matching how
`phase3-backend-foundations/04-postgres-and-sqlx/03-anime-catalog-postgres-backed`'s
lost-update race and `taskforge-api`'s missing idempotency key are both
flagged elsewhere in this repo as genuine, undone work rather than silently
assumed-solved. The standard fix, and the one `capstone-taskforge`'s own
production job queue actually needs to be honest about: a claimed job needs
either a **visibility timeout** (if a job has been `running` for longer
than some threshold with no completion signal, a background sweep resets
it back to claimable — this is exactly what SQS calls a "visibility
timeout" and what a Postgres-backed queue would implement as a periodic
`UPDATE jobs SET status = 'pending' WHERE status = 'running' AND updated_at
< now() - interval '5 minutes'`-style query) or an active **heartbeat**
(the worker periodically touches a `last_heartbeat_at` column while still
processing, and the same kind of sweep reclaims jobs whose heartbeat has
gone stale). Without one of these, a claimed-but-crashed worker's job is
permanently lost — which makes the toy queue, as literally built in that
lesson, provide **at-most-once** delivery for the crash-during-processing
case specifically, not the at-least-once guarantee a production job queue
generally wants.

This isn't a gap unique to the toy version, either — worth being honest
about that too. `capstone-taskforge/docs/adr/0004-worker-failure-handling.md`,
under "Idempotent claiming," states the *identical* gap exists in the real
`taskforge-storage`/`taskforge-worker`: "a worker that crashes mid-job
(process killed, not just the job panicking) leaves that job's row in
`Running` with no worker left to finish it," and names a heartbeat/lease
timeout as an explicit, documented "stretch extension" that v1 does not
ship — in the ADR's own words, "v1 ships without automatic reclaim-on-crash,
and says so explicitly rather than pretending otherwise." So the toy
queue's gap and TaskForge's real, production-shaped gap are the same gap,
not a simplified-toy-version-only problem — a good reminder that "at-least-once"
is a property you have to deliberately build (claim timeout, heartbeat,
reclaim sweep), not something you get for free just because a queue has a
`claim_next` that prevents *double*-claiming. Preventing double-claims and
guaranteeing eventual completion are two different guarantees, and this
repo currently only has the first one, honestly documented as such in both
places.

## Checkpoint

No `cargo test` for this lesson — go straight to `CHECKPOINT.md`.
