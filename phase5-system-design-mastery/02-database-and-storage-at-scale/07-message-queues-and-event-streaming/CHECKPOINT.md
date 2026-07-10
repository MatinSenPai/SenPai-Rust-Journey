# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can name the actual guarantee a system
provides, not recite "at-least-once" as a slogan.

1. Explain the structural difference between a message queue and an event
   stream/log — not "different products," but what actually happens to a
   message after it's consumed in each model. Give an example workload
   that fits each one well, and explain why forcing that workload onto the
   *wrong* model (queue for a fan-out use case, or log for a
   distribute-to-exactly-one-worker use case) causes real friction, not
   just inconvenience.

2. Explain why "exactly-once delivery" is usually not a real, independent
   guarantee — what specific failure (indistinguishable to the sender) makes
   it impossible to guarantee in general, and what two things does a system
   actually combine to approximate it?

3. `phase4-backend-advanced/06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking`
   discusses `taskforge-api`'s `POST /jobs` idempotency gap. Explain how
   that HTTP-layer problem and at-least-once message delivery are "the same
   shape of problem," in your own words.

4. Walk through the exact sequence of events (worker action → failure →
   what the toy queue does or doesn't do) that leaves a job permanently
   stuck in `running` in `phase4-backend-advanced/03-background-jobs-and-message-queues/01-postgres-skip-locked-toy-queue`
   as literally built. What delivery guarantee does this make the toy
   queue actually provide for that specific failure case, and how is that
   different from "at-least-once"?

5. `capstone-taskforge/docs/adr/0004-worker-failure-handling.md` documents
   the *identical* gap in the real `taskforge-storage`/`taskforge-worker`,
   not just the toy queue. Why does this lesson consider that a good sign
   about how this repo handles known limitations, rather than a bad sign
   about TaskForge's quality? What would the alternative (not documenting
   it) have cost a future contributor?

6. Describe, concretely, what you'd need to add to either the toy queue or
   `taskforge-storage` to close this gap (a visibility timeout or a
   heartbeat — pick one and describe it in your own words, you don't need
   to write the SQL). What new column(s) would the `jobs` table need, and
   what would the reclaim check actually look like?
