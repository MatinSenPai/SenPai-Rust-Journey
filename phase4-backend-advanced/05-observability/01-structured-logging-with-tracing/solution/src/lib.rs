//! Structured logging with `tracing` — spans, structured fields, and
//! machine-parseable output, instead of `println!` string interpolation.
//! See `README.md` for the theory. `process_order` simulates handling one
//! request end to end (charge a payment, send a confirmation), instrumented
//! the way a real axum handler chain would be.

use thiserror::Error;

/// Everything that can go wrong while processing an order. `charge_payment`
/// is the only fallible step here — a real system would have far more
/// variants (inventory, shipping, ...), but one is enough to demonstrate
/// error-path logging (`tracing::warn!`) alongside the happy path.
#[derive(Debug, Error)]
pub enum OrderError {
    #[error("payment declined for order {order_id}")]
    PaymentDeclined { order_id: String },
}

/// The entry point: "handle this order end to end." `#[tracing::instrument]`
/// opens a span named `process_order` for the duration of this call,
/// carrying `order_id` and `amount_cents` as fields (instrument records
/// every argument as a span field automatically, using each type's `Debug`
/// impl). Every event emitted by `charge_payment` and `send_confirmation`
/// while they run *underneath* this call is nested inside that span — the
/// mechanism that gives you request correlation without threading an
/// `order_id` parameter into every `tracing::info!` call by hand.
#[tracing::instrument]
pub async fn process_order(order_id: &str, amount_cents: u64) -> Result<(), OrderError> {
    tracing::info!("processing order");
    charge_payment(order_id, amount_cents).await?;
    send_confirmation(order_id).await;
    tracing::info!("order processing complete");
    Ok(())
}

/// Simulates charging a payment provider. In a real service this would be
/// an HTTP call out; here it just decides based on `amount_cents`, so the
/// lesson has both a success path (`info!`) and a failure path (`warn!`)
/// to instrument.
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

/// Simulates sending a confirmation email/notification. No failure mode —
/// kept simple so the lesson's error-path test has exactly one place that
/// can fail (`charge_payment`).
#[tracing::instrument]
async fn send_confirmation(order_id: &str) {
    tracing::info!("confirmation sent");
}

/// Builds a `tracing_subscriber` `fmt` subscriber, but stops short of
/// installing it globally (`.init()`) — that's `main`'s job in a real
/// binary, not a library function's, and a global install can only ever
/// happen once per process, which would make this function untestable (see
/// README.md's "why this function doesn't call `.init()`" section). `json`
/// selects human-readable output (a local dev terminal) vs. one JSON object
/// per event (what a real log aggregator expects to scrape).
pub fn build_subscriber(json: bool) -> Box<dyn tracing::Subscriber + Send + Sync> {
    let filter = tracing_subscriber::EnvFilter::from_default_env();
    if json {
        Box::new(
            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .json()
                .finish(),
        )
    } else {
        Box::new(tracing_subscriber::fmt().with_env_filter(filter).finish())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};
    use tracing::field::{Field, Visit};
    use tracing_subscriber::layer::{Context, SubscriberExt};
    use tracing_subscriber::Layer;

    /// One flattened, captured `tracing` event: the name of whatever span
    /// was open when it fired (if any), its level, its `message` field
    /// (what `info!`/`warn!` implicitly set from the format string), and
    /// every other structured field attached to it. Enough to assert
    /// against without pulling in a whole log-capturing crate.
    #[derive(Debug, Default, Clone)]
    struct RecordedEvent {
        span: Option<String>,
        level: Option<tracing::Level>,
        message: String,
        fields: BTreeMap<String, String>,
    }

    impl Visit for RecordedEvent {
        fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
            let formatted = format!("{value:?}");
            if field.name() == "message" {
                self.message = formatted;
            } else {
                self.fields.insert(field.name().to_string(), formatted);
            }
        }
    }

    /// A minimal `tracing_subscriber::Layer` that records every event into
    /// a shared `Vec` instead of printing anything. This is the
    /// dependency-light alternative to pulling in the `tracing-test` crate
    /// just to assert "did this event fire with these fields" — a `Layer`
    /// is the same extension point real backends (the JSON formatter, an
    /// OpenTelemetry exporter, ...) plug into.
    #[derive(Clone, Default)]
    struct RecordingLayer {
        events: Arc<Mutex<Vec<RecordedEvent>>>,
    }

    impl<S> Layer<S> for RecordingLayer
    where
        S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    {
        fn on_event(&self, event: &tracing::Event<'_>, ctx: Context<'_, S>) {
            let mut recorded = RecordedEvent {
                span: ctx.event_span(event).map(|s| s.name().to_string()),
                level: Some(*event.metadata().level()),
                ..Default::default()
            };
            event.record(&mut recorded);
            self.events.lock().unwrap().push(recorded);
        }
    }

    /// Installs a `RecordingLayer` as the *thread-local* default subscriber
    /// (`tracing::subscriber::set_default`, not the global `.init()`) for
    /// as long as the returned guard is alive, and hands back the shared
    /// event buffer. Thread-local and temporary, not global and permanent,
    /// is exactly why this doesn't collide with `build_subscriber`'s own
    /// tests, or with other tests running on their own threads.
    fn recorder() -> (
        tracing::subscriber::DefaultGuard,
        Arc<Mutex<Vec<RecordedEvent>>>,
    ) {
        let events: Arc<Mutex<Vec<RecordedEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let layer = RecordingLayer {
            events: events.clone(),
        };
        let subscriber = tracing_subscriber::registry().with(layer);
        let guard = tracing::subscriber::set_default(subscriber);
        (guard, events)
    }

    #[tokio::test]
    async fn success_path_emits_events_and_tags_the_charge_with_its_span() {
        let (_guard, events) = recorder();

        process_order("order-1", 500).await.unwrap();

        let events = events.lock().unwrap();

        let charged = events
            .iter()
            .find(|e| e.span.as_deref() == Some("charge_payment") && e.message.contains("charged"))
            .expect("charge_payment should emit an info event tagged with its own span");
        assert_eq!(
            charged.fields.get("amount_cents").map(String::as_str),
            Some("500")
        );

        assert!(
            events
                .iter()
                .any(|e| e.span.as_deref() == Some("send_confirmation")),
            "send_confirmation should run (and emit an event) on the happy path"
        );
        assert!(
            events
                .iter()
                .any(|e| e.span.as_deref() == Some("process_order")
                    && e.message.contains("complete")),
            "process_order itself should log a completion event"
        );
    }

    #[tokio::test]
    async fn zero_amount_orders_are_declined_and_never_reach_send_confirmation() {
        let (_guard, events) = recorder();

        let result = process_order("order-2", 0).await;

        assert!(matches!(result, Err(OrderError::PaymentDeclined { .. })));

        let events = events.lock().unwrap();
        let warning = events
            .iter()
            .find(|e| {
                e.span.as_deref() == Some("charge_payment") && e.level == Some(tracing::Level::WARN)
            })
            .expect("the decline path should emit a warn-level event inside charge_payment's span");
        assert!(warning.message.contains("zero-amount"));

        assert!(
            !events
                .iter()
                .any(|e| e.span.as_deref() == Some("send_confirmation")),
            "a declined payment must short-circuit before send_confirmation runs"
        );
    }

    #[test]
    fn build_subscriber_constructs_cleanly_in_both_formats() {
        // Not installed globally (see the doc comment on `build_subscriber`)
        // so constructing one in a test is safe to do more than once.
        let _human_readable = build_subscriber(false);
        let _json = build_subscriber(true);
    }
}
