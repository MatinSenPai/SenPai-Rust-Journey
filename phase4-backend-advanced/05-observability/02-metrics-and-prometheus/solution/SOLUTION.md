# Solution

```rust
async fn create_widget(
    State(state): State<AppState>,
    Json(req): Json<CreateWidgetRequest>,
) -> (StatusCode, Json<Widget>) {
    let start = Instant::now();

    let id = state.next_id.fetch_add(1, Ordering::SeqCst);
    let widget = Widget { id, name: req.name };
    state.widgets.lock().unwrap().insert(id, widget.clone());

    metrics::counter!("widgets_created_total").increment(1);
    metrics::histogram!("widget_create_duration_seconds").record(start.elapsed().as_secs_f64());

    (StatusCode::CREATED, Json(widget))
}

async fn get_widget(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Widget>, StatusCode> {
    let start = Instant::now();

    let found = state.widgets.lock().unwrap().get(&id).cloned();

    metrics::histogram!("widget_lookup_duration_seconds").record(start.elapsed().as_secs_f64());

    match found {
        Some(widget) => {
            metrics::counter!("widget_lookups_total", "result" => "hit").increment(1);
            Ok(Json(widget))
        }
        None => {
            metrics::counter!("widget_lookups_total", "result" => "miss").increment(1);
            Err(StatusCode::NOT_FOUND)
        }
    }
}
```

## Counters vs. histograms, in the actual code

`widgets_created_total` and `widget_lookups_total` are counters — numbers
that only ever go up, answering "how many total." `widget_create_duration_seconds`
and `widget_lookup_duration_seconds` are histograms — instead of one
number, `metrics-exporter-prometheus` buckets every recorded value
(here, a handler's wall-clock duration) so a Prometheus query can later ask
"what's the p99 latency," not just "what's the average" — an average
hides the slow outliers a histogram's bucket distribution reveals.
`get_widget` records its histogram *before* branching on hit vs. miss,
deliberately: latency is worth measuring for both outcomes, but the counter
underneath it needs a `result` label to tell them apart, since "how many
lookups happened" and "how many of those were hits" are different
questions a dashboard needs to answer separately.

## Why every test can safely call `build_router()`

`PrometheusBuilder::install_recorder()` sets a *global* recorder for the
whole process — the `metrics` crate's macros (`metrics::counter!`, etc.)
write to whatever recorder was last installed globally, not to anything
scoped per-`Router` or per-test. Calling `install_recorder()` a second
time panics. Since this lesson's tests each call `build_router()`
independently (each test wants its own fresh `HashMap` of widgets), a
naive `build_router()` that called `install_recorder()` directly would
panic on the second test. The fix — copied verbatim from
`capstone-taskforge/taskforge-api`, since this exact trap was already hit
once building that crate — is the `OnceLock`: `metrics_handle()` installs
the recorder on the *first* call from anywhere in the process and returns
a cheap `.clone()` of the same handle on every call after, so any number
of `build_router()` calls in the same test binary share one underlying
recorder safely.

## What `/metrics` actually returns

`state.metrics_handle.render()` produces the Prometheus text exposition
format — plain text, one metric per block, e.g.:

```
# TYPE widgets_created_total counter
widgets_created_total 3
# TYPE widget_lookups_total counter
widget_lookups_total{result="hit"} 2
widget_lookups_total{result="miss"} 1
```

This is not a custom format invented for this lesson — it's the literal
response body a real Prometheus server's scrape job parses when it polls a
target's `/metrics` endpoint on an interval. `metrics_endpoint_exposes_the_expected_counters_and_histograms`
asserting the response body `.contains(...)` specific metric/label
substrings is checking the same thing a real Prometheus deployment
would be able to scrape and graph.

## Logs vs. metrics, concretely, in this same lesson

Compare this lesson's `metrics::counter!`/`histogram!` calls to the
previous lesson's `tracing::info!` calls: a `tracing` event fires once per
occurrence and carries rich, request-specific context (which order id,
which user) — great for "what happened on this one request," expensive to
query in aggregate across millions of events. A `metrics` counter/histogram
collapses every occurrence into a running aggregate (a total count, a
bucketed distribution) — terrible for "what happened on this one request"
(there's no per-request detail left at all), but exactly what you want for
"how many requests per second," "what's our p99 latency this hour," the
kind of question a dashboard or an alert threshold needs answered cheaply
even under very high request volume. Real production services run both,
side by side, for exactly this reason — neither replaces the other.
