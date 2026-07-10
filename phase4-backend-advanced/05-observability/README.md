# 05 — Observability

Every earlier Phase 4 module made a system *more capable* (a cache, a rate
limiter, a job queue, a second API protocol). This module makes a running
system *legible* — able to answer "what is it doing right now, and why did
it just do that" without attaching a debugger to a production process. Two
lessons, two complementary tools:

1. [`tracing`: structured logging](01-structured-logging-with-tracing/README.md)
   — discrete events ("this request did X"), with structured fields you can
   query later and spans that correlate everything happening inside one
   unit of work, replacing `println!` debugging entirely.
2. [Metrics and Prometheus](02-metrics-and-prometheus/README.md) —
   aggregated numbers over time ("how many, how often, how long"), scraped
   from a `/metrics` endpoint the same way `capstone-taskforge/taskforge-api`
   already exposes one.

Logs and metrics answer different questions and neither replaces the
other — a log line tells you what happened to *this one* request; a metric
tells you whether the *whole system* is healthy. Real backends run both,
and a third pillar (distributed tracing across services) that's out of
scope here but builds directly on the span concept from lesson 01.
