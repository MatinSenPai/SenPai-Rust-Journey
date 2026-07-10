# Checkpoint

1. Explain concretely why `println!("order {order_id} charged {amount_cents}
   cents")` is worse for a production backend than
   `tracing::info!(order_id, amount_cents, "payment charged")`, even though
   both ultimately produce text somewhere. Name at least two concrete
   capabilities you lose with the `println!` version.
2. What does `#[tracing::instrument]` actually add around a function call?
   In your own words, what's the difference between a "span" and an
   "event" in `tracing`'s model — which one does `charge_payment`'s
   `#[tracing::instrument]` create, and which one does the
   `tracing::info!(...)` call inside its body create?
3. This lesson's tests never call `build_subscriber(...)` followed by
   `.init()`. They install a hand-rolled `RecordingLayer` via
   `tracing::subscriber::set_default` instead. What would go wrong if a
   test (or two tests in the same run) each called `.init()` on a
   `tracing_subscriber::fmt()` subscriber? Why does `set_default`'s
   thread-local, guard-scoped install sidestep that problem?
4. Trace through `process_order("order-x", 0)`. Which function detects the
   problem, what does it log, and at what level? Why is
   `tracing::warn!` there a better choice than silently returning `Err`
   with no log at all, or than `tracing::error!`?
5. Why does JSON output matter specifically once your service is running as
   several replicas behind a load balancer, being scraped by a log
   aggregator, rather than as one process on your laptop? Connect your
   answer to what "structured field" means, as opposed to "a string that
   happens to contain the field's value."
6. `build_subscriber` returns `Box<dyn tracing::Subscriber + Send + Sync>`
   rather than `impl tracing::Subscriber`. Why doesn't `impl Trait` work
   here, given the function's `if json { ... } else { ... }` branches?
