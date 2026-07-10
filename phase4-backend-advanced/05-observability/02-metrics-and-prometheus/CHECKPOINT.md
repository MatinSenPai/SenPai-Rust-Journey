# Checkpoint

1. In your own words, what question does a metric answer that a log line
   can't (and vice versa)? Give a concrete example of a question you'd
   answer by looking at `widget_lookups_total` and one you'd only be able
   to answer by looking at individual log lines.
2. `create_widget` increments `widgets_created_total` with `.increment(1)`
   (a counter) and separately records `widget_create_duration_seconds`
   with `.record(...)` (a histogram). Why are these different metric kinds
   rather than the same kind used twice — what real question does each one
   let you ask that the other can't?
3. `get_widget` labels its counter with `"result" => "hit"` or `"result" =>
   "miss"` rather than using two differently-named counters
   (`widget_hits_total`, `widget_misses_total`). What's the practical
   benefit of one labeled metric name over two separate metric names, once
   you're looking at this in a real Prometheus/Grafana dashboard?
4. `metrics_handle()` uses `OnceLock::get_or_init`. What error would
   `PrometheusBuilder::new().install_recorder()` return if called directly,
   unguarded, from every test in this file? Why does `PrometheusHandle`
   being cheap to `.clone()` matter to this fix working at all?
5. The `metrics_endpoint_exposes_the_expected_counters_and_histograms` test
   asserts `body.contains(...)` for specific metric names rather than
   asserting an exact numeric count (e.g. `widgets_created_total 1`). Why
   would asserting an exact count be flaky in this specific test setup —
   what's actually shared across every test function in this file?
6. A real Prometheus server scrapes `/metrics` on an interval rather than
   your service pushing metrics to it. What does that imply about a metric
   for something that happened once and is already over by the time the
   next scrape happens (say, a single very slow outlier request)? Does a
   counter/histogram, sampled only at scrape time, lose any information
   compared to a log line recorded the instant it happened?
