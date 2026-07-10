# Solution

```rust
#[tracing::instrument]
pub async fn process_order(order_id: &str, amount_cents: u64) -> Result<(), OrderError> {
    tracing::info!("processing order");
    charge_payment(order_id, amount_cents).await?;
    send_confirmation(order_id).await;
    tracing::info!("order processing complete");
    Ok(())
}

#[tracing::instrument]
async fn charge_payment(order_id: &str, amount_cents: u64) -> Result<(), OrderError> {
    if amount_cents == 0 {
        tracing::warn!("refusing to charge a zero-amount order");
        return Err(OrderError::PaymentDeclined {
            order_id: order_id.to_string(),
        });
    }
    tracing::info!(amount_cents, "payment charged");
    Ok(())
}

#[tracing::instrument]
async fn send_confirmation(order_id: &str) {
    tracing::info!("confirmation sent");
}

pub fn build_subscriber(json: bool) -> Box<dyn tracing::Subscriber + Send + Sync> {
    let filter = tracing_subscriber::EnvFilter::from_default_env();
    if json {
        Box::new(tracing_subscriber::fmt().with_env_filter(filter).json().finish())
    } else {
        Box::new(tracing_subscriber::fmt().with_env_filter(filter).finish())
    }
}
```

## Spans vs. events, concretely

`#[tracing::instrument]` on `process_order`, `charge_payment`, and
`send_confirmation` gives each function call its own **span** — a named,
timed region — automatically populated with that function's arguments as
fields (`order_id`, `amount_cents`), no manual field-attaching required.
Each `tracing::info!`/`warn!` call inside those bodies is an **event** — a
point-in-time occurrence, tagged (again automatically) with whichever
span(s) are currently open. Concretely, the `"payment charged"` event fired
inside `charge_payment` carries both its own explicit field
(`amount_cents`, passed directly to `info!`) *and* is attributed to the
`charge_payment` span by the subscriber, without `charge_payment` having to
pass `order_id` to the `info!` call by hand — that's the whole point of
nesting spans instead of manually stitching a request ID through every log
call, the way you'd have to with bare `println!`.

## Why `order_id` unused in `send_confirmation`'s body still compiles clean

`send_confirmation`'s body only calls `tracing::info!("confirmation
sent")` — it never touches `order_id` directly. That's not a bug and
clippy doesn't flag it: `#[tracing::instrument]` macro-expands into code
that *does* read every argument (to record it as a span field), so by the
time the compiler sees the expanded function, `order_id` is used. This is
worth internalizing, because it looks surprising the first time — an
apparently-unused parameter that's actually consumed by an attribute macro
above the function, not by anything visible in the body.

## `Box<dyn Subscriber>`, and why `impl Trait` doesn't work here

`tracing_subscriber::fmt()` returns a builder generic over its formatter;
calling `.json()` swaps in a different formatter type parameter before
`.finish()`. That means the `if json { ... } else { ... }` branches produce
two different concrete types implementing `Subscriber` — `impl
tracing::Subscriber` requires the compiler to pick one single underlying
type for every path through the function, which two genuinely different
struct types can't satisfy. Boxing behind `dyn tracing::Subscriber + Send +
Sync` erases the concrete type entirely, so both branches return the same
thing from the caller's point of view: "some subscriber object." The `Send
+ Sync` bounds matter because a real `main.rs` would move this into
`.init()` (which needs a subscriber usable across threads, since a tokio
runtime schedules work across worker threads by default).

## Testing pattern: `set_default`, not `.init()`

`build_subscriber(...).init()` installs a **global**, **process-wide**,
**permanent** default subscriber — exactly what `main` should do exactly
once, and exactly what a test suite cannot do repeatedly (a second
`.init()` anywhere in the same test binary panics with "a global default
trace dispatcher has already been set"). The `recorder()` helper instead
uses `tracing::subscriber::set_default(subscriber)`, which:

- Only affects the **current thread**, for as long as the returned
  `DefaultGuard` is alive (it resets to whatever was previously active when
  dropped, at the end of the enclosing scope).
- Can be called as many times as you want, once per test, with zero
  collision risk between tests running on separate threads — exactly the
  same reasoning `capstone-taskforge/taskforge-api`'s `OnceLock`-guarded
  `metrics_handle()` uses for a *global*, install-once resource (Prometheus'
  recorder genuinely can't be installed twice), just solved the opposite
  way here: `tracing`'s subscriber has a *scoped* alternative to global
  install, so the test suite reaches for that instead of needing a
  once-guard at all.

## The hand-rolled `Layer`

`RecordingLayer` implements exactly one method, `on_event`, and pushes a
flattened `RecordedEvent` (span name, level, message, structured fields)
into a shared `Vec` for tests to assert against afterward. This is real
production machinery, not a test-only trick — `tracing_subscriber::Layer`
is the same trait a JSON formatter or an OpenTelemetry exporter implements;
this lesson's version just writes into memory instead of stdout or a
network socket. `Context::event_span(event)` is what recovers "which span
was active" from the ambient `Registry` (the `S: LookupSpan` bound is what
makes that lookup possible), which is how the tests can assert that the
`"payment charged"` event specifically came from inside `charge_payment`'s
span rather than merely "some event fired somewhere."

## Why the decline test asserts `send_confirmation` never ran

`charge_payment`'s `Err` return propagates through `process_order`'s `?`
operator, which returns immediately — `send_confirmation(order_id).await`
on the next line never executes. The test proves this at the *observability*
layer (no event tagged with the `send_confirmation` span exists in the
recorded output) rather than by adding a side channel like a counter,
because that's genuinely how you'd audit this in a real deployed service:
if the logs say a step never ran, it never ran.
