# 05.2 — Metrics and Prometheus

## Logs vs. metrics: different questions

Lesson 01 gave you structured, correlated logs — the right tool for "what
happened to *this specific request*." Metrics answer a different question:
**"how is the system doing, in aggregate, over time?"** A log line about one
declined payment is interesting when you're debugging that one order. It's
useless for answering "did our payment decline rate spike in the last ten
minutes across all 50,000 orders we processed" — for that you want a single
number, tracked over time, that you can graph and alert on. That number is
a metric.

Concretely:

- A **log** is a discrete event: "order `abc123` was declined at 14:32:07."
- A **metric** is an aggregated measurement: "1,204 orders were declined in
  the last hour" or "p99 request latency is 340ms."

Both matter, for different jobs. Logs are how you investigate *one* thing
after you already suspect something's wrong. Metrics are how you *notice*
something's wrong in the first place, and how you watch capacity trends
over weeks, not just single incidents.

## Three kinds of metric

This lesson uses the `metrics` crate's macros, all already familiar if
you've read `capstone-taskforge/taskforge-api/src/handlers.rs` (it already
does `metrics::counter!("taskforge_jobs_enqueued_total").increment(1)` on
every successful enqueue):

- **Counter** (`metrics::counter!`) — a number that only ever goes up:
  total requests served, total jobs enqueued, total errors. You read it as
  a *rate* ("counter went up by 340 in the last minute"), not as an
  absolute value.
- **Histogram** (`metrics::histogram!`) — records a *distribution* of
  values, typically durations: not just "average latency was 50ms" (which
  hides a slow tail) but "here's the full spread, so you can ask for the
  p50/p95/p99." This lesson records one histogram value (the handler's
  wall-clock duration) per request.
- **Gauge** (`metrics::gauge!`, not used directly in this lesson's code but
  worth knowing) — a number that goes up *and* down: current queue depth,
  open connections, memory in use. Unlike a counter, its current value is
  meaningful on its own, not just its rate of change.

## `metrics` (the facade) + `metrics-exporter-prometheus` (the backend)

Same split as `tracing`/`tracing_subscriber` from lesson 01: `metrics` is a
facade — `counter!`/`histogram!`/`gauge!` calls don't know or care where
the numbers end up — and `metrics-exporter-prometheus` is one concrete
backend that collects them and can render Prometheus's text exposition
format on demand. A real Prometheus server works by **scraping**: it polls
your service's `/metrics` HTTP endpoint on an interval (typically 15-30s)
and stores every sample it sees, timestamped, so you can later graph
`rate(widgets_created_total[5m])` or similar in a dashboard. Your service
never pushes anywhere; it just always has an up-to-date answer ready
whenever Prometheus asks.

## The `OnceLock<PrometheusHandle>` trap (and its fix)

`PrometheusBuilder::install_recorder()` installs a **global** recorder for
the whole process — and calling it a second time in the same process
**panics**. This repo already hit that trap once, in
`capstone-taskforge/taskforge-api/src/lib.rs`, because `build_router()`
there is called by every single integration test in that crate, and each
call used to try installing a fresh recorder. The fix, copied verbatim into
this lesson's `metrics_handle()`:

```rust
static METRICS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

fn metrics_handle() -> PrometheusHandle {
    METRICS_HANDLE
        .get_or_init(|| {
            PrometheusBuilder::new()
                .install_recorder()
                .expect("installing the Prometheus recorder should only fail if called twice")
        })
        .clone()
}
```

`OnceLock::get_or_init` runs the closure (which calls `install_recorder()`)
at most once per process, no matter how many times `metrics_handle()` — and
therefore `build_router()` — gets called; every call after the first just
clones the already-installed handle (`PrometheusHandle` is cheap to clone,
an `Arc` internally). This lesson's tests call `build_router()` once per
test function, exactly the scenario that broke `taskforge-api` before this
pattern was in place — don't reinvent this, reuse it, exactly as written.

## Reading `src/lib.rs`

`AppState` holds the `PrometheusHandle`, an in-memory `widgets` map
(`Arc<Mutex<HashMap<u64, Widget>>>`), and an `Arc<AtomicU64>` for allocating
ids — the tiny bit of application state this toy service needs to have
something to instrument. `POST /widgets` creates a widget; `GET
/widgets/<id>` looks one up; `GET /metrics` renders whatever the recorder
has collected so far via `state.metrics_handle.render()`, exactly the same
one-liner `taskforge-api`'s own `metrics_handler` uses.

## Your task

Open `src/lib.rs`. Four things are `todo!()`-gated:

1. `create_widget` — allocate an id, store the widget, then increment
   `widgets_created_total` and record `widget_create_duration_seconds`.
2. `get_widget` — look the widget up, then increment `widget_lookups_total`
   with a `result` label (`"hit"` or `"miss"`) and record
   `widget_lookup_duration_seconds`, regardless of which branch you took.
3. `build_router` — wire `AppState` together and register the three routes.

`metrics_handler` (rendering `/metrics`) is given — it's a one-liner
already shown above, not worth re-deriving.

## Next

`cargo test -p p4-05-02-metrics-and-prometheus`, then the recall questions, then
`solution/SOLUTION.md`.
