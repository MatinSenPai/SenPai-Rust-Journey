# Checkpoint

1. A team's job queue (currently Postgres-backed, like lesson 01's toy
   queue) needs to notify five completely independent downstream systems
   whenever an order is placed — billing, shipping, analytics, fraud
   detection, and a recommendation engine — each processing the event at
   its own pace, and a new sixth system should be able to start consuming
   from an arbitrary point in the past when it comes online next quarter.
   Which broker from this lesson fits best, and specifically which feature
   makes it the right fit (not just "it's popular")?
2. A team needs "distribute these image-resize jobs across a worker pool,
   each job processed by exactly one worker, with configurable priority
   levels and automatic dead-lettering after 3 failures." Which broker (or
   this lesson's toy queue) fits best? Would you actually recommend
   introducing a new broker for this, given `capstone-taskforge`'s
   ADR-0002 reasoning — why or why not?
3. Every broker surveyed here (and the toy queue from lesson 01) is
   described as "at-least-once" delivery, not "exactly-once." Concretely,
   what does a consumer need to do in its own logic to be safe under
   at-least-once delivery (i.e., correct even if it processes the same
   message twice)? Give a concrete example of an operation that is *not*
   naturally safe to run twice, and how you'd make it safe.
4. Kafka's ordering guarantee is "within a partition, not across the whole
   topic." Give a concrete example of data where that partial ordering
   would cause a real bug if the consumer assumed full topic-wide ordering
   instead (hint: think about what determines which partition a given
   message lands in).
5. `capstone-taskforge`'s ADR-0002 lists three "revisit triggers" for
   moving off Postgres. Restate them in your own words, and for each one,
   name which broker from this lesson would be the natural fit if that
   trigger were hit.
