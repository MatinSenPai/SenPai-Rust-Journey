# 05.1 — Structured logging with `tracing`

## Why `println!` stops working

Every earlier lesson in this repo that needed to see what was happening
reached for `println!` or `dbg!`, and that was fine — a single-threaded kata
with one thing happening at a time doesn't need more. A real backend breaks
that in three specific ways:

- **No log levels.** `println!` prints unconditionally. In production you
  want `DEBUG`-level detail available but *off* by default (too much volume,
  too slow, often sensitive), `INFO` for normal operation, `WARN`/`ERROR`
  for things that need attention — and the ability to flip the verbosity up
  for one module while debugging, without a redeploy. `println!` has no
  concept of any of this.
- **No structured fields.** `println!("charged order {order_id} for
  {amount_cents} cents")` produces one opaque string. If you later want
  "show me every log line where `amount_cents > 100_00`," you're grepping
  text and hoping the format never changed. A structured log emits
  `order_id` and `amount_cents` as separate, typed key-value fields — a log
  aggregator (Loki, Datadog, CloudWatch Logs Insights, ...) can filter,
  aggregate, and alert on those fields directly, the same way you'd query
  columns in a database instead of parsing a CSV blob by hand.
- **No request correlation.** A real backend handles many requests
  concurrently. If `charge_payment` logs "payment declined," which of the
  47 requests currently in flight was that for? `println!` has no way to
  answer that short of manually threading a request ID string into every
  single log call, by hand, forever.

## `tracing`'s model: spans and events

`tracing` (the crate this lesson uses, already in this repo's
`taskforge-api`) has two core concepts:

- **A span** represents a *unit of work with a duration* — "handling this
  HTTP request," "processing this order." You enter a span, do work
  (possibly calling other instrumented functions), and exit it.
- **An event** is a *point-in-time occurrence* — "payment charged,"
  "connection failed." This is the direct analogue of a single `println!`
  call, except every event carries structured fields and — critically — is
  automatically tagged with whichever span(s) were active when it fired.

```rust
#[tracing::instrument]
pub async fn process_order(order_id: &str, amount_cents: u64) -> Result<(), OrderError> {
    tracing::info!("processing order");
    charge_payment(order_id, amount_cents).await?;
    send_confirmation(order_id).await;
    tracing::info!("order processing complete");
    Ok(())
}
```

`#[tracing::instrument]` is a macro that wraps the whole function body in a
span named after the function (`process_order`), automatically recording
every argument as a structured field on that span (using each type's
`Debug` impl — no manual "attach this field" boilerplate). If
`process_order` calls `charge_payment`, and `charge_payment` is *also*
`#[tracing::instrument]`-annotated, its span nests inside `process_order`'s
— and any `tracing::info!`/`warn!`/`error!` event fired anywhere inside that
nested call is automatically attributed to both spans, no request-ID
parameter threading required. This is the fix for the "no request
correlation" problem above: correlation falls out of the call structure for
free.

## Structured fields, not string interpolation

```rust
tracing::info!(amount_cents, "payment charged");     // structured field
tracing::info!("payment charged for {amount_cents}"); // just a string — don't do this
```

The first form records `amount_cents` as its own field (name inferred from
the local variable), queryable independently of the human-readable
message. The second form bakes the value into an opaque string — you've
thrown away exactly the information a structured backend needs. This
lesson's `charge_payment` uses the first form deliberately; get in the habit
now, since retrofitting structured fields onto a codebase full of
string-interpolated logs later is real, tedious work.

## `tracing_subscriber`: turning events into output

`tracing` itself only defines *how to emit* spans and events — something
else has to decide what to actually do with them (print to stdout, ship to
a log aggregator, both). That's `tracing_subscriber`'s job. `src/lib.rs`'s
`build_subscriber` builds a `tracing_subscriber::fmt()` subscriber two ways:

- **Human-readable** (`json: false`) — colored, single-line-per-event
  output meant for a developer staring at a local terminal.
- **JSON** (`json: true`, via `.json()`) — one JSON object per event, the
  format every real log aggregator expects, so you can query on
  `amount_cents` or `order_id` as first-class fields instead of regex-ing a
  free-text log line.

Both branches also chain `.with_env_filter(EnvFilter::from_default_env())`,
which is what makes the `RUST_LOG=debug` / `RUST_LOG=my_crate=trace`
environment-variable convention work — the level filter lives in the
subscriber, not scattered through `if log_level >= ...` checks in your own
code.

Note the return type: `Box<dyn tracing::Subscriber + Send + Sync>`, not
`impl tracing::Subscriber`. The `if json { ... } else { ... }` branches
produce two genuinely different concrete types (`.json()` changes the
formatter type parameter on the builder) — `impl Trait` requires a single
underlying type, so the two branches have to be erased behind a trait
object to unify.

## Why this function doesn't call `.init()`

A real `main.rs` would call `build_subscriber(true).init()` (or the
`tracing_subscriber` equivalent) exactly once, at startup, installing it as
the process-wide default. But this is a *library* crate with tests, and
installing a **global** default subscriber can only happen once per
process — a second `.init()` call anywhere else in the same test binary
panics. `build_subscriber` stops short of installing anything; it just
constructs one and hands it back, leaving the "install it" decision (and
its exactly-once constraint) to whoever actually owns `main`.

## Testing logs without capturing stdout

You can't easily assert on `println!`-style stdout output in a Rust unit
test. But `tracing`'s subscriber is itself just a trait — nothing stops you
from writing one that records events into a `Vec` instead of printing them.
`src/lib.rs`'s test module does exactly that: a hand-rolled `RecordingLayer`
(implementing `tracing_subscriber::Layer`) that pushes every event's span
name, level, message, and fields into a shared, `Mutex`-guarded `Vec`. Tests
install it with `tracing::subscriber::set_default(...)` — a **thread-local**,
**temporary** override (scoped to a guard, undone when the guard drops),
deliberately different from the global, permanent `.init()` a real `main`
uses. That's what lets multiple tests each install their own recorder
without racing a single global subscriber, or colliding with
`build_subscriber`'s own tests.

A dedicated crate (`tracing-test`) exists for this, but a `Layer` is a small
enough interface that hand-rolling one here is more educational than adding
a dependency — and it's the exact extension point real backends plug a JSON
formatter or an OpenTelemetry exporter into, so understanding it pays off
beyond this one lesson.

## Your task

Open `src/lib.rs`. Three things are `todo!()`-gated:

1. `process_order`'s body — emit start/completion events, call
   `charge_payment` then `send_confirmation`.
2. `charge_payment`'s body — emit a `warn!` event and return `Err` for a
   zero-amount order, otherwise emit an `info!` event with `amount_cents`
   as a structured field and return `Ok(())`.
3. `send_confirmation`'s body — emit one `info!` event.
4. `build_subscriber` — construct the `fmt` subscriber described above,
   toggling `.json()` on `json`, and box the result.

## Next

`cargo test -p p4-05-01-structured-logging-with-tracing`, then
`solution/SOLUTION.md` — but only after a real attempt.
